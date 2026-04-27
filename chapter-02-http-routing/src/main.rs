use axum::{
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use uuid::Uuid;

#[derive(Serialize)]
struct OkBody<T> {
    data: T,
    request_id: String,
}

#[derive(Serialize)]
struct TaskDto {
    id: String,
    title: String,
    done: bool,
}

#[derive(Deserialize)]
struct ListQuery {
    limit: Option<u32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
struct CreateTaskDto {
    title: String,
}

fn request_id(headers: &HeaderMap) -> String {
    headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

async fn list_tasks(Query(query): Query<ListQuery>, headers: HeaderMap) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(20).min(100);
    let tasks = vec![TaskDto {
        id: "task_1".to_string(),
        title: format!("Return at most {limit} tasks"),
        done: false,
    }];

    Json(OkBody {
        data: tasks,
        request_id: request_id(&headers),
    })
}

async fn get_task(Path(id): Path<String>, headers: HeaderMap) -> impl IntoResponse {
    let task = TaskDto {
        id,
        title: "Learn Axum routing".to_string(),
        done: false,
    };

    Json(OkBody {
        data: task,
        request_id: request_id(&headers),
    })
}

async fn create_task(headers: HeaderMap, Json(body): Json<CreateTaskDto>) -> impl IntoResponse {
    let task = TaskDto {
        id: format!("task_{}", Uuid::new_v4()),
        title: body.title,
        done: false,
    };

    (
        StatusCode::CREATED,
        Json(OkBody {
            data: task,
            request_id: request_id(&headers),
        }),
    )
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/tasks", get(list_tasks).post(create_task))
        .route("/tasks/{id}", get(get_task))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_target(false).init();
    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");
    axum::serve(listener, router()).await?;
    Ok(())
}
