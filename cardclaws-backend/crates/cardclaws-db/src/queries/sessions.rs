//! Session (refresh-token) queries. Tokens are stored only as SHA-256 hashes.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::session::SessionRow;
use crate::Db;

pub struct NewSession<'a> {
    pub user_id: Uuid,
    pub refresh_token_hash: &'a str,
    pub device_fingerprint: Option<&'a str>,
    pub expires_at: DateTime<Utc>,
}

pub async fn insert(db: &Db, new: NewSession<'_>) -> Result<SessionRow, sqlx::Error> {
    sqlx::query_as(
        r#"
        INSERT INTO cardclaws_sessions
            (user_id, refresh_token_hash, device_fingerprint, expires_at)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, refresh_token_hash, device_fingerprint,
                  last_active_at, expires_at, created_at
        "#,
    )
    .bind(new.user_id)
    .bind(new.refresh_token_hash)
    .bind(new.device_fingerprint)
    .bind(new.expires_at)
    .fetch_one(db)
    .await
}

/// Look up a live (non-expired) session by its token hash.
pub async fn find_live_by_hash(
    db: &Db,
    token_hash: &str,
) -> Result<Option<SessionRow>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT id, user_id, refresh_token_hash, device_fingerprint,
                  last_active_at, expires_at, created_at
           FROM cardclaws_sessions
           WHERE refresh_token_hash = $1 AND expires_at > now()"#,
    )
    .bind(token_hash)
    .fetch_optional(db)
    .await
}

/// Delete a session by id (used both for rotation and explicit logout).
pub async fn delete(db: &Db, id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM cardclaws_sessions WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}

/// Delete every session for a user (logout-everywhere / account deletion).
pub async fn delete_all_for_user(db: &Db, user_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM cardclaws_sessions WHERE user_id = $1")
        .bind(user_id)
        .execute(db)
        .await?;
    Ok(())
}
