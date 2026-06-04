//! Wallet pass generation (PRD §10). Phase 1 implements the Apple `.pkpass`
//! pipeline; Google Wallet (JWT) lands in Phase 2.
//!
//! KEY CORRECTION vs. the PRD draft (plan review A1): the strip image is a
//! **bundled file** inside the `.pkpass` zip and is included in the manifest
//! SHA-1 hashes — it is NOT an `stripImage` URL. The pipeline here renders the
//! strip, writes it into the bundle, hashes every file in the manifest, signs
//! the manifest (PKCS#7 detached), and zips everything.

pub mod apple;
pub mod error;
pub mod strip_renderer;

pub use error::WalletError;
