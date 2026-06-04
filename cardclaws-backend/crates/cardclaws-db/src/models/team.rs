use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamRow {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub seat_limit: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// Aggregate analytics for one card belonging to a team member (PRD §6.7.3).
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamCardStats {
    pub card_id: Uuid,
    pub handle: String,
    pub owner_display_name: String,
    pub visits: i64,
    pub qr_scans: i64,
    pub contact_saves: i64,
    pub link_clicks: i64,
}

/// A shared team template (PRD §6.1.4).
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamTemplateRow {
    pub id: Uuid,
    pub team_id: Uuid,
    pub name: String,
    pub definition: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// A team member row joined with the user's identity, for member listings.
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemberRow {
    pub user_id: Uuid,
    pub email: String,
    pub display_name: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}
