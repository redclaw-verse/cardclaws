//! Request/response DTOs for the authentication endpoints (PRD §13.1).

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::user::Tier;

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub handle: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MagicLinkRequest {
    pub email: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MagicLinkVerifyRequest {
    pub token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppleOAuthRequest {
    /// The identity token (a JWT) returned by Sign in with Apple.
    pub identity_token: String,
    /// Apple only returns the name on first authorization, so the client passes
    /// it through when present.
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Returned by every successful authentication. Access token is short-lived
/// (15 min), refresh token is long-lived (30 days) and rotates on use.
#[derive(Debug, Clone, Serialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    /// Access-token lifetime in seconds.
    pub expires_in: i64,
    pub user: AuthUserInfo,
}

/// Minimal user info embedded in an auth response so the client need not make a
/// second round-trip after login.
#[derive(Debug, Clone, Serialize)]
pub struct AuthUserInfo {
    pub id: Uuid,
    pub email: String,
    pub handle: String,
    pub display_name: String,
    pub tier: Tier,
}

/// Claims encoded inside the access JWT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessClaims {
    /// Subject — the user id.
    pub sub: Uuid,
    pub tier: Tier,
    /// Expiry (unix seconds).
    pub exp: i64,
    /// Issued-at (unix seconds).
    pub iat: i64,
}
