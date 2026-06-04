use std::str::FromStr;

use cardclaws_types::{Tier, User};
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Raw `users` row. `tier` is stored as TEXT, so we parse it into [`Tier`] when
/// converting to the domain [`User`].
#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub handle: String,
    pub display_name: String,
    pub tier: String,
    pub password_hash: Option<String>,
    pub avatar_r2_key: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl UserRow {
    /// Convert into the domain type. An unrecognized tier string falls back to
    /// `free` rather than panicking — the DB CHECK constraint already guards it.
    pub fn into_domain(self) -> User {
        let tier = Tier::from_str(&self.tier).unwrap_or(Tier::Free);
        User {
            id: self.id,
            email: self.email,
            handle: self.handle,
            display_name: self.display_name,
            tier,
            avatar_r2_key: self.avatar_r2_key,
            password_hash: self.password_hash,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
