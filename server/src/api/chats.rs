// server/src/api/chats.rs
//! API чатов

use axum::{
    extract::{State, Path},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::api::AppState;

#[derive(Serialize)]
pub struct ChatResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub chat_type: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub owner_id: Option<String>,
    pub members: Vec<ChatMember>,
    pub last_message: Option<MessagePreview>,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct ChatMember {
    pub user_id: String,
    pub username: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct MessagePreview {
    pub id: String,
    pub content: String,
    pub sender_id: String,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct CreateChatRequest {
    #[serde(rename = "type")]
    pub chat_type: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub member_ids: Option<Vec<String>>,
}

/// Список чатов пользователя
pub async fn list_chats(
    State(state): State<AppState>,
    // claims: Claims,
) -> Result<Json<Vec<ChatResponse>>, StatusCode> {
    // В реальности здесь будет фильтрация по user_id из токена
    let chats = sqlx::query_as(
        "SELECT id, type, name, description, owner_id, created_at FROM chats ORDER BY updated_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Ошибка получения чатов: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(chats))
}

/// Создать чат
pub async fn create_chat(
    State(state): State<AppState>,
    Json(req): Json<CreateChatRequest>,
    // claims: Claims,
) -> Result<Json<ChatResponse>, StatusCode> {
    let chat_id = Uuid::new_v4().to_string();

    // Создание чата
    sqlx::query(
        "INSERT INTO chats (id, type, name, description, owner_id) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(&chat_id)
    .bind(&req.chat_type)
    .bind(&req.name)
    .bind(&req.description)
    .bind("owner-id") // claims.sub
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Ошибка создания чата: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Добавление участников
    if let Some(member_ids) = req.member_ids {
        for member_id in member_ids {
            sqlx::query("INSERT INTO chat_members (chat_id, user_id) VALUES (?, ?)")
                .bind(&chat_id)
                .bind(&member_id)
                .execute(&state.db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }

    Ok(Json(ChatResponse {
        id: chat_id,
        chat_type: req.chat_type,
        name: req.name,
        description: req.description,
        owner_id: Some("owner-id".to_string()),
        members: vec![],
        last_message: None,
        created_at: chrono::Utc::now().to_rfc3339(),
    }))
}

/// Получить чат по ID
pub async fn get_chat(
    State(state): State<AppState>,
    Path(chat_id): Path<String>,
) -> Result<Json<ChatResponse>, StatusCode> {
    let chat: (String, String, Option<String>, Option<String>, Option<String>, String) = sqlx::query_as(
        "SELECT id, type, name, description, owner_id, created_at FROM chats WHERE id = ?"
    )
    .bind(&chat_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    // Получение участников
    let members = sqlx::query_as(
        "SELECT cm.user_id, u.username, cm.role 
         FROM chat_members cm 
         JOIN users u ON cm.user_id = u.id 
         WHERE cm.chat_id = ?"
    )
    .bind(&chat_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    Ok(Json(ChatResponse {
        id: chat.0,
        chat_type: chat.1,
        name: chat.2,
        description: chat.3,
        owner_id: chat.4,
        members,
        last_message: None,
        created_at: chat.5,
    }))
}
