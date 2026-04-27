use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, sync::RwLock};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    users: Arc<RwLock<HashMap<String, UserRecord>>>,
    jwt_secret: Arc<[u8]>,
}

#[derive(Clone)]
struct UserRecord {
    user_id: Uuid,
    email: String,
    password_hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
}

#[derive(Debug, Clone, Serialize)]
struct UserContext {
    user_id: Uuid,
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
                "invalid credentials or token",
            ),
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

impl FromRequestParts<AppState> for UserContext {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;

        let token = auth.strip_prefix("Bearer ").ok_or(ApiError::Unauthorized)?;

        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(&state.jwt_secret),
            &Validation::default(),
        )
        .map_err(|_| ApiError::Unauthorized)?;

        let user_id = Uuid::parse_str(&data.claims.sub).map_err(|_| ApiError::Unauthorized)?;
        Ok(UserContext { user_id })
    }
}

#[derive(Deserialize)]
struct AuthDto {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct TokenResponse {
    access_token: String,
    token_type: &'static str,
    expires_in_seconds: i64,
}

fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| ApiError::Internal)?;
    Ok(hash.to_string())
}

fn verify_password(password: &str, hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

fn issue_token(user_id: Uuid, secret: &[u8]) -> Result<String, ApiError> {
    let now = Utc::now();
    let exp = now + Duration::minutes(15);
    let claims = Claims {
        sub: user_id.to_string(),
        iat: now.timestamp() as usize,
        exp: exp.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|_| ApiError::Internal)
}

async fn register(
    State(state): State<AppState>,
    Json(body): Json<AuthDto>,
) -> Result<impl IntoResponse, ApiError> {
    if !body.email.contains('@') {
        return Err(ApiError::BadRequest("valid email is required"));
    }
    if body.password.len() < 8 {
        return Err(ApiError::BadRequest(
            "password must be at least 8 characters",
        ));
    }

    let mut users = state.users.write().await;
    if users.contains_key(&body.email) {
        return Err(ApiError::Conflict("email already registered"));
    }

    let user = UserRecord {
        user_id: Uuid::new_v4(),
        email: body.email.clone(),
        password_hash: hash_password(&body.password)?,
    };

    users.insert(body.email, user.clone());
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "user_id": user.user_id })),
    ))
}

async fn login(
    State(state): State<AppState>,
    Json(body): Json<AuthDto>,
) -> Result<Json<TokenResponse>, ApiError> {
    let users = state.users.read().await;
    let user = users.get(&body.email).ok_or(ApiError::Unauthorized)?;

    if !verify_password(&body.password, &user.password_hash) {
        return Err(ApiError::Unauthorized);
    }

    let token = issue_token(user.user_id, &state.jwt_secret)?;
    Ok(Json(TokenResponse {
        access_token: token,
        token_type: "Bearer",
        expires_in_seconds: 15 * 60,
    }))
}

async fn me(user: UserContext) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "user_id": user.user_id }))
}

async fn list_my_tasks(user: UserContext) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "owner": user.user_id,
        "tasks": [
            { "id": "task_1", "title": "protected task", "done": false }
        ]
    }))
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/me", get(me))
        .route("/tasks", get(list_my_tasks))
        .with_state(state)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_target(false).init();

    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "dev-secret-change-me-dev-secret-change-me".to_string());

    let state = AppState {
        users: Arc::new(RwLock::new(HashMap::new())),
        jwt_secret: Arc::from(secret.into_bytes()),
    };

    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");
    axum::serve(listener, router(state)).await?;
    Ok(())
}
