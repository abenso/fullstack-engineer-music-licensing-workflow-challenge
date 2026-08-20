mod common;

use music_licensing_backend::domain::license::LicenseStatus;
use music_licensing_backend::repositories::tracks;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

/// Regression test for a race condition: two requests both read the track
/// as `Draft` and both pass the `can_transition` check before either
/// writes. Without a compare-and-swap on the write, the second one would
/// silently overwrite the first instead of losing gracefully.
///
/// This drives `tracks::try_update_license_status` directly (rather than
/// spawning real concurrent HTTP requests, which wouldn't reliably
/// reproduce the interleaving) to deterministically simulate "two callers,
/// same stale read".
#[sqlx::test]
async fn a_stale_concurrent_update_is_rejected_instead_of_overwriting(pool: PgPool) {
    let app = common::app(pool.clone());
    let (_, scene_id, song_id) = common::seed_movie_scene_song(&app).await;

    let (_, track) = common::request(
        app,
        "POST",
        &format!("/scenes/{scene_id}/tracks"),
        Some(json!({ "song_id": song_id, "start_time_ms": 0, "end_time_ms": 100 })),
    )
    .await;
    let track_id: Uuid = track["id"].as_str().unwrap().parse().unwrap();

    // First caller: Draft -> Requested, expecting Draft. Wins.
    let first = tracks::try_update_license_status(
        &pool,
        track_id,
        LicenseStatus::Draft,
        LicenseStatus::Requested,
    )
    .await
    .unwrap();
    assert!(first.is_some());

    // Second caller raced in with the same stale "Draft" read. Must not
    // overwrite the first caller's change.
    let second = tracks::try_update_license_status(
        &pool,
        track_id,
        LicenseStatus::Draft,
        LicenseStatus::Rejected,
    )
    .await
    .unwrap();
    assert!(second.is_none());

    let current = tracks::find_by_id(&pool, track_id).await.unwrap().unwrap();
    assert_eq!(current.license_status, LicenseStatus::Requested);
}
