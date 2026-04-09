// Secure Messenger Tauri - Binary entry point
// SECURITY: требует аудита перед production
// TODO: pentest перед release

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_logging() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "secure_messenger_tauri=debug,tauri=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

fn main() {
    init_logging();
    tracing::info!("Starting Secure Messenger Tauri v{}", env!("CARGO_PKG_VERSION"));

    let app_state = secure_messenger_tauri::AppState::new();

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            secure_messenger_tauri::commands::ping,
        ])
        .setup(|_app| {
            tracing::info!("App setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
