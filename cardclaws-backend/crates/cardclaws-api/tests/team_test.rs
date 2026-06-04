//! Team data layer + membership management (PRD §21 Phase 4).

mod common;

use axum::http::StatusCode;
use serde_json::json;

use common::TestApp;

/// Register a user, force them to Team tier, and create a team. Returns
/// (token, userId, teamId).
async fn team_owner(app: &TestApp, seat_limit: Option<i32>) -> (String, String, String) {
    let (token, user_id) = app.register_and_user().await;
    app.set_tier(&user_id, "team").await;
    let body = match seat_limit {
        Some(n) => json!({ "name": "RedClaw", "seat_limit": n }),
        None => json!({ "name": "RedClaw" }),
    };
    let (status, team) = app
        .request("POST", "/v1/teams", Some(&token), Some(body))
        .await;
    assert_eq!(status, StatusCode::OK, "create team failed: {team}");
    (token, user_id, team["id"].as_str().unwrap().to_string())
}

/// Register a plain user and return (token, email).
async fn member_account(app: &TestApp) -> (String, String) {
    let (_token, user_id) = app.register_and_user().await;
    let email = app_email(app, &user_id).await;
    (user_id, email)
}

async fn app_email(app: &TestApp, user_id: &str) -> String {
    let id = uuid::Uuid::parse_str(user_id).unwrap();
    let (email,): (String,) = sqlx::query_as("SELECT email FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    email
}

#[tokio::test]
async fn free_tier_cannot_create_team() {
    let app = require_app!();
    let token = app.register_and_token().await; // free
    let (status, body) = app
        .request(
            "POST",
            "/v1/teams",
            Some(&token),
            Some(json!({ "name": "Nope" })),
        )
        .await;
    assert_eq!(status, StatusCode::PAYMENT_REQUIRED);
    assert_eq!(body["code"], "tier_limit");
}

#[tokio::test]
async fn create_team_makes_owner_a_member() {
    let app = require_app!();
    let (token, owner_id, team_id) = team_owner(&app, None).await;

    let (status, members) = app
        .request(
            "GET",
            &format!("/v1/teams/{team_id}/members"),
            Some(&token),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let arr = members.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["userId"], owner_id);
    assert_eq!(arr[0]["role"], "owner");
}

#[tokio::test]
async fn add_and_remove_member() {
    let app = require_app!();
    let (token, _, team_id) = team_owner(&app, None).await;
    let (member_id, member_email) = member_account(&app).await;

    let (add_status, _) = app
        .request(
            "POST",
            &format!("/v1/teams/{team_id}/members"),
            Some(&token),
            Some(json!({ "email": member_email, "role": "admin" })),
        )
        .await;
    assert_eq!(add_status, StatusCode::OK);

    let (_, members) = app
        .request(
            "GET",
            &format!("/v1/teams/{team_id}/members"),
            Some(&token),
            None,
        )
        .await;
    assert_eq!(members.as_array().unwrap().len(), 2);
    // The invitee got a notification email.
    assert!(app
        .email
        .sent
        .lock()
        .unwrap()
        .iter()
        .any(|m| m.to == member_email));

    let (rm_status, _) = app
        .request(
            "DELETE",
            &format!("/v1/teams/{team_id}/members/{member_id}"),
            Some(&token),
            None,
        )
        .await;
    assert_eq!(rm_status, StatusCode::OK);
    let (_, after) = app
        .request(
            "GET",
            &format!("/v1/teams/{team_id}/members"),
            Some(&token),
            None,
        )
        .await;
    assert_eq!(after.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn add_member_unknown_email_is_404() {
    let app = require_app!();
    let (token, _, team_id) = team_owner(&app, None).await;
    let (status, _) = app
        .request(
            "POST",
            &format!("/v1/teams/{team_id}/members"),
            Some(&token),
            Some(json!({ "email": "ghost@nowhere.test", "role": "member" })),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn seat_limit_is_enforced() {
    let app = require_app!();
    // seat_limit = 1; the owner already occupies it.
    let (token, _, team_id) = team_owner(&app, Some(1)).await;
    let (_, member_email) = member_account(&app).await;

    let (status, body) = app
        .request(
            "POST",
            &format!("/v1/teams/{team_id}/members"),
            Some(&token),
            Some(json!({ "email": member_email, "role": "member" })),
        )
        .await;
    assert_eq!(status, StatusCode::PAYMENT_REQUIRED);
    assert_eq!(body["code"], "tier_limit");
}

#[tokio::test]
async fn non_member_cannot_view_or_admin_team() {
    let app = require_app!();
    let (_, _, team_id) = team_owner(&app, None).await;
    let intruder = app.register_and_token().await;

    // Non-member: team existence is hidden (404).
    let (get_status, _) = app
        .request(
            "GET",
            &format!("/v1/teams/{team_id}"),
            Some(&intruder),
            None,
        )
        .await;
    assert_eq!(get_status, StatusCode::NOT_FOUND);

    let (add_status, _) = app
        .request(
            "POST",
            &format!("/v1/teams/{team_id}/members"),
            Some(&intruder),
            Some(json!({ "email": "x@y.test", "role": "member" })),
        )
        .await;
    assert_eq!(add_status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn plain_member_cannot_add_members() {
    let app = require_app!();
    let (owner_token, _, team_id) = team_owner(&app, None).await;
    // Create a plain-member account and add them with role member.
    let (member_token, member_id) = app.register_and_user().await;
    let member_email = app_email(&app, &member_id).await;
    app.request(
        "POST",
        &format!("/v1/teams/{team_id}/members"),
        Some(&owner_token),
        Some(json!({ "email": member_email, "role": "member" })),
    )
    .await;

    // That member (role=member) may not add others → 403 Forbidden.
    let (status, _) = app
        .request(
            "POST",
            &format!("/v1/teams/{team_id}/members"),
            Some(&member_token),
            Some(json!({ "email": "x@y.test", "role": "member" })),
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
