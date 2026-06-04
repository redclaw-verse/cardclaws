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
