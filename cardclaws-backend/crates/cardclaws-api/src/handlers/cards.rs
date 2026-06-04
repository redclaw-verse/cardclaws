//! Card HTTP handlers (PRD §13.2). Thin: extract + delegate to `card_service`.

use axum::extract::{Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::Json;
use cardclaws_db::models::card::CardRow;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::ApiResult;
use crate::middleware::auth::AuthUser;
use crate::services::card_service;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct CreateCardRequest {
    pub handle: String,
    pub definition: serde_json::Value,
}

#[derive(Deserialize)]
pub struct DefinitionBody {
    pub definition: serde_json::Value,
}

pub async fn list_cards(
    State(state): State<AppState>,
    user: AuthUser,
) -> ApiResult<Json<Vec<CardRow>>> {
    Ok(Json(card_service::list(&state, user.user_id).await?))
}

pub async fn create_card(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateCardRequest>,
) -> ApiResult<Json<CardRow>> {
    let card = card_service::create(&state, user.user_id, &req.handle, &req.definition).await?;
    Ok(Json(card))
}

pub async fn get_card(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<CardRow>> {
    Ok(Json(
        card_service::get_owned(&state, id, user.user_id).await?,
    ))
}

pub async fn replace_card(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<DefinitionBody>,
) -> ApiResult<Json<CardRow>> {
    Ok(Json(
        card_service::replace(&state, id, user.user_id, &req.definition).await?,
    ))
}

pub async fn patch_card(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<DefinitionBody>,
) -> ApiResult<Json<CardRow>> {
    Ok(Json(
        card_service::patch(&state, id, user.user_id, &req.definition).await?,
    ))
}

pub async fn delete_card(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<CardRow>> {
    Ok(Json(card_service::archive(&state, id, user.user_id).await?))
}

pub async fn publish_card(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<CardRow>> {
    Ok(Json(
        card_service::publish(&state, id, user.user_id, user.tier).await?,
    ))
}

pub async fn duplicate_card(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<CardRow>> {
    Ok(Json(
        card_service::duplicate(&state, id, user.user_id).await?,
    ))
}

/// Export the card's contact data as a downloadable `.vcf` file.
pub async fn export_vcf(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Response> {
    let body = card_service::export_vcf(&state, id, user.user_id).await?;
    Ok((
        [
            (header::CONTENT_TYPE, "text/vcard; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"contact.vcf\"",
            ),
        ],
        body,
    )
        .into_response())
}

/// Public — no auth. Resolves the active card for a handle (§6.6) and records a
/// best-effort `profile_visit` (PRD §22 acceptance: visit recorded ≤60s).
pub async fn get_card_by_handle(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Path(handle): Path<String>,
) -> ApiResult<Json<card_service::PublicProfile>> {
    let profile = card_service::public_profile(&state, &handle).await?;

    let ip = crate::handlers::analytics::client_ip(&headers);
    let ua = crate::handlers::analytics::header_str(&headers, "user-agent");
    // Best-effort: a failed analytics write must not break the page load.
    let _ = crate::services::analytics_service::record(
        &state,
        profile.card.id,
        "profile_visit",
        None,
        ip.as_deref(),
        ua.as_deref(),
    )
    .await;

    Ok(Json(profile))
}
