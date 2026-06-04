//! Asset upload/delete handlers (PRD §13.7). Uploads are presigned so bytes go
//! directly to R2. Keys are namespaced per user and deletes are restricted to
//! the caller's own prefix.

use axum::extract::{Path, State};
use axum::Json;
use cardclaws_types::AppError;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::ApiResult;
use crate::middleware::auth::AuthUser;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct UploadRequest {
    /// File extension without the dot, e.g. "png", "jpg", "mp4".
    pub ext: String,
}

#[derive(Serialize)]
pub struct UploadResponse {
    /// The object key to store in the card definition / profile.
    pub key: String,
    /// Presigned PUT URL (valid 5 minutes) the client uploads bytes to.
    pub upload_url: String,
}

/// Allowed upload extensions (server-side gate; full MIME re-validation happens
/// on first serve — see assets.rs note).
const ALLOWED_EXT: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "gif", "mp4", "mov", "ttf", "otf",
];

pub async fn presign_upload(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<UploadRequest>,
) -> ApiResult<Json<UploadResponse>> {
    let ext = req.ext.to_ascii_lowercase();
    if !ALLOWED_EXT.contains(&ext.as_str()) {
        return Err(AppError::Validation(format!("unsupported file type: {ext}")).into());
    }

    let key = format!("assets/{}/{}.{}", user.user_id, Uuid::new_v4(), ext);
    let upload_url = state
        .assets
        .presign_put(&key)
        .map_err(|e| AppError::Internal(e.0))?;

    Ok(Json(UploadResponse { key, upload_url }))
}

pub async fn delete_asset(
    State(state): State<AppState>,
    user: AuthUser,
    Path(key): Path<String>,
) -> ApiResult<Json<Value>> {
    // Only allow deleting objects under the caller's own prefix.
    let prefix = format!("assets/{}/", user.user_id);
    if !key.starts_with(&prefix) {
        return Err(AppError::Forbidden.into());
    }
    state
        .assets
        .delete(&key)
        .await
        .map_err(|e| AppError::Internal(e.0))?;
    Ok(Json(json!({ "status": "deleted" })))
}
