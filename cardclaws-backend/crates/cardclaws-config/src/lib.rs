//! Configuration + secret loading.
//!
//! Secrets come from a [`SecretSource`]. In production this is Infisical (PRD
//! §7.2/§9.1); in local dev and tests it is the process environment. Code that
//! needs a secret depends on the trait, never on the concrete source, so tests
//! can inject a fake without any network.

mod loader;

pub use loader::{Config, ConfigError, EnvSecretSource, R2Config, SecretSource, WalletConfig};
