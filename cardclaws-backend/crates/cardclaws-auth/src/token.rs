//! Opaque token generation and hashing for refresh tokens and magic links.
//!
//! The plaintext token is returned to the client exactly once; only its SHA-256
//! hash is ever persisted, so a database leak cannot be replayed.

use base64::Engine;
use rand::RngCore;
use sha2::{Digest, Sha256};

/// Generate a cryptographically-random, URL-safe opaque token (32 bytes of
/// entropy → 43-char base64url string).
pub fn generate_opaque_token() -> String {
    let mut bytes = [0u8; 32];
    let mut rng = rand::rngs::OsRng;
    rng.fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// Hash a token for storage/lookup. Deterministic (no per-call salt) because we
/// must be able to look the token up by hash.
pub fn hash_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    hex(&digest)
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_unique_and_url_safe() {
        let a = generate_opaque_token();
        let b = generate_opaque_token();
        assert_ne!(a, b);
        assert!(!a.contains('+') && !a.contains('/') && !a.contains('='));
    }

    #[test]
    fn hash_is_deterministic_and_64_hex_chars() {
        let t = "some-token";
        assert_eq!(hash_token(t), hash_token(t));
        assert_eq!(hash_token(t).len(), 64);
        assert_ne!(hash_token("a"), hash_token("b"));
    }
}
