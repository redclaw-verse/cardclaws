//! Team management handlers (PRD §21 Phase 4). All require auth; role checks
//! happen in the service.

use axum::extract::{Path, State};
use axum::Json;
use cardclaws_db::models::team::{MemberRow, TeamRow};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::ApiResult;
use crate::middleware::auth::AuthUser;
use crate::services::team_service;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct CreateTeamRequest {
    pub name: String,
    #[serde(default)]
    pub seat_limit: Option<i32>,
}

#[derive(Deserialize)]
pub struct AddMemberRequest {
    pub email: String,
    #[serde(default = "default_role")]
    pub role: String,
}

fn default_role() -> String {
    "member".to_string()
}

pub async fn create_team(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateTeamRequest>,
) -> ApiResult<Json<TeamRow>> {
    Ok(Json(
        team_service::create_team(&state, user.user_id, &req.name, req.seat_limit).await?,
    ))
}

pub async fn get_team(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<TeamRow>> {
    Ok(Json(
        team_service::get_team(&state, id, user.user_id).await?,
    ))
}

pub async fn list_members(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<MemberRow>>> {
    Ok(Json(
        team_service::list_members(&state, id, user.user_id).await?,
    ))
}

pub async fn add_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<AddMemberRequest>,
) -> ApiResult<Json<Value>> {
    team_service::add_member(&state, id, user.user_id, &req.email, &req.role).await?;
    Ok(Json(json!({ "status": "added" })))
}

pub async fn remove_member(
    State(state): State<AppState>,
    user: AuthUser,
    Path((id, target_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<Value>> {
    team_service::remove_member(&state, id, user.user_id, target_id).await?;
    Ok(Json(json!({ "status": "removed" })))
}
