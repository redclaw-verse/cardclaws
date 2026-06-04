//! Team management handlers (PRD §21 Phase 4). All require auth; role checks
//! happen in the service.

use axum::extract::{Path, Query, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::Json;
use cardclaws_db::models::team::{MemberRow, TeamCardStats, TeamRow, TeamTemplateRow};
use cardclaws_db::queries::teams::TeamStatsSort;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::ApiResult;
use crate::middleware::auth::AuthUser;
use crate::services::team_service;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct AnalyticsQuery {
    #[serde(default)]
    pub sort: Option<String>,
}

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

pub async fn analytics(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<AnalyticsQuery>,
) -> ApiResult<Json<Vec<TeamCardStats>>> {
    let sort = TeamStatsSort::parse(q.sort.as_deref());
    Ok(Json(
        team_service::aggregate_analytics(&state, id, user.user_id, sort).await?,
    ))
}

/// Same aggregate, exported as CSV (PRD §6.7.3).
pub async fn analytics_csv(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Query(q): Query<AnalyticsQuery>,
) -> ApiResult<Response> {
    let sort = TeamStatsSort::parse(q.sort.as_deref());
    let rows = team_service::aggregate_analytics(&state, id, user.user_id, sort).await?;
    let mut csv = String::from("handle,owner,visits,qr_scans,contact_saves,link_clicks\n");
    for r in &rows {
        csv.push_str(&format!(
            "{},{},{},{},{},{}\n",
            csv_field(&r.handle),
            csv_field(&r.owner_display_name),
            r.visits,
            r.qr_scans,
            r.contact_saves,
            r.link_clicks,
        ));
    }
    Ok((
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"team-analytics.csv\"",
            ),
        ],
        csv,
    )
        .into_response())
}

#[derive(Deserialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub definition: Value,
}

pub async fn create_template(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateTemplateRequest>,
) -> ApiResult<Json<TeamTemplateRow>> {
    Ok(Json(
        team_service::create_template(&state, id, user.user_id, &req.name, &req.definition).await?,
    ))
}

pub async fn list_templates(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<TeamTemplateRow>>> {
    Ok(Json(
        team_service::list_templates(&state, id, user.user_id).await?,
    ))
}

pub async fn delete_template(
    State(state): State<AppState>,
    user: AuthUser,
    Path((id, template_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<Value>> {
    team_service::delete_template(&state, id, user.user_id, template_id).await?;
    Ok(Json(json!({ "status": "deleted" })))
}

/// Quote a CSV field if it contains a comma, quote, or newline (RFC 4180).
fn csv_field(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
