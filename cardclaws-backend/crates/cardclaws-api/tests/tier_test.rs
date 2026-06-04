//! Tier-gate enforcement (PRD §19.1). Tier is read from the DB so a simulated
//! upgrade (set_tier) takes effect immediately.

mod common;

use axum::http::StatusCode;
use serde_json::{json, Value};

use common::unique_handle;

fn card_with_particle() -> Value {
    json!({
        "face": {
            "layers": [
                { "id": "p1", "type": "particle", "x": 0, "y": 0, "width": 1, "height": 1,
                  "opacity": 1, "zIndex": 1 }
            ],
            "background": { "type": "solid", "value": "#101014" }
        },
        "back": { "layers": [] }
    })
}

#[tokio::test]
async fn free_tier_cannot_use_pro_layers_but_pro_can() {
    let app = require_app!();
    let (token, user_id) = app.register_and_user().await;

    // Free tier: a particle layer is rejected with a tier-limit (402).
    let (status, body) = app
        .request(
            "POST",
            "/v1/cards",
            Some(&token),
            Some(json!({ "handle": unique_handle(), "definition": card_with_particle() })),
        )
        .await;
    assert_eq!(status, StatusCode::PAYMENT_REQUIRED);
    assert_eq!(body["code"], "tier_limit");

    // Upgrade to Pro → the same card is now accepted.
    app.set_tier(&user_id, "pro").await;
    let (status2, _) = app
        .request(
            "POST",
            "/v1/cards",
            Some(&token),
            Some(json!({ "handle": unique_handle(), "definition": card_with_particle() })),
        )
        .await;
    assert_eq!(status2, StatusCode::OK);
}

#[tokio::test]
async fn free_tier_plain_card_still_works() {
    let app = require_app!();
    let (token, _) = app.register_and_user().await;
    let plain = json!({
        "face": { "layers": [], "background": { "type": "solid", "value": "#101014" } },
        "back": { "layers": [] }
    });
    let (status, _) = app
        .request(
            "POST",
            "/v1/cards",
            Some(&token),
            Some(json!({ "handle": unique_handle(), "definition": plain })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn geo_analytics_gated_to_pro() {
    let app = require_app!();
    let (token, user_id) = app.register_and_user().await;
    let (_, created) = app
        .request(
            "POST",
            "/v1/cards",
            Some(&token),
            Some(json!({
                "handle": unique_handle(),
                "definition": { "face": { "layers": [], "background": { "type": "solid", "value": "#101014" } }, "back": { "layers": [] } }
            })),
        )
        .await;
    let id = created["id"].as_str().unwrap();

    // Free tier: geo is gated.
    let (free_status, body) = app
        .request(
            "GET",
            &format!("/v1/cards/{id}/analytics/geo"),
            Some(&token),
            None,
        )
        .await;
    assert_eq!(free_status, StatusCode::PAYMENT_REQUIRED);
    assert_eq!(body["code"], "tier_limit");

    // Pro tier: geo is allowed (empty list here, but 200).
    app.set_tier(&user_id, "pro").await;
    let (pro_status, _) = app
        .request(
            "GET",
            &format!("/v1/cards/{id}/analytics/geo"),
            Some(&token),
            None,
        )
        .await;
    assert_eq!(pro_status, StatusCode::OK);
}
