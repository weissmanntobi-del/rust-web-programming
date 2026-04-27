use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct VersionResponse {
    app: &'static str,
    version: String,
    git_sha: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn hello() -> &'static str {
    "hello from rust"
}

async fn version() -> Json<VersionResponse> {
    Json(VersionResponse {
        app: "tasktracker",
        version: option_env!("CARGO_PKG_VERSION")
            .unwrap_or("unknown")
            .to_string(),
        git_sha: std::env::var("GIT_SHA").unwrap_or_else(|_| "dev".to_string()),
    })
}

fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/hello", get(hello))
        .route("/version", get(version))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_target(false).init();

    let addr: SocketAddr = std::env::var("HTTP_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_string())
        .parse()?;

    let listener = TcpListener::bind(addr).await?;
    info!(%addr, "server starting");

    axum::serve(listener, router()).await?;
    Ok(())
}
