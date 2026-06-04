//! Analytics handlers (PRD §13.5): client event ingest + owner summary.

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::Json;
use cardclaws_db::models::analytics::{AnalyticsSummary, FeedEvent};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::ApiResult;
use crate::middleware::auth::AuthUser;
use crate::services::analytics_service;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct IngestRequest {
    pub card_id: Uuid,
    pub event_type: String,
    #[serde(default)]
    pub share_token: Option<String>,
}

/// Public client-side event ingest (PRD §18.1 secondary path).
pub async fn ingest_event(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<IngestRequest>,
) -> ApiResult<Json<Value>> {
    let ip = client_ip(&headers);
    let ua = header_str(&headers, "user-agent");
    analytics_service::ingest_client_event(
        &state,
        req.card_id,
        &req.event_type,
        req.share_token.as_deref(),
        ip.as_deref(),
        ua.as_deref(),
    )
    .await?;
    Ok(Json(json!({ "status": "recorded" })))
}

/// Owner-only metrics summary for a card.
pub async fn summary(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<AnalyticsSummary>> {
    Ok(Json(
        analytics_service::summary(&state, id, user.user_id).await?,
    ))
}

/// Owner-only chronological event feed.
pub async fn feed(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<FeedEvent>>> {
    Ok(Json(
        analytics_service::feed(&state, id, user.user_id).await?,
    ))
}

/// Best-effort client IP from the proxy headers Cloudflare/Hetzner set. The
/// first IP in `x-forwarded-for` is the original client.
pub fn client_ip(headers: &HeaderMap) -> Option<String> {
    header_str(headers, "cf-connecting-ip")
        .or_else(|| {
            header_str(headers, "x-forwarded-for")
                .map(|xff| xff.split(',').next().unwrap_or("").trim().to_string())
        })
        .or_else(|| header_str(headers, "x-real-ip"))
        .filter(|s| !s.is_empty())
}

pub fn header_str(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}
