//! Billing webhook handling (PRD §19.2). RevenueCat (mirroring Stripe) posts
//! subscription lifecycle events; we map them to `users.tier`, which is the
//! authoritative source for feature gates.

use cardclaws_db::queries::users;
use cardclaws_types::{AppError, Tier};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::SqlxResultExt;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct RevenueCatWebhook {
    pub event: RevenueCatEvent,
}

#[derive(Deserialize)]
pub struct RevenueCatEvent {
    /// The lifecycle event type (INITIAL_PURCHASE, RENEWAL, CANCELLATION, …).
    #[serde(rename = "type")]
    pub event_type: String,
    /// The app user id we set when configuring RevenueCat = our user id.
    pub app_user_id: String,
    /// Active entitlement ids (e.g. ["pro"]). Absent on cancellations.
    #[serde(default)]
    pub entitlement_ids: Vec<String>,
}

/// Constant-time-ish secret comparison (avoids early-exit length leak).
pub fn secret_matches(expected: &str, provided: &str) -> bool {
    if expected.is_empty() || expected.len() != provided.len() {
        return false;
    }
    let mut diff = 0u8;
    for (a, b) in expected.bytes().zip(provided.bytes()) {
        diff |= a ^ b;
    }
    diff == 0
}

/// Apply a verified webhook: resolve the target tier and update the user. Unknown
/// users are a no-op (RevenueCat still gets a 2xx so it won't retry forever).
pub async fn apply_revenuecat(state: &AppState, hook: RevenueCatWebhook) -> Result<(), AppError> {
    let tier = tier_for_event(&hook.event);
    let Ok(user_id) = Uuid::parse_str(&hook.event.app_user_id) else {
        tracing::warn!(app_user_id = %hook.event.app_user_id, "billing webhook: non-uuid app_user_id");
        return Ok(());
    };
    let affected = users::update_tier(&state.db, user_id, tier.as_str())
        .await
        .map_db()?;
    if affected == 0 {
        tracing::warn!(%user_id, "billing webhook: no such user");
    } else {
        tracing::info!(%user_id, tier = tier.as_str(), "tier updated via billing webhook");
    }
    Ok(())
}

/// Map a RevenueCat event to the tier it should produce. Terminal events drop to
/// free; active events derive the tier from the entitlement id.
fn tier_for_event(event: &RevenueCatEvent) -> Tier {
    match event.event_type.as_str() {
        "CANCELLATION" | "EXPIRATION" | "SUBSCRIPTION_PAUSED" | "BILLING_ISSUE" => Tier::Free,
        _ => event
            .entitlement_ids
            .iter()
            .find_map(|e| match e.as_str() {
                "enterprise" => Some(Tier::Enterprise),
                "team" => Some(Tier::Team),
                "pro" => Some(Tier::Pro),
                _ => None,
            })
            .unwrap_or(Tier::Pro),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(t: &str, ents: &[&str]) -> RevenueCatEvent {
        RevenueCatEvent {
            event_type: t.into(),
            app_user_id: "x".into(),
            entitlement_ids: ents.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn maps_events_to_tiers() {
        assert_eq!(
            tier_for_event(&event("INITIAL_PURCHASE", &["pro"])),
            Tier::Pro
        );
        assert_eq!(tier_for_event(&event("RENEWAL", &["team"])), Tier::Team);
        assert_eq!(tier_for_event(&event("CANCELLATION", &["pro"])), Tier::Free);
        assert_eq!(tier_for_event(&event("EXPIRATION", &[])), Tier::Free);
    }

    #[test]
    fn secret_compare() {
        assert!(secret_matches("abc123", "abc123"));
        assert!(!secret_matches("abc123", "abc124"));
        assert!(!secret_matches("abc123", "abc1234"));
        assert!(!secret_matches("", "")); // empty = unconfigured = never matches
    }
}
