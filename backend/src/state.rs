use sqlx::PgPool;

use crate::events::EventSender;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub events_tx: EventSender,
}
