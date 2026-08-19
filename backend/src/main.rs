use axum::{Router, routing::get};
use music_licensing_backend::config::Config;
use music_licensing_backend::db;
use std::net::SocketAddr;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env();

    // The pool isn't consumed anywhere yet — it starts backing API handlers
    // once the repository layer lands in the next change.
    let _pool = db::connect_and_migrate(&config.database_url).await;
    tracing::info!("database migrations applied");

    let app = Router::new().route("/health", get(health));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|err| panic!("failed to bind {addr}: {err}"));

    tracing::info!(%addr, "starting server");
    axum::serve(listener, app).await.expect("server error");
}

async fn health() -> &'static str {
    "ok"
}
