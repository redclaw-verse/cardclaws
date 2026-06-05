//! "We Met" connection queries.

use uuid::Uuid;

use crate::models::connection::ConnectionRow;
use crate::Db;

pub struct NewConnection<'a> {
    pub card_id: Uuid,
    pub owner_id: Uuid,
    pub name: &'a str,
    pub email: Option<&'a str>,
    pub note: Option<&'a str>,
    pub country: Option<&'a str>,
    pub city: Option<&'a str>,
}

pub async fn insert(db: &Db, new: NewConnection<'_>) -> Result<ConnectionRow, sqlx::Error> {
    sqlx::query_as(
        r#"
        INSERT INTO connections (card_id, owner_id, name, email, note, country, city)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, card_id, owner_id, name, email, note, country, city, met_at
        "#,
    )
    .bind(new.card_id)
    .bind(new.owner_id)
    .bind(new.name)
    .bind(new.email)
    .bind(new.note)
    .bind(new.country)
    .bind(new.city)
    .fetch_one(db)
    .await
}

/// Most-recent-first connections captured for a card.
pub async fn list_by_card(db: &Db, card_id: Uuid) -> Result<Vec<ConnectionRow>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT id, card_id, owner_id, name, email, note, country, city, met_at
           FROM connections WHERE card_id = $1 ORDER BY met_at DESC LIMIT 200"#,
    )
    .bind(card_id)
    .fetch_all(db)
    .await
}
