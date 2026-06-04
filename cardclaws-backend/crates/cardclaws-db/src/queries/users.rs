//! User table queries.

use cardclaws_types::User;
use uuid::Uuid;

use crate::models::user::UserRow;
use crate::Db;

/// Parameters for inserting a new user.
pub struct NewUser<'a> {
    pub email: &'a str,
    pub handle: &'a str,
    pub display_name: &'a str,
    pub password_hash: Option<&'a str>,
}

/// Insert a user. Returns a `Conflict`-style sqlx error if email/handle is taken
/// (the caller maps unique-violation to a 409).
pub async fn insert(db: &Db, new: NewUser<'_>) -> Result<User, sqlx::Error> {
    let row: UserRow = sqlx::query_as(
        r#"
        INSERT INTO users (email, handle, display_name, password_hash)
        VALUES ($1, $2, $3, $4)
        RETURNING id, email, handle, display_name, tier, password_hash,
                  avatar_r2_key, created_at, updated_at
        "#,
    )
    .bind(new.email)
    .bind(new.handle)
    .bind(new.display_name)
    .bind(new.password_hash)
    .fetch_one(db)
    .await?;
    Ok(row.into_domain())
}

pub async fn find_by_email(db: &Db, email: &str) -> Result<Option<User>, sqlx::Error> {
    let row: Option<UserRow> = sqlx::query_as(
        r#"SELECT id, email, handle, display_name, tier, password_hash,
                  avatar_r2_key, created_at, updated_at
           FROM users WHERE email = $1"#,
    )
    .bind(email)
    .fetch_optional(db)
    .await?;
    Ok(row.map(UserRow::into_domain))
}

/// Update a user's tier (billing webhook). Returns the number of rows affected
/// (0 = no such user).
pub async fn update_tier(db: &Db, id: Uuid, tier: &str) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("UPDATE users SET tier = $2, updated_at = now() WHERE id = $1")
        .bind(id)
        .bind(tier)
        .execute(db)
        .await?;
    Ok(result.rows_affected())
}

pub async fn find_by_id(db: &Db, id: Uuid) -> Result<Option<User>, sqlx::Error> {
    let row: Option<UserRow> = sqlx::query_as(
        r#"SELECT id, email, handle, display_name, tier, password_hash,
                  avatar_r2_key, created_at, updated_at
           FROM users WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?;
    Ok(row.map(UserRow::into_domain))
}

/// Returns true if the handle is already taken (any user). Used when
/// auto-generating a unique handle for OAuth sign-ups.
pub async fn handle_taken(db: &Db, handle: &str) -> Result<bool, sqlx::Error> {
    let exists: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM users WHERE handle = $1 LIMIT 1")
        .bind(handle)
        .fetch_optional(db)
        .await?;
    Ok(exists.is_some())
}

/// Returns true if either the email or the handle is already taken. Used to give
/// a clear 409 before attempting the insert.
pub async fn email_or_handle_taken(
    db: &Db,
    email: &str,
    handle: &str,
) -> Result<bool, sqlx::Error> {
    let exists: Option<(i32,)> =
        sqlx::query_as("SELECT 1 FROM users WHERE email = $1 OR handle = $2 LIMIT 1")
            .bind(email)
            .bind(handle)
            .fetch_optional(db)
            .await?;
    Ok(exists.is_some())
}
