//! Authentication primitives: password hashing (Argon2id), JWT access tokens,
//! opaque refresh/magic-link tokens, and Sign in with Apple verification.
//!
//! This crate is intentionally storage-agnostic — it produces and validates
//! credentials but does not touch the database or Redis. The API service wires
//! these primitives to `cardclaws-db` and the session store.

pub mod apple;
pub mod jwt;
pub mod magic_link;
pub mod password;
pub mod token;

pub use jwt::{JwtError, JwtKeys};
pub use password::PasswordError;
