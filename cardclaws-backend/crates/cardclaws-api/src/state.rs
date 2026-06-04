//! Shared application state, cloned into every handler by axum.

use std::sync::Arc;

use cardclaws_auth::apple::JwkProvider;
use cardclaws_auth::JwtKeys;
use cardclaws_config::WalletConfig;
use cardclaws_db::Db;
use cardclaws_wallet::apple::signer::PassSigner;
use cardclaws_wallet::apple::BrandAssets;
use cardclaws_wallet::google::jwt_signer::GoogleWalletSigner;

use crate::assets::ObjectStore;
use crate::cache::Cache;
use crate::email::EmailSender;
use crate::geo::GeoResolver;

/// All shared dependencies. `Arc`-wrapped trait objects keep `AppState: Clone`
/// cheap while allowing test doubles to be injected (cache, Apple keys).
#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub cache: Arc<dyn Cache>,
    pub email: Arc<dyn EmailSender>,
    pub assets: Arc<dyn ObjectStore>,
    pub geo: Arc<dyn GeoResolver>,
    pub jwt: JwtKeys,
    pub apple: Arc<dyn JwkProvider>,
    /// Apple Services ID / bundle id the identity token must be addressed to.
    pub apple_audience: String,
    /// e.g. `https://cardclaws.com` — used to build profile URLs.
    pub profile_base_url: String,
    /// Seed for the daily-rotating IP hash salt (PRD §18.3).
    pub ip_hash_secret: String,
    /// Shared secret expected in the billing webhook `Authorization` header.
    pub billing_webhook_secret: String,
    /// Apple Wallet pass identity + signer + bundled brand assets.
    pub wallet: WalletConfig,
    pub pass_signer: Arc<dyn PassSigner>,
    pub google_signer: Arc<dyn GoogleWalletSigner>,
    pub brand: Arc<BrandAssets>,
}
