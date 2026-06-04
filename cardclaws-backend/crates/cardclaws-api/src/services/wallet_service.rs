//! Builds an Apple Wallet `.pkpass` for a card (PRD §10.1). Pulls contact data
//! out of the card definition (reusing the vCard extractor) and the holder name
//! from the account.

use cardclaws_db::queries::users;
use cardclaws_types::AppError;
use cardclaws_wallet::apple::{build_pkpass, PassInput};
use uuid::Uuid;

use crate::error::SqlxResultExt;
use crate::services::{card_service, vcard};
use crate::state::AppState;

pub async fn apple_pkpass(
    state: &AppState,
    card_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<u8>, AppError> {
    let card = card_service::get_owned(state, card_id, user_id).await?;
    let owner = users::find_by_id(&state.db, card.owner_id)
        .await
        .map_db()?
        .ok_or_else(|| AppError::Internal("card owner missing".into()))?;

    let contact = vcard::extract_contact(&card.definition);
    let input = PassInput {
        serial_number: card.id.to_string(),
        pass_type_id: state.wallet.apple_pass_type_id.clone(),
        team_id: state.wallet.apple_team_id.clone(),
        organization_name: state.wallet.organization_name.clone(),
        holder_name: owner.display_name,
        title: contact.title,
        company: contact.company,
        email: contact.email,
        phone: contact.phone,
        website: contact.website,
        profile_url: format!("{}/{}", state.profile_base_url, card.handle),
        background_hex: background_hex(&card.definition),
    };

    build_pkpass(&input, &state.brand, state.pass_signer.as_ref())
        .map_err(|e| AppError::Internal(e.to_string()))
}

/// Extract the face background color (solid) from the definition, defaulting to
/// the CardClaws dark base when absent or non-solid.
fn background_hex(definition: &serde_json::Value) -> String {
    definition
        .get("face")
        .and_then(|f| f.get("background"))
        .and_then(|b| {
            if b.get("type").and_then(|t| t.as_str()) == Some("solid") {
                b.get("value").and_then(|v| v.as_str())
            } else {
                None
            }
        })
        .unwrap_or("#101014")
        .to_string()
}
