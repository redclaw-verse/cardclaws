//! Integration tests for asset presigning + vCard export (PRD §13.2, §13.7).

mod common;

use axum::http::StatusCode;
use serde_json::json;

use common::unique_handle;

#[tokio::test]
async fn presign_upload_returns_key_and_url() {
    let app = require_app!();
    let token = app.register_and_token().await;

    let (status, body) = app
        .request(
            "POST",
            "/v1/assets/upload",
            Some(&token),
            Some(json!({"ext": "png"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let key = body["key"].as_str().unwrap();
    assert!(key.ends_with(".png"));
    assert!(key.starts_with("assets/"));
    assert!(body["upload_url"].as_str().unwrap().starts_with("https://"));
}

#[tokio::test]
async fn presign_upload_requires_auth() {
    let app = require_app!();
    let (status, _) = app
        .request(
            "POST",
            "/v1/assets/upload",
            None,
            Some(json!({"ext": "png"})),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn presign_rejects_unsupported_extension() {
    let app = require_app!();
    let token = app.register_and_token().await;
    let (status, body) = app
        .request(
            "POST",
            "/v1/assets/upload",
            Some(&token),
            Some(json!({"ext": "exe"})),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "validation");
}

#[tokio::test]
async fn delete_rejects_other_users_prefix() {
    let app = require_app!();
    let token = app.register_and_token().await;
    // A key under someone else's prefix must be forbidden.
    let (status, body) = app
        .request(
            "DELETE",
            "/v1/assets/assets/00000000-0000-0000-0000-000000000000/x.png",
            Some(&token),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "forbidden");
}

#[tokio::test]
async fn delete_own_asset_succeeds() {
    let app = require_app!();
    let token = app.register_and_token().await;

    // Presign to learn our own key, then delete it.
    let (_, body) = app
        .request(
            "POST",
            "/v1/assets/upload",
            Some(&token),
            Some(json!({"ext": "png"})),
        )
        .await;
    let key = body["key"].as_str().unwrap().to_string();

    // The delete route is /v1/assets/*key and the key itself starts with
    // "assets/", so the full path is /v1/assets/assets/{user}/{file}.png.
    let (status, _) = app
        .request("DELETE", &format!("/v1/assets/{key}"), Some(&token), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(app.assets.deleted.lock().unwrap().contains(&key));
}

#[tokio::test]
async fn export_vcf_returns_vcard_with_contact() {
    let app = require_app!();
    let token = app.register_and_token().await;

    let handle = unique_handle();
    let definition = json!({
        "face": { "layers": [] },
        "back": { "layers": [
            { "type": "contact", "fields": {
                "phone": "+15551234567",
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

    let (status, content_type, body) = app
        .request_raw("GET", &format!("/v1/cards/{id}/export/vcf"), Some(&token))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(content_type.starts_with("text/vcard"));
    assert!(body.starts_with("BEGIN:VCARD"));
    assert!(body.contains("EMAIL:owner@cardclaws.test"));
    assert!(body.contains("ORG:RedClaw"));
    assert!(body.trim_end().ends_with("END:VCARD"));
}
