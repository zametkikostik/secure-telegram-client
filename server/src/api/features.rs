// server/src/api/features.rs
//! Новые функции: 24h сообщения, семейные статусы, синхронизированные обои

use axum::{
    extract::{State, Path},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};

#[derive(Serialize)]
pub struct FamilyStatus {
    pub status: String,
    pub partner_id: Option<String>,
    pub partner_name: Option<String>,
}

#[derive(Deserialize)]
pub struct SetFamilyStatusRequest {
    pub status: String,
    pub partner_id: Option<String>,
}

#[derive(Serialize)]
pub struct WallpaperResponse {
    pub chat_id: String,
    pub wallpaper_url: String,
    pub wallpaper_type: String,
    pub synced: bool,
}

#[derive(Deserialize)]
pub struct SetWallpaperRequest {
    pub wallpaper_url: String,
    pub wallpaper_type: Option<String>,
    pub sync_to_chat: Option<bool>,
}

#[derive(Serialize)]
pub struct AutoDeleteMessageRequest {
    pub hours: u32,
}

/// Получить семейный статус пользователя
pub async fn get_family_status(
    State(_db): State<crate::api::AppState>,
    user_id: String,
) -> Result<Json<FamilyStatus>, StatusCode> {
    // TODO: Получить из БД
    Ok(Json(FamilyStatus {
        status: "single".to_string(),
        partner_id: None,
        partner_name: None,
    }))
}

/// Установить семейный статус
pub async fn set_family_status(
    State(db): State<crate::api::AppState>,
    user_id: String,
    Json(req): Json<SetFamilyStatusRequest>,
) -> Result<Json<FamilyStatus>, StatusCode> {
    // Обновление статуса пользователя
    sqlx::query("UPDATE users SET family_status = ? WHERE id = ?")
        .bind(&req.status)
        .bind(&user_id)
        .execute(&db.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Если статус "married" или "in_relationship" и указан партнёр
    if let Some(partner_id) = req.partner_id {
        if req.status == "married" || req.status == "in_relationship" {
            // Создание семейной связи
            let relation_id = uuid::Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT OR REPLACE INTO family_relations (id, user_id, relative_id, relation_type)
                 VALUES (?, ?, ?, 'partner')"
            )
            .bind(&relation_id)
            .bind(&user_id)
            .bind(&partner_id)
            .execute(&db.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            // Обновление статуса партнёра
            sqlx::query("UPDATE users SET family_status = ? WHERE id = ?")
                .bind(&req.status)
                .bind(&partner_id)
                .execute(&db.db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }

    Ok(Json(FamilyStatus {
        status: req.status,
        partner_id: req.partner_id,
        partner_name: None, // TODO: Получить имя партнёра
    }))
}

/// Получить обои чата
pub async fn get_chat_wallpaper(
    State(db): State<crate::api::AppState>,
    Path((chat_id, user_id)): Path<(String, String)>,
) -> Result<Json<WallpaperResponse>, StatusCode> {
    let wallpaper = sqlx::query_as(
        "SELECT wallpaper_url, wallpaper_type, synced FROM chat_wallpapers
         WHERE chat_id = ? AND user_id = ?"
    )
    .bind(&chat_id)
    .bind(&user_id)
    .fetch_optional(&db.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match wallpaper {
        Some((url, wtype, synced)) => Ok(Json(WallpaperResponse {
            chat_id,
            wallpaper_url: url,
            wallpaper_type: wtype,
            synced,
        })),
        None => {
            // Обои по умолчанию
            Ok(Json(WallpaperResponse {
                chat_id,
                wallpaper_url: "/wallpapers/default.jpg".to_string(),
                wallpaper_type: "default".to_string(),
                synced: false,
            }))
        }
    }
}

/// Установить обои чата
pub async fn set_chat_wallpaper(
    State(db): State<crate::api::AppState>,
    Path((chat_id, user_id)): Path<(String, String)>,
    Json(req): Json<SetWallpaperRequest>,
) -> Result<Json<WallpaperResponse>, StatusCode> {
    let sync = req.sync_to_chat.unwrap_or(false);
    let wallpaper_type = req.wallpaper_type.unwrap_or_else(|| "custom".to_string());

    // Сохранение обоев для пользователя
    sqlx::query(
        "INSERT OR REPLACE INTO chat_wallpapers (chat_id, user_id, wallpaper_url, wallpaper_type, synced, updated_at)
         VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)"
    )
    .bind(&chat_id)
    .bind(&user_id)
    .bind(&req.wallpaper_url)
    .bind(&wallpaper_type)
    .bind(&sync)
    .execute(&db.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Если синхронизация включена, обновить обои чата для всех участников
    if sync {
        // Обновление общих обоев чата
        sqlx::query("UPDATE chats SET wallpaper_url = ?, wallpaper_sync = TRUE WHERE id = ?")
            .bind(&req.wallpaper_url)
            .bind(&chat_id)
            .execute(&db.db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Обновление обоев для всех участников
        sqlx::query(
            "UPDATE chat_wallpapers SET wallpaper_url = ?, synced = TRUE, updated_at = CURRENT_TIMESTAMP
             WHERE chat_id = ?"
        )
        .bind(&req.wallpaper_url)
        .bind(&chat_id)
        .execute(&db.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(WallpaperResponse {
        chat_id,
        wallpaper_url: req.wallpaper_url,
        wallpaper_type,
        synced: sync,
    }))
}

/// Установить автоудаление сообщений (24 часа)
pub async fn set_auto_delete(
    State(db): State<crate::api::AppState>,
    Path((chat_id, user_id)): Path<(String, String)>,
    Json(req): Json<AutoDeleteMessageRequest>,
) -> Result<StatusCode, StatusCode> {
    // Проверка прав пользователя в чате
    let is_member = sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM chat_members WHERE chat_id = ? AND user_id = ?"
    )
    .bind(&chat_id)
    .bind(&user_id)
    .fetch_one(&db.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !is_member {
        return Err(StatusCode::FORBIDDEN);
    }

    // Обновление настроек чата
    sqlx::query(
        "UPDATE chats SET auto_delete_hours = ? WHERE id = ? AND (owner_id = ? OR EXISTS (
            SELECT 1 FROM chat_members WHERE chat_id = ? AND user_id = ? AND role = 'admin'
        ))"
    )
    .bind(&(req.hours as i64))
    .bind(&chat_id)
    .bind(&user_id)
    .bind(&chat_id)
    .bind(&user_id)
    .execute(&db.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

/// Отправить сообщение с автоудалением
pub async fn send_auto_delete_message(
    State(db): State<crate::api::AppState>,
    Path(chat_id): Path<String>,
    Json(req): Json<crate::api::messages::SendMessageRequest>,
) -> Result<Json<crate::api::messages::MessageResponse>, StatusCode> {
    let message_id = uuid::Uuid::new_v4().to_string();
    let sender_id = "sender-id"; // TODO: Получить из токена
    let message_type = req.message_type.clone().unwrap_or_else(|| "text".to_string());
    
    // Вычисление времени удаления
    let delete_at = Utc::now() + Duration::hours(24);

    // Сохранение сообщения с автоудалением
    sqlx::query(
        "INSERT INTO messages (id, chat_id, sender_id, content, type, file_url, reply_to_id, auto_delete_hours, delete_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&message_id)
    .bind(&chat_id)
    .bind(sender_id)
    .bind(&req.content)
    .bind(&message_type)
    .bind(&req.file_url)
    .bind(&req.reply_to_id)
    .bind(24i64)
    .bind(delete_at)
    .execute(&db.db)
    .await
    .map_err(|e| {
        tracing::error!("Ошибка отправки сообщения: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // TODO: Отправка через WebSocket

    Ok(Json(crate::api::messages::MessageResponse {
        id: message_id,
        chat_id,
        sender_id: sender_id.to_string(),
        content: req.content.clone(),
        translated_content: None,
        message_type,
        file_url: req.file_url.clone(),
        reply_to_id: req.reply_to_id.clone(),
        is_edited: false,
        created_at: Utc::now().to_rfc3339(),
    }))
}

/// Удалить просроченные сообщения (задача по расписанию)
pub async fn cleanup_expired_messages(db: &sqlx::SqlitePool) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM messages WHERE delete_at IS NOT NULL AND delete_at < CURRENT_TIMESTAMP"
    )
    .execute(db)
    .await?;

    Ok(result.rows_affected())
}

/// Предустановленные обои
pub fn get_preset_wallpapers() -> Vec<WallpaperPreset> {
    vec![
        WallpaperPreset {
            id: "roses".to_string(),
            name: "Розы".to_string(),
            url: "/wallpapers/roses.jpg".to_string(),
            category: "nature".to_string(),
        },
        WallpaperPreset {
            id: "nature".to_string(),
            name: "Природа".to_string(),
            url: "/wallpapers/nature.jpg".to_string(),
            category: "nature".to_string(),
        },
        WallpaperPreset {
            id: "mountains".to_string(),
            name: "Горы".to_string(),
            url: "/wallpapers/mountains.jpg".to_string(),
            category: "nature".to_string(),
        },
        WallpaperPreset {
            id: "ocean".to_string(),
            name: "Океан".to_string(),
            url: "/wallpapers/ocean.jpg".to_string(),
            category: "nature".to_string(),
        },
        WallpaperPreset {
            id: "sunset".to_string(),
            name: "Закат".to_string(),
            url: "/wallpapers/sunset.jpg".to_string(),
            category: "nature".to_string(),
        },
        WallpaperPreset {
            id: "abstract".to_string(),
            name: "Абстракция".to_string(),
            url: "/wallpapers/abstract.jpg".to_string(),
            category: "abstract".to_string(),
        },
        WallpaperPreset {
            id: "dark".to_string(),
            name: "Тёмная".to_string(),
            url: "/wallpapers/dark.jpg".to_string(),
            category: "solid".to_string(),
        },
        WallpaperPreset {
            id: "light".to_string(),
            name: "Светлая".to_string(),
            url: "/wallpapers/light.jpg".to_string(),
            category: "solid".to_string(),
        },
    ]
}

#[derive(Serialize)]
pub struct WallpaperPreset {
    pub id: String,
    pub name: String,
    pub url: String,
    pub category: String,
}

/// Получить список обоев
pub async fn list_wallpapers() -> Json<Vec<WallpaperPreset>> {
    Json(get_preset_wallpapers())
}
