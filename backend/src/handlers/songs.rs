use axum::Json;
use axum::extract::State;
use serde::Deserialize;

use crate::domain::entities::Song;
use crate::error::AppError;
use crate::repositories::songs;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateSongRequest {
    pub title: String,
    pub artist: String,
    pub rights_holder: String,
}

pub async fn create_song(
    State(state): State<AppState>,
    Json(payload): Json<CreateSongRequest>,
) -> Result<Json<Song>, AppError> {
    let created = songs::create(
        &state.pool,
        &payload.title,
        &payload.artist,
        &payload.rights_holder,
    )
    .await?;
    Ok(Json(created))
}

pub async fn list_songs(State(state): State<AppState>) -> Result<Json<Vec<Song>>, AppError> {
    let all = songs::list(&state.pool).await?;
    Ok(Json(all))
}
