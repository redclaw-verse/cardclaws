//! Billing webhooks (PRD §19.2). Authenticated by a shared secret in the
//! `Authorization` header (configured in the RevenueCat dashboard).

use axum::extract::State;
use axum::http::{header::AUTHORIZATION, HeaderMap};
use axum::Json;
use cardclaws_types::AppError;
use serde_json::{json, Value};

use crate::error::ApiResult;
use crate::services::billing_service::{self, RevenueCatWebhook};
use crate::state::AppState;

pub async fn revenuecat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(hook): Json<RevenueCatWebhook>,
) -> ApiResult<Json<Value>> {
    let provided = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !billing_service::secret_matches(&state.billing_webhook_secret, provided) {
        return Err(AppError::Unauthorized.into());
    }

    billing_service::apply_revenuecat(&state, hook).await?;
    Ok(Json(json!({ "status": "ok" })))
}
