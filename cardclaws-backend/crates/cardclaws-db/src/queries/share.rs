//! Share-link queries (PRD §17).

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::share_link::ShareLinkRow;
use crate::Db;

pub struct NewShareLink<'a> {
    pub token: &'a str,
    pub card_id: Uuid,
    pub modality: &'a str,
    pub campaign: Option<&'a str>,
    pub expires_at: Option<DateTime<Utc>>,
}

pub async fn insert(db: &Db, new: NewShareLink<'_>) -> Result<ShareLinkRow, sqlx::Error> {
    sqlx::query_as(
        r#"
        INSERT INTO share_links (token, card_id, modality, campaign, expires_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING token, card_id, modality, campaign, created_at, expires_at
        "#,
    )
    .bind(new.token)
    .bind(new.card_id)
    .bind(new.modality)
    .bind(new.campaign)
    .bind(new.expires_at)
    .fetch_one(db)
    .await
}

pub async fn find_by_token(db: &Db, token: &str) -> Result<Option<ShareLinkRow>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT token, card_id, modality, campaign, created_at, expires_at
           FROM share_links WHERE token = $1"#,
    )
    .bind(token)
    .fetch_optional(db)
    .await
}

pub async fn list_by_card(db: &Db, card_id: Uuid) -> Result<Vec<ShareLinkRow>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT token, card_id, modality, campaign, created_at, expires_at
           FROM share_links WHERE card_id = $1 ORDER BY created_at DESC"#,
    )
    .bind(card_id)
    .fetch_all(db)
    .await
}
