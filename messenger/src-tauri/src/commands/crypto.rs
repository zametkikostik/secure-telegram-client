// Crypto commands for Tauri
// SECURITY: требует аудита перед production
// TODO: pentest перед release

use tauri::State;
use crate::state::AppState;

/// Генерация новой пары ключей
#[tauri::command]
pub async fn generate_keypair(state: State<'_, AppState>) -> Result<String, String> {
    tracing::info!("Generating new keypair");

    // TODO: использовать crypto crate для генерации ключей
    *state.user_id.write().await = Some("temp-user-id".to_string());

    Ok("{\"status\": \"keypair_generated\"}".to_string())
}

/// Шифрование сообщения
#[tauri::command]
pub async fn encrypt_message(
    _message: String,
    _recipient_public: String,
) -> Result<String, String> {
    // TODO: реализовать шифрование через HybridEncryptor
    tracing::debug!("Encrypting message for recipient");
    Err("Not implemented yet".to_string())
}

/// Расшифровка сообщения
#[tauri::command]
pub async fn decrypt_message(
    _encrypted_message: String,
) -> Result<String, String> {
    // TODO: реализовать расшифрование через HybridEncryptor
    tracing::debug!("Decrypting message");
    Err("Not implemented yet".to_string())
}
