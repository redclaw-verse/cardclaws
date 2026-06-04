//! Signs the Google Wallet `savetowallet` JWT (RS256) with the issuer service
//! account key. A fake signer (test double) lets the link-assembly logic be
//! verified without a real key.

use base64::Engine;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde_json::Value;

use crate::error::WalletError;

pub trait GoogleWalletSigner: Send + Sync {
    /// Sign the claims into a compact JWT string.
    fn sign(&self, claims: &Value) -> Result<String, WalletError>;
}

/// Production signer: RS256 over the service account private key (PEM).
pub struct Rs256Signer {
    key: EncodingKey,
}

impl Rs256Signer {
    pub fn from_pem(pem: &[u8]) -> Result<Self, WalletError> {
        let key =
            EncodingKey::from_rsa_pem(pem).map_err(|e| WalletError::Signing(e.to_string()))?;
        Ok(Self { key })
    }
}

impl GoogleWalletSigner for Rs256Signer {
    fn sign(&self, claims: &Value) -> Result<String, WalletError> {
        jsonwebtoken::encode(&Header::new(Algorithm::RS256), claims, &self.key)
            .map_err(|e| WalletError::Signing(e.to_string()))
    }
}

/// Test double: emits a structurally-valid `header.payload.signature` token with
/// a non-cryptographic signature, so tests can decode and assert the payload.
pub struct FakeGoogleSigner;

impl GoogleWalletSigner for FakeGoogleSigner {
    fn sign(&self, claims: &Value) -> Result<String, WalletError> {
        let b64 = |bytes: &[u8]| base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
        let header = b64(br#"{"alg":"RS256","typ":"JWT"}"#);
        let payload =
            b64(&serde_json::to_vec(claims).map_err(|e| WalletError::Signing(e.to_string()))?);
        Ok(format!("{header}.{payload}.fakesig"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn fake_signer_payload_is_decodable() {
        let claims = json!({ "aud": "google", "typ": "savetowallet" });
        let jwt = FakeGoogleSigner.sign(&claims).unwrap();
        let parts: Vec<&str> = jwt.split('.').collect();
        assert_eq!(parts.len(), 3);

        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(parts[1])
            .unwrap();
        let decoded: Value = serde_json::from_slice(&payload).unwrap();
        assert_eq!(decoded["aud"], "google");
    }
}
