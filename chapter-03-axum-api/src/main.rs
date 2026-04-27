use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    tasks: Arc<RwLock<HashMap<Uuid, Task>>>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case")]
struct Task {
    id: Uuid,
    title: String,
    done: bool,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
struct CreateTaskDto {
    title: String,
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
    request_id: String,
}

#[derive(Debug)]
enum ApiError {
    BadRequest(&'static str),
    NotFound,
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, "INVALID_INPUT", message),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "TASK_NOT_FOUND", "task not found"),
            ApiError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL",
                "try again later",
            ),
        };

        (
            status,
            Json(ErrorBody {
                code,
                message: message.to_string(),
                request_id: Uuid::new_v4().to_string(),
            }),
        )
            .into_response()
    }
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn list_tasks(State(state): State<AppState>) -> Json<Vec<Task>> {
    let tasks = state.tasks.read().await;
    Json(tasks.values().cloned().collect())
}

async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Task>, ApiError> {
    let tasks = state.tasks.read().await;
    let task = tasks.get(&id).cloned().ok_or(ApiError::NotFound)?;
    Ok(Json(task))
}

async fn create_task(
    State(state): State<AppState>,
    Json(body): Json<CreateTaskDto>,
) -> Result<impl IntoResponse, ApiError> {
    let title = body.title.trim();
    if title.is_empty() {
        return Err(ApiError::BadRequest("title must not be empty"));
    }

    let task = Task {
        id: Uuid::new_v4(),
        title: title.to_string(),
        done: false,
        created_at: Utc::now(),
    };

    state.tasks.write().await.insert(task.id, task.clone());
    Ok((StatusCode::CREATED, Json(task)))
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/tasks", get(list_tasks).post(create_task))
        .route("/tasks/{id}", get(get_task))
        .with_state(state)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_target(false).init();

    let state = AppState {
        tasks: Arc::new(RwLock::new(HashMap::new())),
    };

    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");
    axum::serve(listener, router(state)).await?;
    Ok(())
}
