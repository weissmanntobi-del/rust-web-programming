use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::{
    http::{header, HeaderName, HeaderValue, Method},
    middleware::Next,
    response::Response,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::{net::SocketAddr, time::Duration};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer}
    ,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer}
    ,
    trace::TraceLayer,
};

#[derive(Deserialize)]
struct LoginDto {
    email: String,
    password: String,
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn login(Json(body): Json<LoginDto>) -> Json<serde_json::Value> {
    let _password_len = body.password.len();
    Json(
        serde_json::json!({ "message": "login endpoint protected by body limit and timeout", "email": body.email }),
    )
}

async fn security_headers(request: axum::extract::Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    response
}

async fn request_timeout(
    request: axum::extract::Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    match tokio::time::timeout(Duration::from_secs(10), next.run(request)).await {
        Ok(response) => Ok(response),

        Err(_) => Err((
            StatusCode::REQUEST_TIMEOUT,
            Json(serde_json::json!({
                "code": "REQUEST_TIMEOUT",
                "message": "request took too long"
            })),
        )),
    }
}

fn app() -> Router {
    let request_id_header = HeaderName::from_static("x-request-id");

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health))
        .route("/login", post(login))
        .layer(axum::middleware::from_fn(security_headers))
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(axum::middleware::from_fn(request_timeout))
        .layer(axum::middleware::from_fn(security_headers))
        .layer(
            ServiceBuilder::new()
                .layer(SetRequestIdLayer::new(
                    request_id_header.clone(),
                    MakeRequestUuid,
                ))
                .layer(TraceLayer::new_for_http())
                .layer(PropagateRequestIdLayer::new(request_id_header))
                .layer(cors),
        )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .init();

    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");
    axum::serve(listener, app()).await?;
    Ok(())
}
