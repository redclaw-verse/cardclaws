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
