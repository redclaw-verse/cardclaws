//! Auth HTTP handlers (PRD §13.1). Thin: validate shape via deserialization,
//! delegate to `auth_service`, return typed JSON.

use axum::extract::State;
use axum::Json;
use cardclaws_types::auth::*;

use crate::error::ApiResult;
use crate::services::auth_service;
use crate::state::AppState;

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> ApiResult<Json<TokenPair>> {
    Ok(Json(auth_service::register(&state, req).await?))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> ApiResult<Json<TokenPair>> {
    Ok(Json(auth_service::login(&state, req, None).await?))
}

pub async fn magic_link_request(
    State(state): State<AppState>,
    Json(req): Json<MagicLinkRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    auth_service::magic_link_request(&state, req).await?;
    Ok(Json(serde_json::json!({ "status": "sent" })))
}

pub async fn magic_link_verify(
    State(state): State<AppState>,
    Json(req): Json<MagicLinkVerifyRequest>,
) -> ApiResult<Json<TokenPair>> {
    Ok(Json(auth_service::magic_link_verify(&state, req).await?))
}

pub async fn oauth_apple(
    State(state): State<AppState>,
    Json(req): Json<AppleOAuthRequest>,
) -> ApiResult<Json<TokenPair>> {
    Ok(Json(auth_service::oauth_apple(&state, req).await?))
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> ApiResult<Json<TokenPair>> {
    Ok(Json(auth_service::refresh(&state, req).await?))
}

pub async fn logout(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    auth_service::logout(&state, &req.refresh_token).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}
