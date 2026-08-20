use axum::Router;
use axum::routing::{get, patch, post};
use tower_http::cors::CorsLayer;

use crate::handlers::{events, movies, songs, tracks};
use crate::paths;
use crate::state::AppState;

pub fn build(state: AppState) -> Router {
    Router::new()
        .route(paths::HEALTH, get(health))
        .route(paths::EVENTS, get(events::stream))
        .route(
            paths::MOVIES,
            post(movies::create_movie).get(movies::list_movies),
        )
        .route(paths::MOVIE_BY_ID, get(movies::get_movie))
        .route(
            paths::MOVIE_SCENES,
            post(movies::create_scene).get(movies::list_scenes),
        )
        .route(paths::MOVIE_TRACKS, get(tracks::list_tracks_for_movie))
        .route(
            paths::SONGS,
            post(songs::create_song).get(songs::list_songs),
        )
        .route(
            paths::SCENE_TRACKS,
            post(tracks::create_track).get(tracks::list_tracks_for_scene),
        )
        .route(paths::TRACK_BY_ID, get(tracks::get_track))
        .route(paths::TRACK_LICENSE, patch(tracks::update_license))
        // No auth/cookies to protect, so a permissive policy is enough to
        // let a browser-based frontend on a different origin call this API
        // (including EventSource against GET /events).
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}
