mod common;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test]
async fn creates_and_lists_songs(pool: PgPool) {
    let app = common::app(pool);

    let (status, song) = common::request(
        app.clone(),
        "POST",
        "/songs",
        Some(json!({ "title": "Eye of the Tiger", "artist": "Survivor", "rights_holder": "Sony" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let song_id = song["id"].as_str().unwrap();

    let (status, body) = common::request(app, "GET", "/songs", None).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.as_array().unwrap().iter().any(|s| s["id"] == song_id));
}

#[sqlx::test]
async fn rejects_a_blank_song_field(pool: PgPool) {
    let app = common::app(pool);

    let (status, body) = common::request(
        app,
        "POST",
        "/songs",
        Some(json!({ "title": "Eye of the Tiger", "artist": "", "rights_holder": "Sony" })),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("artist"));
}
