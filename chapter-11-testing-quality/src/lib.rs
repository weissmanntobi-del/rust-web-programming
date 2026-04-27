use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskTitle(String);

impl TaskTitle {
    pub fn parse(raw: String) -> Result<Self, DomainError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Validation("title is empty"));
        }
        if trimmed.len() > 120 {
            return Err(DomainError::Validation("title too long"));
        }
        Ok(Self(trimmed.to_string()))
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("validation error: {0}")]
    Validation(&'static str),
}

#[derive(Debug)]
enum ApiError {
    BadRequest(&'static str),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let ApiError::BadRequest(message) = self;
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "code": "INVALID_INPUT",
                "message": message
            })),
        )
            .into_response()
    }
}

#[derive(Deserialize)]
struct CreateTaskDto {
    title: String,
}

#[derive(Serialize)]
struct TaskDto {
    id: Uuid,
    title: String,
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn create_task(Json(body): Json<CreateTaskDto>) -> Result<impl IntoResponse, ApiError> {
    let title =
        TaskTitle::parse(body.title).map_err(|_| ApiError::BadRequest("title is invalid"))?;
    Ok((
        StatusCode::CREATED,
        Json(TaskDto {
            id: Uuid::new_v4(),
            title: title.0,
        }),
    ))
}

pub fn app() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/tasks", post(create_task))
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn task_title_rejects_empty() {
        let err = TaskTitle::parse(" ".to_string()).unwrap_err();
        assert_eq!(err, DomainError::Validation("title is empty"));
    }

    #[test]
    fn task_title_rejects_too_long() {
        let err = TaskTitle::parse("x".repeat(121)).unwrap_err();
        assert_eq!(err, DomainError::Validation("title too long"));
    }

    #[test]
    fn task_title_accepts_normal_value() {
        let title = TaskTitle::parse("ship tests".to_string()).unwrap();
        assert_eq!(title.0, "ship tests");
    }
}
