//! CardClaws HTTP API.
//!
//! Exposed as a library so integration tests can build the router with injected
//! test doubles (in-memory cache, capturing email, fake Apple keys). The binary
//! (`main.rs`) wires the production implementations.

pub mod ai;
pub mod assets;
pub mod cache;
pub mod email;
pub mod error;
pub mod geo;
pub mod handlers;
pub mod middleware;
pub mod router;
pub mod services;
pub mod state;
pub mod validation;

pub use router::build_router;
pub use state::AppState;
