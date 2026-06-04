//! Route table (PRD §13). Only the auth surface and health are mounted in B1;
//! card/wallet/profile/analytics routes attach in later phases.

use axum::routing::{get, post};
use axum::Router;

use crate::handlers::{
    account, analytics, assets, auth, cards, health, profile, share, teams, wallet, webhooks,
};
use crate::middleware::cors;
use crate::state::AppState;

pub fn build_router(state: AppState) -> Router {
    let auth_routes = Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/magic-link/request", post(auth::magic_link_request))
        .route("/magic-link/verify", post(auth::magic_link_verify))
        .route("/oauth/apple", post(auth::oauth_apple))
        .route("/refresh", post(auth::refresh))
        .route("/logout", post(auth::logout));

    let v1 = Router::new()
        .nest("/auth", auth_routes)
        .route("/cards", get(cards::list_cards).post(cards::create_card))
        .route(
            "/cards/:id",
            get(cards::get_card)
                .put(cards::replace_card)
                .patch(cards::patch_card)
                .delete(cards::delete_card),
        )
        .route("/cards/:id/publish", post(cards::publish_card))
        .route("/cards/:id/duplicate", post(cards::duplicate_card))
        .route("/cards/:id/export/vcf", get(cards::export_vcf))
        .route("/cards/:id/wallet/apple", post(wallet::apple_pass))
        .route("/cards/:id/wallet/google", post(wallet::google_pass))
        .route("/cards/handle/:handle", get(cards::get_card_by_handle))
        .route("/cards/:id/analytics", get(analytics::summary))
        .route("/cards/:id/analytics/feed", get(analytics::feed))
        .route("/cards/:id/analytics/geo", get(analytics::geo))
        .route("/cards/:id/share", post(share::create_share))
        .route("/cards/:id/share-links", get(share::list_shares))
        .route("/s/:token", get(share::resolve_share))
        .route("/profile/:handle/contact", post(profile::submit_contact))
        .route("/webhooks/revenuecat", post(webhooks::revenuecat))
        .route("/teams", post(teams::create_team))
        .route("/teams/:id", get(teams::get_team))
        .route(
            "/teams/:id/members",
            get(teams::list_members).post(teams::add_member),
        )
        .route(
            "/teams/:id/members/:userId",
            axum::routing::delete(teams::remove_member),
        )
        .route("/teams/:id/analytics", get(teams::analytics))
        .route("/teams/:id/analytics.csv", get(teams::analytics_csv))
        .route(
            "/teams/:id/templates",
            get(teams::list_templates).post(teams::create_template),
        )
        .route(
            "/teams/:id/templates/:templateId",
            axum::routing::delete(teams::delete_template),
        )
        .route("/analytics/event", post(analytics::ingest_event))
        .route("/account/export", get(account::export_data))
        .route("/account", axum::routing::delete(account::delete_account))
        .route("/assets/upload", post(assets::presign_upload))
        .route("/assets/*key", axum::routing::delete(assets::delete_asset));

    Router::new()
        .route("/health", get(health::health))
        .nest("/v1", v1)
        .layer(cors::layer())
        .with_state(state)
}
