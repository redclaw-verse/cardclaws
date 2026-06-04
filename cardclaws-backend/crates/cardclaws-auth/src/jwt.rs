//! Access-token (JWT) encode/decode. HS256 over the configured secret. Access
//! tokens are short-lived (15 min, PRD §6.8.1).

use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use thiserror::Error;
use uuid::Uuid;

use cardclaws_types::auth::AccessClaims;
use cardclaws_types::Tier;

/// Access-token lifetime in seconds (15 minutes).
pub const ACCESS_TOKEN_TTL_SECS: i64 = 15 * 60;

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("failed to encode token")]
    Encode,
    #[error("invalid or expired token")]
    Invalid,
}

/// Holds the symmetric signing key. Cheap to clone (key bytes are shared).
#[derive(Clone)]
pub struct JwtKeys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl JwtKeys {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret.as_bytes()),
            decoding: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    /// Mint an access token for a user at a given tier.
    pub fn encode_access(&self, user_id: Uuid, tier: Tier) -> Result<String, JwtError> {
        let now = Utc::now().timestamp();
        let claims = AccessClaims {
            sub: user_id,
            tier,
            iat: now,
            exp: now + ACCESS_TOKEN_TTL_SECS,
        };
        encode(&Header::default(), &claims, &self.encoding).map_err(|_| JwtError::Encode)
    }

    /// Validate an access token and return its claims. Expiry is enforced.
    pub fn decode_access(&self, token: &str) -> Result<AccessClaims, JwtError> {
        let mut validation = Validation::default();
        // Access tokens carry no `aud` claim; only `exp` is enforced.
        validation.validate_aud = false;
        decode::<AccessClaims>(token, &self.decoding, &validation)
            .map(|data| data.claims)
            .map_err(|_| JwtError::Invalid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_then_decode_roundtrips() {
        let keys = JwtKeys::new("super-secret");
        let id = Uuid::new_v4();
        let token = keys.encode_access(id, Tier::Pro).unwrap();
        let claims = keys.decode_access(&token).unwrap();
        assert_eq!(claims.sub, id);
        assert_eq!(claims.tier, Tier::Pro);
        assert!(claims.exp > claims.iat);
    }

    #[test]
    fn wrong_secret_rejected() {
        let keys = JwtKeys::new("secret-a");
        let other = JwtKeys::new("secret-b");
        let token = keys.encode_access(Uuid::new_v4(), Tier::Free).unwrap();
        assert!(matches!(
            other.decode_access(&token),
            Err(JwtError::Invalid)
        ));
    }

    #[test]
    fn garbage_token_rejected() {
        let keys = JwtKeys::new("secret");
        assert!(matches!(
            keys.decode_access("not.a.jwt"),
            Err(JwtError::Invalid)
        ));
    }
}
