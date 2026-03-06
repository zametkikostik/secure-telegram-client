// server/src/api/messages.rs
//! API сообщений

use axum::{
    extract::{State, Path, Query},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::api::AppState;

#[derive(Serialize)]
pub struct MessageResponse {
    pub id: String,
    pub chat_id: String,
    pub sender_id: String,
    pub content: String,
    pub translated_content: Option<String>,
    #[serde(rename = "type")]
    pub message_type: String,
    pub file_url: Option<String>,
    pub reply_to_id: Option<String>,
    pub is_edited: bool,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    #[serde(rename = "type")]
    pub message_type: Option<String>,
    pub file_url: Option<String>,
    pub reply_to_id: Option<String>,
}

#[derive(Deserialize)]
pub struct ListMessagesQuery {
    limit: Option<u32>,
    offset: Option<u32>,
}

/// Список сообщений чата
pub async fn list_messages(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    Query(query): Query<ListMessagesQuery>,
) -> Result<Json<Vec<MessageResponse>>, StatusCode> {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    let messages = sqlx::query_as(
        "SELECT id, chat_id, sender_id, content, translated_content, type, file_url, reply_to_id, is_edited, created_at 
         FROM messages 
         WHERE chat_id = ? 
         ORDER BY created_at DESC 
         LIMIT ? OFFSET ?"
    )
    .bind(&chat_id)
    .bind(limit as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Ошибка получения сообщений: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(messages))
}

/// Отправить сообщение
pub async fn send_message(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
    Json(req): Json<SendMessageRequest>,
    // claims: Claims,
) -> Result<Json<MessageResponse>, StatusCode> {
    let message_id = Uuid::new_v4().to_string();
    let sender_id = "sender-id"; // claims.sub
    let message_type = req.message_type.unwrap_or_else(|| "text".to_string());

    // Сохранение сообщения
    sqlx::query(
        "INSERT INTO messages (id, chat_id, sender_id, content, type, file_url, reply_to_id) 
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&message_id)
    .bind(&chat_id)
    .bind(sender_id)
    .bind(&req.content)
    .bind(&message_type)
    .bind(&req.file_url)
    .bind(&req.reply_to_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Ошибка отправки сообщения: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // TODO: Отправка через WebSocket

    Ok(Json(MessageResponse {
        id: message_id,
        chat_id,
        sender_id: sender_id.to_string(),
        content: req.content,
        translated_content: None,
        message_type,
        file_url: req.file_url,
        reply_to_id: req.reply_to_id,
        is_edited: false,
        created_at: chrono::Utc::now().to_rfc3339(),
    }))
}
