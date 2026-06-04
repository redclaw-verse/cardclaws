//! A tiny cache abstraction over the operations the API needs from Redis:
//! counters-with-expiry (rate limiting, login lockout) and short-lived
//! key/value (magic links). Behind a trait so tests use an in-memory fake and
//! don't require a running Redis (test-double policy, plan A3).

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use redis::AsyncCommands;

#[async_trait]
pub trait Cache: Send + Sync {
    /// Increment a counter, setting `ttl_secs` expiry on first creation. Returns
    /// the new value. Used for fixed-window rate limits and login lockouts.
    async fn incr(&self, key: &str, ttl_secs: u64) -> Result<i64, CacheError>;

    /// Store a value with a TTL.
    async fn set_ex(&self, key: &str, value: &str, ttl_secs: u64) -> Result<(), CacheError>;

    /// Atomically fetch and delete a value (single-use tokens).
    async fn get_del(&self, key: &str) -> Result<Option<String>, CacheError>;
}

#[derive(Debug, thiserror::Error)]
#[error("cache error: {0}")]
pub struct CacheError(pub String);

// ---- Redis implementation -------------------------------------------------

pub struct RedisCache {
    conn: redis::aio::ConnectionManager,
}

impl RedisCache {
    pub async fn connect(url: &str) -> Result<Self, CacheError> {
        let client = redis::Client::open(url).map_err(|e| CacheError(e.to_string()))?;
        let conn = redis::aio::ConnectionManager::new(client)
            .await
            .map_err(|e| CacheError(e.to_string()))?;
        Ok(Self { conn })
    }
}

#[async_trait]
impl Cache for RedisCache {
    async fn incr(&self, key: &str, ttl_secs: u64) -> Result<i64, CacheError> {
        let mut conn = self.conn.clone();
        let value: i64 = conn
            .incr(key, 1)
            .await
            .map_err(|e| CacheError(e.to_string()))?;
        if value == 1 {
            let _: () = conn
                .expire(key, ttl_secs as i64)
                .await
                .map_err(|e| CacheError(e.to_string()))?;
        }
        Ok(value)
    }

    async fn set_ex(&self, key: &str, value: &str, ttl_secs: u64) -> Result<(), CacheError> {
        let mut conn = self.conn.clone();
        let _: () = conn
            .set_ex(key, value, ttl_secs)
            .await
            .map_err(|e| CacheError(e.to_string()))?;
        Ok(())
    }

    async fn get_del(&self, key: &str) -> Result<Option<String>, CacheError> {
        let mut conn = self.conn.clone();
        // GETDEL is atomic (Redis 6.2+).
        let value: Option<String> = redis::cmd("GETDEL")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| CacheError(e.to_string()))?;
        Ok(value)
    }
}

// ---- In-memory fake (tests) ----------------------------------------------

/// Non-expiring in-memory cache for tests. TTLs are accepted and ignored — tests
/// assert on counts/values, not on real-time expiry.
#[derive(Default)]
pub struct InMemoryCache {
    counters: Mutex<HashMap<String, i64>>,
    values: Mutex<HashMap<String, String>>,
}

#[async_trait]
impl Cache for InMemoryCache {
    async fn incr(&self, key: &str, _ttl_secs: u64) -> Result<i64, CacheError> {
        let mut map = self.counters.lock().unwrap();
        let entry = map.entry(key.to_string()).or_insert(0);
        *entry += 1;
        Ok(*entry)
    }

    async fn set_ex(&self, key: &str, value: &str, _ttl_secs: u64) -> Result<(), CacheError> {
        self.values
            .lock()
            .unwrap()
            .insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn get_del(&self, key: &str) -> Result<Option<String>, CacheError> {
        Ok(self.values.lock().unwrap().remove(key))
    }
}
