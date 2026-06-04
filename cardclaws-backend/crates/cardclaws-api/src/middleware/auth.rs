//! `AuthUser` extractor: validates the `Authorization: Bearer <jwt>` header and
//! yields the authenticated user id + tier. Any handler that takes `AuthUser`
//! as an argument is automatically protected.

use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use cardclaws_types::{AppError, Tier};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

pub struct AuthUser {
    pub user_id: Uuid,
    pub tier: Tier,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized)?;

        let claims = state
            .jwt
            .decode_access(token)
            .map_err(|_| AppError::Unauthorized)?;

        Ok(AuthUser {
            user_id: claims.sub,
            tier: claims.tier,
        })
    }
}
