use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

/// Connects to Postgres and runs any pending migrations from `migrations/`.
///
/// Migrations run on every startup (idempotent — sqlx tracks what already
/// applied), so `docker compose up` always brings the schema up to date
/// without a separate manual step.
pub async fn connect_and_migrate(database_url: &str) -> PgPool {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .expect("failed to connect to database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("failed to run database migrations");

    pool
}
