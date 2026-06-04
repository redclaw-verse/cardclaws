use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

/// Raw `share_links` row (PRD §17.1). The token is an opaque base62 id; it
/// encodes nothing and resolves purely via DB lookup.
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareLinkRow {
    pub token: String,
    pub card_id: Uuid,
    pub modality: String,
    pub campaign: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}
