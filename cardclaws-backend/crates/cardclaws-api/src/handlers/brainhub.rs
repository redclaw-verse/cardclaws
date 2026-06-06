//! ClawBrainHub handlers: list agent brains and pull one (normalized for a
//! trading-card-style agent card). Reads proxy the public registry; lightly
//! rate-limited per IP to avoid hammering the upstream.

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::Json;
use cardclaws_types::AppError;
use serde::Deserialize;

use crate::brainhub::{AgentBrain, BrainSummary};
use crate::error::ApiResult;
use crate::handlers::analytics::client_ip;
use crate::middleware::rate_limit;
use crate::state::AppState;

fn rl_key(op: &str, headers: &HeaderMap) -> String {
    let ip = client_ip(headers).unwrap_or_else(|| "unknown".into());
    format!("brainhub:{op}:{ip}")
}

pub async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<BrainSummary>>> {
    rate_limit::check(state.cache.as_ref(), &rl_key("list", &headers), 60, 3600).await?;
    let brains = state
        .brainhub
        .list_brains()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(brains))
}

#[derive(Deserialize)]
pub struct PullQuery {
    pub owner: String,
    pub name: String,
    pub version: String,
}

pub async fn pull(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<PullQuery>,
) -> ApiResult<Json<AgentBrain>> {
    rate_limit::check(state.cache.as_ref(), &rl_key("pull", &headers), 60, 3600).await?;
    let brain = state
        .brainhub
        .pull_brain(&q.owner, &q.name, &q.version)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(brain))
}
