//! Integration test for the Apple Wallet pass endpoint (PRD §13.3). Exercises
//! the full path: card -> service -> wallet crate -> `.pkpass` bytes.

mod common;

use std::io::{Cursor, Read};

use axum::http::StatusCode;
use serde_json::json;

use common::unique_handle;

#[tokio::test]
async fn apple_pass_endpoint_returns_valid_pkpass() {
    let app = require_app!();
    let token = app.register_and_token().await;

    let handle = unique_handle();
    let definition = json!({
        "face": { "layers": [], "background": { "type": "solid", "value": "#202028" } },
        "back": { "layers": [
            { "type": "contact", "fields": {
                "email": "owner@cardclaws.test",
                "company": "RedClaw",
                "title": "Founder"
            }}
        ]}
    });
    let (_, created) = app
        .request(
            "POST",
            "/v1/cards",
            Some(&token),
            Some(json!({"handle": handle, "definition": definition})),
        )
        .await;
    let id = created["id"].as_str().unwrap();

    let (status, content_type, bytes) = app
        .request_bytes(
            "POST",
            &format!("/v1/cards/{id}/wallet/apple"),
            Some(&token),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(content_type, "application/vnd.apple.pkpass");
    // A .pkpass is a zip — magic bytes "PK".
    assert_eq!(&bytes[0..2], b"PK");

    // Open the bundle and confirm pass.json's QR points at this card's profile.
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();
    assert!(names.iter().any(|n| n == "pass.json"));
    assert!(names.iter().any(|n| n == "strip.png"));
    assert!(names.iter().any(|n| n == "manifest.json"));
    assert!(names.iter().any(|n| n == "signature"));

    let mut pass_json = String::new();
    archive
        .by_name("pass.json")
        .unwrap()
        .read_to_string(&mut pass_json)
        .unwrap();
    let pass: serde_json::Value = serde_json::from_str(&pass_json).unwrap();
    assert_eq!(
        pass["barcode"]["message"],
        format!("https://cardclaws.test/{handle}")
    );
}

#[tokio::test]
async fn google_pass_endpoint_returns_save_url_with_object() {
    use base64::Engine;

    let app = require_app!();
    let token = app.register_and_token().await;

    let handle = unique_handle();
    let definition = json!({
        "face": { "layers": [], "background": { "type": "solid", "value": "#202028" } },
        "back": { "layers": [
            { "type": "contact", "fields": { "title": "Founder", "company": "RedClaw" } }
        ]}
    });
    let (_, created) = app
        .request(
            "POST",
            "/v1/cards",
            Some(&token),
            Some(json!({"handle": handle, "definition": definition})),
        )
        .await;
    let id = created["id"].as_str().unwrap();

    let (status, body) = app
        .request(
            "POST",
            &format!("/v1/cards/{id}/wallet/google"),
            Some(&token),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let save_url = body["saveUrl"].as_str().unwrap();
    assert!(save_url.starts_with("https://pay.google.com/gp/v/save/"));

    // Decode the JWT payload and confirm it carries our object + QR barcode.
    let jwt = save_url.trim_start_matches("https://pay.google.com/gp/v/save/");
    let parts: Vec<&str> = jwt.split('.').collect();
    assert_eq!(parts.len(), 3);
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[1])
        .unwrap();
    let claims: serde_json::Value = serde_json::from_slice(&payload).unwrap();
    assert_eq!(claims["typ"], "savetowallet");
    let obj = &claims["payload"]["genericObjects"][0];
    assert_eq!(
        obj["barcode"]["value"],
        format!("https://cardclaws.test/{handle}")
    );
    assert_eq!(obj["header"]["defaultValue"]["value"], "Card Owner");
}

#[tokio::test]
async fn apple_pass_requires_auth() {
    let app = require_app!();
    let (status, _, _) = app
        .request_bytes(
            "POST",
            "/v1/cards/00000000-0000-0000-0000-000000000000/wallet/apple",
            None,
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
