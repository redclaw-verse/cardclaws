//! Manifest signing. PassKit requires `signature` to be a PKCS#7 **detached**
//! DER signature over `manifest.json`, made with the Pass Type ID certificate,
//! its private key, and the Apple WWDR intermediate in the chain (PRD §20.3).
//!
//! The real signer uses OpenSSL and is gated behind the `apple-signing` feature
//! so default builds/tests stay fast. Tests use [`FakePassSigner`].

use crate::error::WalletError;

pub trait PassSigner: Send + Sync {
    /// Produce the detached PKCS#7 DER signature over `manifest`.
    fn sign_manifest(&self, manifest: &[u8]) -> Result<Vec<u8>, WalletError>;
}

/// Deterministic non-cryptographic signer for tests. Produces a stable,
/// non-empty byte string so bundle assembly can be verified without certs.
pub struct FakePassSigner;

impl PassSigner for FakePassSigner {
    fn sign_manifest(&self, manifest: &[u8]) -> Result<Vec<u8>, WalletError> {
        let mut sig = b"FAKE-PKCS7-SIGNATURE:".to_vec();
        sig.extend_from_slice(&(manifest.len() as u32).to_be_bytes());
        Ok(sig)
    }
}

#[cfg(feature = "apple-signing")]
mod openssl_signer {
    use openssl::pkcs7::{Pkcs7, Pkcs7Flags};
    use openssl::pkey::{PKey, Private};
    use openssl::stack::Stack;
    use openssl::x509::X509;

    use super::PassSigner;
    use crate::error::WalletError;

    /// Production signer holding the Pass Type ID cert + key and the WWDR
    /// intermediate, loaded once at startup and kept in memory (never on disk).
    pub struct OpenSslSigner {
        cert: X509,
        pkey: PKey<Private>,
        wwdr: X509,
    }

    impl OpenSslSigner {
        /// Load from a PKCS#12 (P12) blob (cert + key) and the WWDR intermediate
        /// in PEM. `p12_password` unlocks the P12.
        pub fn from_p12(
            p12_der: &[u8],
            p12_password: &str,
            wwdr_pem: &[u8],
        ) -> Result<Self, WalletError> {
            let p12 = openssl::pkcs12::Pkcs12::from_der(p12_der)
                .map_err(|e| WalletError::Signing(e.to_string()))?;
            let parsed = p12
                .parse2(p12_password)
                .map_err(|e| WalletError::Signing(e.to_string()))?;
            let cert = parsed
                .cert
                .ok_or_else(|| WalletError::Signing("P12 missing certificate".into()))?;
            let pkey = parsed
                .pkey
                .ok_or_else(|| WalletError::Signing("P12 missing private key".into()))?;
            let wwdr = X509::from_pem(wwdr_pem).map_err(|e| WalletError::Signing(e.to_string()))?;
            Ok(Self { cert, pkey, wwdr })
        }
    }

    impl PassSigner for OpenSslSigner {
        fn sign_manifest(&self, manifest: &[u8]) -> Result<Vec<u8>, WalletError> {
            let mut certs = Stack::new().map_err(|e| WalletError::Signing(e.to_string()))?;
            certs
                .push(self.wwdr.clone())
                .map_err(|e| WalletError::Signing(e.to_string()))?;
            // Detached + binary: the signature does not embed the manifest.
            let flags = Pkcs7Flags::DETACHED | Pkcs7Flags::BINARY;
            let pkcs7 = Pkcs7::sign(&self.cert, &self.pkey, &certs, manifest, flags)
                .map_err(|e| WalletError::Signing(e.to_string()))?;
            pkcs7
                .to_der()
                .map_err(|e| WalletError::Signing(e.to_string()))
        }
    }
}

#[cfg(feature = "apple-signing")]
pub use openssl_signer::OpenSslSigner;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_signer_is_deterministic_and_nonempty() {
        let s = FakePassSigner;
        let a = s.sign_manifest(b"manifest").unwrap();
        let b = s.sign_manifest(b"manifest").unwrap();
        assert_eq!(a, b);
        assert!(!a.is_empty());
    }
}
