//! Team management (PRD §21 Phase 4): create teams, manage members with roles,
//! and enforce the seat cap. Only Team/Enterprise tiers may own a team.

use cardclaws_db::models::team::{MemberRow, TeamCardStats, TeamRow, TeamTemplateRow};
use cardclaws_db::queries::teams::TeamStatsSort;
use cardclaws_db::queries::{teams, users};
use cardclaws_types::AppError;
use uuid::Uuid;

use crate::error::SqlxResultExt;
use crate::state::AppState;

/// Create a team owned by `owner_id` (who becomes the first member, role owner).
pub async fn create_team(
    state: &AppState,
    owner_id: Uuid,
    name: &str,
    seat_limit: Option<i32>,
) -> Result<TeamRow, AppError> {
    if name.trim().is_empty() {
        return Err(AppError::Validation("team name is required".into()));
    }
    let owner = users::find_by_id(&state.db, owner_id)
        .await
        .map_db()?
        .ok_or(AppError::Unauthorized)?;
    if !owner.tier.can_own_team() {
        return Err(AppError::TierLimit(
            "creating a team requires the Team plan".into(),
        ));
    }
    if let Some(limit) = seat_limit {
        if limit < 1 {
            return Err(AppError::Validation("seat_limit must be at least 1".into()));
        }
    }

    let team = teams::insert_team(&state.db, name, owner_id, seat_limit)
        .await
        .map_db()?;
    teams::insert_membership(&state.db, team.id, owner_id, "owner")
        .await
        .map_db()?;
    Ok(team)
}

pub async fn get_team(
    state: &AppState,
    team_id: Uuid,
    actor_id: Uuid,
) -> Result<TeamRow, AppError> {
    require_member(state, team_id, actor_id).await?;
    teams::find_team(&state.db, team_id)
        .await
        .map_db()?
        .ok_or_else(|| AppError::NotFound("team".into()))
}

pub async fn list_members(
    state: &AppState,
    team_id: Uuid,
    actor_id: Uuid,
) -> Result<Vec<MemberRow>, AppError> {
    require_member(state, team_id, actor_id).await?;
    teams::list_members(&state.db, team_id).await.map_db()
}

/// Add a member by email. Admin/owner only; enforces the seat cap. The invitee
/// must already have a CardClaws account (pending-invite signup is future work).
pub async fn add_member(
    state: &AppState,
    team_id: Uuid,
    actor_id: Uuid,
    email: &str,
    role: &str,
) -> Result<(), AppError> {
    require_admin(state, team_id, actor_id).await?;
    if !matches!(role, "member" | "admin") {
        return Err(AppError::Validation("role must be member or admin".into()));
    }

    let team = teams::find_team(&state.db, team_id)
        .await
        .map_db()?
        .ok_or_else(|| AppError::NotFound("team".into()))?;

    if let Some(limit) = team.seat_limit {
        let count = teams::count_members(&state.db, team_id).await.map_db()?;
        if count >= limit as i64 {
            return Err(AppError::TierLimit(format!(
                "team is at its {limit}-seat limit; upgrade to add more"
            )));
        }
    }

    let invitee = users::find_by_email(&state.db, email)
        .await
        .map_db()?
        .ok_or_else(|| AppError::NotFound("no CardClaws account for that email".into()))?;

    teams::insert_membership(&state.db, team_id, invitee.id, role)
        .await
        .map_err(map_membership_conflict)?;

    let html = format!(
        "<p>You've been added to the team <strong>{}</strong> on CardClaws.</p>",
        team.name
    );
    let _ = state
        .email
        .send(
            &invitee.email,
            "You've been added to a CardClaws team",
            &html,
        )
        .await;
    Ok(())
}

pub async fn remove_member(
    state: &AppState,
    team_id: Uuid,
    actor_id: Uuid,
    target_id: Uuid,
) -> Result<(), AppError> {
    require_admin(state, team_id, actor_id).await?;
    // The owner can't be removed (transfer/delete the team instead).
    if teams::member_role(&state.db, team_id, target_id)
        .await
        .map_db()?
        .as_deref()
        == Some("owner")
    {
        return Err(AppError::Forbidden);
    }
    let removed = teams::delete_membership(&state.db, team_id, target_id)
        .await
        .map_db()?;
    if removed == 0 {
        return Err(AppError::NotFound("member".into()));
    }
    Ok(())
}

/// Per-card aggregate analytics across the team (admin/owner only, PRD §6.7.3).
pub async fn aggregate_analytics(
    state: &AppState,
    team_id: Uuid,
    actor_id: Uuid,
    sort: TeamStatsSort,
) -> Result<Vec<TeamCardStats>, AppError> {
    require_admin(state, team_id, actor_id).await?;
    teams::team_card_stats(&state.db, team_id, sort)
        .await
        .map_db()
}

// ---- Template library (PRD §6.1.4) ----------------------------------------

pub async fn create_template(
    state: &AppState,
    team_id: Uuid,
    actor_id: Uuid,
    name: &str,
    definition: &serde_json::Value,
) -> Result<TeamTemplateRow, AppError> {
    require_admin(state, team_id, actor_id).await?;
    if name.trim().is_empty() {
        return Err(AppError::Validation("template name is required".into()));
    }
    if !definition.is_object() {
        return Err(AppError::Validation(
            "definition must be a JSON object".into(),
        ));
    }
    teams::insert_template(&state.db, team_id, name, definition, actor_id)
        .await
        .map_db()
}

pub async fn list_templates(
    state: &AppState,
    team_id: Uuid,
    actor_id: Uuid,
) -> Result<Vec<TeamTemplateRow>, AppError> {
    require_member(state, team_id, actor_id).await?;
    teams::list_templates(&state.db, team_id).await.map_db()
}

pub async fn delete_template(
    state: &AppState,
    team_id: Uuid,
    actor_id: Uuid,
    template_id: Uuid,
) -> Result<(), AppError> {
    require_admin(state, team_id, actor_id).await?;
    let removed = teams::delete_template(&state.db, team_id, template_id)
        .await
        .map_db()?;
    if removed == 0 {
        return Err(AppError::NotFound("template".into()));
    }
    Ok(())
}

// ---- Authorization helpers ------------------------------------------------

async fn require_member(
    state: &AppState,
    team_id: Uuid,
    user_id: Uuid,
) -> Result<String, AppError> {
    teams::member_role(&state.db, team_id, user_id)
        .await
        .map_db()?
        // NotFound (not Forbidden) so team existence isn't leaked to non-members.
        .ok_or_else(|| AppError::NotFound("team".into()))
}

async fn require_admin(state: &AppState, team_id: Uuid, user_id: Uuid) -> Result<(), AppError> {
    let role = require_member(state, team_id, user_id).await?;
    if role == "owner" || role == "admin" {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

fn map_membership_conflict(e: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &e {
        if db_err.is_unique_violation() {
            return AppError::Conflict("already a member of this team".into());
        }
    }
    AppError::Internal(format!("db: {e}"))
}
