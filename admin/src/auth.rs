//! Admin Authentication

use axum::{
    extract::{State, CookieJar},
    Json,
    http::StatusCode,
};
use axum_extra::extract::Cookie;
use serde::{Deserialize, Serialize};
use crate::AppState;

#[derive(Serialize)]
pub struct Admin {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub role: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub admin: Admin,
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Поиск админа
    let admin = sqlx::query_as::<_, (String, String, String, Option<String>, String)>(
        "SELECT id, username, password_hash, email, role FROM admins WHERE username = ?"
    )
    .bind(&req.username)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    let (id, username, password_hash, email, role) = admin;

    // Проверка пароля
    let password_valid = argon2::verify_encoded(&password_hash, req.password.as_bytes())
        .unwrap_or(false);

    if !password_valid {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Генерация токена
    let token = generate_admin_token(&id, &username, &role);

    // Обновление last_login
    sqlx::query("UPDATE admins SET last_login = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .ok();

    Ok(Json(LoginResponse {
        token,
        admin: Admin {
            id,
            username,
            email,
            role,
        },
    }))
}

pub async fn logout(
    _jar: CookieJar,
) -> Result<StatusCode, StatusCode> {
    // TODO: Инвалидация токена
    Ok(StatusCode::OK)
}

pub async fn get_current_admin(
    // claims: Claims, // TODO: Извлечь из токена
) -> Result<Json<Admin>, StatusCode> {
    // TODO: Получить из токена
    Ok(Json(Admin {
        id: "admin_1".to_string(),
        username: "admin".to_string(),
        email: Some("admin@secure-telegram.io".to_string()),
        role: "superadmin".to_string(),
    }))
}

fn generate_admin_token(admin_id: &str, username: &str, role: &str) -> String {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use chrono::Utc;

    #[derive(Serialize)]
    struct Claims {
        sub: String,
        username: String,
        role: String,
        exp: usize,
        iat: usize,
    }

    let now = Utc::now();
    let exp = now + chrono::Duration::hours(24);

    let claims = Claims {
        sub: admin_id.to_string(),
        username: username.to_string(),
        role: role.to_string(),
        exp: exp.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "admin-secret".to_string());
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map(|t| t.unwrap_or_default())
    .unwrap_or_default()
}
