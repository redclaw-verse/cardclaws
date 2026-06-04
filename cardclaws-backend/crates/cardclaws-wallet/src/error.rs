use thiserror::Error;

#[derive(Debug, Error)]
pub enum WalletError {
    #[error("pass build error: {0}")]
    Build(String),
    #[error("strip render error: {0}")]
    Render(String),
    #[error("manifest error: {0}")]
    Manifest(String),
    #[error("signing error: {0}")]
    Signing(String),
    #[error("packaging error: {0}")]
    Packaging(String),
}
