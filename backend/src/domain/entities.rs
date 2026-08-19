use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use super::license::LicenseStatus;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Movie {
    pub id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Scene {
    pub id: Uuid,
    pub movie_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Song {
    pub id: Uuid,
    pub title: String,
    pub artist: String,
    pub rights_holder: String,
    pub created_at: DateTime<Utc>,
}

/// A song's use within a single scene: the time window it plays for, and the
/// licensing status of that specific use (see `domain::license`).
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Track {
    pub id: Uuid,
    pub scene_id: Uuid,
    pub song_id: Uuid,
    pub start_time_ms: i32,
    pub end_time_ms: i32,
    pub license_status: LicenseStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// One row per `license_status` transition on a track — the audit trail.
#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct LicenseStatusEvent {
    pub id: Uuid,
    pub track_id: Uuid,
    pub from_status: Option<LicenseStatus>,
    pub to_status: LicenseStatus,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}
