use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Token expired")]
    TokenExpired,
    #[error("Token invalid: {0}")]
    TokenInvalid(String),
    #[error("User not found")]
    UserNotFound,
    #[error("Username taken")]
    UsernameTaken,
    #[error("DB: {0}")]
    Database(String),
    #[error("No auth")]
    MissingAuth,
    #[error("Permission denied")]
    PermissionDenied,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AuthError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string())
            }
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "Token expired".to_string()),
            AuthError::TokenInvalid(e) => (StatusCode::UNAUTHORIZED, format!("Bad token: {}", e)),
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, "User not found".to_string()),
            AuthError::UsernameTaken => (StatusCode::CONFLICT, "Username taken".to_string()),
            AuthError::Database(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("DB: {}", e)),
            AuthError::MissingAuth => (StatusCode::UNAUTHORIZED, "Missing auth".to_string()),
            AuthError::PermissionDenied => (StatusCode::FORBIDDEN, "Permission denied".to_string()),
        };
        (status, Json(serde_json::json!({"error": msg}))).into_response()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Clone)]
pub struct AuthState {
    pub jwt_secret: String,
    pub jwt_expiry: usize,
}

impl AuthState {
    pub fn new(s: &str, e: usize) -> Self {
        Self {
            jwt_secret: s.into(),
            jwt_expiry: e,
        }
    }

    pub fn generate_token(&self, uid: &str, uname: &str) -> Result<String, AuthError> {
        let now = chrono::Utc::now().timestamp() as usize;
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &Claims {
                sub: uid.into(),
                username: uname.into(),
                exp: now + self.jwt_expiry,
                iat: now,
            },
            &jsonwebtoken::EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| AuthError::TokenInvalid(e.to_string()))
    }

    pub fn validate_token(&self, t: &str) -> Result<Claims, AuthError> {
        decode::<Claims>(
            t,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map(|d| d.claims)
        .map_err(|e| match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::TokenInvalid(e.to_string()),
        })
    }

    pub fn hash_password(&self, p: &str) -> Result<String, AuthError> {
        bcrypt::hash(p, bcrypt::DEFAULT_COST).map_err(|e| AuthError::Database(e.to_string()))
    }

    pub fn verify_password(&self, p: &str, h: &str) -> Result<bool, AuthError> {
        bcrypt::verify(p, h).map_err(|e| AuthError::Database(e.to_string()))
    }
}

/// Extract user ID from Authorization header
pub fn get_user_id_from_header(
    auth_header: Option<String>,
    auth: &AuthState,
) -> Result<String, AuthError> {
    let auth_str = auth_header.ok_or(AuthError::MissingAuth)?;
    let token = auth_str
        .strip_prefix("Bearer ")
        .ok_or(AuthError::MissingAuth)?;
    let claims = auth.validate_token(token)?;
    Ok(claims.sub)
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub display_name: Option<String>,
    pub public_key_x25519: Option<String>,
    pub public_key_ed25519: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: String,
    pub username: String,
    pub display_name: Option<String>,
}
