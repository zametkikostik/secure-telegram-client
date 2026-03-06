// server/src/api/auth.rs
//! API аутентификации

use axum::{
    extract::State,
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{api::AppState, auth};

/// Запрос регистрации
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
}

/// Запрос входа
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Ответ аутентификации
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user_id: String,
    pub username: String,
    pub token: String,
    pub public_key: String,
}

/// Регистрация нового пользователя
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    // Хэширование пароля
    let password_hash = auth::hash_password(&req.password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Генерация ключей Ed25519
    let (signing_key, verifying_key) = auth::generate_keypair();
    let user_id = Uuid::new_v4().to_string();

    // Сохранение в базу данных
    let result = sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, public_key) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&user_id)
    .bind(&req.username)
    .bind(&req.email)
    .bind(&password_hash)
    .bind(hex::encode(verifying_key.to_bytes()))
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Ошибка регистрации: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Создание JWT токена
    let token = auth::create_token(&user_id, &req.username)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthResponse {
        user_id,
        username: req.username,
        token,
        public_key: hex::encode(verifying_key.to_bytes()),
    }))
}

/// Вход пользователя
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    // Поиск пользователя
    let user: (String, String, String, String) = sqlx::query_as(
        "SELECT id, username, password_hash, public_key FROM users WHERE username = ?"
    )
    .bind(&req.username)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Верификация пароля
    if !auth::verify_password(&req.password, &user.2) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Создание JWT токена
    let token = auth::create_token(&user.0, &user.1)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthResponse {
        user_id: user.0,
        username: user.1,
        token,
        public_key: user.3,
    }))
}

/// Верификация токена
pub async fn verify_token(
    State(state): State<AppState>,
    axum::extract::Headers(headers): axum::extract::Headers<axum::http::HeaderMap>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = auth::verify_token(auth_header)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(Json(serde_json::json!({
        "user_id": claims.sub,
        "username": claims.username,
        "valid": true
    })))
}
