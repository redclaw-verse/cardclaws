//! Public profile handlers (PRD §13.6).

use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::ApiResult;
use crate::services::profile_service;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ContactFormRequest {
    pub name: String,
    pub email: String,
    pub message: String,
}

pub async fn submit_contact(
    State(state): State<AppState>,
    Path(handle): Path<String>,
    Json(req): Json<ContactFormRequest>,
) -> ApiResult<Json<Value>> {
    profile_service::submit_contact(&state, &handle, &req.name, &req.email, &req.message).await?;
    Ok(Json(json!({ "status": "sent" })))
}

#[derive(Deserialize)]
pub struct ConnectRequest {
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

/// "We Met" — a scanner shares back; capture the connection (with coarse geo).
pub async fn submit_connection(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(handle): Path<String>,
    Json(req): Json<ConnectRequest>,
) -> ApiResult<Json<Value>> {
    let ip = crate::handlers::analytics::client_ip(&headers);
    profile_service::submit_connection(
        &state,
        &handle,
        &req.name,
        req.email.as_deref(),
        req.note.as_deref(),
        ip.as_deref(),
    )
    .await?;
    Ok(Json(json!({ "status": "connected" })))
}
