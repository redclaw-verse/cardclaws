//! Card table queries.

use uuid::Uuid;

use crate::models::card::CardRow;
use crate::Db;

pub struct NewCard<'a> {
    pub owner_id: Uuid,
    pub handle: &'a str,
    pub definition: &'a serde_json::Value,
}

pub async fn insert(db: &Db, new: NewCard<'_>) -> Result<CardRow, sqlx::Error> {
    sqlx::query_as(
        r#"
        INSERT INTO cards (owner_id, handle, definition)
        VALUES ($1, $2, $3)
        RETURNING id, owner_id, handle, status, definition, version,
                  created_at, updated_at
        "#,
    )
    .bind(new.owner_id)
    .bind(new.handle)
    .bind(new.definition)
    .fetch_one(db)
    .await
}

pub async fn find_by_id(db: &Db, id: Uuid) -> Result<Option<CardRow>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT id, owner_id, handle, status, definition, version, created_at, updated_at
           FROM cards WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn list_by_owner(db: &Db, owner_id: Uuid) -> Result<Vec<CardRow>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT id, owner_id, handle, status, definition, version, created_at, updated_at
           FROM cards WHERE owner_id = $1 ORDER BY created_at DESC"#,
    )
    .bind(owner_id)
    .fetch_all(db)
    .await
}

/// Public lookup: only `active` cards are resolvable by handle (PRD §6.6,
/// review A2 — the profile resolves the active card's handle).
pub async fn find_active_by_handle(db: &Db, handle: &str) -> Result<Option<CardRow>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT id, owner_id, handle, status, definition, version, created_at, updated_at
           FROM cards WHERE handle = $1 AND status = 'active'"#,
    )
    .bind(handle)
    .fetch_optional(db)
    .await
}

/// Count a user's `active` cards — used to enforce per-tier limits (§6.8.2).
pub async fn count_active_for_owner(db: &Db, owner_id: Uuid) -> Result<i64, sqlx::Error> {
    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM cards WHERE owner_id = $1 AND status = 'active'")
            .bind(owner_id)
            .fetch_one(db)
            .await?;
    Ok(count)
}

/// Replace the definition and bump the version atomically.
pub async fn update_definition(
    db: &Db,
    id: Uuid,
    definition: &serde_json::Value,
) -> Result<CardRow, sqlx::Error> {
    sqlx::query_as(
        r#"
        UPDATE cards
        SET definition = $2, version = version + 1, updated_at = now()
        WHERE id = $1
        RETURNING id, owner_id, handle, status, definition, version, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(definition)
    .fetch_one(db)
    .await
}

pub async fn set_status(db: &Db, id: Uuid, status: &str) -> Result<CardRow, sqlx::Error> {
    sqlx::query_as(
        r#"
        UPDATE cards SET status = $2, updated_at = now()
        WHERE id = $1
        RETURNING id, owner_id, handle, status, definition, version, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(status)
    .fetch_one(db)
    .await
}
