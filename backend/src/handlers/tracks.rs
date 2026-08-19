use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::license::{LicenseStatus, can_transition};
use crate::domain::track::is_valid_time_range;
use crate::dto::{SceneTracks, TrackDetail, TrackSummary};
use crate::error::AppError;
use crate::events::LicenseStatusChanged;
use crate::repositories::{license_events, scenes, songs, tracks};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateTrackRequest {
    pub song_id: Uuid,
    pub start_time_ms: i32,
    pub end_time_ms: i32,
}

pub async fn create_track(
    State(state): State<AppState>,
    Path(scene_id): Path<Uuid>,
    Json(payload): Json<CreateTrackRequest>,
) -> Result<Json<TrackDetail>, AppError> {
    if !is_valid_time_range(payload.start_time_ms, payload.end_time_ms) {
        return Err(AppError::BadRequest(
            "start_time_ms must be >= 0 and end_time_ms must be greater than start_time_ms"
                .to_string(),
        ));
    }

    scenes::find_by_id(&state.pool, scene_id)
        .await?
        .ok_or(AppError::NotFound)?;

    songs::find_by_id(&state.pool, payload.song_id)
        .await?
        .ok_or(AppError::NotFound)?;

    let created = tracks::create(
        &state.pool,
        scene_id,
        payload.song_id,
        payload.start_time_ms,
        payload.end_time_ms,
    )
    .await?;

    let detail = tracks::find_detail_by_id(&state.pool, created.id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(detail))
}

pub async fn list_tracks_for_scene(
    State(state): State<AppState>,
    Path(scene_id): Path<Uuid>,
) -> Result<Json<Vec<TrackSummary>>, AppError> {
    let all = tracks::list_for_scene(&state.pool, scene_id).await?;
    Ok(Json(all))
}

pub async fn list_tracks_for_movie(
    State(state): State<AppState>,
    Path(movie_id): Path<Uuid>,
) -> Result<Json<Vec<SceneTracks>>, AppError> {
    let all = tracks::list_for_movie(&state.pool, movie_id).await?;
    Ok(Json(all))
}

pub async fn get_track(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TrackDetail>, AppError> {
    let detail = tracks::find_detail_by_id(&state.pool, id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(detail))
}

#[derive(Debug, Deserialize)]
pub struct UpdateLicenseRequest {
    pub status: LicenseStatus,
    pub note: Option<String>,
}

pub async fn update_license(
    State(state): State<AppState>,
    Path(track_id): Path<Uuid>,
    Json(payload): Json<UpdateLicenseRequest>,
) -> Result<Json<TrackDetail>, AppError> {
    let current = tracks::find_by_id(&state.pool, track_id)
        .await?
        .ok_or(AppError::NotFound)?;

    if !can_transition(current.license_status, payload.status) {
        return Err(AppError::InvalidTransition {
            from: current.license_status,
            to: payload.status,
        });
    }

    // The status update and the audit event must land together: begin a
    // transaction, and guard the update on the status we just read still
    // being current. If it isn't (another request changed it in between),
    // roll back and report the *actual* current status instead of silently
    // overwriting someone else's change.
    let mut tx = state.pool.begin().await?;

    let updated = tracks::try_update_license_status(
        &mut *tx,
        track_id,
        current.license_status,
        payload.status,
    )
    .await?;

    if updated.is_none() {
        let actual = tracks::find_by_id(&state.pool, track_id)
            .await?
            .ok_or(AppError::NotFound)?;
        return Err(AppError::InvalidTransition {
            from: actual.license_status,
            to: payload.status,
        });
    }

    license_events::record(
        &mut *tx,
        track_id,
        Some(current.license_status),
        payload.status,
        payload.note.as_deref(),
    )
    .await?;

    tx.commit().await?;

    let detail = tracks::find_detail_by_id(&state.pool, track_id)
        .await?
        .ok_or(AppError::NotFound)?;

    // No connected SSE clients is a normal case, not an error — send()
    // only errors when the channel is fully closed, which never happens
    // while AppState holds the sender.
    let _ = state.events_tx.send(LicenseStatusChanged {
        track_id,
        scene_id: current.scene_id,
        from: Some(current.license_status),
        to: payload.status,
        at: detail.track.updated_at,
    });

    Ok(Json(detail))
}
