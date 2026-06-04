//! Analytics event writes and summary aggregation (PRD §18).
//!
//! Phase 1 writes events directly to `analytics_events`. The Redis write-buffer
//! + hourly rollup path (PRD §18.1/§18.2) is a Phase 2 optimization.

use uuid::Uuid;

use crate::models::analytics::AnalyticsSummary;
use crate::Db;

pub struct NewEvent<'a> {
    pub card_id: Uuid,
    pub event_type: &'a str,
    pub ip_hash: Option<&'a str>,
    pub user_agent: Option<&'a str>,
}

pub async fn insert_event(db: &Db, ev: NewEvent<'_>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO analytics_events (card_id, event_type, ip_hash, user_agent)
           VALUES ($1, $2, $3, $4)"#,
    )
    .bind(ev.card_id)
    .bind(ev.event_type)
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
