//! Billing webhook → tier update (PRD §19.2).

mod common;

use axum::http::StatusCode;
use serde_json::json;

const SECRET: &str = "test-webhook-secret";

fn rc_event(event_type: &str, user_id: &str, ents: &[&str]) -> serde_json::Value {
    json!({
        "event": {
            "type": event_type,
            "app_user_id": user_id,
            "entitlement_ids": ents,
        }
    })
}

#[tokio::test]
async fn webhook_upgrades_then_cancels_tier() {
    let app = require_app!();
    let (_, user_id) = app.register_and_user().await;
    assert_eq!(app.get_tier(&user_id).await, "free");

    // Purchase → pro.
    let (status, _) = app
        .request_with_headers(
            "POST",
            "/v1/webhooks/revenuecat",
            None,
            Some(rc_event("INITIAL_PURCHASE", &user_id, &["pro"])),
            &[("authorization", SECRET)],
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(app.get_tier(&user_id).await, "pro");

    // Cancellation → back to free.
    app.request_with_headers(
        "POST",
        "/v1/webhooks/revenuecat",
        None,
        Some(rc_event("CANCELLATION", &user_id, &[])),
        &[("authorization", SECRET)],
    )
    .await;
    assert_eq!(app.get_tier(&user_id).await, "free");
}

#[tokio::test]
async fn webhook_rejects_bad_secret_and_does_not_change_tier() {
    let app = require_app!();
    let (_, user_id) = app.register_and_user().await;

    let (status, _) = app
        .request_with_headers(
            "POST",
            "/v1/webhooks/revenuecat",
            None,
            Some(rc_event("INITIAL_PURCHASE", &user_id, &["pro"])),
            &[("authorization", "wrong-secret")],
        )
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(app.get_tier(&user_id).await, "free");
}

#[tokio::test]
async fn webhook_unknown_user_is_ok_noop() {
    let app = require_app!();
    let (status, _) = app
        .request_with_headers(
            "POST",
            "/v1/webhooks/revenuecat",
            None,
            Some(rc_event(
                "INITIAL_PURCHASE",
                "00000000-0000-0000-0000-000000000000",
                &["pro"],
            )),
            &[("authorization", SECRET)],
        )
        .await;
    // 2xx so RevenueCat doesn't retry forever.
    assert_eq!(status, StatusCode::OK);
}
