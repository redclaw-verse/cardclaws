//! Team + membership queries (PRD §21 Phase 4).

use uuid::Uuid;

use crate::models::team::{MemberRow, TeamRow};
use crate::Db;

pub async fn insert_team(
    db: &Db,
    name: &str,
    owner_id: Uuid,
    seat_limit: Option<i32>,
) -> Result<TeamRow, sqlx::Error> {
    sqlx::query_as(
        r#"INSERT INTO teams (name, owner_id, seat_limit)
           VALUES ($1, $2, $3)
           RETURNING id, name, owner_id, seat_limit, created_at"#,
    )
    .bind(name)
    .bind(owner_id)
    .bind(seat_limit)
    .fetch_one(db)
    .await
}

pub async fn find_team(db: &Db, id: Uuid) -> Result<Option<TeamRow>, sqlx::Error> {
    sqlx::query_as(r#"SELECT id, name, owner_id, seat_limit, created_at FROM teams WHERE id = $1"#)
        .bind(id)
        .fetch_optional(db)
        .await
}

pub async fn insert_membership(
    db: &Db,
    team_id: Uuid,
    user_id: Uuid,
    role: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(r#"INSERT INTO team_memberships (team_id, user_id, role) VALUES ($1, $2, $3)"#)
        .bind(team_id)
        .bind(user_id)
        .bind(role)
        .execute(db)
        .await?;
    Ok(())
}

/// Return the caller's role in a team, if any.
pub async fn member_role(
    db: &Db,
    team_id: Uuid,
    user_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(String,)> =
        sqlx::query_as(r#"SELECT role FROM team_memberships WHERE team_id = $1 AND user_id = $2"#)
            .bind(team_id)
            .bind(user_id)
            .fetch_optional(db)
            .await?;
    Ok(row.map(|r| r.0))
}

pub async fn list_members(db: &Db, team_id: Uuid) -> Result<Vec<MemberRow>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT u.id AS user_id, u.email, u.display_name, m.role, m.created_at
           FROM team_memberships m
           JOIN users u ON u.id = m.user_id
           WHERE m.team_id = $1
           ORDER BY m.created_at ASC"#,
    )
    .bind(team_id)
    .fetch_all(db)
    .await
}

pub async fn count_members(db: &Db, team_id: Uuid) -> Result<i64, sqlx::Error> {
    let (n,): (i64,) =
        sqlx::query_as(r#"SELECT COUNT(*) FROM team_memberships WHERE team_id = $1"#)
            .bind(team_id)
            .fetch_one(db)
            .await?;
    Ok(n)
}

pub async fn delete_membership(db: &Db, team_id: Uuid, user_id: Uuid) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(r#"DELETE FROM team_memberships WHERE team_id = $1 AND user_id = $2"#)
        .bind(team_id)
        .bind(user_id)
        .execute(db)
        .await?;
    Ok(r.rows_affected())
}
