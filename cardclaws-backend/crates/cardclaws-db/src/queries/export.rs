//! GDPR data export (PRD §18.3). Each table is serialized to JSON in Postgres
//! via `json_agg`/`row_to_json`, so the export is assembled without a struct per
//! table. The user's `password_hash` is deliberately never selected.

use serde_json::Value;
use uuid::Uuid;

use crate::Db;

/// The user's own record (without the password hash). `None` if no such user.
pub async fn user_json(db: &Db, id: Uuid) -> Result<Option<Value>, sqlx::Error> {
    let row: Option<(Value,)> = sqlx::query_as(
        r#"SELECT row_to_json(t) FROM (
               SELECT id, email, handle, display_name, tier, avatar_r2_key,
                      created_at, updated_at
               FROM users WHERE id = $1
           ) t"#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?;
    Ok(row.map(|r| r.0))
}

async fn agg(db: &Db, sql: &str, owner: Uuid) -> Result<Value, sqlx::Error> {
    let (v,): (Value,) = sqlx::query_as(sql).bind(owner).fetch_one(db).await?;
    Ok(v)
}

pub async fn cards_json(db: &Db, owner: Uuid) -> Result<Value, sqlx::Error> {
    agg(
        db,
        r#"SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json) FROM (
               SELECT id, handle, status, definition, version, created_at, updated_at
               FROM cards WHERE owner_id = $1 ORDER BY created_at
           ) t"#,
        owner,
    )
    .await
}

pub async fn share_links_json(db: &Db, owner: Uuid) -> Result<Value, sqlx::Error> {
    agg(
        db,
        r#"SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json) FROM (
               SELECT s.token, s.card_id, s.modality, s.campaign, s.created_at, s.expires_at
               FROM share_links s JOIN cards c ON c.id = s.card_id
               WHERE c.owner_id = $1 ORDER BY s.created_at
           ) t"#,
        owner,
    )
    .await
}

pub async fn analytics_json(db: &Db, owner: Uuid) -> Result<Value, sqlx::Error> {
    agg(
        db,
        r#"SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json) FROM (
               SELECT e.id, e.card_id, e.event_type, e.country, e.city, e.occurred_at
               FROM analytics_events e JOIN cards c ON c.id = e.card_id
               WHERE c.owner_id = $1 ORDER BY e.occurred_at
           ) t"#,
        owner,
    )
    .await
}

pub async fn wallet_json(db: &Db, owner: Uuid) -> Result<Value, sqlx::Error> {
    agg(
        db,
        r#"SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json) FROM (
               SELECT w.id, w.card_id, w.platform, w.registered_at, w.last_updated_at
               FROM wallet_registrations w JOIN cards c ON c.id = w.card_id
               WHERE c.owner_id = $1
           ) t"#,
        owner,
    )
    .await
}

pub async fn memberships_json(db: &Db, user_id: Uuid) -> Result<Value, sqlx::Error> {
    agg(
        db,
        r#"SELECT COALESCE(json_agg(row_to_json(t)), '[]'::json) FROM (
               SELECT m.team_id, t2.name AS team_name, m.role, m.created_at
               FROM team_memberships m JOIN teams t2 ON t2.id = m.team_id
               WHERE m.user_id = $1
           ) t"#,
        user_id,
    )
    .await
}
