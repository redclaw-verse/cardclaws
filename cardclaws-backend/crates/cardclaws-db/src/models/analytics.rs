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
