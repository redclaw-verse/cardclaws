//! Integration tests for the auth surface (PRD §13.1, §15.2). Run against a real
//! Postgres via `TEST_DATABASE_URL`; skipped cleanly when that is unset.

mod common;

use axum::http::StatusCode;
use serde_json::json;

use common::unique_identity;

fn register_body(email: &str, handle: &str) -> serde_json::Value {
    json!({
        "email": email,
        "password": "correct horse battery",
        "handle": handle,
        "display_name": "Test User",
    })
}

#[tokio::test]
async fn register_success_returns_tokens() {
    let app = require_app!();
    let (email, handle) = unique_identity();
    let (status, body) = app
        .post("/v1/auth/register", register_body(&email, &handle))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["access_token"].as_str().is_some());
    assert!(body["refresh_token"].as_str().is_some());
    assert_eq!(body["user"]["handle"], handle);
    assert_eq!(body["user"]["tier"], "free");
}

#[tokio::test]
async fn register_duplicate_is_conflict() {
    let app = require_app!();
    let (email, handle) = unique_identity();
    let (s1, _) = app
        .post("/v1/auth/register", register_body(&email, &handle))
        .await;
    assert_eq!(s1, StatusCode::OK);

    let (s2, body) = app
        .post("/v1/auth/register", register_body(&email, &handle))
        .await;
    assert_eq!(s2, StatusCode::CONFLICT);
    assert_eq!(body["code"], "conflict");
}

#[tokio::test]
async fn register_rejects_invalid_handle() {
    let app = require_app!();
    let (email, _) = unique_identity();
    let (status, body) = app
        .post("/v1/auth/register", register_body(&email, "ab"))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "validation");
}

#[tokio::test]
async fn login_success_then_wrong_password() {
    let app = require_app!();
    let (email, handle) = unique_identity();
    app.post("/v1/auth/register", register_body(&email, &handle))
        .await;

    let (ok_status, body) = app
        .post(
            "/v1/auth/login",
            json!({"email": email, "password": "correct horse battery"}),
        )
        .await;
    assert_eq!(ok_status, StatusCode::OK);
    assert!(body["access_token"].as_str().is_some());

    let (bad_status, _) = app
        .post(
            "/v1/auth/login",
            json!({"email": email, "password": "wrong"}),
        )
        .await;
    assert_eq!(bad_status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_locks_out_after_ten_failures() {
    let app = require_app!();
    let (email, handle) = unique_identity();
    app.post("/v1/auth/register", register_body(&email, &handle))
        .await;

    // 10 wrong attempts are unauthorized; the 11th trips the lockout.
    for _ in 0..10 {
        let (status, _) = app
            .post(
                "/v1/auth/login",
                json!({"email": email, "password": "nope"}),
            )
            .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
    let (status, body) = app
        .post(
            "/v1/auth/login",
            json!({"email": email, "password": "nope"}),
        )
        .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["code"], "rate_limited");
}

#[tokio::test]
async fn refresh_rotates_and_invalidates_old_token() {
    let app = require_app!();
    let (email, handle) = unique_identity();
    let (_, reg) = app
        .post("/v1/auth/register", register_body(&email, &handle))
        .await;
    let refresh = reg["refresh_token"].as_str().unwrap().to_string();

    let (status, body) = app
        .post("/v1/auth/refresh", json!({"refresh_token": refresh}))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["access_token"].as_str().is_some());

    // The original refresh token must no longer work (rotation).
    let (reuse_status, _) = app
        .post("/v1/auth/refresh", json!({"refresh_token": refresh}))
        .await;
    assert_eq!(reuse_status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn logout_invalidates_refresh_token() {
    let app = require_app!();
    let (email, handle) = unique_identity();
    let (_, reg) = app
        .post("/v1/auth/register", register_body(&email, &handle))
        .await;
    let refresh = reg["refresh_token"].as_str().unwrap().to_string();

    let (status, _) = app
        .post("/v1/auth/logout", json!({"refresh_token": refresh}))
        .await;
    assert_eq!(status, StatusCode::OK);

    let (after, _) = app
        .post("/v1/auth/refresh", json!({"refresh_token": refresh}))
        .await;
    assert_eq!(after, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn magic_link_sends_email_and_verifies() {
    let app = require_app!();
    let (email, handle) = unique_identity();
    app.post("/v1/auth/register", register_body(&email, &handle))
        .await;

    let (status, _) = app
        .post("/v1/auth/magic-link/request", json!({"email": email}))
        .await;
    assert_eq!(status, StatusCode::OK);

    let token = {
        let sent = app.email.sent.lock().unwrap();
        let msg = sent.iter().find(|m| m.to == email).expect("email sent");
        extract_token(&msg.html)
    };

    let (verify_status, body) = app
        .post("/v1/auth/magic-link/verify", json!({"token": token}))
        .await;
    assert_eq!(verify_status, StatusCode::OK);
    assert_eq!(body["user"]["handle"], handle);
}

#[tokio::test]
async fn magic_link_unknown_email_sends_nothing() {
    let app = require_app!();
    let (email, _) = unique_identity();
    let (status, _) = app
        .post("/v1/auth/magic-link/request", json!({"email": email}))
        .await;
    // Always 200 to avoid enumeration, but no email is dispatched.
    assert_eq!(status, StatusCode::OK);
    let sent = app.email.sent.lock().unwrap();
    assert!(sent.iter().all(|m| m.to != email));
}

/// Pull the `token=...` value out of the magic-link email HTML.
fn extract_token(html: &str) -> String {
    let start = html.find("token=").expect("token in link") + "token=".len();
    let rest = &html[start..];
    let end = rest.find('"').unwrap_or(rest.len());
    rest[..end].to_string()
}
