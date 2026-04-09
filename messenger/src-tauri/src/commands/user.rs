// User commands
// TODO: реализовать управление профилем

use tauri::State;
use crate::state::AppState;

/// Получение профиля пользователя
#[tauri::command]
pub async fn get_profile(state: State<'_, AppState>) -> Result<String, String> {
    let user_id = state.user_id.read().await;
    match user_id.as_ref() {
        Some(id) => Ok(format!("{{\"id\":\"{}\"}}", id)),
        None => Ok("{}".to_string()),
    }
}

/// Обновление профиля
#[tauri::command]
pub async fn update_profile(
    _display_name: Option<String>,
    _avatar: Option<String>,
    _state: State<'_, AppState>,
) -> Result<(), String> {
    // TODO: реализовать обновление профиля
    Ok(())
}
