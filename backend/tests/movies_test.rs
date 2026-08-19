mod common;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test]
async fn creates_and_fetches_a_movie(pool: PgPool) {
    let app = common::app(pool);

    let (status, body) = common::request(
        app.clone(),
        "POST",
        "/movies",
        Some(json!({ "title": "Rocky IV" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["title"], "Rocky IV");
    let movie_id = body["id"].as_str().unwrap();

    let (status, body) = common::request(app, "GET", &format!("/movies/{movie_id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], movie_id);
}

#[sqlx::test]
async fn unknown_movie_returns_404(pool: PgPool) {
    let app = common::app(pool);

    let (status, body) = common::request(
        app,
        "GET",
        "/movies/00000000-0000-0000-0000-000000000000",
        None,
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "resource not found");
}

#[sqlx::test]
async fn creating_a_scene_under_an_unknown_movie_returns_404(pool: PgPool) {
    let app = common::app(pool);

    let (status, _) = common::request(
        app,
        "POST",
        "/movies/00000000-0000-0000-0000-000000000000/scenes",
        Some(json!({ "name": "Training Montage" })),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}
