mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use sqlx::PgPool;
use tower::ServiceExt;

/// The SSE stream never ends on its own, so this only checks that the
/// connection opens correctly (status + content-type) — it doesn't try to
/// read the (infinite) body. The actual event delivery is exercised
/// manually with `curl -N /events` (see the PR description).
#[sqlx::test]
async fn events_stream_opens_with_the_right_content_type(pool: PgPool) {
    let app = common::app(pool);

    let request = Request::builder()
        .method("GET")
        .uri("/events")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "text/event-stream"
    );
}
