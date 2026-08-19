mod common;

use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;

#[sqlx::test]
async fn rejects_a_track_with_an_invalid_time_range(pool: PgPool) {
    let app = common::app(pool);
    let (_, scene_id, song_id) = common::seed_movie_scene_song(&app).await;

    let (status, body) = common::request(
        app,
        "POST",
        &format!("/scenes/{scene_id}/tracks"),
        Some(json!({ "song_id": song_id, "start_time_ms": 100, "end_time_ms": 50 })),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("end_time_ms"));
}

#[sqlx::test]
async fn lists_tracks_by_scene_and_by_movie(pool: PgPool) {
    let app = common::app(pool);
    let (movie_id, scene_id, song_id) = common::seed_movie_scene_song(&app).await;

    common::request(
        app.clone(),
        "POST",
        &format!("/scenes/{scene_id}/tracks"),
        Some(json!({ "song_id": song_id, "start_time_ms": 750_000, "end_time_ms": 840_000 })),
    )
    .await;

    let (status, body) = common::request(
        app.clone(),
        "GET",
        &format!("/scenes/{scene_id}/tracks"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let tracks = body.as_array().unwrap();
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0]["song"]["title"], "Eye of the Tiger");

    let (status, body) =
        common::request(app, "GET", &format!("/movies/{movie_id}/tracks"), None).await;
    assert_eq!(status, StatusCode::OK);
    let scenes = body.as_array().unwrap();
    assert_eq!(scenes.len(), 1);
    assert_eq!(scenes[0]["scene"]["id"], scene_id);
    assert_eq!(scenes[0]["tracks"].as_array().unwrap().len(), 1);
}

#[sqlx::test]
async fn rejects_an_out_of_order_license_transition(pool: PgPool) {
    let app = common::app(pool);
    let (_, scene_id, song_id) = common::seed_movie_scene_song(&app).await;

    let (_, track) = common::request(
        app.clone(),
        "POST",
        &format!("/scenes/{scene_id}/tracks"),
        Some(json!({ "song_id": song_id, "start_time_ms": 750_000, "end_time_ms": 840_000 })),
    )
    .await;
    let track_id = track["id"].as_str().unwrap();
    assert_eq!(track["license_status"], "Draft");

    // Draft -> Licensed skips Requested/InNegotiation/Approved.
    let (status, body) = common::request(
        app,
        "PATCH",
        &format!("/tracks/{track_id}/license"),
        Some(json!({ "status": "Licensed" })),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body["error"].as_str().unwrap().contains("Draft"));
}

#[sqlx::test]
async fn a_valid_transition_updates_status_and_appends_history(pool: PgPool) {
    let app = common::app(pool);
    let (_, scene_id, song_id) = common::seed_movie_scene_song(&app).await;

    let (_, track) = common::request(
        app.clone(),
        "POST",
        &format!("/scenes/{scene_id}/tracks"),
        Some(json!({ "song_id": song_id, "start_time_ms": 750_000, "end_time_ms": 840_000 })),
    )
    .await;
    let track_id = track["id"].as_str().unwrap().to_string();

    let (status, body) = common::request(
        app.clone(),
        "PATCH",
        &format!("/tracks/{track_id}/license"),
        Some(json!({ "status": "Requested", "note": "Sent to Sony legal team" })),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["license_status"], "Requested");

    let history = body["history"].as_array().unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0]["from_status"], "Draft");
    assert_eq!(history[0]["to_status"], "Requested");
    assert_eq!(history[0]["note"], "Sent to Sony legal team");

    // GET should reflect the same state, not just the PATCH response.
    let (status, body) = common::request(app, "GET", &format!("/tracks/{track_id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["license_status"], "Requested");
}
