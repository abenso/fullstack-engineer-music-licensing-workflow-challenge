use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::Song;

pub async fn create(
    pool: &PgPool,
    title: &str,
    artist: &str,
    rights_holder: &str,
) -> sqlx::Result<Song> {
    sqlx::query_as!(
        Song,
        r#"
        INSERT INTO songs (title, artist, rights_holder)
        VALUES ($1, $2, $3)
        RETURNING id, title, artist, rights_holder, created_at
        "#,
        title,
        artist,
        rights_holder
    )
    .fetch_one(pool)
    .await
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Song>> {
    sqlx::query_as!(
        Song,
        "SELECT id, title, artist, rights_holder, created_at FROM songs WHERE id = $1",
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn list(pool: &PgPool) -> sqlx::Result<Vec<Song>> {
    sqlx::query_as!(
        Song,
        "SELECT id, title, artist, rights_holder, created_at FROM songs ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await
}
