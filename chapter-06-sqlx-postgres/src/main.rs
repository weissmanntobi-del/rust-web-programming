use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{net::SocketAddr, time::Duration};
use tokio::net::TcpListener;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "snake_case")]
struct TaskRow {
    id: Uuid,
    user_id: Uuid,
    title: String,
    done: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
struct ListQuery {
    limit: Option<i64>,
}

#[derive(Deserialize)]
struct CreateTaskDto {
    title: String,
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

#[derive(Debug)]
enum ApiError {
    BadRequest(&'static str),
    Unauthorized,
    NotFound,
    Conflict(&'static str),
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, "INVALID_INPUT", message),
            ApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "missing x-user-id header",
            ),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "TASK_NOT_FOUND", "task not found"),
            ApiError::Conflict(message) => (StatusCode::CONFLICT, "CONFLICT", message),
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
            }),
        )
            .into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        if let sqlx::Error::Database(db_err) = &err {
            if db_err.is_unique_violation() {
                return ApiError::Conflict("unique constraint violation");
            }
        }
        tracing::error!(error = ?err, "database error");
        ApiError::Internal
    }
}

fn user_id_from_headers(headers: &HeaderMap) -> Result<Uuid, ApiError> {
    headers
        .get("x-user-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| Uuid::parse_str(v).ok())
        .ok_or(ApiError::Unauthorized)
}

async fn make_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5))
        .connect(database_url)
        .await
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

async fn list_tasks(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
    headers: HeaderMap,
) -> Result<Json<Vec<TaskRow>>, ApiError> {
    let user_id = user_id_from_headers(&headers)?;
    let limit = q.limit.unwrap_or(20).clamp(1, 100);

    let tasks = sqlx::query_as::<_, TaskRow>(
        r#"
        SELECT id, user_id, title, done, created_at, updated_at
        FROM tasks
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(tasks))
}

async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<TaskRow>, ApiError> {
    let user_id = user_id_from_headers(&headers)?;

    let task = sqlx::query_as::<_, TaskRow>(
        r#"
        SELECT id, user_id, title, done, created_at, updated_at
        FROM tasks
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(task_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(task))
}

async fn create_task(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateTaskDto>,
) -> Result<impl IntoResponse, ApiError> {
    let user_id = user_id_from_headers(&headers)?;
    let title = body.title.trim();
    if title.is_empty() {
        return Err(ApiError::BadRequest("title must not be empty"));
    }

    let mut tx = state.db.begin().await?;
    let task = sqlx::query_as::<_, TaskRow>(
        r#"
        INSERT INTO tasks (id, user_id, title)
        VALUES ($1, $2, $3)
        RETURNING id, user_id, title, done, created_at, updated_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(title)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok((StatusCode::CREATED, Json(task)))
}

async fn mark_done(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<StatusCode, ApiError> {
    let user_id = user_id_from_headers(&headers)?;
    let result = sqlx::query(
        r#"
        UPDATE tasks
        SET done = true, updated_at = now()
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(task_id)
    .bind(user_id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/tasks", get(list_tasks).post(create_task))
        .route("/tasks/{id}", get(get_task))
        .route("/tasks/{id}/done", put(mark_done))
        .with_state(state)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_target(false).init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set, e.g. postgres://tasktracker:tasktracker@localhost:5432/tasktracker");
    let db = make_pool(&database_url).await?;

    if std::env::var("RUN_MIGRATIONS").as_deref() == Ok("true") {
        sqlx::migrate!("./migrations").run(&db).await?;
    }

    let state = AppState { db };
    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");
    axum::serve(listener, router(state)).await?;
    Ok(())
}
