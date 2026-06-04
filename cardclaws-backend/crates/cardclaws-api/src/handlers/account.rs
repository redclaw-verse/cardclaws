//! Account self-service handlers (PRD §18.3): data export + deletion.

use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

use crate::error::ApiResult;
use crate::middleware::auth::AuthUser;
use crate::services::account_service;
use crate::state::AppState;

pub async fn export_data(State(state): State<AppState>, user: AuthUser) -> ApiResult<Json<Value>> {
    Ok(Json(
        account_service::export_data(&state, user.user_id).await?,
    ))
}

pub async fn delete_account(
    State(state): State<AppState>,
    user: AuthUser,
) -> ApiResult<Json<Value>> {
    account_service::delete_account(&state, user.user_id).await?;
    Ok(Json(json!({ "status": "deleted" })))
}
