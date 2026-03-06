// server/tests/api_test.rs
//! Интеграционные тесты для API

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::sync::Arc;
use tower::ServiceExt;

type AppState = liberty_reach_server::api::AppState;

async fn create_test_db() -> SqlitePool {
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite::memory:")
        .await
        .unwrap()
}

async fn create_app(db: SqlitePool) -> Router {
    let state = AppState {
        db: Arc::new(db),
        jwt_secret: "test-secret".to_string(),
        uploads_dir: "./uploads".to_string(),
    };
    
    liberty_reach_server::api::create_router(state)
}

#[tokio::test]
async fn test_health_endpoint() {
    let db = create_test_db().await;
    let app = create_app(db).await;
    
    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_register_user() {
    let db = create_test_db().await;
    let app = create_app(db).await;
    
    let body = serde_json::json!({
        "username": "testuser",
        "password": "password123",
        "email": "test@example.com"
    });
    
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["username"], "testuser");
    assert!(json["token"].as_str().is_some());
}

#[tokio::test]
async fn test_login() {
    let db = create_test_db().await;
    let app = create_app(db).await;
    
    // Сначала регистрируем пользователя
    let register_body = serde_json::json!({
        "username": "loginuser",
        "password": "password123"
    });
    
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(register_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Теперь логинимся
    let login_body = serde_json::json!({
        "username": "loginuser",
        "password": "password123"
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(login_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["username"], "loginuser");
    assert!(json["token"].as_str().is_some());
}

#[tokio::test]
async fn test_login_wrong_password() {
    let db = create_test_db().await;
    let app = create_app(db).await;
    
    // Регистрируем
    let register_body = serde_json::json!({
        "username": "wrongpassuser",
        "password": "password123"
    });
    
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/register")
                .header("Content-Type", "application/json")
                .body(Body::from(register_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    // Логин с неправильным паролем
    let login_body = serde_json::json!({
        "username": "wrongpassuser",
        "password": "wrongpassword"
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("Content-Type", "application/json")
                .body(Body::from(login_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
