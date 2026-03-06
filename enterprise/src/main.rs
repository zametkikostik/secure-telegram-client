//! Secure Telegram Enterprise Server
//! 
//! Корпоративная версия с поддержкой:
//! - SSO (OAuth2, SAML, LDAP, OpenID Connect)
//! - Централизованное аудирование
//! - Админ-панель
//! - Управление пользователями и группами
//! - Compliance и отчётность

mod auth;
mod audit;
mod admin;
mod sso;
mod compliance;
mod api;

use axum::{Router, middleware};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Инициализация логгера
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "enterprise=info,tower_http=debug".into()))
        )
        .with(tracing_appender::rolling::daily("/var/log/secure-telegram", "enterprise.log"))
        .init();

    tracing::info!("🚀 Запуск Secure Telegram Enterprise Server");

    // Инициализация базы данных
    let db = sqlx::PgPool::connect(&std::env::var("DATABASE_URL")?)
        .await?;
    sqlx::migrate!().run(&db).await?;

    // Инициализация Redis
    let redis = redis::Client::open(std::env::var("REDIS_URL")?)?;

    // Инициализация SSO провайдеров
    let sso_config = sso::SSOConfig::load_from_file("config/sso.toml")?;
    let sso_providers = sso::init_providers(&sso_config).await?;

    // Инициализация аудита
    let audit_logger = audit::AuditLogger::new(&db).await?;

    // Создание роутов
    let app = Router::new()
        // API
        .nest("/api/v1", api::create_router())
        // SSO endpoints
        .nest("/auth", sso::create_router(sso_providers))
        // Admin panel
        .nest("/admin", admin::create_router())
        // Health checks
        .route("/health", axum::routing::get(health_check))
        .route("/ready", axum::routing::get(ready_check))
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(middleware::map_request_with_state(
            db.clone(),
            audit::log_request,
        ))
        .with_state(AppState {
            db,
            redis,
            audit_logger,
        });

    // Запуск сервера
    let addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    tracing::info!("📡 Сервер слушает на {}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

#[derive(Clone)]
struct AppState {
    db: sqlx::PgPool,
    redis: redis::Client,
    audit_logger: audit::AuditLogger,
}

async fn health_check() -> &'static str {
    "OK"
}

async fn ready_check() -> &'static str {
    "Ready"
}
