//! AI card-image generation handlers (PRD §21 Phase 5). Two steps the mobile
//! wizard calls: refine the brief, then generate the image (nano-banana). Heavily
//! rate-limited because each call costs money; production also gates these behind
//! auth + the Pro tier.

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use base64::Engine;
use cardclaws_types::AppError;
use serde::{Deserialize, Serialize};

use crate::ai::{AiError, SceneBrief, VideoStatus};
use crate::error::ApiResult;
use crate::handlers::analytics::client_ip;
use crate::middleware::rate_limit;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct RefineRequest {
    pub scene: String,
    #[serde(default)]
    pub style: Option<String>,
    #[serde(default)]
    pub mood: Option<String>,
}

#[derive(Serialize)]
pub struct RefineResponse {
    pub prompt: String,
}

#[derive(Deserialize)]
pub struct ImageRequest {
    pub prompt: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageResponse {
    pub mime_type: String,
    /// Base64-encoded image; the client renders it as a `data:` URI.
    pub image_base64: String,
}

fn map_ai_err(e: AiError) -> AppError {
    match e {
        AiError::NotConfigured => AppError::Internal("AI is not configured".into()),
        other => AppError::Internal(other.to_string()),
    }
}

pub async fn refine(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RefineRequest>,
) -> ApiResult<Json<RefineResponse>> {
    rate_limit::check(state.cache.as_ref(), &rl_key("refine", &headers), 60, 3600).await?;
    if req.scene.trim().is_empty() {
        return Err(AppError::Validation("scene is required".into()).into());
    }
    let brief = SceneBrief {
        scene: req.scene,
        style: req.style,
        mood: req.mood,
    };
    let prompt = state.ai.refine_prompt(&brief).await.map_err(map_ai_err)?;
    Ok(Json(RefineResponse { prompt }))
}

pub async fn image(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ImageRequest>,
) -> ApiResult<Json<ImageResponse>> {
    // Image generation is the expensive call — tighter limit.
    rate_limit::check(state.cache.as_ref(), &rl_key("image", &headers), 20, 3600).await?;
    if req.prompt.trim().is_empty() {
        return Err(AppError::Validation("prompt is required".into()).into());
    }
    let img = state
        .ai
        .generate_image(&req.prompt)
        .await
        .map_err(map_ai_err)?;
    Ok(Json(ImageResponse {
        mime_type: img.mime_type,
        image_base64: img.base64,
    }))
}

// ---- Video (Veo, async) ---------------------------------------------------

#[derive(Deserialize)]
pub struct VideoRequest {
    pub prompt: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartVideoResponse {
    pub operation_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoStatusRequest {
    pub operation_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoStatusResponse {
    /// "pending" | "done" | "failed".
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_base64: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Submit a Veo job. Very tightly rate-limited — each clip is costly.
pub async fn start_video(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<VideoRequest>,
) -> ApiResult<Json<StartVideoResponse>> {
    rate_limit::check(state.cache.as_ref(), &rl_key("video", &headers), 5, 3600).await?;
    if req.prompt.trim().is_empty() {
        return Err(AppError::Validation("prompt is required".into()).into());
    }
    let operation_id = state
        .ai
        .start_video(&req.prompt)
        .await
        .map_err(map_ai_err)?;
    Ok(Json(StartVideoResponse { operation_id }))
}

/// Poll a Veo operation. Not rate-limited (the client polls repeatedly); the
/// backend proxies the download so the API key never reaches the client.
pub async fn video_status(
    State(state): State<AppState>,
    Json(req): Json<VideoStatusRequest>,
) -> ApiResult<Json<VideoStatusResponse>> {
    let resp = match state
        .ai
        .poll_video(&req.operation_id)
        .await
        .map_err(map_ai_err)?
    {
        VideoStatus::Pending => VideoStatusResponse {
            status: "pending".into(),
            mime_type: None,
            video_base64: None,
            error: None,
        },
        VideoStatus::Failed(e) => VideoStatusResponse {
            status: "failed".into(),
            mime_type: None,
            video_base64: None,
            error: Some(e),
        },
        VideoStatus::Done(bytes) => VideoStatusResponse {
            status: "done".into(),
            mime_type: Some("video/mp4".into()),
            video_base64: Some(base64::engine::general_purpose::STANDARD.encode(bytes)),
            error: None,
        },
    };
    Ok(Json(resp))
}

fn rl_key(op: &str, headers: &HeaderMap) -> String {
    let ip = client_ip(headers).unwrap_or_else(|| "unknown".into());
    format!("ai:{op}:{ip}")
}
