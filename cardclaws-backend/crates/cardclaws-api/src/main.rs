//! Production entrypoint: load config + secrets, run migrations, wire the real
//! implementations into `AppState`, and serve.

use std::sync::Arc;

use cardclaws_api::assets::R2Store;
use cardclaws_api::cache::RedisCache;
use cardclaws_api::email::ResendEmailSender;
use cardclaws_api::geo::{GeoResolver, NullGeoResolver};
use cardclaws_api::{build_router, AppState};
use cardclaws_auth::apple::HttpJwkProvider;
use cardclaws_auth::JwtKeys;
use cardclaws_config::{Config, EnvSecretSource, SecretSource};
use cardclaws_wallet::apple::signer::PassSigner;
use cardclaws_wallet::apple::BrandAssets;
use cardclaws_wallet::google::jwt_signer::{FakeGoogleSigner, GoogleWalletSigner, Rs256Signer};
use cardclaws_wallet::strip_renderer;

type BoxError = Box<dyn std::error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,cardclaws_api=debug".into()),
        )
        .init();

    let secrets = EnvSecretSource;
    let config = Config::load(&secrets).await?;

    // Extra secrets not part of the core Config struct.
    let resend_key = secrets.get("RESEND_API_KEY").await.unwrap_or_default();
    let email_from = secrets
        .get("EMAIL_FROM")
        .await
        .unwrap_or_else(|| "CardClaws <auth@cardclaws.com>".into());
    let apple_audience = secrets
        .get("APPLE_AUDIENCE")
        .await
        .unwrap_or_else(|| "com.cardclaws.app".into());

    let db = cardclaws_db::connect(&config.database_url).await?;
    cardclaws_db::migrate(&db).await?;
    tracing::info!("migrations applied");

    let cache = RedisCache::connect(&config.redis_url).await?;
    let assets = R2Store::new(&config.r2)?;
    let pass_signer = build_pass_signer(&secrets).await?;
    let google_signer = build_google_signer(&secrets).await;
    let geo = build_geo_resolver(&secrets).await;

    // Brand glyphs bundled into every pass. Solid-fill placeholders for now;
    // replaced by real CardClaws artwork when design assets land.
    let brand = BrandAssets {
        icon_png: strip_renderer::render_solid(58, 58, "#ff3b30")?,
        logo_png: strip_renderer::render_solid(160, 50, "#ffffff")?,
    };

    let state = AppState {
        db,
        cache: Arc::new(cache),
        email: Arc::new(ResendEmailSender::new(resend_key, email_from)),
        assets: Arc::new(assets),
        geo,
        jwt: JwtKeys::new(&config.jwt_secret),
        apple: Arc::new(HttpJwkProvider::new()),
        apple_audience,
        profile_base_url: config.profile_base_url.clone(),
        ip_hash_secret: config.ip_hash_secret.clone(),
        wallet: config.wallet.clone(),
        pass_signer,
        google_signer,
        brand: Arc::new(brand),
    };

    // Hourly analytics rollup (PRD §18.2).
    spawn_rollup_task(state.db.clone());

    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!(addr = %config.bind_addr, "cardclaws-api listening");
    axum::serve(listener, app).await?;
    Ok(())
}

/// Spawn the background task that recomputes hourly analytics rollups. The first
/// tick fires immediately, then once per hour.
fn spawn_rollup_task(db: cardclaws_db::Db) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(3600));
        loop {
            ticker.tick().await;
            match cardclaws_db::queries::analytics::run_rollup(&db).await {
                Ok(n) => tracing::debug!(buckets = n, "analytics rollup complete"),
                Err(e) => tracing::warn!(error = %e, "analytics rollup failed"),
            }
        }
    });
}

/// Build the Google Wallet signer. Uses the service account RSA key from
/// `GOOGLE_SA_KEY` (PEM) when present; otherwise a fake signer (dev) that
/// produces structurally-valid but unverifiable save links, with a warning.
async fn build_google_signer(secrets: &dyn SecretSource) -> Arc<dyn GoogleWalletSigner> {
    if let Some(pem) = secrets.get("GOOGLE_SA_KEY").await {
        match Rs256Signer::from_pem(pem.as_bytes()) {
            Ok(s) => {
                tracing::info!("google wallet signing enabled (RS256)");
                return Arc::new(s);
            }
            Err(e) => tracing::warn!(error = %e, "invalid GOOGLE_SA_KEY; google wallet disabled"),
        }
    }
    tracing::warn!("GOOGLE_SA_KEY not set: google wallet save links are unsigned (dev only)");
    Arc::new(FakeGoogleSigner)
}

/// Build the geo resolver. With `geoip` enabled and `MAXMIND_DB_PATH` set, uses
/// the MaxMind City database; otherwise geo is unresolved (analytics still work).
#[cfg(feature = "geoip")]
async fn build_geo_resolver(secrets: &dyn SecretSource) -> Arc<dyn GeoResolver> {
    use cardclaws_api::geo::MaxMindGeoResolver;
    if let Some(path) = secrets.get("MAXMIND_DB_PATH").await {
        match MaxMindGeoResolver::open(&path) {
            Ok(r) => {
                tracing::info!("geoip enabled");
                return Arc::new(r);
            }
            Err(e) => tracing::warn!(error = %e, "failed to open MaxMind db; geo disabled"),
        }
    }
    Arc::new(NullGeoResolver)
}

#[cfg(not(feature = "geoip"))]
async fn build_geo_resolver(_secrets: &dyn SecretSource) -> Arc<dyn GeoResolver> {
    Arc::new(NullGeoResolver)
}

/// Build the Apple pass signer. With `apple-signing` enabled, loads the Pass
/// Type ID P12 + WWDR intermediate and signs for real; otherwise uses a fake
/// signer (dev only) and warns loudly.
#[cfg(feature = "apple-signing")]
async fn build_pass_signer(secrets: &dyn SecretSource) -> Result<Arc<dyn PassSigner>, BoxError> {
    use base64::Engine;
    use cardclaws_wallet::apple::signer::OpenSslSigner;

    let p12_b64 = secrets
        .get("APPLE_PASS_P12_BASE64")
        .await
        .ok_or("APPLE_PASS_P12_BASE64 not set")?;
    let p12 = base64::engine::general_purpose::STANDARD.decode(p12_b64.trim())?;
    let password = secrets
        .get("APPLE_PASS_P12_PASSWORD")
        .await
        .unwrap_or_default();
    let wwdr = secrets
        .get("APPLE_WWDR_PEM")
        .await
        .ok_or("APPLE_WWDR_PEM not set")?;
    let signer = OpenSslSigner::from_p12(&p12, &password, wwdr.as_bytes())?;
    tracing::info!("apple pass signing enabled (PKCS#7)");
    Ok(Arc::new(signer))
}

#[cfg(not(feature = "apple-signing"))]
async fn build_pass_signer(_secrets: &dyn SecretSource) -> Result<Arc<dyn PassSigner>, BoxError> {
    use cardclaws_wallet::apple::signer::FakePassSigner;
    tracing::warn!(
        "apple-signing feature is OFF: wallet passes are UNSIGNED and will not \
         load on a real device. Build with --features apple-signing for production."
    );
    Ok(Arc::new(FakePassSigner))
}
