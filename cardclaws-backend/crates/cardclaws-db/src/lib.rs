//! Database access layer (sqlx / PostgreSQL).
//!
//! Queries use the runtime `query_as` API (not the compile-time `query!`
//! macros) so the workspace builds without a live database or an offline query
//! cache. Each `queries` module owns the SQL for one table.

pub mod models;
pub mod pool;
pub mod queries;

pub use pool::{connect, migrate, Db};
