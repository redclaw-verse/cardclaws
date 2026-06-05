use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// A "We Met" connection — someone who scanned a card and shared back.
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionRow {
    pub id: Uuid,
    pub card_id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub note: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    pub met_at: DateTime<Utc>,
}
