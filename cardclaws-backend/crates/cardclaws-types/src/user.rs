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

    /// Pro-and-above unlock the animated layer types (PRD §6.1.1, §19.1):
    /// video background, particle systems, animated (shader) gradients.
    pub fn allows_pro_layers(self) -> bool {
        !matches!(self, Tier::Free)
    }

    /// Geographic analytics are Pro-and-above (PRD §19.1).
    pub fn allows_geo_analytics(self) -> bool {
        !matches!(self, Tier::Free)
    }

    /// Custom domains are Pro-and-above (PRD §19.1).
    pub fn allows_custom_domain(self) -> bool {
        !matches!(self, Tier::Free)
    }

    /// Analytics retention window in days; `None` = unlimited (PRD §19.1).
    pub fn analytics_retention_days(self) -> Option<i64> {
        match self {
            Tier::Free => Some(7),
            Tier::Pro => Some(90),
            Tier::Team => Some(365),
            Tier::Enterprise => None,
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

#[cfg(test)]
mod tests {
    use super::Tier;

    #[test]
    fn capability_matrix() {
        assert_eq!(Tier::Free.active_card_limit(), Some(1));
        assert_eq!(Tier::Pro.active_card_limit(), Some(5));
        assert_eq!(Tier::Team.active_card_limit(), None);

        assert!(!Tier::Free.allows_pro_layers());
        assert!(Tier::Pro.allows_pro_layers());

        assert!(!Tier::Free.allows_geo_analytics());
        assert!(Tier::Enterprise.allows_geo_analytics());

        assert_eq!(Tier::Free.analytics_retention_days(), Some(7));
        assert_eq!(Tier::Pro.analytics_retention_days(), Some(90));
        assert_eq!(Tier::Enterprise.analytics_retention_days(), None);
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
