//! Integration tests for the card surface (PRD §13.2, §15.2).

mod common;

use axum::http::StatusCode;
use serde_json::{json, Value};

use common::{unique_handle, TestApp};

fn definition() -> Value {
    json!({
        "face": { "layers": [], "background": { "type": "solid", "value": "#101014" } },
        "back": { "layers": [], "background": { "type": "solid", "value": "#101014" } }
    })
}

/// Create a draft card; returns its id and handle.
async fn create_card(app: &TestApp, token: &str) -> (String, String) {
    let handle = unique_handle();
    let (status, body) = app
        .request(
            "POST",
            "/v1/cards",
            Some(token),
            Some(json!({ "handle": handle, "definition": definition() })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "create failed: {body}");
    (body["id"].as_str().unwrap().to_string(), handle)
}

#[tokio::test]
async fn create_card_success() {
    let app = require_app!();
    let token = app.register_and_token().await;
    let (id, handle) = create_card(&app, &token).await;
    assert!(!id.is_empty());

    let (status, body) = app
        .request("GET", &format!("/v1/cards/{id}"), Some(&token), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["handle"], handle);
    assert_eq!(body["status"], "draft");
    assert_eq!(body["version"], 1);
}

#[tokio::test]
async fn create_card_unauthorized() {
    let app = require_app!();
    let (status, _) = app
        .request(
            "POST",
            "/v1/cards",
            None,
            Some(json!({ "handle": unique_handle(), "definition": definition() })),
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn update_card_increments_version() {
    let app = require_app!();
    let token = app.register_and_token().await;
    let (id, _) = create_card(&app, &token).await;

    let (s1, b1) = app
        .request(
            "PUT",
            &format!("/v1/cards/{id}"),
            Some(&token),
            Some(json!({ "definition": definition() })),
        )
        .await;
    assert_eq!(s1, StatusCode::OK);
    assert_eq!(b1["version"], 2);

    let (_, b2) = app
        .request(
            "PUT",
            &format!("/v1/cards/{id}"),
            Some(&token),
            Some(json!({ "definition": definition() })),
        )
        .await;
    assert_eq!(b2["version"], 3);
}

#[tokio::test]
async fn patch_merges_definition_keys() {
    let app = require_app!();
    let token = app.register_and_token().await;
    let (id, _) = create_card(&app, &token).await;

    let (status, body) = app
        .request(
            "PATCH",
            &format!("/v1/cards/{id}"),
            Some(&token),
            Some(json!({ "definition": { "settings": { "flipDurationMs": 250 } } })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    // Original face key survives; new settings key is merged in.
    assert!(body["definition"]["face"].is_object());
    assert_eq!(body["definition"]["settings"]["flipDurationMs"], 250);
}

#[tokio::test]
async fn delete_card_archives_not_destroys() {
    let app = require_app!();
    let token = app.register_and_token().await;
    let (id, _) = create_card(&app, &token).await;

    let (status, body) = app
        .request("DELETE", &format!("/v1/cards/{id}"), Some(&token), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "archived");

    // Row still exists and is fetchable by its owner.
    let (get_status, _) = app
        .request("GET", &format!("/v1/cards/{id}"), Some(&token), None)
        .await;
    assert_eq!(get_status, StatusCode::OK);
}

#[tokio::test]
async fn publish_respects_free_tier_limit() {
    let app = require_app!();
    let token = app.register_and_token().await; // free tier => 1 active card
    let (id_a, _) = create_card(&app, &token).await;
    let (id_b, _) = create_card(&app, &token).await;

    let (s_a, _) = app
        .request(
            "POST",
            &format!("/v1/cards/{id_a}/publish"),
            Some(&token),
            None,
        )
        .await;
    assert_eq!(s_a, StatusCode::OK);

    let (s_b, body) = app
        .request(
            "POST",
            &format!("/v1/cards/{id_b}/publish"),
            Some(&token),
            None,
        )
        .await;
    assert_eq!(s_b, StatusCode::PAYMENT_REQUIRED);
    assert_eq!(body["code"], "tier_limit");
}

#[tokio::test]
async fn get_card_by_handle_public_then_404_when_archived() {
    let app = require_app!();
    let token = app.register_and_token().await;
    let (id, handle) = create_card(&app, &token).await;

    // Draft is not publicly resolvable.
    let (draft_status, _) = app
        .request("GET", &format!("/v1/cards/handle/{handle}"), None, None)
        .await;
    assert_eq!(draft_status, StatusCode::NOT_FOUND);

    // Publish -> public access works without auth.
    app.request(
        "POST",
        &format!("/v1/cards/{id}/publish"),
        Some(&token),
        None,
    )
    .await;
    let (active_status, body) = app
        .request("GET", &format!("/v1/cards/handle/{handle}"), None, None)
        .await;
    assert_eq!(active_status, StatusCode::OK);
    assert_eq!(body["handle"], handle);

    // Archive -> no longer publicly resolvable.
    app.request("DELETE", &format!("/v1/cards/{id}"), Some(&token), None)
        .await;
    let (archived_status, _) = app
        .request("GET", &format!("/v1/cards/handle/{handle}"), None, None)
        .await;
    assert_eq!(archived_status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn cannot_access_another_users_card() {
    let app = require_app!();
    let owner = app.register_and_token().await;
    let (id, _) = create_card(&app, &owner).await;

    let intruder = app.register_and_token().await;
    let (status, _) = app
        .request("GET", &format!("/v1/cards/{id}"), Some(&intruder), None)
        .await;
    // NotFound (not Forbidden) so existence isn't leaked.
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn duplicate_creates_independent_draft() {
    let app = require_app!();
    let token = app.register_and_token().await;
    let (id, handle) = create_card(&app, &token).await;

    let (status, body) = app
        .request(
            "POST",
            &format!("/v1/cards/{id}/duplicate"),
            Some(&token),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_ne!(body["id"].as_str().unwrap(), id);
    assert_ne!(body["handle"].as_str().unwrap(), handle);
    assert_eq!(body["status"], "draft");
}

#[tokio::test]
async fn welcome_generates_and_appears_on_public_profile() {
    let app = require_app!();
    let token = app.register_and_token().await;
    let (id, handle) = create_card(&app, &token).await;

    // Generate the AI welcome (FakeAiClient returns a 1x1 png).
    let (status, body) = app
        .request(
            "POST",
            &format!("/v1/cards/{id}/welcome"),
            Some(&token),
            Some(json!({ "prompt": "a calm forest at dawn", "message": "Great to meet you, I'm Omar" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "set welcome failed: {body}");
    assert_eq!(body["welcome"]["kind"], "image");
    assert!(body["welcome"]["imageDataUrl"]
        .as_str()
        .unwrap()
        .starts_with("data:image/png;base64,"));

    // Publish -> the public profile exposes the welcome.
    app.request(
        "POST",
        &format!("/v1/cards/{id}/publish"),
        Some(&token),
        None,
    )
    .await;
    let (status, profile) = app
        .request("GET", &format!("/v1/cards/handle/{handle}"), None, None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(profile["welcome"]["message"], "Great to meet you, I'm Omar");
    assert!(profile["welcome"]["imageDataUrl"]
        .as_str()
        .unwrap()
        .starts_with("data:image/png"));
}

#[tokio::test]
async fn welcome_requires_ownership() {
    let app = require_app!();
    let owner = app.register_and_token().await;
    let (id, _handle) = create_card(&app, &owner).await;
    let other = app.register_and_token().await;
    let (status, _) = app
        .request(
            "POST",
            &format!("/v1/cards/{id}/welcome"),
            Some(&other),
            Some(json!({ "prompt": "x" })),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn we_met_capture_and_owner_list() {
    let app = require_app!();
    let token = app.register_and_token().await;
    let (id, handle) = create_card(&app, &token).await;
    app.request(
        "POST",
        &format!("/v1/cards/{id}/publish"),
        Some(&token),
        None,
    )
    .await;

    // A scanner shares back (public, unauthenticated).
    let (s, _) = app
        .request(
            "POST",
            &format!("/v1/profile/{handle}/connect"),
            None,
            Some(json!({ "name": "Dana Scanner", "email": "dana@example.com", "note": "met at SXSW" })),
        )
        .await;
    assert_eq!(s, StatusCode::OK);

    // The owner sees it in their connections.
    let (s, body) = app
        .request(
            "GET",
            &format!("/v1/cards/{id}/connections"),
            Some(&token),
            None,
        )
        .await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body[0]["name"], "Dana Scanner");
    assert_eq!(body[0]["note"], "met at SXSW");

    // Name is required.
    let (s, _) = app
        .request(
            "POST",
            &format!("/v1/profile/{handle}/connect"),
            None,
            Some(json!({ "name": "  " })),
        )
        .await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn connections_list_is_owner_only() {
    let app = require_app!();
    let token = app.register_and_token().await;
    let (id, _handle) = create_card(&app, &token).await;
    let other = app.register_and_token().await;
    let (status, _) = app
        .request(
            "GET",
            &format!("/v1/cards/{id}/connections"),
            Some(&other),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
