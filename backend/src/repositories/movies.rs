use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::Movie;

pub async fn create(pool: &PgPool, title: &str) -> sqlx::Result<Movie> {
    sqlx::query_as!(
        Movie,
        r#"
        INSERT INTO movies (title)
        VALUES ($1)
        RETURNING id, title, created_at
        "#,
        title
    )
    .fetch_one(pool)
    .await
}

pub async fn list(pool: &PgPool) -> sqlx::Result<Vec<Movie>> {
    sqlx::query_as!(
        Movie,
        "SELECT id, title, created_at FROM movies ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Movie>> {
    sqlx::query_as!(
        Movie,
        "SELECT id, title, created_at FROM movies WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await
}
