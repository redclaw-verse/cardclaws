//! Apple `.pkpass` generation.

pub mod manifest;
pub mod packager;
pub mod pass_builder;
pub mod signer;

use std::collections::BTreeMap;

use crate::error::WalletError;
use crate::strip_renderer;
use signer::PassSigner;

/// Everything needed to build one pass. Identity (`pass_type_id`, `team_id`) and
/// brand assets come from config; the rest from the card.
pub struct PassInput {
    pub serial_number: String,
    pub pass_type_id: String,
    pub team_id: String,
    pub organization_name: String,
    pub holder_name: String,
    pub title: Option<String>,
    pub company: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    /// The profile URL encoded in the QR barcode (PRD §6.3.1).
    pub profile_url: String,
    /// Hex background color used to render the strip image.
    pub background_hex: String,
}

/// Brand assets bundled into every pass (CardClaws icon + logo PNGs).
pub struct BrandAssets {
    pub icon_png: Vec<u8>,
    pub logo_png: Vec<u8>,
}

/// Build a complete, signed `.pkpass` zip in memory.
///
/// Pipeline (PRD §10.1, corrected): build pass.json → render strip → assemble
/// the file set → hash every file into manifest.json → sign the manifest →
/// zip it all.
pub fn build_pkpass(
    input: &PassInput,
    brand: &BrandAssets,
    signer: &dyn PassSigner,
) -> Result<Vec<u8>, WalletError> {
    let pass_json = serde_json::to_vec(&pass_builder::build_pass_json(input))
        .map_err(|e| WalletError::Build(e.to_string()))?;
    let strip_png = strip_renderer::render_strip(&input.background_hex)?;

    // BTreeMap keeps file ordering deterministic (stable manifest + zip).
    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    files.insert("pass.json".into(), pass_json);
    files.insert("icon.png".into(), brand.icon_png.clone());
    files.insert("logo.png".into(), brand.logo_png.clone());
    files.insert("strip.png".into(), strip_png);

    let manifest_json = manifest::build_manifest(&files);
    let signature = signer.sign_manifest(manifest_json.as_bytes())?;

    packager::package(&files, &manifest_json, &signature)
}
