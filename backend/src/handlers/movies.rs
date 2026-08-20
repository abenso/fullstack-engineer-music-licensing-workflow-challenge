use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::entities::{Movie, Scene};
use crate::error::AppError;
use crate::repositories::{movies, scenes};
use crate::state::AppState;
use crate::validation::require_non_blank;

#[derive(Debug, Deserialize)]
pub struct CreateMovieRequest {
    pub title: String,
}

pub async fn create_movie(
    State(state): State<AppState>,
    Json(payload): Json<CreateMovieRequest>,
) -> Result<Json<Movie>, AppError> {
    require_non_blank("title", &payload.title)?;

    let created = movies::create(&state.pool, &payload.title).await?;
    Ok(Json(created))
}

pub async fn list_movies(State(state): State<AppState>) -> Result<Json<Vec<Movie>>, AppError> {
    let all = movies::list(&state.pool).await?;
    Ok(Json(all))
}

pub async fn get_movie(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Movie>, AppError> {
    let found = movies::find_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(found))
}

#[derive(Debug, Deserialize)]
pub struct CreateSceneRequest {
    pub name: String,
}

pub async fn create_scene(
    State(state): State<AppState>,
    Path(movie_id): Path<Uuid>,
    Json(payload): Json<CreateSceneRequest>,
) -> Result<Json<Scene>, AppError> {
    require_non_blank("name", &payload.name)?;

    // Fail with a clear 404 instead of letting the foreign key violation
    // surface as an opaque 500 if the movie doesn't exist.
    movies::find_by_id(&state.pool, movie_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let created = scenes::create(&state.pool, movie_id, &payload.name).await?;
    Ok(Json(created))
}

pub async fn list_scenes(
    State(state): State<AppState>,
    Path(movie_id): Path<Uuid>,
) -> Result<Json<Vec<Scene>>, AppError> {
    let all = scenes::list_for_movie(&state.pool, movie_id).await?;
    Ok(Json(all))
}
