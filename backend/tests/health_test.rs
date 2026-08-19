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

/// A browser-based frontend on a different origin needs this header on
/// every response, not just on a pre-flight OPTIONS request.
#[sqlx::test]
async fn responses_allow_cross_origin_requests(pool: PgPool) {
    let app = common::app(pool);

    let request = Request::builder()
        .uri("/health")
        .header("origin", "http://localhost:3000")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(
        response
            .headers()
            .get("access-control-allow-origin")
            .unwrap(),
        "*"
    );
}
