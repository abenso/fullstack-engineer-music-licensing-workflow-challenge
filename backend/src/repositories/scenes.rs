use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::Scene;

pub async fn create(pool: &PgPool, movie_id: Uuid, name: &str) -> sqlx::Result<Scene> {
    sqlx::query_as!(
        Scene,
        r#"
        INSERT INTO scenes (movie_id, name)
        VALUES ($1, $2)
        RETURNING id, movie_id, name, created_at
        "#,
        movie_id,
        name
    )
    .fetch_one(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Scene>> {
    sqlx::query_as!(
        Scene,
        "SELECT id, movie_id, name, created_at FROM scenes WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_for_movie(pool: &PgPool, movie_id: Uuid) -> sqlx::Result<Vec<Scene>> {
    sqlx::query_as!(
        Scene,
        r#"
        SELECT id, movie_id, name, created_at
        FROM scenes
        WHERE movie_id = $1
        ORDER BY created_at
        "#,
        movie_id
    )
    .fetch_all(pool)
    .await
}
