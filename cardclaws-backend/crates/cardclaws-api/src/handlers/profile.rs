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
