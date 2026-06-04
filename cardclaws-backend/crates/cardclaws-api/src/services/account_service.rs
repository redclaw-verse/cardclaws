//! Account self-service: GDPR data export and account deletion (PRD §18.3).

use cardclaws_db::queries::{export, users};
use cardclaws_types::AppError;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::error::SqlxResultExt;
use crate::state::AppState;

/// Assemble a full export of everything tied to a user.
pub async fn export_data(state: &AppState, user_id: Uuid) -> Result<Value, AppError> {
    let user = export::user_json(&state.db, user_id)
        .await
        .map_db()?
        .ok_or(AppError::Unauthorized)?;

    Ok(json!({
        "user": user,
        "cards": export::cards_json(&state.db, user_id).await.map_db()?,
        "shareLinks": export::share_links_json(&state.db, user_id).await.map_db()?,
        "analyticsEvents": export::analytics_json(&state.db, user_id).await.map_db()?,
        "walletRegistrations": export::wallet_json(&state.db, user_id).await.map_db()?,
        "teamMemberships": export::memberships_json(&state.db, user_id).await.map_db()?,
    }))
}

/// Permanently delete the account and everything it owns (FK cascade).
pub async fn delete_account(state: &AppState, user_id: Uuid) -> Result<(), AppError> {
    let removed = users::delete(&state.db, user_id).await.map_db()?;
    if removed == 0 {
        return Err(AppError::Unauthorized);
    }
    Ok(())
}
