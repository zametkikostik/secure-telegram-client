//! Admin Panel
//! 
//! Функции:
//! - Управление пользователями
//! - Верификация с бейджами
//! - Модерация контента
//! - Статистика платформы
//! - Управление ботами

mod api;
mod auth;
mod templates;

use axum::{Router, routing::{get, post}};
use tower_http::{cors::{Any, CorsLayer}, trace::TraceLayer};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone)]
pub struct AppState {
    db: Arc<SqlitePool>,
    admin_secret: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Инициализация логгера
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "admin=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🛡️ Запуск Admin Panel");

    // Инициализация базы данных
    let db = SqlitePoolOptions::new()
        .max_connections(10)
        .connect("sqlite:./admin.db")
        .await?;

    // Создание таблиц
    init_database(&db).await?;

    // Создание состояния приложения
    let state = AppState {
        db: Arc::new(db),
        admin_secret: std::env::var("ADMIN_SECRET").unwrap_or_else(|| "admin-secret-change-me".into()),
    };

    // CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Создание роутера
    let app = Router::new()
        .route("/health", get(health))
        // Auth
        .route("/admin/login", post(auth::login))
        .route("/admin/logout", post(auth::logout))
        .route("/admin/me", get(auth::get_current_admin))
        // Dashboard
        .route("/admin/dashboard", get(api::get_dashboard))
        // Users
        .route("/admin/users", get(api::list_users))
        .route("/admin/users/:user_id", get(api::get_user))
        .route("/admin/users/:user_id/ban", post(api::ban_user))
        .route("/admin/users/:user_id/unban", post(api::unban_user))
        // Verification
        .route("/admin/verification/requests", get(api::list_verification_requests))
        .route("/admin/verification/:user_id", post(api::verify_user))
        .route("/admin/verification/:user_id/revoke", post(api::revoke_verification))
        // Badges
        .route("/admin/badges", get(api::list_badges))
        .route("/admin/badges", post(api::create_badge))
        .route("/admin/badges/:badge_id", delete(api::delete_badge))
        .route("/admin/users/:user_id/badges", post(api::assign_badge))
        // Moderation
        .route("/admin/reports", get(api::list_reports))
        .route("/admin/reports/:report_id", post(api::handle_report))
        // Bots
        .route("/admin/bots", get(api::list_bots))
        .route("/admin/bots/:bot_id/verify", post(api::verify_bot))
        // System
        .route("/admin/settings", get(api::get_settings))
        .route("/admin/settings", put(api::update_settings))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Запуск сервера
    let addr = std::env::var("ADMIN_ADDR").unwrap_or_else(|_| "0.0.0.0:8082".into());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("📡 Admin Panel слушает на {}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health() -> &'static str {
    "OK"
}

async fn init_database(db: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(
        r#"
        -- Админы
        CREATE TABLE IF NOT EXISTS admins (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            email TEXT,
            role TEXT DEFAULT 'moderator',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            last_login DATETIME
        );

        -- Верификации
        CREATE TABLE IF NOT EXISTS verifications (
            id TEXT PRIMARY KEY,
            user_id TEXT NOT NULL,
            type TEXT NOT NULL,
            document_url TEXT,
            status TEXT DEFAULT 'pending',
            reviewed_by TEXT REFERENCES admins(id),
            reviewed_at DATETIME,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Бейджи
        CREATE TABLE IF NOT EXISTS badges (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            icon_url TEXT,
            color TEXT DEFAULT '#3390EC',
            is_verified_badge BOOLEAN DEFAULT FALSE,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Бейджи пользователей
        CREATE TABLE IF NOT EXISTS user_badges (
            user_id TEXT NOT NULL,
            badge_id TEXT REFERENCES badges(id) ON DELETE CASCADE,
            assigned_by TEXT REFERENCES admins(id),
            assigned_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (user_id, badge_id)
        );

        -- Жалобы
        CREATE TABLE IF NOT EXISTS reports (
            id TEXT PRIMARY KEY,
            reporter_id TEXT,
            reported_user_id TEXT,
            reported_message_id TEXT,
            reason TEXT NOT NULL,
            description TEXT,
            status TEXT DEFAULT 'pending',
            reviewed_by TEXT REFERENCES admins(id),
            reviewed_at DATETIME,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Настройки системы
        CREATE TABLE IF NOT EXISTS system_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_by TEXT REFERENCES admins(id),
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Сессии админов
        CREATE TABLE IF NOT EXISTS admin_sessions (
            id TEXT PRIMARY KEY,
            admin_id TEXT REFERENCES admins(id) ON DELETE CASCADE,
            token TEXT UNIQUE NOT NULL,
            expires_at DATETIME NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Индексы
        CREATE INDEX IF NOT EXISTS idx_verifications_user ON verifications(user_id);
        CREATE INDEX IF NOT EXISTS idx_verifications_status ON verifications(status);
        CREATE INDEX IF NOT EXISTS idx_user_badges_user ON user_badges(user_id);
        CREATE INDEX IF NOT EXISTS idx_reports_status ON reports(status);
        CREATE INDEX IF NOT EXISTS idx_admin_sessions_token ON admin_sessions(token);

        -- Данные по умолчанию
        INSERT OR IGNORE INTO badges (id, name, description, icon_url, color, is_verified_badge)
        VALUES 
            ('verified', '✓', 'Верифицированный пользователь', '/badges/verified.svg', '#3390EC', TRUE),
            ('premium', '★', 'Premium подписчик', '/badges/premium.svg', '#FFD700', FALSE),
            ('bot', '🤖', 'Официальный бот', '/badges/bot.svg', '#FF6B6B', FALSE),
            ('support', '🎧', 'Поддержка', '/badges/support.svg', '#4ECDC4', FALSE),
            ('admin', '🛡️', 'Администратор', '/badges/admin.svg', '#FF4757', FALSE);

        INSERT OR IGNORE INTO system_settings (key, value, updated_at)
        VALUES 
            ('registration_enabled', 'true', CURRENT_TIMESTAMP),
            ('verification_enabled', 'true', CURRENT_TIMESTAMP),
            ('max_bots_per_user', '10', CURRENT_TIMESTAMP);
        "#,
    )
    .execute(db)
    .await?;

    tracing::info!("📊 База данных админ-панели инициализирована");

    Ok(())
}
