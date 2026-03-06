// server/src/api/extra.rs
//! Дополнительные функции: стикеры, GIF, эмодзи, закреплённые, избранные, отложенные сообщения, био, темы, демонстрация экрана

use axum::{
    extract::{State, Path, Query},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use uuid::Uuid;

// ==================== Стикеры ====================

#[derive(Serialize)]
pub struct Sticker {
    pub id: String,
    pub name: String,
    pub url: String,
    pub emoji: Option<String>,
    pub pack_id: Option<String>,
}

#[derive(Serialize)]
pub struct StickerPack {
    pub id: String,
    pub name: String,
    pub cover_url: String,
    pub stickers_count: i64,
    pub is_animated: bool,
}

/// Получить список стикеров
pub async fn list_stickers() -> Json<Vec<Sticker>> {
    Json(vec![
        Sticker { id: "1".into(), name: "❤️".into(), url: "/stickers/heart.png".into(), emoji: Some("❤️".into()), pack_id: Some("1".into()) },
        Sticker { id: "2".into(), name: "👍".into(), url: "/stickers/like.png".into(), emoji: Some("👍".into()), pack_id: Some("1".into()) },
        Sticker { id: "3".into(), name: "😂".into(), url: "/stickers/laugh.png".into(), emoji: Some("😂".into()), pack_id: Some("1".into()) },
    ])
}

/// Получить паки стикеров
pub async fn list_sticker_packs() -> Json<Vec<StickerPack>> {
    Json(vec![
        StickerPack { id: "1".into(), name: "Эмодзи".into(), cover_url: "/stickers/emoji-pack.png".into(), stickers_count: 50, is_animated: false },
        StickerPack { id: "2".into(), name: "Животные".into(), cover_url: "/stickers/animals-pack.png".into(), stickers_count: 30, is_animated: false },
    ])
}

// ==================== GIF ====================

#[derive(Serialize)]
pub struct Gif {
    pub id: String,
    pub url: String,
    pub title: String,
    pub width: i32,
    pub height: i32,
}

/// Получить популярные GIF
pub async fn list_gifs() -> Json<Vec<Gif>> {
    Json(vec![
        Gif { id: "1".into(), url: "/gifs/cat.gif".into(), title: "Cat dancing".into(), width: 480, height: 270 },
        Gif { id: "2".into(), url: "/gifs/dog.gif".into(), title: "Dog happy".into(), width: 480, height: 270 },
        Gif { id: "3".into(), url: "/gifs/reaction.gif".into(), title: "Reaction wow".into(), width: 480, height: 270 },
    ])
}

// ==================== Эмодзи реакции ====================

#[derive(Serialize)]
pub struct Reaction {
    pub emoji: String,
    pub count: i64,
    pub users: Vec<String>,
}

#[derive(Deserialize)]
pub struct AddReactionRequest {
    pub emoji: String,
}

/// Добавить реакцию на сообщение
pub async fn add_reaction(
    State(db): State<crate::api::AppState>,
    Path((message_id, user_id)): Path<(String, String)>,
    Json(req): Json<AddReactionRequest>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query(
        "INSERT OR REPLACE INTO message_reactions (message_id, user_id, emoji, created_at)
         VALUES (?, ?, ?, CURRENT_TIMESTAMP)"
    )
    .bind(&message_id)
    .bind(&user_id)
    .bind(&req.emoji)
    .execute(&db.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

/// Получить реакции на сообщение
pub async fn get_reactions(
    State(db): State<crate::api::AppState>,
    Path(message_id): Path<String>,
) -> Result<Json<Vec<Reaction>>, StatusCode> {
    let rows = sqlx::query_as(
        "SELECT emoji, COUNT(*) as count, GROUP_CONCAT(user_id) as users
         FROM message_reactions WHERE message_id = ?
         GROUP BY emoji"
    )
    .bind(&message_id)
    .fetch_all(&db.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let reactions = rows.into_iter().map(|(emoji, count, users): (String, i64, String)| {
        Reaction {
            emoji,
            count,
            users: users.split(',').map(String::from).collect(),
        }
    }).collect();

    Ok(Json(reactions))
}

// ==================== Закреплённые сообщения ====================

#[derive(Serialize)]
pub struct PinnedMessage {
    pub message_id: String,
    pub chat_id: String,
    pub content: String,
    pub sender_id: String,
    pub pinned_at: String,
    pub pinned_by: String,
}

#[derive(Deserialize)]
pub struct PinMessageRequest {
    pub message_id: String,
}

/// Закрепить сообщение
pub async fn pin_message(
    State(db): State<crate::api::AppState>,
    Path((chat_id, user_id)): Path<(String, String)>,
    Json(req): Json<PinMessageRequest>,
) -> Result<Json<PinnedMessage>, StatusCode> {
    // Получение информации о сообщении
    let message = sqlx::query_as(
        "SELECT id, chat_id, content, sender_id FROM messages WHERE id = ? AND chat_id = ?"
    )
    .bind(&req.message_id)
    .bind(&chat_id)
    .fetch_optional(&db.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    // Закрепление
    sqlx::query(
        "INSERT OR REPLACE INTO pinned_messages (chat_id, message_id, pinned_by, pinned_at)
         VALUES (?, ?, ?, CURRENT_TIMESTAMP)"
    )
    .bind(&chat_id)
    .bind(&req.message_id)
    .bind(&user_id)
    .execute(&db.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Обновление сообщения
    sqlx::query("UPDATE messages SET is_pinned = 1, pinned_at = CURRENT_TIMESTAMP, pinned_by = ? WHERE id = ?")
        .bind(&user_id)
        .bind(&req.message_id)
        .execute(&db.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (id, chat, content, sender): (String, String, String, String) = message;

    Ok(Json(PinnedMessage {
        message_id: id,
        chat_id: chat,
        content,
        sender_id: sender,
        pinned_at: Utc::now().to_rfc3339(),
        pinned_by: user_id,
    }))
}

/// Открепить сообщение
pub async fn unpin_message(
    State(db): State<crate::api::AppState>,
    Path((chat_id, message_id)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM pinned_messages WHERE chat_id = ? AND message_id = ?")
        .bind(&chat_id)
        .bind(&message_id)
        .execute(&db.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query("UPDATE messages SET is_pinned = 0 WHERE id = ?")
        .bind(&message_id)
        .execute(&db.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

/// Получить закреплённые сообщения чата
pub async fn get_pinned_messages(
    State(db): State<crate::api::AppState>,
    Path(chat_id): Path<String>,
) -> Result<Json<Vec<PinnedMessage>>, StatusCode> {
    let messages = sqlx::query_as(
        "SELECT m.id, m.chat_id, m.content, m.sender_id, pm.pinned_at, pm.pinned_by
         FROM pinned_messages pm
         JOIN messages m ON pm.message_id = m.id
         WHERE pm.chat_id = ?
         ORDER BY pm.pinned_at DESC"
    )
    .bind(&chat_id)
    .fetch_all(&db.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(messages))
}

// ==================== Избранные сообщения (Заметки) ====================

#[derive(Serialize, Deserialize)]
pub struct SavedMessage {
    pub id: String,
    pub user_id: String,
    pub content: String,
    pub message_type: String,
    pub file_url: Option<String>,
    pub tags: Option<String>,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct SaveMessageRequest {
    pub content: String,
    pub message_type: Option<String>,
    pub file_url: Option<String>,
    pub tags: Option<String>,
}

/// Сохранить сообщение в избранное
pub async fn save_message(
    State(db): State<crate::api::AppState>,
    user_id: String,
    Json(req): Json<SaveMessageRequest>,
) -> Result<Json<SavedMessage>, StatusCode> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO saved_messages (id, user_id, content, message_type, file_url, tags, created_at)
         VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)"
    )
    .bind(&id)
    .bind(&user_id)
    .bind(&req.content)
    .bind(req.message_type.as_deref().unwrap_or("text"))
    .bind(&req.file_url)
    .bind(&req.tags)
    .execute(&db.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SavedMessage {
        id,
        user_id,
        content: req.content,
        message_type: req.message_type.unwrap_or_else(|| "text".to_string()),
        file_url: req.file_url,
        tags: req.tags,
        created_at: Utc::now().to_rfc3339(),
    }))
}

/// Получить избранные сообщения
pub async fn get_saved_messages(
    State(db): State<crate::api::AppState>,
    user_id: String,
    Query(tags): Query<Option<String>>,
) -> Result<Json<Vec<SavedMessage>>, StatusCode> {
    let messages = if let Some(tag) = tags {
        sqlx::query_as(
            "SELECT * FROM saved_messages WHERE user_id = ? AND tags LIKE ? ORDER BY created_at DESC"
        )
        .bind(&user_id)
        .bind(&format!("%{}%", tag))
        .fetch_all(&db.db)
        .await
    } else {
        sqlx::query_as(
            "SELECT * FROM saved_messages WHERE user_id = ? ORDER BY created_at DESC"
        )
        .bind(&user_id)
        .fetch_all(&db.db)
        .await
    }
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(messages))
}

/// Удалить из избранного
pub async fn delete_saved_message(
    State(db): State<crate::api::AppState>,
    Path((user_id, message_id)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM saved_messages WHERE user_id = ? AND id = ?")
        .bind(&user_id)
        .bind(&message_id)
        .execute(&db.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

// ==================== Отложенные сообщения ====================

#[derive(Serialize)]
pub struct ScheduledMessage {
    pub id: String,
    pub chat_id: String,
    pub content: String,
    pub send_at: String,
    pub status: String,
}

#[derive(Deserialize)]
pub struct ScheduleMessageRequest {
    pub content: String,
    pub send_at: String,
    pub message_type: Option<String>,
    pub file_url: Option<String>,
}

/// Запланировать сообщение
pub async fn schedule_message(
    State(db): State<crate::api::AppState>,
    Path((chat_id, user_id)): Path<(String, String)>,
    Json(req): Json<ScheduleMessageRequest>,
) -> Result<Json<ScheduledMessage>, StatusCode> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO scheduled_messages (id, chat_id, sender_id, content, message_type, file_url, send_at, status)
         VALUES (?, ?, ?, ?, ?, ?, ?, 'pending')"
    )
    .bind(&id)
    .bind(&chat_id)
    .bind(&user_id)
    .bind(&req.content)
    .bind(req.message_type.as_deref().unwrap_or("text"))
    .bind(&req.file_url)
    .bind(&req.send_at)
    .execute(&db.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ScheduledMessage {
        id,
        chat_id,
        content: req.content,
        send_at: req.send_at,
        status: "pending".to_string(),
    }))
}

/// Получить отложенные сообщения
pub async fn get_scheduled_messages(
    State(db): State<crate::api::AppState>,
    Path((chat_id, user_id)): Path<(String, String)>,
) -> Result<Json<Vec<ScheduledMessage>>, StatusCode> {
    let messages = sqlx::query_as(
        "SELECT * FROM scheduled_messages WHERE chat_id = ? AND sender_id = ? AND status = 'pending' ORDER BY send_at ASC"
    )
    .bind(&chat_id)
    .bind(&user_id)
    .fetch_all(&db.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(messages))
}

/// Отменить отложенное сообщение
pub async fn cancel_scheduled_message(
    State(db): State<crate::api::AppState>,
    Path((chat_id, message_id)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("UPDATE scheduled_messages SET status = 'cancelled' WHERE chat_id = ? AND id = ?")
        .bind(&chat_id)
        .bind(&message_id)
        .execute(&db.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

// ==================== Био пользователя ====================

#[derive(Serialize, Deserialize)]
pub struct Bio {
    pub user_id: String,
    pub text: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct SetBioRequest {
    pub bio: String,
}

/// Получить био пользователя
pub async fn get_bio(
    State(db): State<crate::api::AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<Bio>, StatusCode> {
    let bio = sqlx::query_scalar(
        "SELECT bio FROM users WHERE id = ?"
    )
    .bind(&user_id)
    .fetch_optional(&db.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match bio {
        Some(text) => Ok(Json(Bio {
            user_id,
            text: text.unwrap_or_default(),
            updated_at: Utc::now().to_rfc3339(),
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Установить био
pub async fn set_bio(
    State(db): State<crate::api::AppState>,
    Path(user_id): Path<String>,
    Json(req): Json<SetBioRequest>,
) -> Result<Json<Bio>, StatusCode> {
    sqlx::query("UPDATE users SET bio = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(&req.bio)
        .bind(&user_id)
        .execute(&db.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(Bio {
        user_id,
        text: req.bio,
        updated_at: Utc::now().to_rfc3339(),
    }))
}

// ==================== Темы оформления ====================

#[derive(Serialize)]
pub struct Theme {
    pub id: String,
    pub name: String,
    pub colors: ThemeColors,
}

#[derive(Serialize)]
pub struct ThemeColors {
    pub primary: String,
    pub background: String,
    pub text: String,
    pub secondary: String,
}

/// Получить список тем
pub async fn list_themes() -> Json<Vec<Theme>> {
    Json(vec![
        Theme {
            id: "light".into(),
            name: "Светлая".into(),
            colors: ThemeColors {
                primary: "#3390EC".into(),
                background: "#FFFFFF".into(),
                text: "#000000".into(),
                secondary: "#707579".into(),
            },
        },
        Theme {
            id: "dark".into(),
            name: "Тёмная".into(),
            colors: ThemeColors {
                primary: "#8774E1".into(),
                background: "#0F0F0F".into(),
                text: "#FFFFFF".into(),
                secondary: "#AAAAAA".into(),
            },
        },
        Theme {
            id: "night".into(),
            name: "Ночная".into(),
            colors: ThemeColors {
                primary: "#6C5CE7".into(),
                background: "#1A1A2E".into(),
                text: "#EAEAEA".into(),
                secondary: "#888888".into(),
            },
        },
    ])
}

/// Установить тему пользователю
pub async fn set_user_theme(
    State(db): State<crate::api::AppState>,
    Path((user_id, theme)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("UPDATE users SET theme = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(&theme)
        .bind(&user_id)
        .execute(&db.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

/// Установить ночной режим
pub async fn set_night_mode(
    State(db): State<crate::api::AppState>,
    Path((user_id, enabled)): Path<(String, bool)>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("UPDATE users SET night_mode = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(&enabled)
        .bind(&user_id)
        .execute(&db.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

// ==================== Демонстрация экрана ====================

#[derive(Serialize)]
pub struct ScreenShareSession {
    pub id: String,
    pub chat_id: String,
    pub user_id: String,
    pub stream_url: String,
    pub started_at: String,
}

#[derive(Deserialize)]
pub struct StartScreenShareRequest {
    pub chat_id: String,
    pub user_id: String,
    pub stream_url: String,
}

/// Начать демонстрацию экрана
pub async fn start_screen_share(
    State(db): State<crate::api::AppState>,
    Json(req): Json<StartScreenShareRequest>,
) -> Result<Json<ScreenShareSession>, StatusCode> {
    let id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO screen_share_sessions (id, chat_id, user_id, stream_url, started_at)
         VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP)"
    )
    .bind(&id)
    .bind(&req.chat_id)
    .bind(&req.user_id)
    .bind(&req.stream_url)
    .execute(&db.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ScreenShareSession {
        id,
        chat_id: req.chat_id,
        user_id: req.user_id,
        stream_url: req.stream_url,
        started_at: Utc::now().to_rfc3339(),
    }))
}

/// Завершить демонстрацию экрана
pub async fn stop_screen_share(
    State(db): State<crate::api::AppState>,
    Path(session_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("UPDATE screen_share_sessions SET ended_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(&session_id)
        .execute(&db.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

/// Получить активные сессии демонстрации
pub async fn get_active_screen_shares(
    State(db): State<crate::api::AppState>,
    Path(chat_id): Path<String>,
) -> Result<Json<Vec<ScreenShareSession>>, StatusCode> {
    let sessions = sqlx::query_as(
        "SELECT * FROM screen_share_sessions WHERE chat_id = ? AND ended_at IS NULL"
    )
    .bind(&chat_id)
    .fetch_all(&db.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(sessions))
}

// ==================== Таймер самоуничтожения сообщений ====================

#[derive(Serialize)]
pub struct SelfDestructConfig {
    pub chat_id: String,
    pub timer_seconds: i64,
    pub enabled: bool,
}

#[derive(Deserialize)]
pub struct SetSelfDestructRequest {
    pub timer_seconds: i64,
}

/// Установить таймер самоуничтожения для чата
pub async fn set_chat_self_destruct(
    State(db): State<crate::api::AppState>,
    Path((chat_id, user_id)): Path<(String, String)>,
    Json(req): Json<SetSelfDestructRequest>,
) -> Result<Json<SelfDestructConfig>, StatusCode> {
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
        "UPDATE chats SET self_destruct_timer = ? WHERE id = ?"
    )
    .bind(&req.timer_seconds)
    .bind(&chat_id)
    .execute(&db.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SelfDestructConfig {
        chat_id,
        timer_seconds: req.timer_seconds,
        enabled: req.timer_seconds > 0,
    }))
}

/// Отправить сообщение с таймером самоуничтожения
pub async fn send_self_destruct_message(
    State(db): State<crate::api::AppState>,
    Path(chat_id): Path<String>,
    Json(req): Json<SendSelfDestructRequest>,
) -> Result<Json<crate::api::messages::MessageResponse>, StatusCode> {
    let message_id = uuid::Uuid::new_v4().to_string();
    let sender_id = "sender-id"; // TODO: Получить из токена
    let message_type = req.message_type.clone().unwrap_or_else(|| "text".to_string());
    
    // Вычисление времени удаления на основе таймера
    let delete_at = chrono::Utc::now() + chrono::Duration::seconds(req.timer_seconds);

    // Сохранение сообщения с таймером самоуничтожения
    sqlx::query(
        "INSERT INTO messages (id, chat_id, sender_id, content, type, file_url, reply_to_id, self_destruct_timer, delete_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&message_id)
    .bind(&chat_id)
    .bind(sender_id)
    .bind(&req.content)
    .bind(&message_type)
    .bind(&req.file_url)
    .bind(&req.reply_to_id)
    .bind(&req.timer_seconds)
    .bind(delete_at)
    .execute(&db.db)
    .await
    .map_err(|e| {
        tracing::error!("Ошибка отправки сообщения с таймером: {}", e);
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
        created_at: chrono::Utc::now().to_rfc3339(),
    }))
}

#[derive(Deserialize)]
pub struct SendSelfDestructRequest {
    pub content: String,
    pub timer_seconds: i64,
    pub message_type: Option<String>,
    pub file_url: Option<String>,
    pub reply_to_id: Option<String>,
}

/// Отключить таймер самоуничтожения для чата
pub async fn disable_self_destruct(
    State(db): State<crate::api::AppState>,
    Path((chat_id, user_id)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    // Проверка прав
    let is_admin = sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM chat_members WHERE chat_id = ? AND user_id = ? AND role = 'admin'"
    )
    .bind(&chat_id)
    .bind(&user_id)
    .fetch_one(&db.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !is_admin {
        return Err(StatusCode::FORBIDDEN);
    }

    sqlx::query("UPDATE chats SET self_destruct_timer = NULL WHERE id = ?")
        .bind(&chat_id)
        .execute(&db.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

/// Получить настройки таймера чата
pub async fn get_self_destruct_settings(
    State(db): State<crate::api::AppState>,
    Path(chat_id): Path<String>,
) -> Result<Json<SelfDestructConfig>, StatusCode> {
    let timer = sqlx::query_scalar(
        "SELECT COALESCE(self_destruct_timer, 0) FROM chats WHERE id = ?"
    )
    .bind(&chat_id)
    .fetch_one(&db.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SelfDestructConfig {
        chat_id,
        timer_seconds: timer,
        enabled: timer > 0,
    }))
}

// ==================== Очистка отложенных сообщений ====================

/// Отправить просроченные отложенные сообщения
pub async fn send_scheduled_messages(db: &sqlx::SqlitePool) -> Result<u64, sqlx::Error> {
    // Получаем сообщения, которые нужно отправить
    let messages = sqlx::query_as::<_, (String, String, String, String, Option<String>, Option<String>)>(
        "SELECT id, chat_id, sender_id, content, message_type, file_url
         FROM scheduled_messages
         WHERE send_at <= CURRENT_TIMESTAMP AND status = 'pending'"
    )
    .fetch_all(db)
    .await?;

    let count = messages.len() as u64;

    // Помечаем как отправленные
    sqlx::query("UPDATE scheduled_messages SET status = 'sent' WHERE send_at <= CURRENT_TIMESTAMP AND status = 'pending'")
        .execute(db)
        .await?;

    // TODO: Здесь должна быть логика отправки сообщений через WebSocket
    // Для каждого сообщения создать событие и отправить через WebSocket

    Ok(count)
}
