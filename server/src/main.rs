mod enterprise;
mod routes;

mod e2ee;
mod middleware;
mod models;
mod ws;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use enterprise::admin::AdminState;
use middleware::auth::AuthState;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use ws::WsManager;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<SqlitePool>,
    pub auth: Arc<AuthState>,
    pub ws_manager: Arc<WsManager>,
    pub admin_state: Arc<AdminState>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    if let Err(e) = dotenv::dotenv() {
        eprintln!("Warning: Could not load .env file: {}", e);
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    tracing::info!(
        "Starting Secure Messenger Server v{}",
        env!("CARGO_PKG_VERSION")
    );

    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".into());
    let db = SqlitePool::connect(&db_url)
        .await
        .map_err(|e| format!("DB: {}", e))?;
    init_db(&db).await?;

    // Start background self-destruct message cleanup
    start_message_cleanup(db.clone()).await;

    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret".into());
    let jwt_expiry: usize = std::env::var("JWT_EXPIRY")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(86400);

    let state = AppState {
        db: Arc::new(db),
        auth: Arc::new(AuthState::new(&jwt_secret, jwt_expiry)),
        ws_manager: Arc::new(WsManager::new()),
        admin_state: Arc::new(AdminState::new()),
    };

    let app = Router::new()
        // Public routes
        .route("/health", get(health))
        .route("/api/v1/auth/register", post(routes::users::register))
        .route("/api/v1/auth/login", post(routes::users::login))
        .route("/api/v1/users/:id", get(routes::users::get_user))
        .route("/api/v1/users/:id/keys", get(routes::users::get_keys))
        // WebSocket
        .route("/api/v1/ws", get(ws::ws_handler))
        // Auth required routes
        .route("/api/v1/users/me", get(routes::users::get_me))
        .route("/api/v1/users/me", post(routes::users::update_me))
        .route("/api/v1/users/keys", post(routes::users::update_keys))
        .route(
            "/api/v1/users/:id/status",
            get(routes::users::get_user_status),
        )
        .route("/api/v1/users/status", post(routes::users::update_status))
        .route("/api/v1/users/search", get(routes::users::search_users))
        .route("/api/v1/chats", get(routes::chats::list_chats))
        .route("/api/v1/chats", post(routes::chats::create_chat))
        .route("/api/v1/chats/:id", get(routes::chats::get_chat))
        .route(
            "/api/v1/chats/:id/messages",
            get(routes::messages::list_messages),
        )
        .route(
            "/api/v1/chats/:id/messages",
            post(routes::messages::send_message),
        )
        .route(
            "/api/v1/chats/:id/messages/e2ee",
            post(routes::messages::send_message_e2ee),
        )
        .route(
            "/api/v1/messages/:id/read",
            post(routes::messages::mark_read),
        )
        .route(
            "/api/v1/messages/:id/reactions",
            post(routes::messages::add_reaction),
        )
        .route(
            "/api/v1/messages/:id/reactions",
            delete(routes::messages::remove_reaction),
        )
        .route(
            "/api/v1/messages/:id/reactions",
            get(routes::messages::get_message_reactions),
        )
        .route(
            "/api/v1/chats/:id/typing",
            post(routes::messages::typing_indicator),
        )
        .route(
            "/api/v1/messages/:id/receipts",
            get(routes::messages::get_read_receipts),
        )
        .route("/api/v1/messages/:id", get(routes::messages::get_message))
        .route("/api/v1/messages/:id", put(routes::messages::edit_message))
        .route(
            "/api/v1/messages/:id",
            delete(routes::messages::delete_message),
        )
        .route(
            "/api/v1/messages/:id/pin",
            post(routes::messages::pin_message),
        )
        .route(
            "/api/v1/messages/:id/unpin",
            post(routes::messages::unpin_message),
        )
        .route(
            "/api/v1/chats/:id/pinned",
            get(routes::messages::get_pinned_messages),
        )
        .route(
            "/api/v1/chats/:id/unread",
            get(routes::messages::get_unread_count),
        )
        .route(
            "/api/v1/chats/:id/participants",
            get(routes::chats::get_participants),
        )
        .route(
            "/api/v1/chats/:id/participants",
            post(routes::chats::add_participant),
        )
        .route(
            "/api/v1/chats/:id/participants/remove",
            delete(routes::chats::remove_participant),
        )
        .route(
            "/api/v1/chats/:id/wallpaper",
            get(routes::chats::get_wallpaper),
        )
        .route(
            "/api/v1/chats/:id/wallpaper",
            put(routes::chats::set_wallpaper),
        )
        .route("/api/v1/chats/:id/leave", post(routes::chats::leave_chat))
        // Bot Platform routes
        .route("/api/v1/bots", get(routes::bots::list_my_bots))
        .route("/api/v1/bots", post(routes::bots::create_bot))
        .route("/api/v1/bots/:id", get(routes::bots::get_bot))
        .route("/api/v1/bots/:id", put(routes::bots::update_bot))
        .route("/api/v1/bots/:id", delete(routes::bots::delete_bot))
        .route(
            "/api/v1/bots/:id/token/rotate",
            post(routes::bots::rotate_token),
        )
        .route(
            "/api/v1/bots/:id/commands",
            get(routes::bots::list_commands),
        )
        .route(
            "/api/v1/bots/:id/commands",
            post(routes::bots::create_command),
        )
        .route(
            "/api/v1/bots/:id/commands/:cmd",
            delete(routes::bots::delete_command),
        )
        .route(
            "/api/v1/bots/:id/webhooks",
            get(routes::bots::list_webhooks),
        )
        .route(
            "/api/v1/bots/:id/webhooks",
            post(routes::bots::create_webhook),
        )
        .route(
            "/api/v1/bots/:id/webhooks/:wh",
            delete(routes::bots::delete_webhook),
        )
        .route("/api/v1/bots/store", get(routes::bots::list_store))
        .route(
            "/api/v1/bots/store/:id/install",
            post(routes::bots::install_from_store),
        )
        .route(
            "/api/v1/bots/store/:id/uninstall",
            post(routes::bots::uninstall_from_store),
        )
        .route("/api/v1/bots/search", get(routes::bots::search_bots))
        // Payment routes
        .route("/api/v1/credits", get(routes::payments::get_credits))
        .route(
            "/api/v1/credits/history",
            get(routes::payments::get_credit_history),
        )
        .route(
            "/api/v1/credits/purchase",
            post(routes::payments::add_credits),
        )
        .route("/api/v1/tips", post(routes::payments::send_tip))
        .route(
            "/api/v1/payments/stripe/webhook",
            post(routes::payments::stripe_webhook),
        )
        // File routes
        .route("/api/v1/files", post(routes::files::upload_file))
        .route("/api/v1/files/:id", get(routes::files::download_file))
        .route("/api/v1/files/:id/info", get(routes::files::get_file_info))
        // Stories routes
        .route("/api/v1/stories", post(routes::stories::create_story))
        .route("/api/v1/stories", get(routes::stories::get_stories))
        .route(
            "/api/v1/stories/user/:id",
            get(routes::stories::get_user_stories),
        )
        .route(
            "/api/v1/stories/media/:name",
            get(routes::stories::get_story_media),
        )
        .route(
            "/api/v1/stories/:id/view",
            post(routes::stories::view_story),
        )
        .route("/api/v1/stories/:id", delete(routes::stories::delete_story))
        // Admin routes
        .merge(enterprise::admin::admin_router())
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Listening on {}", addr);

    axum::serve(tokio::net::TcpListener::bind(&addr).await?, app).await?;

    Ok(())
}

async fn health() -> impl axum::response::IntoResponse {
    axum::Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "ts": chrono::Utc::now().timestamp()
    }))
}

async fn init_db(db: &SqlitePool) -> Result<(), String> {
    // Users table with E2EE keys
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            display_name TEXT,
            public_key_x25519 TEXT,
            public_key_ed25519 TEXT,
            avatar_url TEXT,
            family_status TEXT DEFAULT 'none',
            is_online INTEGER DEFAULT 0,
            last_seen TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        )",
    )
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    // Migration: add family_status if not exists
    sqlx::query("ALTER TABLE users ADD COLUMN family_status TEXT DEFAULT 'none'")
        .execute(db)
        .await
        .ok();

    // Chat wallpapers
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS chat_wallpapers (
            chat_id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            color TEXT DEFAULT '#1e293b',
            pattern TEXT DEFAULT 'solid',
            custom_url TEXT,
            updated_at TEXT DEFAULT (datetime('now'))
        )",
    )
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    // Pinned messages
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS pinned_messages (
            chat_id TEXT NOT NULL,
            message_id TEXT NOT NULL,
            pinned_by TEXT NOT NULL,
            pinned_at TEXT DEFAULT (datetime('now')),
            PRIMARY KEY (chat_id, message_id)
        )",
    )
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    // Chats table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS chats (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            chat_type TEXT DEFAULT 'direct',
            created_by TEXT NOT NULL,
            created_at TEXT DEFAULT (datetime('now'))
        )",
    )
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    // Chat participants
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS chat_participants (
            chat_id TEXT,
            user_id TEXT,
            role TEXT DEFAULT 'member',
            joined_at TEXT DEFAULT (datetime('now')),
            PRIMARY KEY (chat_id, user_id)
        )",
    )
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    // Messages with E2EE support
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            chat_id TEXT NOT NULL,
            sender_id TEXT NOT NULL,
            content TEXT NOT NULL,
            msg_type TEXT DEFAULT 'text',
            reply_to TEXT,
            created_at TEXT DEFAULT (datetime('now')),
            edited_at TEXT,
            is_encrypted INTEGER DEFAULT 0,
            ephemeral_key TEXT,
            nonce TEXT,
            destroy_at TEXT,
            scheduled_for TEXT
        )",
    )
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    // Migration: add missing columns to messages
    sqlx::query("ALTER TABLE messages ADD COLUMN destroy_at TEXT")
        .execute(db)
        .await
        .ok();
    sqlx::query("ALTER TABLE messages ADD COLUMN scheduled_for TEXT")
        .execute(db)
        .await
        .ok();
    sqlx::query("ALTER TABLE messages ADD COLUMN edited_at TEXT")
        .execute(db)
        .await
        .ok();

    // Read receipts
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS read_receipts (
            message_id TEXT,
            user_id TEXT,
            read_at TEXT DEFAULT (datetime('now')),
            PRIMARY KEY (message_id, user_id)
        )",
    )
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_mc ON messages(chat_id)")
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_mt ON messages(created_at DESC)")
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_rr_user ON read_receipts(user_id)")
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

    // Files table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            owner_id TEXT NOT NULL,
            file_name TEXT NOT NULL,
            storage_name TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            mime_type TEXT NOT NULL,
            created_at TEXT DEFAULT (datetime('now')),
            download_count INTEGER DEFAULT 0
        )",
    )
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_files_owner ON files(owner_id)")
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

    // Reactions table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS reactions (
            message_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            emoji TEXT NOT NULL,
            created_at TEXT DEFAULT (datetime('now')),
            PRIMARY KEY (message_id, user_id, emoji)
        )",
    )
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_reactions_msg ON reactions(message_id)")
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

    // Stories tables
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS stories (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            media_url TEXT NOT NULL,
            media_type TEXT DEFAULT 'image',
            caption TEXT,
            created_at TEXT DEFAULT (datetime('now')),
            expires_at TEXT NOT NULL,
            view_count INTEGER DEFAULT 0
        )",
    )
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS story_views (
            story_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            viewed_at TEXT DEFAULT (datetime('now')),
            PRIMARY KEY (story_id, user_id)
        )",
    )
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_stories_user ON stories(user_id)")
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_stories_expires ON stories(expires_at)")
        .execute(db)
        .await
        .map_err(|e| e.to_string())?;

    // Delete expired self-destruct messages on startup
    let _ = sqlx::query(
        "DELETE FROM messages WHERE destroy_at IS NOT NULL AND destroy_at < datetime('now')",
    )
    .execute(db)
    .await;

    tracing::info!("DB initialized with E2EE, files, reactions, stories and self-destruct support");
    Ok(())
}

/// Background task: delete expired self-destruct messages periodically
async fn start_message_cleanup(db: SqlitePool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            // Delete expired self-destruct messages
            if let Ok(result) = sqlx::query("DELETE FROM messages WHERE destroy_at IS NOT NULL AND destroy_at < datetime('now')")
                .execute(&db).await {
                if result.rows_affected() > 0 {
                    tracing::info!("Deleted {} expired self-destruct messages", result.rows_affected());
                }
            }
            // TODO: Send scheduled messages where scheduled_for <= now()
        }
    });
}
