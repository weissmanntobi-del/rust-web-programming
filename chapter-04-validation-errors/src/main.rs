mod domain;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use domain::{DomainError, TaskTitle};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
struct CreateTaskDto {
    title: String,
}

#[derive(Debug, Serialize)]
struct FieldError {
    field: &'static str,
    message: String,
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
    request_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    fields: Vec<FieldError>,
}

#[derive(Debug)]
enum ApiError {
    BadRequest(Vec<FieldError>),
    Unauthorized,
    NotFound,
    Conflict(&'static str),
    Internal,
}

impl From<DomainError> for ApiError {
    fn from(value: DomainError) -> Self {
        match value {
            DomainError::NotFound => ApiError::NotFound,
            DomainError::Conflict(message) => ApiError::Conflict(message),
            DomainError::Unauthorized => ApiError::Unauthorized,
            DomainError::Validation(message) => ApiError::BadRequest(vec![FieldError {
                field: "title",
                message: message.to_string(),
            }]),
            DomainError::Unexpected => ApiError::Internal,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message, fields) = match self {
            ApiError::BadRequest(fields) => (
                StatusCode::BAD_REQUEST,
                "INVALID_INPUT",
                "request validation failed".to_string(),
                fields,
            ),
            ApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "login required".to_string(),
                vec![],
            ),
            ApiError::NotFound => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                "resource not found".to_string(),
                vec![],
            ),
            ApiError::Conflict(message) => (
                StatusCode::CONFLICT,
                "CONFLICT",
                message.to_string(),
                vec![],
            ),
            ApiError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL",
                "try again later".to_string(),
                vec![],
            ),
        };

        (
            status,
            Json(ErrorBody {
                code,
                message,
                request_id: Uuid::new_v4().to_string(),
                fields,
            }),
        )
            .into_response()
    }
}

#[derive(Serialize)]
struct TaskDto {
    id: String,
    title: String,
}

async fn create_task(Json(body): Json<CreateTaskDto>) -> Result<impl IntoResponse, ApiError> {
    let title = TaskTitle::parse(body.title)?;
    let task = TaskDto {
        id: format!("task_{}", Uuid::new_v4()),
        title: title.as_str().to_string(),
    };
    Ok((StatusCode::CREATED, Json(task)))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_target(false).init();
    let app = Router::new().route("/tasks", post(create_task));
    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
