//! Object storage for user assets (logos, media) on Cloudflare R2.
//!
//! Uploads use **presigned PUT URLs**: the client uploads bytes directly to R2,
//! so large files never transit the API. Presigning is a pure HMAC operation
//! (no network), generated via `rusty-s3`. Behind an [`ObjectStore`] trait so
//! tests use an in-memory fake.
//!
//! NOTE: because uploads go straight to R2, server-side MIME re-validation and
//! ClamAV scanning (PRD §20.2) happen lazily on first access, not here — that
//! scan-on-serve path is a later milestone.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use cardclaws_config::R2Config;
use rusty_s3::actions::S3Action;
use rusty_s3::{Bucket, Credentials, UrlStyle};

/// How long a presigned upload URL stays valid (PRD §10.1 uses 5 min for passes;
/// reuse that here).
const PRESIGN_TTL: Duration = Duration::from_secs(300);

#[derive(Debug, thiserror::Error)]
#[error("object store error: {0}")]
pub struct StoreError(pub String);

#[async_trait]
pub trait ObjectStore: Send + Sync {
    /// Return a presigned PUT URL the client can upload `key` to.
    fn presign_put(&self, key: &str) -> Result<String, StoreError>;

    /// Upload bytes server-side (e.g. an AI-generated image). The object must be
    /// publicly readable so the web profile can fetch it.
    async fn put_object(
        &self,
        key: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<(), StoreError>;

    /// The public URL an uploaded object is served from.
    fn public_url(&self, key: &str) -> String;

    /// Delete an object.
    async fn delete(&self, key: &str) -> Result<(), StoreError>;
}

// ---- R2 -------------------------------------------------------------------

pub struct R2Store {
    bucket: Bucket,
    creds: Credentials,
    http: reqwest::Client,
    /// Base URL objects are publicly served from (e.g. the R2 public domain, or
    /// `http://localhost:9000/<bucket>` for a local MinIO). S3-compatible, so the
    /// same store works against R2 in prod and MinIO in dev.
    public_base: String,
}

impl R2Store {
    pub fn new(cfg: &R2Config) -> Result<Self, StoreError> {
        let url = cfg
            .endpoint
            .parse()
            .map_err(|e| StoreError(format!("{e}")))?;
        // R2 ignores the region but the S3 signer requires one; "auto" is
        // Cloudflare's convention (MinIO ignores it too).
        let bucket = Bucket::new(url, UrlStyle::Path, cfg.bucket.clone(), "auto")
            .map_err(|e| StoreError(e.to_string()))?;
        let creds = Credentials::new(cfg.access_key.clone(), cfg.secret_key.clone());
        let public_base = if cfg.public_base.is_empty() {
            format!("{}/{}", cfg.endpoint.trim_end_matches('/'), cfg.bucket)
        } else {
            cfg.public_base.trim_end_matches('/').to_string()
        };
        Ok(Self {
            bucket,
            creds,
            http: reqwest::Client::new(),
            public_base,
        })
    }
}

#[async_trait]
impl ObjectStore for R2Store {
    fn presign_put(&self, key: &str) -> Result<String, StoreError> {
        let action = self.bucket.put_object(Some(&self.creds), key);
        Ok(action.sign(PRESIGN_TTL).to_string())
    }

    async fn put_object(
        &self,
        key: &str,
        bytes: Vec<u8>,
        content_type: &str,
    ) -> Result<(), StoreError> {
        let url = self
            .bucket
            .put_object(Some(&self.creds), key)
            .sign(PRESIGN_TTL);
        let resp = self
            .http
            .put(url)
            .header("content-type", content_type)
            .body(bytes)
            .send()
            .await
            .map_err(|e| StoreError(e.to_string()))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(StoreError(format!("put status {}", resp.status())))
        }
    }

    fn public_url(&self, key: &str) -> String {
        format!("{}/{}", self.public_base, key)
    }

    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        let url = self
            .bucket
            .delete_object(Some(&self.creds), key)
            .sign(PRESIGN_TTL);
        let resp = self
            .http
            .delete(url)
            .send()
            .await
            .map_err(|e| StoreError(e.to_string()))?;
        if resp.status().is_success() || resp.status().as_u16() == 404 {
            Ok(())
        } else {
            Err(StoreError(format!("delete status {}", resp.status())))
        }
    }
}

// ---- In-memory fake (tests) ----------------------------------------------

#[derive(Default)]
pub struct InMemoryStore {
    pub deleted: Mutex<Vec<String>>,
    pub objects: Mutex<HashMap<String, Vec<u8>>>,
}

#[async_trait]
impl ObjectStore for InMemoryStore {
    fn presign_put(&self, key: &str) -> Result<String, StoreError> {
        Ok(format!("https://upload.test/{key}"))
    }

    async fn put_object(
        &self,
        key: &str,
        bytes: Vec<u8>,
        _content_type: &str,
    ) -> Result<(), StoreError> {
        self.objects.lock().unwrap().insert(key.to_string(), bytes);
        Ok(())
    }

    fn public_url(&self, key: &str) -> String {
        format!("https://assets.test/{key}")
    }

    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        self.deleted.lock().unwrap().push(key.to_string());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r2_presign_produces_https_url_containing_key() {
        let cfg = R2Config {
            endpoint: "https://acct.r2.cloudflarestorage.com".into(),
            bucket: "cardclaws-assets".into(),
            access_key: "AKIA_TEST".into(),
            secret_key: "secret_test".into(),
            public_base: String::new(),
        };
        let store = R2Store::new(&cfg).unwrap();
        let url = store.presign_put("assets/abc/logo.png").unwrap();
        assert!(url.starts_with("https://"));
        assert!(url.contains("cardclaws-assets"));
        assert!(url.contains("logo.png"));
        assert!(url.contains("X-Amz-Signature"));

        // Falls back to endpoint/bucket when no explicit public base is set.
        assert_eq!(
            store.public_url("welcome/x.png"),
            "https://acct.r2.cloudflarestorage.com/cardclaws-assets/welcome/x.png"
        );
    }
}
