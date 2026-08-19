use std::convert::Infallible;

use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use crate::state::AppState;

/// Streams `LicenseStatusChanged` events (see src/events.rs) to the client
/// as they happen. A lagged/closed receiver just drops those items instead
/// of ending the stream — the client keeps the connection and gets the
/// next event normally.
pub async fn stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let receiver = state.events_tx.subscribe();

    let stream = BroadcastStream::new(receiver).filter_map(|result| {
        let payload = result.ok()?;
        let data = serde_json::to_string(&payload).ok()?;
        Some(Ok(Event::default()
            .event("license_status_changed")
            .data(data)))
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
