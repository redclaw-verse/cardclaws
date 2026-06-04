//! Wallet pass handlers (PRD §13.3). Returns the `.pkpass` bytes with the
//! PassKit content type so the client can hand it straight to `PKAddPasses`.

use axum::extract::{Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::ApiResult;
use crate::middleware::auth::AuthUser;
use crate::services::wallet_service;
use crate::state::AppState;

pub async fn apple_pass(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Response> {
    let bytes = wallet_service::apple_pkpass(&state, id, user.user_id).await?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/vnd.apple.pkpass"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"card.pkpass\"",
            ),
        ],
        bytes,
    )
        .into_response())
}

/// Returns the "Add to Google Wallet" save URL (PRD §13.3).
pub async fn google_pass(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Value>> {
    let save_url = wallet_service::google_save_link(&state, id, user.user_id).await?;
    Ok(Json(json!({ "saveUrl": save_url })))
}
