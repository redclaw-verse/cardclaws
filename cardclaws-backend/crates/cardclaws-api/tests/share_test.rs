//! Integration tests for the share system (PRD §13.4, §17).

mod common;

use axum::http::StatusCode;
use serde_json::{json, Value};

use common::{unique_handle, TestApp};

async fn create_card(app: &TestApp, token: &str) -> String {
    let handle = unique_handle();
    let def = json!({
        "face": { "layers": [], "background": { "type": "solid", "value": "#101014" } },
        "back": { "layers": [] }
    });
    let (_, created) = app
        .request(
            "POST",
            "/v1/cards",
            Some(token),
            Some(json!({"handle": handle, "definition": def})),
        )
        .await;
    created["id"].as_str().unwrap().to_string()
}

async fn create_share(
    app: &TestApp,
    token: &str,
    card_id: &str,
    modality: &str,
) -> (StatusCode, Value) {
    app.request(
        "POST",
        &format!("/v1/cards/{card_id}/share"),
        Some(token),
        Some(json!({ "modality": modality })),
    )
    .await
}

#[tokio::test]
async fn create_share_link_returns_token_and_url() {
    let app = require_app!();
    let token = app.register_and_token().await;
    let card_id = create_card(&app, &token).await;

    let (status, body) = create_share(&app, &token, &card_id, "qr").await;
    assert_eq!(status, StatusCode::OK);
    let tok = body["token"].as_str().unwrap();
    assert_eq!(tok.len(), 12);
    assert!(body["url"]
        .as_str()
        .unwrap()
        .ends_with(&format!("/s/{tok}")));
}

#[tokio::test]
async fn create_share_requires_auth_and_valid_modality() {
    let app = require_app!();
    let token = app.register_and_token().await;
    let card_id = create_card(&app, &token).await;

    let (unauth, _) = app
        .request(
            "POST",
            &format!("/v1/cards/{card_id}/share"),
            None,
            Some(json!({"modality": "qr"})),
        )
        .await;
    assert_eq!(unauth, StatusCode::UNAUTHORIZED);

    let (bad, body) = create_share(&app, &token, &card_id, "telepathy").await;
    assert_eq!(bad, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "validation");
}

#[tokio::test]
async fn resolve_redirects_and_attributes_qr_scan() {
    let app = require_app!();
    let token = app.register_and_token().await;
    let card_id = create_card(&app, &token).await;
    let (_, share) = create_share(&app, &token, &card_id, "qr").await;
    let tok = share["token"].as_str().unwrap();

    let (status, location) = app.get_redirect(&format!("/v1/s/{tok}")).await;
    assert_eq!(status, StatusCode::FOUND);
    assert!(location.unwrap().starts_with("https://cardclaws.test/"));

    // The resolution recorded a qr_scan attributed to the card.
    let (_, summary) = app
        .request(
            "GET",
            &format!("/v1/cards/{card_id}/analytics"),
            Some(&token),
            None,
        )
        .await;
    assert_eq!(summary["qrScans"], 1);
}

#[tokio::test]
async fn feed_shows_attributed_event_after_resolve() {
    let app = require_app!();
    let token = app.register_and_token().await;
    let card_id = create_card(&app, &token).await;
    let (_, share) = create_share(&app, &token, &card_id, "qr").await;
    let tok = share["token"].as_str().unwrap().to_string();

    app.get_redirect(&format!("/v1/s/{tok}")).await;

    let (status, body) = app
        .request(
            "GET",
            &format!("/v1/cards/{card_id}/analytics/feed"),
            Some(&token),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let events = body.as_array().unwrap();
    assert!(events
        .iter()
        .any(|e| e["eventType"] == "qr_scan" && e["shareToken"] == tok));
}

#[tokio::test]
async fn resolve_unknown_token_is_404() {
    let app = require_app!();
    let (status, _) = app.get_redirect("/v1/s/doesnotexist1").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_share_links_returns_created_links() {
    let app = require_app!();
    let token = app.register_and_token().await;
    let card_id = create_card(&app, &token).await;
    create_share(&app, &token, &card_id, "qr").await;
    create_share(&app, &token, &card_id, "nfc").await;

    let (status, body) = app
        .request(
            "GET",
            &format!("/v1/cards/{card_id}/share-links"),
            Some(&token),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    let modalities: Vec<&str> = arr
        .iter()
        .map(|l| l["modality"].as_str().unwrap())
        .collect();
    assert!(modalities.contains(&"qr") && modalities.contains(&"nfc"));
}
