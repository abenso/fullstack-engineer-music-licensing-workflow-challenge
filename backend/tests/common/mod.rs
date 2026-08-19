use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

use music_licensing_backend::state::AppState;
use music_licensing_backend::{events, routes};

/// Builds the same router the real server uses, wired to the isolated
/// per-test database `#[sqlx::test]` hands us.
pub fn app(pool: PgPool) -> Router {
    routes::build(AppState {
        pool,
        events_tx: events::channel(),
    })
}

/// Sends one request through the router and returns (status, JSON body).
/// `Router` is cheap to `.clone()`, so callers can reuse `app` across calls.
pub async fn request(
    app: Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let body = match body {
        Some(value) => Body::from(value.to_string()),
        None => Body::empty(),
    };

    let http_request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(body)
        .unwrap();

    let response = app.oneshot(http_request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json_body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };

    (status, json_body)
}

/// Creates a movie -> scene -> song chain, returning their ids. Used by
/// tests that only care about the track/license behavior and don't want
/// to repeat this setup every time.
///
/// Each file under tests/ compiles as its own crate, so this module is
/// recompiled per test binary — `#[allow(dead_code)]` because not every
/// binary that includes `common` happens to call this particular helper.
#[allow(dead_code)]
pub async fn seed_movie_scene_song(app: &Router) -> (String, String, String) {
    let (_, movie) = request(
        app.clone(),
        "POST",
        "/movies",
        Some(json!({ "title": "Rocky IV" })),
    )
    .await;
    let movie_id = movie["id"].as_str().unwrap().to_string();

    let (_, scene) = request(
        app.clone(),
        "POST",
        &format!("/movies/{movie_id}/scenes"),
        Some(json!({ "name": "Training Montage" })),
    )
    .await;
    let scene_id = scene["id"].as_str().unwrap().to_string();

    let (_, song) = request(
        app.clone(),
        "POST",
        "/songs",
        Some(json!({ "title": "Eye of the Tiger", "artist": "Survivor", "rights_holder": "Sony" })),
    )
    .await;
    let song_id = song["id"].as_str().unwrap().to_string();

    (movie_id, scene_id, song_id)
}
