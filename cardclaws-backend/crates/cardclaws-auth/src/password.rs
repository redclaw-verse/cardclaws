//! Argon2id password hashing with the parameters mandated by PRD §20.1
//! (time cost 3, memory 64 MiB, parallelism 4).

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("password hashing failed")]
    Hash,
    #[error("invalid stored hash")]
    InvalidHash,
}

fn argon2() -> Argon2<'static> {
    // m_cost is in KiB: 65536 KiB = 64 MiB.
    let params = Params::new(65_536, 3, 4, None).expect("static argon2 params are valid");
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

/// Hash a plaintext password into a PHC string suitable for storage.
pub fn hash_password(plaintext: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    argon2()
        .hash_password(plaintext.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| PasswordError::Hash)
}

/// Verify a plaintext password against a stored PHC hash. Returns `Ok(false)`
/// for a well-formed hash that simply does not match; `Err` only for a
/// malformed stored hash.
pub fn verify_password(plaintext: &str, stored_hash: &str) -> Result<bool, PasswordError> {
    let parsed = PasswordHash::new(stored_hash).map_err(|_| PasswordError::InvalidHash)?;
    Ok(argon2()
        .verify_password(plaintext.as_bytes(), &parsed)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_then_verify_roundtrips() {
        let hash = hash_password("correct horse battery staple").unwrap();
        assert!(verify_password("correct horse battery staple", &hash).unwrap());
        assert!(!verify_password("wrong password", &hash).unwrap());
    }

    #[test]
    fn distinct_salts_produce_distinct_hashes() {
        let a = hash_password("same").unwrap();
        let b = hash_password("same").unwrap();
        assert_ne!(a, b, "each hash must use a fresh random salt");
    }

    #[test]
    fn malformed_hash_errors() {
        assert!(matches!(
            verify_password("x", "not-a-phc-string"),
            Err(PasswordError::InvalidHash)
        ));
    }
}
