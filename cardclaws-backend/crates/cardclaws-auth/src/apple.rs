//! Sign in with Apple: verify the identity token (a JWT signed by Apple, RS256)
//! against Apple's published JWKS.
//!
//! The JWKS fetch is behind a [`JwkProvider`] trait so tests can supply keys
//! without network access (and so the real provider can cache them).

use async_trait::async_trait;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use thiserror::Error;

const APPLE_ISSUER: &str = "https://appleid.apple.com";
const APPLE_JWKS_URL: &str = "https://appleid.apple.com/auth/keys";

#[derive(Debug, Error)]
pub enum AppleAuthError {
    #[error("malformed identity token")]
    MalformedToken,
    #[error("no matching Apple signing key for kid")]
    UnknownKey,
    #[error("identity token verification failed")]
    Verification,
    #[error("failed to fetch Apple keys: {0}")]
    Fetch(String),
}

/// A single JSON Web Key (RSA) from Apple's JWKS.
#[derive(Debug, Clone, Deserialize)]
pub struct AppleJwk {
    pub kid: String,
    pub n: String,
    pub e: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppleJwks {
    pub keys: Vec<AppleJwk>,
}

impl AppleJwks {
    fn find(&self, kid: &str) -> Option<&AppleJwk> {
        self.keys.iter().find(|k| k.kid == kid)
    }
}

/// Verified claims we care about from the Apple identity token.
#[derive(Debug, Clone, Deserialize)]
pub struct AppleClaims {
    /// Stable, unique Apple user identifier.
    pub sub: String,
    pub email: Option<String>,
}

/// Source of Apple's signing keys.
#[async_trait]
pub trait JwkProvider: Send + Sync {
    async fn jwks(&self) -> Result<AppleJwks, AppleAuthError>;
}

/// Production provider that fetches the JWKS over HTTPS.
pub struct HttpJwkProvider {
    client: reqwest::Client,
}

impl HttpJwkProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl Default for HttpJwkProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl JwkProvider for HttpJwkProvider {
    async fn jwks(&self) -> Result<AppleJwks, AppleAuthError> {
        self.client
            .get(APPLE_JWKS_URL)
            .send()
            .await
            .map_err(|e| AppleAuthError::Fetch(e.to_string()))?
            .json::<AppleJwks>()
            .await
            .map_err(|e| AppleAuthError::Fetch(e.to_string()))
    }
}

/// Verify an Apple identity token. `audience` is the app's client_id (the
/// Services ID / bundle id) the token must be addressed to.
pub async fn verify_identity_token(
    provider: &dyn JwkProvider,
    token: &str,
    audience: &str,
) -> Result<AppleClaims, AppleAuthError> {
    let header = decode_header(token).map_err(|_| AppleAuthError::MalformedToken)?;
    let kid = header.kid.ok_or(AppleAuthError::MalformedToken)?;

    let jwks = provider.jwks().await?;
    let jwk = jwks.find(&kid).ok_or(AppleAuthError::UnknownKey)?;

    let key =
        DecodingKey::from_rsa_components(&jwk.n, &jwk.e).map_err(|_| AppleAuthError::UnknownKey)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[APPLE_ISSUER]);
    validation.set_audience(&[audience]);

    decode::<AppleClaims>(token, &key, &validation)
        .map(|d| d.claims)
        .map_err(|_| AppleAuthError::Verification)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EmptyProvider;

    #[async_trait]
    impl JwkProvider for EmptyProvider {
        async fn jwks(&self) -> Result<AppleJwks, AppleAuthError> {
            Ok(AppleJwks { keys: vec![] })
        }
    }

    #[tokio::test]
    async fn malformed_token_rejected() {
        let err = verify_identity_token(&EmptyProvider, "garbage", "com.cardclaws.app")
            .await
            .unwrap_err();
        assert!(matches!(err, AppleAuthError::MalformedToken));
    }
}
