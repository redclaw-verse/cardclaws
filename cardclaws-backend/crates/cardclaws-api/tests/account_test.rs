//! GDPR data export + account deletion (PRD §18.3).

mod common;

use axum::http::StatusCode;
use serde_json::json;

use common::{unique_handle, TestApp};

async fn create_card(app: &TestApp, token: &str) -> (String, String) {
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
            Some(json!({ "handle": handle, "definition": def })),
        )
        .await;
    (created["id"].as_str().unwrap().to_string(), handle)
}

async fn count(app: &TestApp, sql: &str, id: &str) -> i64 {
    let uuid = uuid::Uuid::parse_str(id).unwrap();
    let (n,): (i64,) = sqlx::query_as(sql)
        .bind(uuid)
        .fetch_one(&app.db)
        .await
        .unwrap();
    n
}

#[tokio::test]
async fn export_includes_user_and_cards() {
    let app = require_app!();
    let (token, user_id) = app.register_and_user().await;
    let (_, handle) = create_card(&app, &token).await;

    let (status, body) = app
        .request("GET", "/v1/account/export", Some(&token), None)
        .await;
    assert_eq!(status, StatusCode::OK);

    assert_eq!(body["user"]["id"], user_id);
    // The password hash must never be exported.
    assert!(body["user"].get("passwordHash").is_none());
    assert!(body["user"].get("password_hash").is_none());

    let cards = body["cards"].as_array().unwrap();
    assert_eq!(cards.len(), 1);
    assert_eq!(cards[0]["handle"], handle);

    // Other sections are present as arrays.
    for key in [
        "shareLinks",
        "analyticsEvents",
        "walletRegistrations",
        "teamMemberships",
    ] {
        assert!(body[key].is_array(), "{key} should be an array");
    }
}

#[tokio::test]
async fn export_requires_auth() {
    let app = require_app!();
    let (status, _) = app.request("GET", "/v1/account/export", None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn delete_account_cascades() {
    let app = require_app!();
    let (token, user_id) = app.register_and_user().await;
    let (card_id, _) = create_card(&app, &token).await;
    // Give the card a share link + event so cascade coverage is meaningful.
    app.request(
        "POST",
        &format!("/v1/cards/{card_id}/share"),
        Some(&token),
        Some(json!({"modality": "qr"})),
    )
    .await;

    let (status, _) = app
        .request("DELETE", "/v1/account", Some(&token), None)
        .await;
    assert_eq!(status, StatusCode::OK);

    // User and all owned rows are gone.
    assert_eq!(
        count(&app, "SELECT COUNT(*) FROM users WHERE id = $1", &user_id).await,
        0
    );
    assert_eq!(
        count(
            &app,
            "SELECT COUNT(*) FROM cards WHERE owner_id = $1",
            &user_id
        )
        .await,
        0
    );
    assert_eq!(
        count(
            &app,
            "SELECT COUNT(*) FROM share_links WHERE card_id = $1",
            &card_id
        )
        .await,
        0
    );
}
