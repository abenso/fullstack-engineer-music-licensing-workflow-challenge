mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use sqlx::PgPool;
use tower::ServiceExt;

/// /health returns plain text, not JSON, so this bypasses common::request
/// (which assumes a JSON body) and checks the raw response directly.
#[sqlx::test]
async fn health_returns_ok(pool: PgPool) {
    let app = common::app(pool);

    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
