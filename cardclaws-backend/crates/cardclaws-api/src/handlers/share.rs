//! Share-link handlers (PRD §13.4).

use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use cardclaws_db::models::share_link::ShareLinkRow;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ApiResult;
use crate::handlers::analytics::{client_ip, header_str};
use crate::middleware::auth::AuthUser;
use crate::services::share_service;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct CreateShareRequest {
    pub modality: String,
    #[serde(default)]
    pub campaign: Option<String>,
}

#[derive(Serialize)]
pub struct CreateShareResponse {
    pub token: String,
    /// The short link the recipient opens, e.g. `https://cardclaws.com/s/ab12…`.
    pub url: String,
}

pub async fn create_share(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateShareRequest>,
) -> ApiResult<Json<CreateShareResponse>> {
    let link = share_service::create(
        &state,
        id,
        user.user_id,
        &req.modality,
        req.campaign.as_deref(),
    )
    .await?;
    Ok(Json(CreateShareResponse {
        url: format!("{}/s/{}", state.profile_base_url, link.token),
        token: link.token,
    }))
}

pub async fn list_shares(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<ShareLinkRow>>> {
    Ok(Json(share_service::list(&state, id, user.user_id).await?))
}

/// Public: resolve a token, record the attributed event, and 302 to the profile.
pub async fn resolve_share(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(token): Path<String>,
) -> ApiResult<Response> {
    let ip = client_ip(&headers);
    let ua = header_str(&headers, "user-agent");
    let target = share_service::resolve(&state, &token, ip.as_deref(), ua.as_deref()).await?;
    Ok((StatusCode::FOUND, [(header::LOCATION, target)]).into_response())
}
