use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// Raw `cards` row. The `definition` column holds the full `CardDefinition`
/// JSON; callers deserialize it with `serde_json` as needed. Serializable so it
/// can be returned directly as the card API response body.
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardRow {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub handle: String,
    pub status: String,
    pub definition: serde_json::Value,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
