//! Share-link creation + resolution (PRD §6.5.2, §17.1). Tokens are opaque
//! 12-char base62 ids that resolve via DB lookup — they encode nothing, so the
//! token structure leaks nothing.

use chrono::Utc;
use rand::Rng;
use uuid::Uuid;

use cardclaws_db::models::share_link::ShareLinkRow;
use cardclaws_db::queries::{cards, share};

use cardclaws_types::AppError;

use crate::error::SqlxResultExt;
use crate::services::{analytics_service, card_service};
use crate::state::AppState;

const BASE62: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const TOKEN_LEN: usize = 12;

/// Share modalities (PRD §6.5.1). The modality is recorded so analytics can
/// attribute a visit to how it was shared.
const MODALITIES: &[&str] = &[
    "nfc", "qr", "airdrop", "imessage", "email", "link", "wallet", "contact",
];

fn gen_token() -> String {
    let mut rng = rand::thread_rng();
    (0..TOKEN_LEN)
        .map(|_| BASE62[rng.gen_range(0..BASE62.len())] as char)
        .collect()
}

pub async fn create(
    state: &AppState,
    card_id: Uuid,
    user_id: Uuid,
    modality: &str,
    campaign: Option<&str>,
) -> Result<ShareLinkRow, AppError> {
    card_service::get_owned(state, card_id, user_id).await?;
    if !MODALITIES.contains(&modality) {
        return Err(AppError::Validation(format!(
            "unknown modality: {modality}"
        )));
    }

    // Retry on the (vanishingly unlikely) token collision.
    for _ in 0..5 {
        let token = gen_token();
        match share::insert(
            &state.db,
            share::NewShareLink {
                token: &token,
                card_id,
                modality,
                campaign,
                expires_at: None,
            },
        )
        .await
        {
            Ok(row) => return Ok(row),
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() => continue,
            Err(e) => return Err(AppError::Internal(format!("db: {e}"))),
        }
    }
    Err(AppError::Internal(
        "could not allocate a share token".into(),
    ))
}

pub async fn list(
    state: &AppState,
    card_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<ShareLinkRow>, AppError> {
    card_service::get_owned(state, card_id, user_id).await?;
    share::list_by_card(&state.db, card_id).await.map_db()
}

/// Resolve a share token: record the attributed analytics event and return the
/// profile URL to redirect to. Unknown or expired tokens are `NotFound`.
pub async fn resolve(
    state: &AppState,
    token: &str,
    ip: Option<&str>,
    user_agent: Option<&str>,
) -> Result<String, AppError> {
    let link = share::find_by_token(&state.db, token)
        .await
        .map_db()?
        .ok_or_else(|| AppError::NotFound("share link".into()))?;

    if let Some(exp) = link.expires_at {
        if exp < Utc::now() {
            return Err(AppError::NotFound("share link".into()));
        }
    }

    let card = cards::find_by_id(&state.db, link.card_id)
        .await
        .map_db()?
        .ok_or_else(|| AppError::NotFound("card".into()))?;

    // Best-effort attributed event; never block the redirect on analytics.
    let _ = analytics_service::record(
        state,
        link.card_id,
        event_type_for(&link.modality),
        Some(token),
        ip,
        user_agent,
    )
    .await;

    Ok(format!("{}/{}", state.profile_base_url, card.handle))
}

/// Map a share modality to the analytics event type it produces (PRD §12.2).
fn event_type_for(modality: &str) -> &'static str {
    match modality {
        "qr" => "qr_scan",
        "nfc" => "nfc_tap",
        _ => "profile_visit",
    }
}
