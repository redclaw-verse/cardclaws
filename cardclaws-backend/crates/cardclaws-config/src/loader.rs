use std::collections::HashMap;

use async_trait::async_trait;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("missing required config key: {0}")]
    Missing(String),
    #[error("invalid value for {key}: {reason}")]
    Invalid { key: String, reason: String },
}

/// Source of named secrets/config values. The production implementation talks to
/// Infisical; tests and local dev use [`EnvSecretSource`].
#[async_trait]
pub trait SecretSource: Send + Sync {
    async fn get(&self, key: &str) -> Option<String>;
}

/// Reads secrets from process environment variables. Loaded once at startup.
#[derive(Debug, Default)]
pub struct EnvSecretSource;

#[async_trait]
impl SecretSource for EnvSecretSource {
    async fn get(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}

/// In-memory source, primarily for tests and callers that assemble config from
/// a non-environment source.
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct MapSecretSource(pub HashMap<String, String>);

#[async_trait]
impl SecretSource for MapSecretSource {
    async fn get(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }
}

/// Fully-resolved application configuration.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    /// Base URL for public profiles, e.g. `https://cardclaws.com`.
    pub profile_base_url: String,
    /// Per-day rotating salt seed used for hashing visitor IPs (PRD §18.3).
    pub ip_hash_secret: String,
    pub bind_addr: String,
    pub r2: R2Config,
    pub wallet: WalletConfig,
}

/// Apple Wallet pass identity (PRD §10). Signing material (P12/WWDR) is loaded
/// separately and only when the `apple-signing` feature is enabled.
#[derive(Debug, Clone)]
pub struct WalletConfig {
    pub apple_pass_type_id: String,
    pub apple_team_id: String,
    pub organization_name: String,
}

/// Cloudflare R2 (S3-compatible) object storage config. Defaults are dev
/// placeholders; production supplies real values via Infisical.
#[derive(Debug, Clone)]
pub struct R2Config {
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
}

impl Config {
    /// Resolve all required config from a [`SecretSource`]. Fails fast with a
    /// precise error if anything required is missing.
    pub async fn load(src: &dyn SecretSource) -> Result<Self, ConfigError> {
        async fn require(src: &dyn SecretSource, key: &str) -> Result<String, ConfigError> {
            src.get(key)
                .await
                .ok_or_else(|| ConfigError::Missing(key.to_string()))
        }
        async fn optional(src: &dyn SecretSource, key: &str, default: &str) -> String {
            src.get(key).await.unwrap_or_else(|| default.to_string())
        }

        Ok(Config {
            database_url: require(src, "DATABASE_URL").await?,
            redis_url: optional(src, "REDIS_URL", "redis://127.0.0.1:6379").await,
            jwt_secret: require(src, "JWT_SECRET").await?,
            profile_base_url: optional(src, "PROFILE_BASE_URL", "https://cardclaws.com").await,
            ip_hash_secret: require(src, "IP_HASH_SECRET").await?,
            bind_addr: optional(src, "BIND_ADDR", "0.0.0.0:8080").await,
            r2: R2Config {
                endpoint: optional(
                    src,
                    "R2_ENDPOINT",
                    "https://example.r2.cloudflarestorage.com",
                )
                .await,
                bucket: optional(src, "R2_BUCKET", "cardclaws-assets").await,
                access_key: optional(src, "R2_ACCESS_KEY", "").await,
                secret_key: optional(src, "R2_SECRET_KEY", "").await,
            },
            wallet: WalletConfig {
                apple_pass_type_id: optional(src, "APPLE_PASS_TYPE_ID", "pass.com.cardclaws.card")
                    .await,
                apple_team_id: optional(src, "APPLE_TEAM_ID", "TEAMID0000").await,
                organization_name: optional(src, "ORGANIZATION_NAME", "CardClaws").await,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn src() -> MapSecretSource {
        MapSecretSource(HashMap::from([
            ("DATABASE_URL".into(), "postgres://localhost/cc".into()),
            ("JWT_SECRET".into(), "test-secret".into()),
            ("IP_HASH_SECRET".into(), "salt-seed".into()),
        ]))
    }

    #[tokio::test]
    async fn loads_required_and_defaults() {
        let cfg = Config::load(&src()).await.unwrap();
        assert_eq!(cfg.database_url, "postgres://localhost/cc");
        assert_eq!(cfg.profile_base_url, "https://cardclaws.com");
        assert_eq!(cfg.bind_addr, "0.0.0.0:8080");
    }

    #[tokio::test]
    async fn missing_required_fails() {
        let empty = MapSecretSource(HashMap::new());
        let err = Config::load(&empty).await.unwrap_err();
        assert!(matches!(err, ConfigError::Missing(k) if k == "DATABASE_URL"));
    }
}
