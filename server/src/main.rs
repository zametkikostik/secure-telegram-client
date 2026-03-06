// server/src/main.rs
#![recursion_limit = "256"]

mod api;
mod db;
mod websocket;
mod auth;
mod middleware;

use axum::{
    Router,
    routing::{get, post},
    extract::WebSocketUpgrade,
    response::Response,
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
    services::ServeDir,
};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Инициализация логгера
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Загрузка .env
    dotenvy::dotenv().ok();

    // Инициализация базы данных
    let db = db::init_database().await?;
    tracing::info!("База данных инициализирована");

    // Создание состояния приложения
    let app_state = api::AppState {
        db: db.clone(),
        jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret".to_string()),
        uploads_dir: std::env::var("UPLOADS_DIR").unwrap_or_else(|_| "./uploads".to_string()),
    };

    // CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Роуты API
    let api_routes = Router::new()
        .route("/health", get(api::health))
        .route("/auth/register", post(api::auth::register))
        .route("/auth/login", post(api::auth::login))
        .route("/auth/verify", post(api::auth::verify_token))
        .route("/users/me", get(api::users::get_current_user))
        .route("/users/:id", get(api::users::get_user))
        .route("/chats", get(api::chats::list_chats))
        .route("/chats", post(api::chats::create_chat))
        .route("/chats/:chat_id", get(api::chats::get_chat))
        .route("/chats/:chat_id/messages", get(api::messages::list_messages))
        .route("/chats/:chat_id/messages", post(api::messages::send_message))
        .route("/files/upload", post(api::files::upload_file))
        .route("/files/:file_id", get(api::files::get_file))
        .route("/web3/balance", get(api::web3::get_balance))
        .route("/web3/swap", post(api::web3::swap_tokens))
        .route("/ai/translate", post(api::ai::translate))
        .route("/ai/summarize", post(api::ai::summarize))
        .route("/ai/chat", post(api::ai::chat));

    // WebSocket для реального времени
    let ws_routes = Router::new()
        .route("/ws", get(websocket_handler));

    // Основное приложение
    let app = Router::new()
        .nest("/api/v1", api_routes)
        .nest("/ws", ws_routes)
        .nest_service("/uploads", ServeDir::new("./uploads"))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Запуск сервера
    let addr: SocketAddr = std::env::var("SERVER_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8008".to_string())
        .parse()?;

    tracing::info!("🚀 Liberty Reach сервер запущен на {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    // state: State<api::AppState>,
) -> Response {
    ws.on_upgrade(websocket::handle_socket)
}
