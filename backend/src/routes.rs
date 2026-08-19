use axum::Router;
use axum::routing::{get, post};

use crate::handlers::{movies, songs};
use crate::state::AppState;

pub fn build(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route(
            "/movies",
            post(movies::create_movie).get(movies::list_movies),
        )
        .route("/movies/{id}", get(movies::get_movie))
        .route(
            "/movies/{id}/scenes",
            post(movies::create_scene).get(movies::list_scenes),
        )
        .route("/songs", post(songs::create_song).get(songs::list_songs))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}
