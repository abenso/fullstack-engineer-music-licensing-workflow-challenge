use music_licensing_backend::config::Config;
use music_licensing_backend::state::AppState;
use music_licensing_backend::{db, routes};
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

    let pool = db::connect_and_migrate(&config.database_url).await;
    tracing::info!("database migrations applied");

    let app = routes::build(AppState { pool });

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|err| panic!("failed to bind {addr}: {err}"));

    tracing::info!(%addr, "starting server");
    axum::serve(listener, app).await.expect("server error");
}
