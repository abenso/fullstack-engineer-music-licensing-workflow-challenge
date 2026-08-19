use axum::Router;
use axum::routing::{get, post};

use crate::handlers::{movies, songs};
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
        .route(
            paths::SONGS,
            post(songs::create_song).get(songs::list_songs),
        )
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}
