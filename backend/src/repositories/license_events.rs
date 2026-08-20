use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::entities::LicenseStatusEvent;
use crate::domain::license::LicenseStatus;

/// Takes a generic executor so this can run inside the same transaction as
/// the track's status update — see `tracks::try_update_license_status`.
pub async fn record<'e, E>(
    executor: E,
    track_id: Uuid,
    from_status: Option<LicenseStatus>,
    to_status: LicenseStatus,
    note: Option<&str>,
) -> sqlx::Result<LicenseStatusEvent>
where
    E: sqlx::PgExecutor<'e>,
{
    sqlx::query_as!(
        LicenseStatusEvent,
        r#"
        INSERT INTO license_status_events (track_id, from_status, to_status, note)
        VALUES ($1, $2, $3, $4)
        RETURNING id, track_id,
                  from_status AS "from_status: LicenseStatus",
                  to_status AS "to_status: LicenseStatus",
                  note, created_at
        "#,
        track_id,
        from_status as Option<LicenseStatus>,
        to_status as LicenseStatus,
        note
    )
    .fetch_one(executor)
    .await
}

pub async fn list_for_track(
    pool: &PgPool,
    track_id: Uuid,
) -> sqlx::Result<Vec<LicenseStatusEvent>> {
    sqlx::query_as!(
        LicenseStatusEvent,
        r#"
        SELECT id, track_id,
               from_status AS "from_status: LicenseStatus",
               to_status AS "to_status: LicenseStatus",
               note, created_at
        FROM license_status_events
        WHERE track_id = $1
        ORDER BY created_at
        "#,
        track_id
    )
    .fetch_all(pool)
    .await
}
