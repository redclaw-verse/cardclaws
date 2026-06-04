//! Integration tests for the public profile contact form (PRD §11.4, §13.6).

mod common;

use axum::http::StatusCode;
use serde_json::{json, Value};

use common::{unique_handle, TestApp};

async fn published_card(app: &TestApp, token: &str) -> String {
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
    handle
}

#[tokio::test]
async fn contact_form_emails_the_owner() {
    let app = require_app!();
    let token = app.register_and_token().await;
    let handle = published_card(&app, &token).await;

    let (status, _) = app
        .request(
            "POST",
            &format!("/v1/profile/{handle}/contact"),
            None,
            Some(json!({ "name": "Visitor", "email": "visitor@example.com", "message": "Loved your card!" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let sent = app.email.sent.lock().unwrap();
    let msg = sent.last().expect("an email was sent");
    assert!(msg.subject.contains("contact form"));
    assert!(msg.html.contains("Loved your card!"));
    assert!(msg.html.contains("visitor@example.com"));
}

#[tokio::test]
async fn contact_form_rejects_bad_input() {
    let app = require_app!();
    let token = app.register_and_token().await;
    let handle = published_card(&app, &token).await;

    let bad_email: Value = json!({ "name": "V", "email": "not-an-email", "message": "hi" });
    let (s1, b1) = app
        .request(
            "POST",
            &format!("/v1/profile/{handle}/contact"),
            None,
            Some(bad_email),
        )
        .await;
    assert_eq!(s1, StatusCode::BAD_REQUEST);
    assert_eq!(b1["code"], "validation");

    let empty_name = json!({ "name": "  ", "email": "a@b.com", "message": "hi" });
    let (s2, _) = app
        .request(
            "POST",
            &format!("/v1/profile/{handle}/contact"),
            None,
            Some(empty_name),
        )
        .await;
    assert_eq!(s2, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn contact_form_unknown_handle_is_404() {
    let app = require_app!();
    let (status, _) = app
        .request(
            "POST",
            "/v1/profile/nobody-here/contact",
            None,
            Some(json!({ "name": "V", "email": "a@b.com", "message": "hi" })),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
