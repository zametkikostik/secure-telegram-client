// server/src/api/users.rs
//! API пользователей

use axum::{
    extract::{State, Path},
    Json,
    http::StatusCode,
};
use serde::Serialize;
use crate::api::AppState;

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub status: String,
    pub public_key: String,
}

/// Получить текущего пользователя
pub async fn get_current_user(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<UserResponse>, StatusCode> {
    use crate::auth;
    
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = auth::verify_token(auth_header)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user: (String, String, Option<String>, Option<String>, String, String) = sqlx::query_as(
        "SELECT id, username, email, avatar_url, status, public_key FROM users WHERE id = ?"
    )
    .bind(&claims.sub)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(UserResponse {
        id: user.0,
        username: user.1,
        email: user.2,
        avatar_url: user.3,
        status: user.4,
        public_key: user.5,
    }))
}

/// Получить пользователя по ID
pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<UserResponse>, StatusCode> {
    let user: (String, String, Option<String>, Option<String>, String, String) = sqlx::query_as(
        "SELECT id, username, email, avatar_url, status, public_key FROM users WHERE id = ?"
    )
    .bind(&user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(UserResponse {
        id: user.0,
        username: user.1,
        email: user.2,
        avatar_url: user.3,
        status: user.4,
        public_key: user.5,
    }))
}
