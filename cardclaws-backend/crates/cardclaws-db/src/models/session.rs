use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Raw `cardclaws_sessions` row.
#[derive(Debug, Clone, FromRow)]
pub struct SessionRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token_hash: String,
    pub device_fingerprint: Option<String>,
    pub last_active_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
