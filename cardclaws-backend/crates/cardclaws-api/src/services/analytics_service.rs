//! Analytics ingestion + summary (PRD §18). IPs are hashed with a per-day
//! rotating salt before storage — never persisted in plaintext (§18.3).

use cardclaws_db::models::analytics::{AnalyticsSummary, FeedEvent};
use cardclaws_db::queries::analytics;
use cardclaws_types::AppError;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::SqlxResultExt;
use crate::middleware::rate_limit;
use crate::services::card_service;
use crate::state::AppState;

/// Client-ingestible event types (PRD §12.2). Server-originated types
/// (`profile_visit`) are recorded internally and not accepted from clients.
const CLIENT_EVENT_TYPES: &[&str] = &["qr_scan", "contact_save", "link_click", "wallet_add"];

/// All recordable event types (client + server-originated).
const ALL_EVENT_TYPES: &[&str] = &[
    "profile_visit",
    "qr_scan",
    "nfc_tap",
    "contact_save",
    "link_click",
    "wallet_add",
    "contact_form_submission",
    "card_share_event",
];

/// Record a client-reported event (PRD §18.1 secondary path). Rate-limited per
/// card. Rejects server-only event types.
pub async fn ingest_client_event(
    state: &AppState,
    card_id: Uuid,
    event_type: &str,
    share_token: Option<&str>,
    ip: Option<&str>,
    user_agent: Option<&str>,
) -> Result<(), AppError> {
    if !CLIENT_EVENT_TYPES.contains(&event_type) {
        return Err(AppError::Validation(format!(
            "event_type '{event_type}' is not client-ingestible"
        )));
    }
    // 100 events/min per card (§20.4).
    rate_limit::check(
        state.cache.as_ref(),
        &format!("analytics:ingest:{card_id}"),
        100,
        60,
    )
    .await?;

    record(state, card_id, event_type, share_token, ip, user_agent).await
}

/// Record any event type internally (used for server-originated events like
/// `profile_visit` and share-link resolutions). Errors are swallowed by callers
/// that treat analytics as best-effort.
pub async fn record(
    state: &AppState,
    card_id: Uuid,
    event_type: &str,
    share_token: Option<&str>,
    ip: Option<&str>,
    user_agent: Option<&str>,
) -> Result<(), AppError> {
    debug_assert!(ALL_EVENT_TYPES.contains(&event_type));
    let ip_hash = ip.map(|raw| hash_ip(&state.ip_hash_secret, raw));
    // Resolve geo from the raw IP, then immediately drop the raw IP — only the
    // hash and coarse country/city are persisted (PRD §18.3).
    let geo = ip.map(|raw| state.geo.resolve(raw));
    let country = geo.as_ref().and_then(|g| g.country.as_deref());
    let city = geo.as_ref().and_then(|g| g.city.as_deref());
    analytics::insert_event(
        &state.db,
        analytics::NewEvent {
            card_id,
            event_type,
            share_token,
            ip_hash: ip_hash.as_deref(),
            country,
            city,
            user_agent,
        },
    )
    .await
    .map_db()
}

/// Owner-only geo distribution for a card.
pub async fn geo_breakdown(
    state: &AppState,
    card_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<cardclaws_db::models::analytics::GeoCount>, AppError> {
    card_service::get_owned(state, card_id, user_id).await?;
    analytics::geo_breakdown(&state.db, card_id).await.map_db()
}

/// Owner-only metrics summary for a card.
pub async fn summary(
    state: &AppState,
    card_id: Uuid,
    user_id: Uuid,
) -> Result<AnalyticsSummary, AppError> {
    card_service::get_owned(state, card_id, user_id).await?;
    analytics::summary(&state.db, card_id).await.map_db()
}

/// Owner-only chronological event feed (capped).
pub async fn feed(
    state: &AppState,
    card_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<FeedEvent>, AppError> {
    card_service::get_owned(state, card_id, user_id).await?;
    analytics::feed(&state.db, card_id, 100).await.map_db()
}

/// SHA-256 of `date:secret:ip`. The date component rotates the salt daily so a
/// hash cannot be correlated across days, while same-day uniqueness is
/// preserved for unique-visitor counting (§18.3).
fn hash_ip(secret: &str, ip: &str) -> String {
    let day = chrono::Utc::now().format("%Y-%m-%d");
    let mut hasher = Sha256::new();
    hasher.update(format!("{day}:{secret}:{ip}").as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ip_hash_is_stable_within_day_and_ip_specific() {
        let a = hash_ip("seed", "1.2.3.4");
        let b = hash_ip("seed", "1.2.3.4");
        let c = hash_ip("seed", "5.6.7.8");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.len(), 64);
    }
}
