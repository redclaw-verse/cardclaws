//! User account types shared between the DB layer and the API surface.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Subscription tier. Authoritative source is the `users.tier` column; client
/// state is never trusted for feature gating (PRD §19.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Free,
    Pro,
    Team,
    Enterprise,
}

impl Tier {
    /// Maximum number of `active` cards allowed for this tier (PRD §6.8.2).
    /// `None` means unlimited.
    pub fn active_card_limit(self) -> Option<i64> {
        match self {
            Tier::Free => Some(1),
            Tier::Pro => Some(5),
            Tier::Team | Tier::Enterprise => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Tier::Free => "free",
            Tier::Pro => "pro",
            Tier::Team => "team",
            Tier::Enterprise => "enterprise",
        }
    }
}

impl std::str::FromStr for Tier {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "free" => Ok(Tier::Free),
            "pro" => Ok(Tier::Pro),
            "team" => Ok(Tier::Team),
            "enterprise" => Ok(Tier::Enterprise),
            other => Err(format!("unknown tier: {other}")),
        }
    }
}

/// A user account. The `password_hash` is never serialized to clients — it is
/// `#[serde(skip)]` so an accidental `Json(user)` cannot leak it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub handle: String,
    pub display_name: String,
    pub tier: Tier,
    pub avatar_r2_key: Option<String>,
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
