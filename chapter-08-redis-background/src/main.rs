use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::{net::TcpListener, sync::RwLock};
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    tasks: Arc<RwLock<Vec<TaskDto>>>,
    redis: Option<redis::Client>,
}

#[derive(Clone, Serialize, Deserialize)]
struct TaskDto {
    id: Uuid,
    title: String,
    done: bool,
    created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct CreateTaskDto {
    title: String,
}

fn tasks_key(user_id: Uuid) -> String {
    format!("tasks:v1:user:{user_id}")
}

async fn get_cached_tasks(redis: &redis::Client, key: &str) -> Option<Vec<TaskDto>> {
    let mut conn = tokio::time::timeout(
        Duration::from_millis(100),
        redis.get_multiplexed_async_connection(),
    )
    .await
    .ok()?
    .ok()?;

    let raw: Option<String> = tokio::time::timeout(Duration::from_millis(100), conn.get(key))
        .await
        .ok()?
        .ok()?;

    raw.and_then(|json| serde_json::from_str(&json).ok())
}

async fn set_cached_tasks(redis: &redis::Client, key: &str, tasks: &[TaskDto]) {
    let Ok(json) = serde_json::to_string(tasks) else {
        return;
    };

    let Ok(Ok(mut conn)) = tokio::time::timeout(
        Duration::from_millis(100),
        redis.get_multiplexed_async_connection(),
    )
    .await
    else {
        warn!("redis connection failed; ignoring cache write");
        return;
    };

    let _: Result<(), _> =
        tokio::time::timeout(Duration::from_millis(100), conn.set_ex(key, json, 30))
            .await
            .unwrap_or_else(|_| {
                Err(redis::RedisError::from((
                    redis::ErrorKind::IoError,
                    "timeout",
                )))
            });
}

async fn delete_cache(redis: &redis::Client, key: &str) {
    if let Ok(Ok(mut conn)) = tokio::time::timeout(
        Duration::from_millis(100),
        redis.get_multiplexed_async_connection(),
    )
    .await
    {
        let _: Result<(), _> = conn.del(key).await;
    }
}

async fn list_tasks(State(state): State<AppState>) -> Json<Vec<TaskDto>> {
    let user_id = Uuid::nil();
    let key = tasks_key(user_id);

    if let Some(redis) = &state.redis {
        if let Some(tasks) = get_cached_tasks(redis, &key).await {
            info!(cache_hit = true, endpoint = "/tasks");
            return Json(tasks);
        }
    }

    info!(cache_hit = false, endpoint = "/tasks");
    let tasks = state.tasks.read().await.clone();

    if let Some(redis) = &state.redis {
        set_cached_tasks(redis, &key, &tasks).await;
    }

    Json(tasks)
}

async fn create_task(
    State(state): State<AppState>,
    Json(body): Json<CreateTaskDto>,
) -> (StatusCode, Json<TaskDto>) {
    let task = TaskDto {
        id: Uuid::new_v4(),
        title: body.title,
        done: false,
        created_at: Utc::now(),
    };

    state.tasks.write().await.push(task.clone());

    if let Some(redis) = &state.redis {
        delete_cache(redis, &tasks_key(Uuid::nil())).await;
    }

    let audit_task = task.clone();
    tokio::spawn(async move {
        for attempt in 1..=3 {
            info!(job = "audit_log", task_id = %audit_task.id, attempt, "audit job started");
            tokio::time::sleep(Duration::from_millis(50)).await;
            info!(job = "audit_log", task_id = %audit_task.id, attempt, outcome = "ok");
            break;
        }
    });

    (StatusCode::CREATED, Json(task))
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/tasks", get(list_tasks).post(create_task))
        .with_state(state)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_target(false).init();

    let redis = match std::env::var("REDIS_URL") {
        Ok(url) => Some(redis::Client::open(url)?),
        Err(_) => None,
    };

    let state = AppState {
        tasks: Arc::new(RwLock::new(vec![])),
        redis,
    };

    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");
    axum::serve(listener, router(state)).await?;
    Ok(())
}
