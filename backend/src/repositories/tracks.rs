use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::Track;
use crate::domain::license::LicenseStatus;
use crate::dto::{
    LicenseEventSummary, SceneSummary, SceneTracks, SongSummary, TrackDetail, TrackSummary,
};

use super::license_events;

pub async fn create(
    pool: &PgPool,
    scene_id: Uuid,
    song_id: Uuid,
    start_time_ms: i32,
    end_time_ms: i32,
) -> sqlx::Result<Track> {
    sqlx::query_as!(
        Track,
        r#"
        INSERT INTO tracks (scene_id, song_id, start_time_ms, end_time_ms)
        VALUES ($1, $2, $3, $4)
        RETURNING id, scene_id, song_id, start_time_ms, end_time_ms,
                  license_status AS "license_status: LicenseStatus",
                  created_at, updated_at
        "#,
        scene_id,
        song_id,
        start_time_ms,
        end_time_ms
    )
    .fetch_one(pool)
    .await
}

/// Plain row lookup — used internally (e.g. to read the current
/// `license_status` before validating a transition). For API responses
/// that need the associated song, see `find_detail_by_id`.
pub async fn find_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Track>> {
    sqlx::query_as!(
        Track,
        r#"
        SELECT id, scene_id, song_id, start_time_ms, end_time_ms,
               license_status AS "license_status: LicenseStatus",
               created_at, updated_at
        FROM tracks
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await
}

/// Moves a track to `new_status`, but only if it's still at
/// `expected_status`. Returns `None` if it isn't — i.e. someone else
/// already changed it between the caller's read and this write — so the
/// caller can treat that as a conflict instead of silently overwriting it.
///
/// Takes a generic executor (rather than `&PgPool`) so it can run inside
/// the same transaction as the `license_status_events` insert: those two
/// writes must succeed or fail together, or the audit trail could disagree
/// with the track's actual status.
pub async fn try_update_license_status<'e, E>(
    executor: E,
    id: Uuid,
    expected_status: LicenseStatus,
    new_status: LicenseStatus,
) -> sqlx::Result<Option<Track>>
where
    E: sqlx::PgExecutor<'e>,
{
    sqlx::query_as!(
        Track,
        r#"
        UPDATE tracks
        SET license_status = $1, updated_at = now()
        WHERE id = $2 AND license_status = $3
        RETURNING id, scene_id, song_id, start_time_ms, end_time_ms,
                  license_status AS "license_status: LicenseStatus",
                  created_at, updated_at
        "#,
        new_status as LicenseStatus,
        id,
        expected_status as LicenseStatus,
    )
    .fetch_optional(executor)
    .await
}

pub async fn list_for_scene(pool: &PgPool, scene_id: Uuid) -> sqlx::Result<Vec<TrackSummary>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            t.id, t.start_time_ms, t.end_time_ms, t.updated_at,
            t.license_status AS "license_status: LicenseStatus",
            s.id AS song_id, s.title AS song_title, s.artist AS song_artist
        FROM tracks t
        JOIN songs s ON s.id = t.song_id
        WHERE t.scene_id = $1
        ORDER BY t.created_at
        "#,
        scene_id
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| TrackSummary {
            id: row.id,
            song: SongSummary {
                id: row.song_id,
                title: row.song_title,
                artist: row.song_artist,
            },
            start_time_ms: row.start_time_ms,
            end_time_ms: row.end_time_ms,
            license_status: row.license_status,
            updated_at: row.updated_at,
        })
        .collect())
}

/// Tracks for every scene in a movie, grouped by scene. Scenes with no
/// tracks yet are omitted (this endpoint is about tracks, not scenes).
pub async fn list_for_movie(pool: &PgPool, movie_id: Uuid) -> sqlx::Result<Vec<SceneTracks>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            sc.id AS scene_id, sc.name AS scene_name,
            t.id AS track_id, t.start_time_ms, t.end_time_ms, t.updated_at,
            t.license_status AS "license_status: LicenseStatus",
            s.id AS song_id, s.title AS song_title, s.artist AS song_artist
        FROM scenes sc
        JOIN tracks t ON t.scene_id = sc.id
        JOIN songs s ON s.id = t.song_id
        WHERE sc.movie_id = $1
        ORDER BY sc.created_at, t.created_at
        "#,
        movie_id
    )
    .fetch_all(pool)
    .await?;

    let mut grouped: Vec<SceneTracks> = Vec::new();
    for row in rows {
        let track = TrackSummary {
            id: row.track_id,
            song: SongSummary {
                id: row.song_id,
                title: row.song_title,
                artist: row.song_artist,
            },
            start_time_ms: row.start_time_ms,
            end_time_ms: row.end_time_ms,
            license_status: row.license_status,
            updated_at: row.updated_at,
        };

        match grouped.last_mut() {
            Some(entry) if entry.scene.id == row.scene_id => entry.tracks.push(track),
            _ => grouped.push(SceneTracks {
                scene: SceneSummary {
                    id: row.scene_id,
                    name: row.scene_name,
                },
                tracks: vec![track],
            }),
        }
    }

    Ok(grouped)
}

pub async fn find_detail_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<TrackDetail>> {
    let row = sqlx::query!(
        r#"
        SELECT
            t.id, t.start_time_ms, t.end_time_ms, t.updated_at,
            t.license_status AS "license_status: LicenseStatus",
            s.id AS song_id, s.title AS song_title, s.artist AS song_artist
        FROM tracks t
        JOIN songs s ON s.id = t.song_id
        WHERE t.id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let history = license_events::list_for_track(pool, id).await?;

    Ok(Some(TrackDetail {
        track: TrackSummary {
            id: row.id,
            song: SongSummary {
                id: row.song_id,
                title: row.song_title,
                artist: row.song_artist,
            },
            start_time_ms: row.start_time_ms,
            end_time_ms: row.end_time_ms,
            license_status: row.license_status,
            updated_at: row.updated_at,
        },
        history: history
            .into_iter()
            .map(|event| LicenseEventSummary {
                from_status: event.from_status,
                to_status: event.to_status,
                note: event.note,
                created_at: event.created_at,
            })
            .collect(),
    }))
}
