//! The single application error type. Every crate returns `Result<_, AppError>`
//! and the API layer is responsible for turning it into an HTTP response.

use thiserror::Error;

/// Stable machine-readable error code returned to clients in the JSON body.
/// Kept separate from the HTTP status so clients can branch on a stable string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    Validation,
    RateLimited,
    TierLimit,
    Internal,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("{0} not found")]
    NotFound(String),

    #[error("{0}")]
    Conflict(String),

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("rate limit exceeded")]
    RateLimited,

    /// The caller's tier does not permit this action (e.g. card-count cap).
    #[error("tier limit reached: {0}")]
    TierLimit(String),

    /// Any unexpected internal failure. The inner string is logged but never
    /// surfaced verbatim to clients (see the API error mapping).
    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn code(&self) -> ErrorCode {
        match self {
            AppError::BadRequest(_) => ErrorCode::BadRequest,
            AppError::Unauthorized => ErrorCode::Unauthorized,
            AppError::Forbidden => ErrorCode::Forbidden,
            AppError::NotFound(_) => ErrorCode::NotFound,
            AppError::Conflict(_) => ErrorCode::Conflict,
            AppError::Validation(_) => ErrorCode::Validation,
            AppError::RateLimited => ErrorCode::RateLimited,
            AppError::TierLimit(_) => ErrorCode::TierLimit,
            AppError::Internal(_) => ErrorCode::Internal,
        }
    }

    /// The client-safe message. Internal errors are redacted to avoid leaking
    /// implementation detail; everything else echoes its `Display`.
    pub fn public_message(&self) -> String {
        match self {
            AppError::Internal(_) => "internal error".to_string(),
            other => other.to_string(),
        }
    }
}
