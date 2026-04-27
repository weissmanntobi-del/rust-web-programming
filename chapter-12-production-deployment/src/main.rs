use axum::error_handling::HandleErrorLayer;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{routing::get, BoxError, Json, Router};
use serde::Serialize;
use std::{net::SocketAddr, time::Duration};
use tokio::{net::TcpListener, signal};
use tower::ServiceBuilder;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};
use tracing::info;

#[derive(Clone, Debug)]
struct Config {
    database_url: String,
    jwt_secret: String,
    http_addr: SocketAddr,
}

impl Config {
    fn from_env() -> Result<Self, String> {
        let database_url =
            std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL is required".to_string())?;
        let jwt_secret =
            std::env::var("JWT_SECRET").map_err(|_| "JWT_SECRET is required".to_string())?;
        if jwt_secret.len() < 32 {
            return Err("JWT_SECRET must be at least 32 characters".to_string());
        }

        let http_addr = std::env::var("HTTP_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:3000".to_string())
            .parse::<SocketAddr>()
            .map_err(|err| format!("HTTP_ADDR is invalid: {err}"))?;

        Ok(Self {
            database_url,
            jwt_secret,
            http_addr,
        })
    }
}

#[derive(Serialize)]
struct HealthBody {
    status: &'static str,
}

async fn health() -> Json<HealthBody> {
    Json(HealthBody { status: "ok" })
}

async fn livez() -> Json<HealthBody> {
    Json(HealthBody { status: "alive" })
}

async fn readyz() -> Json<HealthBody> {
    // In a real service, check DB pool and critical dependencies here.
    Json(HealthBody { status: "ready" })
}

fn app() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/livez", get(livez))
        .route("/readyz", get(readyz))
        .layer(DefaultBodyLimit::max(1 * 1024 * 1024))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::with_status_code(
                    StatusCode::REQUEST_TIMEOUT,
                    Duration::from_secs(10),
                )),
        )
}

async fn handle_middleware_error(err: BoxError) -> axum::response::Response {
    if err.is::<tower::timeout::error::Elapsed>() {
        (StatusCode::REQUEST_TIMEOUT, "request timed out").into_response()
    } else {
        eprintln!("middleware error: {err}");

        (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("shutdown signal received");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    let config = Config::from_env().map_err(anyhow::Error::msg)?;
    info!(addr = %config.http_addr, "server starting");
    info!(
        database_url_configured = !config.database_url.is_empty(),
        jwt_secret_configured = !config.jwt_secret.is_empty()
    );

    let listener = TcpListener::bind(config.http_addr).await?;
    axum::serve(listener, app())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
