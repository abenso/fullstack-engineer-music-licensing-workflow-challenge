//! Response shapes that combine data from more than one table (e.g. a
//! track together with its song), as opposed to `domain::entities`, which
//! map 1:1 to a single table.

use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::domain::license::LicenseStatus;

#[derive(Debug, Serialize)]
pub struct SongSummary {
    pub id: Uuid,
    pub title: String,
    pub artist: String,
}

#[derive(Debug, Serialize)]
pub struct SceneSummary {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct TrackSummary {
    pub id: Uuid,
    pub song: SongSummary,
    pub start_time_ms: i32,
    pub end_time_ms: i32,
    pub license_status: LicenseStatus,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct LicenseEventSummary {
    pub from_status: Option<LicenseStatus>,
    pub to_status: LicenseStatus,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TrackDetail {
    #[serde(flatten)]
    pub track: TrackSummary,
    pub history: Vec<LicenseEventSummary>,
}

/// One entry of `GET /movies/{id}/tracks` — tracks grouped by the scene
/// they belong to. Scenes with no tracks are omitted (see repositories::tracks::list_for_movie).
#[derive(Debug, Serialize)]
pub struct SceneTracks {
    pub scene: SceneSummary,
    pub tracks: Vec<TrackSummary>,
}
