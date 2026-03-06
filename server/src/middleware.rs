// server/src/middleware.rs
//! Middleware для аутентификации, rate limiting, и логирования

use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use serde_json::json;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use std::collections::HashMap;
use crate::auth;

/// Rate limiter state
pub struct RateLimiter {
    requests: HashMap<String, Vec<Instant>>,
    limit: u64,
    window: Duration,
}

impl RateLimiter {
    pub fn new(limit: u64, window_secs: u64) -> Self {
        Self {
            requests: HashMap::new(),
            limit,
            window: Duration::from_secs(window_secs),
        }
    }
    
    pub fn is_allowed(&mut self, key: &str) -> bool {
        let now = Instant::now();
        let requests = self.requests.entry(key.to_string()).or_insert_with(Vec::new);
        
        // Удаляем старые запросы
        requests.retain(|&time| now.duration_since(time) < self.window);
        
        if requests.len() < self.limit as usize {
            requests.push(now);
            true
        } else {
            false
        }
    }
}

/// Глобальный rate limiter
pub static RATE_LIMITER: RwLock<Option<RateLimiter>> = RwLock::const_new(None);

/// Инициализация rate limiter
pub async fn init_rate_limiter(limit: u64, window_secs: u64) {
    let mut limiter = RATE_LIMITER.write().await;
    *limiter = Some(RateLimiter::new(limit, window_secs));
}

/// Middleware для проверки JWT токена
pub async fn auth_middleware<B>(
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    let token = auth_header.ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = auth::verify_token(token).map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Добавление claims в request extensions
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

/// Middleware для rate limiting
pub async fn rate_limit_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let client_ip = request
        .headers()
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let mut limiter = RATE_LIMITER.write().await;
    if let Some(limiter) = limiter.as_mut() {
        if !limiter.is_allowed(&client_ip) {
            return Ok((
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({
                    "error": "Too many requests",
                    "retry_after": limiter.window.as_secs()
                }))
            ).into_response());
        }
    }

    Ok(next.run(request).await)
}

/// Middleware для CORS
pub fn cors_middleware() -> tower_http::cors::CorsLayer {
    use tower_http::cors::{Any, CorsLayer};
    
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers([
            axum::http::header::CONTENT_LENGTH,
            axum::http::header::CONTENT_TYPE,
        ])
        .max_age(Duration::from_secs(3600))
}

/// Middleware для security headers
pub async fn security_headers_middleware<B>(
    mut request: Request<B>,
    next: Next<B>,
) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Security headers
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert("Strict-Transport-Security", "max-age=31536000; includeSubDomains".parse().unwrap());
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(3, 60);
        
        assert!(limiter.is_allowed("user1"));
        assert!(limiter.is_allowed("user1"));
        assert!(limiter.is_allowed("user1"));
        assert!(!limiter.is_allowed("user1")); // Превышен лимит
        
        // Другой пользователь
        assert!(limiter.is_allowed("user2"));
    }
}
