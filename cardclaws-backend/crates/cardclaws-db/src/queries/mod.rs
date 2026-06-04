//! SQL query functions, one module per table. Functions take `&Db` (or a
//! transaction) and return domain types or row models.

pub mod analytics;
pub mod cards;
pub mod sessions;
pub mod share;
pub mod users;
