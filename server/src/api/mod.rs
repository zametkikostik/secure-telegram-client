// server/src/api/mod.rs
//! API роуты и обработчики

pub mod auth;
pub mod users;
pub mod chats;
pub mod messages;
pub mod files;
pub mod web3;
pub mod ai;
pub mod nodes;
pub mod features;

use axum::{Router, routing::{get, post}};
use sqlx::SqlitePool;
use std::sync::Arc;
use tower_http::{cors::{Any, CorsLayer}, trace::TraceLayer};

/// Состояние приложения
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<SqlitePool>,
    pub jwt_secret: String,
    pub uploads_dir: String,
}

/// Проверка здоровья сервера
pub async fn health() -> &'static str {
    "OK"
}

/// Создание роутера приложения
pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health))
        // Auth
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/verify", post(auth::verify_token))
        // Users
        .route("/users/me", get(users::get_current_user))
        .route("/users/:id", get(users::get_user))
        // Chats
        .route("/chats", get(chats::list_chats))
        .route("/chats", post(chats::create_chat))
        .route("/chats/:chat_id", get(chats::get_chat))
        .route("/chats/:chat_id/messages", get(messages::list_messages))
        .route("/chats/:chat_id/messages", post(messages::send_message))
        // New Features
        .route("/users/:user_id/family-status", get(features::get_family_status))
        .route("/users/:user_id/family-status", post(features::set_family_status))
        .route("/chats/:chat_id/wallpaper", get(features::get_chat_wallpaper))
        .route("/chats/:chat_id/wallpaper", post(features::set_chat_wallpaper))
        .route("/wallpapers", get(features::list_wallpapers))
        .route("/chats/:chat_id/auto-delete", post(features::set_auto_delete))
        .route("/chats/:chat_id/messages/auto-delete", post(features::send_auto_delete_message))
        // Files
        .route("/files/upload", post(files::upload_file))
        .route("/files/:file_id", get(files::get_file))
        // Web3
        .route("/web3/balance", get(web3::get_balance))
        .route("/web3/swap", post(web3::swap_tokens))
        // AI
        .route("/ai/translate", post(ai::translate))
        .route("/ai/summarize", post(ai::summarize))
        .route("/ai/chat", post(ai::chat))
        // Nodes
        .route("/nodes/register", post(nodes::register_node))
        .route("/nodes/list", get(nodes::get_peer_list))
        .route("/nodes/heartbeat", post(nodes::node_heartbeat))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
