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
pub mod extra;

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
        .route("/users/:user_id/bio", get(features::get_family_status))
        .route("/users/:user_id/bio", post(features::set_family_status))
        // Chats
        .route("/chats", get(chats::list_chats))
        .route("/chats", post(chats::create_chat))
        .route("/chats/:chat_id", get(chats::get_chat))
        .route("/chats/:chat_id/messages", get(messages::list_messages))
        .route("/chats/:chat_id/messages", post(messages::send_message))
        // Pinned Messages
        .route("/chats/:chat_id/pinned", get(extra::get_pinned_messages))
        .route("/chats/:chat_id/pin", post(extra::pin_message))
        .route("/chats/:chat_id/unpin/:message_id", post(extra::unpin_message))
        // Saved Messages (Favorites/Notes)
        .route("/users/:user_id/saved", get(extra::get_saved_messages))
        .route("/users/:user_id/saved", post(extra::save_message))
        .route("/users/:user_id/saved/:message_id", delete(extra::delete_saved_message))
        // Scheduled Messages
        .route("/chats/:chat_id/scheduled", get(extra::get_scheduled_messages))
        .route("/chats/:chat_id/schedule", post(extra::schedule_message))
        .route("/chats/:chat_id/scheduled/:message_id", post(extra::cancel_scheduled_message))
        // Stickers & GIFs
        .route("/stickers", get(extra::list_stickers))
        .route("/sticker-packs", get(extra::list_sticker_packs))
        .route("/gifs", get(extra::list_gifs))
        // Reactions
        .route("/messages/:message_id/reactions", get(extra::get_reactions))
        .route("/messages/:message_id/reactions", post(extra::add_reaction))
        // Themes
        .route("/themes", get(extra::list_themes))
        .route("/users/:user_id/theme/:theme", post(extra::set_user_theme))
        .route("/users/:user_id/night-mode/:enabled", post(extra::set_night_mode))
        // Screen Share
        .route("/screen-share", post(extra::start_screen_share))
        .route("/screen-share/:session_id", post(extra::stop_screen_share))
        .route("/chats/:chat_id/screen-share", get(extra::get_active_screen_shares))
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

use axum::routing::delete;
