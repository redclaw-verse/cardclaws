//! AI generator endpoints (PRD §21 Phase 5). Uses the fake AI client; a live
//! Gemini/nano-banana run is exercised manually with a real key.

mod common;

use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn refine_returns_a_prompt() {
    let app = require_app!();
    let (status, body) = app
        .request(
            "POST",
            "/v1/ai/refine",
            None,
            Some(json!({ "scene": "neon Tokyo street", "style": "Cinematic", "mood": "Bold" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let prompt = body["prompt"].as_str().unwrap();
    assert!(prompt.contains("neon Tokyo street"));
}

#[tokio::test]
async fn refine_includes_persona() {
    let app = require_app!();
    let (status, body) = app
        .request(
            "POST",
            "/v1/ai/refine",
            None,
            Some(json!({
                "scene": "a calm forest at dawn",
                "persona": { "role": "Founder", "vibe": "Cinematic, Luxe", "colors": ["#ff3b30"], "goal": "networking" }
            })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    // FakeAiClient echoes the persona role into the refined prompt.
    assert!(body["prompt"].as_str().unwrap().contains("Founder"));
}

#[tokio::test]
async fn refine_requires_a_scene() {
    let app = require_app!();
    let (status, body) = app
        .request(
            "POST",
            "/v1/ai/refine",
            None,
            Some(json!({ "scene": "   " })),
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "validation");
}

#[tokio::test]
async fn image_returns_base64_png() {
    let app = require_app!();
    let (status, body) = app
        .request(
            "POST",
            "/v1/ai/image",
            None,
            Some(json!({ "prompt": "a cinematic neon street" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["mimeType"], "image/png");
    assert!(!body["imageBase64"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn video_submit_then_poll_returns_clip() {
    let app = require_app!();
    let (status, body) = app
        .request(
            "POST",
            "/v1/ai/video",
            None,
            Some(json!({ "prompt": "a calm forest at dawn" })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let op = body["operationId"].as_str().unwrap().to_string();
    assert!(!op.is_empty());

    let (status, body) = app
        .request(
            "POST",
            "/v1/ai/video/status",
            None,
            Some(json!({ "operationId": op })),
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "done");
    assert_eq!(body["mimeType"], "video/mp4");
    assert!(!body["videoBase64"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn video_requires_a_prompt() {
    let app = require_app!();
    let (status, body) = app
        .request("POST", "/v1/ai/video", None, Some(json!({ "prompt": "" })))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "validation");
}
