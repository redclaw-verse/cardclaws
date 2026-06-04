//! Team + membership queries (PRD §21 Phase 4).

use uuid::Uuid;

use crate::models::team::{MemberRow, TeamCardStats, TeamRow, TeamTemplateRow};
use crate::Db;

pub async fn insert_template(
    db: &Db,
    team_id: Uuid,
    name: &str,
    definition: &serde_json::Value,
    created_by: Uuid,
) -> Result<TeamTemplateRow, sqlx::Error> {
    sqlx::query_as(
        r#"INSERT INTO team_templates (team_id, name, definition, created_by)
           VALUES ($1, $2, $3, $4)
           RETURNING id, team_id, name, definition, created_by, created_at"#,
    )
    .bind(team_id)
    .bind(name)
    .bind(definition)
    .bind(created_by)
    .fetch_one(db)
    .await
}

pub async fn list_templates(db: &Db, team_id: Uuid) -> Result<Vec<TeamTemplateRow>, sqlx::Error> {
    sqlx::query_as(
        r#"SELECT id, team_id, name, definition, created_by, created_at
           FROM team_templates WHERE team_id = $1 ORDER BY created_at ASC"#,
    )
    .bind(team_id)
    .fetch_all(db)
    .await
}

pub async fn delete_template(
    db: &Db,
    team_id: Uuid,
    template_id: Uuid,
) -> Result<u64, sqlx::Error> {
    let r = sqlx::query(r#"DELETE FROM team_templates WHERE team_id = $1 AND id = $2"#)
        .bind(team_id)
        .bind(template_id)
        .execute(db)
        .await?;
    Ok(r.rows_affected())
}

/// Sortable columns for team analytics. Whitelisted so the caller's sort key can
/// never be interpolated into SQL directly.
#[derive(Debug, Clone, Copy)]
pub enum TeamStatsSort {
    Visits,
    QrScans,
    ContactSaves,
    LinkClicks,
}

impl TeamStatsSort {
    pub fn parse(s: Option<&str>) -> Self {
        match s {
            Some("qr_scans") => TeamStatsSort::QrScans,
            Some("contact_saves") => TeamStatsSort::ContactSaves,
            Some("link_clicks") => TeamStatsSort::LinkClicks,
            _ => TeamStatsSort::Visits,
        }
    }

    fn column(self) -> &'static str {
        match self {
            TeamStatsSort::Visits => "visits",
            TeamStatsSort::QrScans => "qr_scans",
            TeamStatsSort::ContactSaves => "contact_saves",
            TeamStatsSort::LinkClicks => "link_clicks",
        }
    }
}

/// Per-card aggregate stats across all of a team's members (PRD §6.7.3).
/// Sorted descending by the chosen (whitelisted) metric.
pub async fn team_card_stats(
    db: &Db,
    team_id: Uuid,
    sort: TeamStatsSort,
) -> Result<Vec<TeamCardStats>, sqlx::Error> {
    // The ORDER BY column comes from the whitelist above — never user input.
    let sql = format!(
        r#"
        SELECT c.id AS card_id, c.handle, u.display_name AS owner_display_name,
          COUNT(e.id) FILTER (WHERE e.event_type = 'profile_visit') AS visits,
          COUNT(e.id) FILTER (WHERE e.event_type = 'qr_scan')       AS qr_scans,
          COUNT(e.id) FILTER (WHERE e.event_type = 'contact_save')  AS contact_saves,
          COUNT(e.id) FILTER (WHERE e.event_type = 'link_click')    AS link_clicks
        FROM cards c
        JOIN team_memberships m ON m.user_id = c.owner_id AND m.team_id = $1
        JOIN users u ON u.id = c.owner_id
        LEFT JOIN analytics_events e ON e.card_id = c.id
        GROUP BY c.id, c.handle, u.display_name
        ORDER BY {} DESC, c.handle ASC
        "#,
        sort.column()
    );
    sqlx::query_as(&sql).bind(team_id).fetch_all(db).await
}

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
