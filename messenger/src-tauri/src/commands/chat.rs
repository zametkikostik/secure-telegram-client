// Chat commands
// TODO: реализовать P2P чат (Фаза 3)

use tauri::State;
use crate::state::AppState;

/// Отправка сообщения
#[tauri::command]
pub async fn send_message(
    _chat_id: String,
    _message: String,
    _state: State<'_, AppState>,
) -> Result<String, String> {
    // TODO: реализовать P2P отправку
    Err("Not implemented yet".to_string())
}

/// Получение сообщений
#[tauri::command]
pub async fn get_messages(
    _chat_id: String,
    _limit: usize,
    _state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    // TODO: реализовать получение сообщений
    Ok(vec![])
}
