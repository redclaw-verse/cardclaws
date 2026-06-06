//! Anonymous demo "publish to web" handler (no auth). Rate-limited because it
//! creates a card + may run AI generation.

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use serde::Serialize;

use crate::error::ApiResult;
use crate::middleware::rate_limit;
use crate::services::demo_service::{self, DemoPublishRequest};
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoPublishResponse {
    pub handle: String,
    pub profile_url: String,
}

pub async fn publish(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<DemoPublishRequest>,
) -> ApiResult<Json<DemoPublishResponse>> {
    let ip = crate::handlers::analytics::client_ip(&headers).unwrap_or_else(|| "unknown".into());
    rate_limit::check(
        state.cache.as_ref(),
        &format!("demo_publish:{ip}"),
        10,
        3600,
    )
    .await?;

    let published = demo_service::publish(&state, &req).await?;
    Ok(Json(DemoPublishResponse {
        handle: published.handle,
        profile_url: published.profile_url,
    }))
}
