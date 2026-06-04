//! Google Wallet generic pass (PRD §10.3). The "Add to Google Wallet" button
//! deep-links to `https://pay.google.com/gp/v/save/{JWT}`, where the JWT is an
//! RS256-signed `savetowallet` token carrying the GenericObject inline (so no
//! Google Wallet API round-trip is needed to mint the link).

pub mod jwt_signer;
pub mod object_builder;

use crate::error::WalletError;
use jwt_signer::GoogleWalletSigner;

const SAVE_URL_PREFIX: &str = "https://pay.google.com/gp/v/save/";

/// Inputs for one Google Wallet object (mirrors the Apple `PassInput`).
pub struct GoogleInput {
    /// Service account email (the JWT `iss`).
    pub issuer_email: String,
    /// Fully-qualified class id, e.g. `3388000000022195611.cardclaws_generic`.
    pub class_id: String,
    /// Fully-qualified object id, e.g. `3388000000022195611.<card-id>`.
    pub object_id: String,
    pub holder_name: String,
    pub title: Option<String>,
    pub company: Option<String>,
    pub profile_url: String,
    pub background_hex: String,
}

/// Build the full "Add to Google Wallet" save URL.
pub fn build_save_link(
    input: &GoogleInput,
    signer: &dyn GoogleWalletSigner,
) -> Result<String, WalletError> {
    let claims = object_builder::build_claims(input);
    let jwt = signer.sign(&claims)?;
    Ok(format!("{SAVE_URL_PREFIX}{jwt}"))
}
