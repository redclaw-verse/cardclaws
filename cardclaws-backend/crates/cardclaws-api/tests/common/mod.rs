//! Shared test harness: builds the real router against a test Postgres with
//! in-memory cache, capturing email, and a no-network Apple key provider.
//!
//! Tests are skipped (not failed) when `TEST_DATABASE_URL` is unset, so a plain
//! `cargo test` without a database still passes; CI sets it (see ci.yml).

// Each test binary (`auth_test`, `cards_test`, …) includes this whole module but
// uses only a subset of its helpers, so unused-helper warnings are expected.
#![allow(dead_code)]

use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

use cardclaws_api::assets::InMemoryStore;
use cardclaws_api::cache::InMemoryCache;
use cardclaws_api::email::CapturingEmailSender;
use cardclaws_api::{build_router, AppState};
use cardclaws_auth::apple::{AppleAuthError, AppleJwks, JwkProvider};
use cardclaws_auth::JwtKeys;
use cardclaws_config::WalletConfig;
use cardclaws_wallet::apple::signer::FakePassSigner;
use cardclaws_wallet::apple::BrandAssets;
use cardclaws_wallet::strip_renderer;

/// Apple provider that never returns a usable key — fine for tests that don't
/// exercise the Apple happy path (which needs a signed token + private key).
struct NoopApple;

#[async_trait]
impl JwkProvider for NoopApple {
    async fn jwks(&self) -> Result<AppleJwks, AppleAuthError> {
        Ok(AppleJwks { keys: vec![] })
    }
}

pub struct TestApp {
    pub router: Router,
    pub email: Arc<CapturingEmailSender>,
    pub assets: Arc<InMemoryStore>,
}

/// Returns `None` when no test DB is configured (test should early-return).
pub async fn try_setup() -> Option<TestApp> {
    let url = std::env::var("TEST_DATABASE_URL").ok()?;
    let db = cardclaws_db::connect(&url).await.expect("connect test db");
    cardclaws_db::migrate(&db).await.expect("run migrations");

    let email = Arc::new(CapturingEmailSender::default());
    let assets = Arc::new(InMemoryStore::default());
    let state = AppState {
        db,
        cache: Arc::new(InMemoryCache::default()),
        email: email.clone(),
        assets: assets.clone(),
        jwt: JwtKeys::new("test-jwt-secret"),
        apple: Arc::new(NoopApple),
        apple_audience: "com.cardclaws.test".into(),
        profile_base_url: "https://cardclaws.test".into(),
        ip_hash_secret: "test-ip-salt".into(),
        wallet: WalletConfig {
            apple_pass_type_id: "pass.com.cardclaws.test".into(),
            apple_team_id: "TEST123".into(),
            organization_name: "CardClaws".into(),
        },
        pass_signer: Arc::new(FakePassSigner),
        brand: Arc::new(BrandAssets {
            icon_png: strip_renderer::render_solid(58, 58, "#ff3b30").unwrap(),
            logo_png: strip_renderer::render_solid(160, 50, "#ffffff").unwrap(),
        }),
    };

    Some(TestApp {
        router: build_router(state),
        email,
        assets,
    })
}

/// Print a skip notice and return — keeps `cargo test` green without a DB.
#[macro_export]
macro_rules! require_app {
    () => {{
        match $crate::common::try_setup().await {
            Some(app) => app,
            None => {
                eprintln!("skipping: TEST_DATABASE_URL not set");
                return;
            }
        }
    }};
}

impl TestApp {
    pub async fn post(&self, path: &str, body: Value) -> (StatusCode, Value) {
        let req = Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = self.router.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        (status, json)
    }

    /// Issue an arbitrary request, optionally authenticated and/or with a JSON
    /// body. Returns the status and parsed JSON body.
    pub async fn request(
        &self,
        method: &str,
        path: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let mut builder = Request::builder().method(method).uri(path);
        if let Some(t) = token {
            builder = builder.header("authorization", format!("Bearer {t}"));
        }
        let req = match body {
            Some(b) => builder
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&b).unwrap()))
                .unwrap(),
            None => builder.body(Body::empty()).unwrap(),
        };
        let resp = self.router.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        (status, json)
    }

    /// Like `request`, but returns the raw (status, content-type, body string)
    /// for non-JSON responses such as vCard downloads.
    pub async fn request_raw(
        &self,
        method: &str,
        path: &str,
        token: Option<&str>,
    ) -> (StatusCode, String, String) {
        let mut builder = Request::builder().method(method).uri(path);
        if let Some(t) = token {
            builder = builder.header("authorization", format!("Bearer {t}"));
        }
        let resp = self
            .router
            .clone()
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = resp.status();
        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes();
        (
            status,
            content_type,
            String::from_utf8_lossy(&bytes).to_string(),
        )
    }

    /// Like `request`, but returns the raw response bytes (for binary downloads
    /// such as `.pkpass`). Returns (status, content-type, body bytes).
    pub async fn request_bytes(
        &self,
        method: &str,
        path: &str,
        token: Option<&str>,
    ) -> (StatusCode, String, Vec<u8>) {
        let mut builder = Request::builder().method(method).uri(path);
        if let Some(t) = token {
            builder = builder.header("authorization", format!("Bearer {t}"));
        }
        let resp = self
            .router
            .clone()
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = resp.status();
        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let bytes = resp
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec();
        (status, content_type, bytes)
    }

    /// GET a path without following redirects; returns (status, Location header).
    pub async fn get_redirect(&self, path: &str) -> (StatusCode, Option<String>) {
        let req = Request::builder()
            .method("GET")
            .uri(path)
            .body(Body::empty())
            .unwrap();
        let resp = self.router.clone().oneshot(req).await.unwrap();
        let location = resp
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        (resp.status(), location)
    }

    /// Register a fresh user and return its access token.
    pub async fn register_and_token(&self) -> String {
        let (email, handle) = unique_identity();
        let (status, body) = self
            .post(
                "/v1/auth/register",
                serde_json::json!({
                    "email": email,
                    "password": "correct horse battery",
                    "handle": handle,
                    "display_name": "Card Owner",
                }),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "registration failed: {body}");
        body["access_token"].as_str().unwrap().to_string()
    }
}

/// Generate a unique (email, handle) pair so tests sharing one DB never collide.
pub fn unique_identity() -> (String, String) {
    let id = uuid::Uuid::new_v4().simple().to_string();
    let short = &id[..12];
    (format!("u{short}@cardclaws.test"), format!("u{short}"))
}

/// Generate a unique card handle.
pub fn unique_handle() -> String {
    let id = uuid::Uuid::new_v4().simple().to_string();
    format!("c{}", &id[..12])
}
