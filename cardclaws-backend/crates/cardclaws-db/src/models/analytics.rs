use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

/// Aggregated card metrics for the dashboard summary (PRD §6.7.1).
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsSummary {
    pub total_visits: i64,
    pub visits_7d: i64,
    pub visits_24h: i64,
    pub qr_scans: i64,
    pub contact_saves: i64,
    pub link_clicks: i64,
}

/// One hourly rollup bucket (PRD §9.3 `analytics_rollups_hourly`).
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RollupRow {
    pub hour: DateTime<Utc>,
    pub visits: i32,
    pub qr_scans: i32,
    pub nfc_taps: i32,
    pub saves: i32,
    pub link_clicks: i32,
}

/// A country's visit count for the geo distribution (PRD §6.7.1).
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeoCount {
    pub country: String,
    pub visits: i64,
}

/// One row of the chronological event feed (PRD §6.7.2).
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedEvent {
    pub id: i64,
    pub event_type: String,
    pub share_token: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub occurred_at: DateTime<Utc>,
}
