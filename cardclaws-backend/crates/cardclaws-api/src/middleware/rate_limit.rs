//! Redis-backed fixed-window rate limiting (PRD §20.2). Exposed as a guard
//! function that handlers/services call with a key, limit, and window. Fails
//! **open** on cache errors so a Redis blip never takes the API down.

use cardclaws_types::AppError;

use crate::cache::Cache;

/// Authenticated default: 100 requests / minute.
pub const AUTHED_PER_MIN: i64 = 100;
/// Unauthenticated default: 20 requests / minute.
pub const UNAUTHED_PER_MIN: i64 = 20;
/// Login lockout: 10 failed attempts / 5 minutes (PRD §20.1).
pub const LOGIN_MAX_FAILS: i64 = 10;
pub const LOGIN_WINDOW_SECS: u64 = 5 * 60;

/// Increment the counter at `key` and reject if it exceeds `limit` within
/// `window_secs`. Returns the current count on success.
pub async fn check(
    cache: &dyn Cache,
    key: &str,
    limit: i64,
    window_secs: u64,
) -> Result<i64, AppError> {
    match cache.incr(key, window_secs).await {
        Ok(count) if count > limit => Err(AppError::RateLimited),
        Ok(count) => Ok(count),
        // Fail open: never let cache unavailability break the request path.
        Err(_) => Ok(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::InMemoryCache;

    #[tokio::test]
    async fn blocks_after_limit() {
        let cache = InMemoryCache::default();
        for _ in 0..3 {
            assert!(check(&cache, "k", 3, 60).await.is_ok());
        }
        assert!(matches!(
            check(&cache, "k", 3, 60).await,
            Err(AppError::RateLimited)
        ));
    }
}
