// Tauri commands
// SECURITY: требует аудита перед production
// TODO: pentest перед release

pub mod crypto;
pub mod chat;
pub mod user;

use serde::Serialize;

#[derive(Serialize)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
}

/// Простая проверка связи
#[tauri::command]
pub fn ping() -> CommandResult {
    CommandResult { success: true, message: "pong".into() }
}
