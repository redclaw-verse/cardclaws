//! Wallet pass handlers (PRD §13.3). Returns the `.pkpass` bytes with the
//! PassKit content type so the client can hand it straight to `PKAddPasses`.

use axum::extract::{Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
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
