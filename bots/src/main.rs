//! Secure Telegram Bots Platform
//! 
//! BotFather аналог + ManyChat конструктор ботов
//! 
//! Функции:
//! - Создание ботов через @BotFather
//! - Конструктор ботов (drag-and-drop)
//! - Webhooks для ботов
//! - IPFS интеграция через Pinata.cloud

mod api;
mod bot_engine;
mod builder;
mod ipfs;

use axum::{Router, routing::{get, post}};
use tower_http::{cors::{Any, CorsLayer}, trace::TraceLayer};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    db: Arc<SqlitePool>,
    pinata_api_key: String,
    pinata_secret_key: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Инициализация логгера
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "bots=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🤖 Запуск Secure Telegram Bots Platform");

    // Инициализация базы данных
    let db = SqlitePoolOptions::new()
        .max_connections(10)
        .connect("sqlite:./bots.db")
        .await?;

    // Создание таблиц
    init_database(&db).await?;

    // Создание состояния приложения
    let state = AppState {
        db: Arc::new(db),
        pinata_api_key: std::env::var("PINATA_API_KEY").unwrap_or_default(),
        pinata_secret_key: std::env::var("PINATA_SECRET_KEY").unwrap_or_default(),
    };

    // CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Создание роутера
    let app = Router::new()
        .route("/health", get(health))
        // BotFather API
        .route("/api/v1/bots", get(api::list_bots))
        .route("/api/v1/bots", post(api::create_bot))
        .route("/api/v1/bots/:bot_id", get(api::get_bot))
        .route("/api/v1/bots/:bot_id", delete(api::delete_bot))
        .route("/api/v1/bots/:bot_id/token", post(api::regenerate_token))
        // Bot Builder (ManyChat analog)
        .route("/api/v1/bots/:bot_id/flows", get(api::list_flows))
        .route("/api/v1/bots/:bot_id/flows", post(api::create_flow))
        .route("/api/v1/bots/:bot_id/flows/:flow_id", get(api::get_flow))
        .route("/api/v1/bots/:bot_id/flows/:flow_id", put(api::update_flow))
        .route("/api/v1/bots/:bot_id/flows/:flow_id", delete(api::delete_flow))
        // Bot blocks
        .route("/api/v1/bots/:bot_id/blocks", get(api::list_blocks))
        .route("/api/v1/bots/:bot_id/blocks", post(api::create_block))
        .route("/api/v1/blocks/:block_id", put(api::update_block))
        .route("/api/v1/blocks/:block_id", delete(api::delete_block))
        // Webhooks
        .route("/api/v1/bots/:bot_id/webhook", get(api::get_webhook))
        .route("/api/v1/bots/:bot_id/webhook", post(api::set_webhook))
        .route("/api/v1/bots/:bot_id/webhook", delete(api::delete_webhook))
        // Bot stats
        .route("/api/v1/bots/:bot_id/stats", get(api::get_bot_stats))
        // IPFS
        .route("/api/v1/ipfs/upload", post(ipfs::upload_to_ipfs))
        .route("/api/v1/ipfs/:cid", get(ipfs::get_from_ipfs))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Запуск сервера
    let addr = std::env::var("BOTS_ADDR").unwrap_or_else(|_| "0.0.0.0:8081".into());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("📡 Bots Platform слушает на {}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health() -> &'static str {
    "OK"
}

async fn init_database(db: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        -- Боты
        CREATE TABLE IF NOT EXISTS bots (
            id TEXT PRIMARY KEY,
            owner_id TEXT NOT NULL,
            username TEXT UNIQUE NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            token TEXT UNIQUE NOT NULL,
            avatar_url TEXT,
            is_verified BOOLEAN DEFAULT FALSE,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Flow (сценарии бота)
        CREATE TABLE IF NOT EXISTS bot_flows (
            id TEXT PRIMARY KEY,
            bot_id TEXT REFERENCES bots(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            description TEXT,
            is_active BOOLEAN DEFAULT TRUE,
            trigger_type TEXT DEFAULT 'keyword',
            trigger_value TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Блоки (шаги в flow)
        CREATE TABLE IF NOT EXISTS bot_blocks (
            id TEXT PRIMARY KEY,
            flow_id TEXT REFERENCES bot_flows(id) ON DELETE CASCADE,
            block_type TEXT NOT NULL,
            content TEXT,
            position INTEGER DEFAULT 0,
            next_block_id TEXT,
            conditions TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Webhooks
        CREATE TABLE IF NOT EXISTS bot_webhooks (
            id TEXT PRIMARY KEY,
            bot_id TEXT REFERENCES bots(id) ON DELETE CASCADE,
            url TEXT NOT NULL,
            secret TEXT,
            events TEXT,
            is_active BOOLEAN DEFAULT TRUE,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Сообщения от ботов
        CREATE TABLE IF NOT EXISTS bot_messages (
            id TEXT PRIMARY KEY,
            bot_id TEXT REFERENCES bots(id) ON DELETE CASCADE,
            chat_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            content TEXT NOT NULL,
            message_type TEXT DEFAULT 'text',
            file_url TEXT,
            sent_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Подписчики ботов
        CREATE TABLE IF NOT EXISTS bot_subscribers (
            bot_id TEXT REFERENCES bots(id) ON DELETE CASCADE,
            user_id TEXT NOT NULL,
            subscribed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            last_interaction DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (bot_id, user_id)
        );

        -- IPFS файлы
        CREATE TABLE IF NOT EXISTS ipfs_files (
            id TEXT PRIMARY KEY,
            cid TEXT NOT NULL,
            filename TEXT NOT NULL,
            owner_id TEXT,
            size INTEGER,
            uploaded_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Индексы
        CREATE INDEX IF NOT EXISTS idx_bots_owner ON bots(owner_id);
        CREATE INDEX IF NOT EXISTS idx_flows_bot ON bot_flows(bot_id);
        CREATE INDEX IF NOT EXISTS idx_blocks_flow ON bot_blocks(flow_id);
        CREATE INDEX IF NOT EXISTS idx_messages_bot ON bot_messages(bot_id);
        CREATE INDEX IF NOT EXISTS idx_subscribers_bot ON bot_subscribers(bot_id);
        "#,
    )
    .execute(db)
    .await?;

    tracing::info!("📊 База данных ботов инициализирована");

    Ok(())
}
