//! Integration tests for analytics ingest + summary (PRD §13.5, §18).

mod common;

use axum::http::StatusCode;
use serde_json::json;

use common::{unique_handle, TestApp};

async fn create_and_publish(app: &TestApp, token: &str) -> (String, String) {
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
    let id = created["id"].as_str().unwrap().to_string();
    app.request(
        "POST",
        &format!("/v1/cards/{id}/publish"),
        Some(token),
        None,
    )
    .await;
    (id, handle)
}

#[tokio::test]
async fn profile_visit_recorded_on_public_lookup() {
    let app = require_app!();
    let token = app.register_and_token().await;
    let (id, handle) = create_and_publish(&app, &token).await;

    // Two public lookups => two profile visits.
    app.request("GET", &format!("/v1/cards/handle/{handle}"), None, None)
        .await;
    app.request("GET", &format!("/v1/cards/handle/{handle}"), None, None)
        .await;

    let (status, body) = app
        .request(
            "GET",
            &format!("/v1/cards/{id}/analytics"),
            Some(&token),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["totalVisits"], 2);
    assert_eq!(body["visits24h"], 2);
}

#[tokio::test]
async fn client_event_ingest_increments_summary() {
    let app = require_app!();
    let token = app.register_and_token().await;
    let (id, _) = create_and_publish(&app, &token).await;

    let (status, _) = app
        .request(
            "POST",
            "/v1/analytics/event",
            None,
            Some(json!({"card_id": id, "event_type": "contact_save"})),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let (_, body) = app
        .request(
            "GET",
            &format!("/v1/cards/{id}/analytics"),
            Some(&token),
            None,
        )
        .await;
    assert_eq!(body["contactSaves"], 1);
}

#[tokio::test]
async fn ingest_rejects_server_only_event_type() {
    let app = require_app!();
    let token = app.register_and_token().await;
    let (id, _) = create_and_publish(&app, &token).await;

    // profile_visit is server-originated; clients can't post it.
    let (status, body) = app
        .request(
            "POST",
            "/v1/analytics/event",
            None,
            Some(json!({"card_id": id, "event_type": "profile_visit"})),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "validation");
}

#[tokio::test]
async fn analytics_summary_requires_ownership() {
    let app = require_app!();
    let owner = app.register_and_token().await;
    let (id, _) = create_and_publish(&app, &owner).await;

    let intruder = app.register_and_token().await;
    let (status, _) = app
        .request(
            "GET",
            &format!("/v1/cards/{id}/analytics"),
            Some(&intruder),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
