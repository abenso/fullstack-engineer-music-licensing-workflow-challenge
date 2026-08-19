//! In-process pub/sub for license status changes. Handlers publish here on
//! every successful `PATCH /tracks/{id}/license`; the SSE endpoint
//! (added separately) subscribes and forwards events to connected clients.

use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::domain::license::LicenseStatus;

#[derive(Debug, Clone, Serialize)]
pub struct LicenseStatusChanged {
    pub track_id: Uuid,
    pub scene_id: Uuid,
    pub from: Option<LicenseStatus>,
    pub to: LicenseStatus,
    pub at: DateTime<Utc>,
}

pub type EventSender = broadcast::Sender<LicenseStatusChanged>;

/// Bounded so a slow subscriber can't grow memory unbounded — it just
/// misses old events instead of blocking publishers.
pub fn channel() -> EventSender {
    let (tx, _rx) = broadcast::channel(64);
    tx
}
