//! Analytics event writes and summary aggregation (PRD §18).
//!
//! Phase 1 writes events directly to `analytics_events`. The Redis write-buffer
//! + hourly rollup path (PRD §18.1/§18.2) is a Phase 2 optimization.

use uuid::Uuid;

use crate::models::analytics::{AnalyticsSummary, FeedEvent, GeoCount, RollupRow};
use crate::Db;

pub struct NewEvent<'a> {
    pub card_id: Uuid,
    pub event_type: &'a str,
    /// Correlates the event to the share link that produced it (PRD §6.5.2).
    pub share_token: Option<&'a str>,
    pub ip_hash: Option<&'a str>,
    pub country: Option<&'a str>,
    pub city: Option<&'a str>,
    pub user_agent: Option<&'a str>,
}

pub async fn insert_event(db: &Db, ev: NewEvent<'_>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO analytics_events
             (card_id, event_type, share_token, ip_hash, country, city, user_agent)
           VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
    )
    .bind(ev.card_id)
    .bind(ev.event_type)
    .bind(ev.share_token)
    .bind(ev.ip_hash)
    .bind(ev.country)
    .bind(ev.city)
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

/// Visit counts grouped by country, descending (PRD §6.7.1 geo distribution).
/// Rows with no resolved country are excluded.
pub async fn geo_breakdown(db: &Db, card_id: Uuid) -> Result<Vec<GeoCount>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT country, COUNT(*) AS visits
           FROM analytics_events
           WHERE card_id = $1 AND country IS NOT NULL
           GROUP BY country
           ORDER BY visits DESC"#,
    )
    .bind(card_id)
    .fetch_all(db)
    .await
}

/// Read a card's hourly rollup buckets, newest first.
pub async fn rollups(db: &Db, card_id: Uuid) -> Result<Vec<RollupRow>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT hour, visits, qr_scans, nfc_taps, saves, link_clicks
           FROM analytics_rollups_hourly
           WHERE card_id = $1
           ORDER BY hour DESC"#,
    )
    .bind(card_id)
    .fetch_all(db)
    .await
}

/// Recompute the hourly rollups from the raw event log (PRD §18.2). Idempotent:
/// `ON CONFLICT DO UPDATE` overwrites each (card, hour) bucket, so re-running is
/// safe and self-correcting. Returns the number of buckets written.
pub async fn run_rollup(db: &Db) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO analytics_rollups_hourly
            (card_id, hour, visits, qr_scans, nfc_taps, saves, link_clicks)
        SELECT
            card_id,
            date_trunc('hour', occurred_at) AS hour,
            COUNT(*) FILTER (WHERE event_type = 'profile_visit'),
            COUNT(*) FILTER (WHERE event_type = 'qr_scan'),
            COUNT(*) FILTER (WHERE event_type = 'nfc_tap'),
            COUNT(*) FILTER (WHERE event_type = 'contact_save'),
            COUNT(*) FILTER (WHERE event_type = 'link_click')
        FROM analytics_events
        GROUP BY card_id, date_trunc('hour', occurred_at)
        ON CONFLICT (card_id, hour) DO UPDATE SET
            visits = EXCLUDED.visits,
            qr_scans = EXCLUDED.qr_scans,
            nfc_taps = EXCLUDED.nfc_taps,
            saves = EXCLUDED.saves,
            link_clicks = EXCLUDED.link_clicks
        "#,
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected())
}
