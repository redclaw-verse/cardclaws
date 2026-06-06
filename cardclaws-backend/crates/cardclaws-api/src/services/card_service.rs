//! Card CRUD business logic (PRD §13.2). Ownership is enforced on every
//! mutating/owned-read path; non-owned access returns `NotFound` rather than
//! `Forbidden` so card existence is not leaked.

use base64::Engine;
use cardclaws_db::models::card::CardRow;
use cardclaws_db::queries::{cards, users};
use cardclaws_types::{AppError, Tier};
use uuid::Uuid;

use crate::error::SqlxResultExt;
use crate::services::vcard;
use crate::state::AppState;

/// Load a card the caller owns, or `NotFound`.
pub async fn get_owned(state: &AppState, id: Uuid, user_id: Uuid) -> Result<CardRow, AppError> {
    let card = cards::find_by_id(&state.db, id)
        .await
        .map_db()?
        .filter(|c| c.owner_id == user_id)
        .ok_or_else(|| AppError::NotFound("card".into()))?;
    Ok(card)
}

/// Generate an AI welcome image (nano-banana) and attach it to the card the
/// caller owns — shown on the public profile when the QR is scanned.
pub async fn set_welcome(
    state: &AppState,
    id: Uuid,
    user_id: Uuid,
    prompt: &str,
    message: Option<&str>,
    style: Option<&str>,
    mood: Option<&str>,
) -> Result<CardRow, AppError> {
    get_owned(state, id, user_id).await?;
    if prompt.trim().is_empty() {
        return Err(AppError::Validation("prompt is required".into()));
    }
    let brief = crate::ai::SceneBrief {
        scene: prompt.to_string(),
        style: style.map(str::to_string),
        mood: mood.map(str::to_string),
        persona: None,
    };
    let refined = state.ai.refine_prompt(&brief).await.map_err(ai_to_app)?;
    let img = state.ai.generate_image(&refined).await.map_err(ai_to_app)?;

    // Upload the generated image to object storage (R2 in prod, MinIO in dev)
    // and store its public URL — keeps the profile payload tiny.
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(img.base64.as_bytes())
        .map_err(|e| AppError::Internal(format!("decode image: {e}")))?;
    let ext = if img.mime_type.contains("jpeg") {
        "jpg"
    } else {
        "png"
    };
    let key = format!("welcome/{id}.{ext}");
    state
        .assets
        .put_object(&key, bytes, &img.mime_type)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    let welcome = serde_json::json!({
        "kind": "image",
        "message": message.unwrap_or("Great to meet you"),
        "imageUrl": state.assets.public_url(&key),
    });
    cards::update_welcome(&state.db, id, &welcome)
        .await
        .map_db()
}

fn ai_to_app(e: crate::ai::AiError) -> AppError {
    AppError::Internal(e.to_string())
}

/// List the "We Met" connections captured for a card the caller owns.
pub async fn list_connections(
    state: &AppState,
    id: Uuid,
    user_id: Uuid,
) -> Result<Vec<cardclaws_db::models::connection::ConnectionRow>, AppError> {
    get_owned(state, id, user_id).await?;
    cardclaws_db::queries::connections::list_by_card(&state.db, id)
        .await
        .map_db()
}

pub async fn list(state: &AppState, user_id: Uuid) -> Result<Vec<CardRow>, AppError> {
    cards::list_by_owner(&state.db, user_id).await.map_db()
}

pub async fn create(
    state: &AppState,
    user_id: Uuid,
    handle: &str,
    definition: &serde_json::Value,
) -> Result<CardRow, AppError> {
    crate::validation::validate_handle(handle)?;
    require_object(definition)?;
    enforce_layer_tier(definition, current_tier(state, user_id).await?)?;

    cards::insert(
        &state.db,
        cards::NewCard {
            owner_id: user_id,
            handle,
            definition,
        },
    )
    .await
    .map_err(map_handle_conflict)
}

pub async fn replace(
    state: &AppState,
    id: Uuid,
    user_id: Uuid,
    definition: &serde_json::Value,
) -> Result<CardRow, AppError> {
    get_owned(state, id, user_id).await?;
    require_object(definition)?;
    enforce_layer_tier(definition, current_tier(state, user_id).await?)?;
    cards::update_definition(&state.db, id, definition)
        .await
        .map_db()
}

/// Shallow-merge the provided top-level keys into the existing definition.
pub async fn patch(
    state: &AppState,
    id: Uuid,
    user_id: Uuid,
    partial: &serde_json::Value,
) -> Result<CardRow, AppError> {
    let existing = get_owned(state, id, user_id).await?;
    let mut merged = existing.definition.clone();
    let (Some(base), Some(patch)) = (merged.as_object_mut(), partial.as_object()) else {
        return Err(AppError::BadRequest(
            "definition and patch must be JSON objects".into(),
        ));
    };
    for (k, v) in patch {
        base.insert(k.clone(), v.clone());
    }
    enforce_layer_tier(&merged, current_tier(state, user_id).await?)?;
    cards::update_definition(&state.db, id, &merged)
        .await
        .map_db()
}

/// Archive (soft-delete): status -> archived. Never destroys the row (§15.2).
pub async fn archive(state: &AppState, id: Uuid, user_id: Uuid) -> Result<CardRow, AppError> {
    get_owned(state, id, user_id).await?;
    cards::set_status(&state.db, id, "archived").await.map_db()
}

/// Publish: enforce the tier's active-card limit, then status -> active.
pub async fn publish(
    state: &AppState,
    id: Uuid,
    user_id: Uuid,
    tier: Tier,
) -> Result<CardRow, AppError> {
    let card = get_owned(state, id, user_id).await?;

    // Already-active cards re-publish freely; only a draft/archived card
    // becoming active consumes a slot.
    if card.status != "active" {
        if let Some(limit) = tier.active_card_limit() {
            let active = cards::count_active_for_owner(&state.db, user_id)
                .await
                .map_db()?;
            if active >= limit {
                return Err(AppError::TierLimit(format!(
                    "your plan allows {limit} active card(s); archive one or upgrade"
                )));
            }
        }
    }

    require_object(&card.definition)?;
    cards::set_status(&state.db, id, "active").await.map_db()
}

pub async fn duplicate(state: &AppState, id: Uuid, user_id: Uuid) -> Result<CardRow, AppError> {
    let src = get_owned(state, id, user_id).await?;
    let new_handle = format!("{}-copy-{}", src.handle, short_id());
    let new_handle = truncate_handle(&new_handle);

    cards::insert(
        &state.db,
        cards::NewCard {
            owner_id: user_id,
            handle: &new_handle,
            definition: &src.definition,
        },
    )
    .await
    .map_err(map_handle_conflict)
}

/// Public, unauthenticated lookup — active cards only (§6.6, review A2).
pub async fn get_public_by_handle(state: &AppState, handle: &str) -> Result<CardRow, AppError> {
    cards::find_active_by_handle(&state.db, handle)
        .await
        .map_db()?
        .ok_or_else(|| AppError::NotFound("card".into()))
}

/// Public profile response: the active card plus the owner's display name (the
/// name the web profile renders in the hero). Used by the Astro profile.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicProfile {
    #[serde(flatten)]
    pub card: CardRow,
    pub owner_display_name: String,
}

pub async fn public_profile(state: &AppState, handle: &str) -> Result<PublicProfile, AppError> {
    let card = get_public_by_handle(state, handle).await?;
    let owner = users::find_by_id(&state.db, card.owner_id)
        .await
        .map_db()?
        .ok_or_else(|| AppError::Internal("card owner missing".into()))?;
    Ok(PublicProfile {
        owner_display_name: owner.display_name,
        card,
    })
}

/// Render the card's contact data as an RFC 6350 vCard (PRD §13.2, §17.3).
pub async fn export_vcf(state: &AppState, id: Uuid, user_id: Uuid) -> Result<String, AppError> {
    let card = get_owned(state, id, user_id).await?;
    let owner = users::find_by_id(&state.db, card.owner_id)
        .await
        .map_db()?
        .ok_or_else(|| AppError::Internal("card owner missing".into()))?;
    let contact = vcard::extract_contact(&card.definition);
    Ok(vcard::build_vcard(&owner.display_name, &contact))
}

// ---- Helpers --------------------------------------------------------------

/// The caller's current tier from the DB (authoritative — reflects webhook
/// upgrades immediately, unlike the tier embedded in the access token).
async fn current_tier(state: &AppState, user_id: Uuid) -> Result<Tier, AppError> {
    let user = users::find_by_id(&state.db, user_id)
        .await
        .map_db()?
        .ok_or(AppError::Unauthorized)?;
    Ok(user.tier)
}

/// Pro-and-above layer types (PRD §6.1.1). Free cards may not use them.
const PRO_LAYER_TYPES: &[&str] = &["video", "particle", "animatedGradient"];

/// Reject Pro-only layer types for tiers that don't allow them.
fn enforce_layer_tier(definition: &serde_json::Value, tier: Tier) -> Result<(), AppError> {
    if tier.allows_pro_layers() {
        return Ok(());
    }
    for side in ["face", "back"] {
        let layers = definition
            .get(side)
            .and_then(|s| s.get("layers"))
            .and_then(|l| l.as_array());
        for layer in layers.into_iter().flatten() {
            if let Some(t) = layer.get("type").and_then(|t| t.as_str()) {
                if PRO_LAYER_TYPES.contains(&t) {
                    return Err(AppError::TierLimit(format!(
                        "{t} layers require a Pro plan"
                    )));
                }
            }
        }
    }
    Ok(())
}

fn require_object(definition: &serde_json::Value) -> Result<(), AppError> {
    if definition.is_object() {
        Ok(())
    } else {
        Err(AppError::Validation(
            "card definition must be a JSON object".into(),
        ))
    }
}

fn map_handle_conflict(e: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &e {
        if db_err.is_unique_violation() {
            return AppError::Conflict("handle already in use".into());
        }
    }
    AppError::Internal(format!("db: {e}"))
}

fn short_id() -> String {
    Uuid::new_v4().simple().to_string()[..8].to_string()
}

fn truncate_handle(handle: &str) -> String {
    handle.chars().take(30).collect()
}
