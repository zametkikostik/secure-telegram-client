// Secure Messenger Bot Platform
// SECURITY: требует аудита перед production
// TODO: pentest перед release

use axum::{Router, routing::get, Json};
use serde::Serialize;
use std::net::SocketAddr;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "secure_messenger_bots=info".into()),
        )
        .init();

    tracing::info!("Starting Secure Messenger Bots v{}", env!("CARGO_PKG_VERSION"));

    let app = Router::new()
        .route("/health", get(health));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    tracing::info!("Bot server listening on {}", addr);

    axum::serve(
        tokio::net::TcpListener::bind(&addr).await?,
        app,
    )
    .await?;

    Ok(())
}
