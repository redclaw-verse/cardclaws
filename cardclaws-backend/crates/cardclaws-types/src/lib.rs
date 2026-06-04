//! Shared types across all CardClaws backend crates.
//!
//! This crate holds the canonical data shapes (users, cards, auth DTOs) and the
//! single application error enum that every crate maps into. It has no runtime
//! dependencies beyond serde/uuid/chrono so it can be linked by leaf crates.

pub mod auth;
pub mod card;
pub mod errors;
pub mod user;

pub use errors::{AppError, ErrorCode};
pub use user::{Tier, User};
