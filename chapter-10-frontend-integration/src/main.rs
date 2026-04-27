use axum::{routing::get, Json, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};

async fn api_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "service": "tasktracker-api" }))
}

async fn api_tasks() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "tasks": [
            { "id": "task_1", "title": "Serve frontend from Rust", "done": false }
        ]
    }))
}

fn app() -> Router {
    Router::new()
        .route("/api/health", get(api_health))
        .route("/api/tasks", get(api_tasks))
        .nest_service("/assets", ServeDir::new("public/assets"))
        .fallback_service(ServeFile::new("public/index.html"))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_target(false).init();
    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let listener = TcpListener::bind(addr).await?;
    println!("open http://{addr}");
    axum::serve(listener, app()).await?;
    Ok(())
}
