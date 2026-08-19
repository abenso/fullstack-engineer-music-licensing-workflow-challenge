use axum::Router;
use axum::routing::{get, patch, post};

use crate::handlers::{movies, songs, tracks};
use crate::paths;
use crate::state::AppState;

pub fn build(state: AppState) -> Router {
    Router::new()
        .route(paths::HEALTH, get(health))
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
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}
