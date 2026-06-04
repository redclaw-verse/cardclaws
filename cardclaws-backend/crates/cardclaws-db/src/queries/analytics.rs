//! Analytics event writes and summary aggregation (PRD §18).
//!
//! Phase 1 writes events directly to `analytics_events`. The Redis write-buffer
//! + hourly rollup path (PRD §18.1/§18.2) is a Phase 2 optimization.

use uuid::Uuid;

use crate::models::analytics::{AnalyticsSummary, FeedEvent};
use crate::Db;

pub struct NewEvent<'a> {
    pub card_id: Uuid,
    pub event_type: &'a str,
    /// Correlates the event to the share link that produced it (PRD §6.5.2).
    pub share_token: Option<&'a str>,
    pub ip_hash: Option<&'a str>,
    pub user_agent: Option<&'a str>,
}

pub async fn insert_event(db: &Db, ev: NewEvent<'_>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO analytics_events (card_id, event_type, share_token, ip_hash, user_agent)
           VALUES ($1, $2, $3, $4, $5)"#,
    )
    .bind(ev.card_id)
    .bind(ev.event_type)
    .bind(ev.share_token)
    .bind(ev.ip_hash)
    .bind(ev.user_agent)
    .execute(db)
    .await?;
    Ok(())
}

/// Aggregate metrics for one card across the standard windows.
pub async fn summary(db: &Db, card_id: Uuid) -> Result<AnalyticsSummary, sqlx::Error> {
    sqlx::query_as(
        r#"
        SELECT
          COUNT(*) FILTER (WHERE event_type = 'profile_visit')                                              AS total_visits,
          COUNT(*) FILTER (WHERE event_type = 'profile_visit' AND occurred_at > now() - interval '7 days')  AS visits_7d,
          COUNT(*) FILTER (WHERE event_type = 'profile_visit' AND occurred_at > now() - interval '24 hours') AS visits_24h,
          COUNT(*) FILTER (WHERE event_type = 'qr_scan')                                                     AS qr_scans,
          COUNT(*) FILTER (WHERE event_type = 'contact_save')                                                AS contact_saves,
          COUNT(*) FILTER (WHERE event_type = 'link_click')                                                  AS link_clicks
        FROM analytics_events
        WHERE card_id = $1
        "#,
    )
    .bind(card_id)
    .fetch_one(db)
    .await
}

/// Most recent events for a card, newest first (PRD §6.7.2).
pub async fn feed(db: &Db, card_id: Uuid, limit: i64) -> Result<Vec<FeedEvent>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT id, event_type, share_token, country, city, occurred_at
           FROM analytics_events
           WHERE card_id = $1
           ORDER BY occurred_at DESC, id DESC
           LIMIT $2"#,
    )
    .bind(card_id)
    .bind(limit)
    .fetch_all(db)
    .await
}
