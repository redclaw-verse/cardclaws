//! HTTP error mapping. `AppError` lives in `cardclaws-types`, and `IntoResponse`
//! lives in axum, so the orphan rule forces a local newtype to bridge them.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use cardclaws_types::{AppError, ErrorCode};
use serde::Serialize;

/// Newtype wrapper so we can implement `IntoResponse` for our error.
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self {
        ApiError(e)
    }
}

/// Any sqlx error that escapes the query layer is an internal failure. Unique
/// violations are mapped to 409 at the call site before reaching here.
impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        ApiError(AppError::Internal(format!("db: {e}")))
    }
}

#[derive(Serialize)]
struct ErrorBody {
    code: ErrorCode,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let err = self.0;
        let status = match err.code() {
            ErrorCode::BadRequest | ErrorCode::Validation => StatusCode::BAD_REQUEST,
            ErrorCode::Unauthorized => StatusCode::UNAUTHORIZED,
            ErrorCode::Forbidden => StatusCode::FORBIDDEN,
            ErrorCode::NotFound => StatusCode::NOT_FOUND,
            ErrorCode::Conflict => StatusCode::CONFLICT,
            ErrorCode::TierLimit => StatusCode::PAYMENT_REQUIRED,
            ErrorCode::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            ErrorCode::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };

        // Internal errors are logged with detail but redacted in the response.
        if let AppError::Internal(detail) = &err {
            tracing::error!(error = %detail, "internal error");
        }

        let body = ErrorBody {
            code: err.code(),
            message: err.public_message(),
        };
        (status, Json(body)).into_response()
    }
}

/// Convenience alias for handler return types.
pub type ApiResult<T> = Result<T, ApiError>;

/// Map a raw sqlx error into an `AppError::Internal`. Service code uses this so
/// it can keep returning the domain `AppError` (which has no sqlx dependency)
/// while still using `?` on database calls.
pub trait SqlxResultExt<T> {
    fn map_db(self) -> Result<T, cardclaws_types::AppError>;
}

impl<T> SqlxResultExt<T> for Result<T, sqlx::Error> {
    fn map_db(self) -> Result<T, cardclaws_types::AppError> {
        self.map_err(|e| cardclaws_types::AppError::Internal(format!("db: {e}")))
    }
}
