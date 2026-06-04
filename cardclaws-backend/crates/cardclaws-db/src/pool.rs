use std::time::Duration;

use sqlx::postgres::PgPoolOptions;

/// The shared connection pool type used throughout the backend.
pub type Db = sqlx::PgPool;

/// Open a pooled connection to PostgreSQL.
pub async fn connect(database_url: &str) -> Result<Db, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
}

/// Run all pending migrations embedded from `./migrations`.
pub async fn migrate(db: &Db) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(db).await
}
