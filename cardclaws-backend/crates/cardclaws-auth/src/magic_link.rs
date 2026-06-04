//! Magic-link tokens: single-use, 32 bytes of entropy, 15-minute expiry
//! (PRD §20.1). Storage (Redis, keyed by hash → email) lives in the API service;
//! this module just mints the token and its lookup hash.

use crate::token::{generate_opaque_token, hash_token};

/// Magic-link validity window in seconds (15 minutes).
pub const MAGIC_LINK_TTL_SECS: u64 = 15 * 60;

/// A freshly minted magic-link token. `token` is emailed to the user; `hash` is
/// the Redis key the verify step looks up.
pub struct MintedToken {
    pub token: String,
    pub hash: String,
}

pub fn mint() -> MintedToken {
    let token = generate_opaque_token();
    let hash = hash_token(&token);
    MintedToken { token, hash }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::hash_token;

    #[test]
    fn minted_hash_matches_token() {
        let m = mint();
        assert_eq!(m.hash, hash_token(&m.token));
    }
}
