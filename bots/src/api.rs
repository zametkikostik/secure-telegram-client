//! Bots Platform API
//! 
//! BotFather аналог + ManyChat конструктор

use axum::{
    extract::{State, Path},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::AppState;

// ==================== BotFather API ====================

/// Бот
#[derive(Serialize, Deserialize, Clone)]
pub struct Bot {
    pub id: String,
    pub owner_id: String,
    pub username: String,
    pub name: String,
    pub description: Option<String>,
    pub token: String,
    pub avatar_url: Option<String>,
    pub is_verified: bool,
    pub created_at: String,
}

/// Создать бота (BotFather)
#[derive(Deserialize)]
pub struct CreateBotRequest {
    pub username: String,
    pub name: String,
    pub description: Option<String>,
}

/// Список ботов пользователя
pub async fn list_bots(
    State(state): State<AppState>,
    user_id: String, // TODO: Получить из токена
) -> Result<Json<Vec<Bot>>, StatusCode> {
    let bots = sqlx::query_as(
        "SELECT id, owner_id, username, name, description, token, avatar_url, is_verified, created_at
         FROM bots WHERE owner_id = ?"
    )
    .bind(&user_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Ошибка получения ботов: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(bots))
}

/// Создать нового бота
pub async fn create_bot(
    State(state): State<AppState>,
    user_id: String, // TODO: Получить из токена
    Json(req): Json<CreateBotRequest>,
) -> Result<Json<Bot>, StatusCode> {
    let bot_id = Uuid::new_v4().to_string();
    let token = generate_bot_token();

    sqlx::query(
        "INSERT INTO bots (id, owner_id, username, name, description, token, created_at)
         VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)"
    )
    .bind(&bot_id)
    .bind(&user_id)
    .bind(&req.username)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&token)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Ошибка создания бота: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    Ok(Json(Bot {
        id: bot_id,
        owner_id: user_id,
        username: req.username,
        name: req.name,
        description: req.description,
        token,
        avatar_url: None,
        is_verified: false,
        created_at: chrono::Utc::now().to_rfc3339(),
    }))
}

/// Получить информацию о боте
pub async fn get_bot(
    State(state): State<AppState>,
    Path(bot_id): Path<String>,
) -> Result<Json<Bot>, StatusCode> {
    let bot = sqlx::query_as(
        "SELECT id, owner_id, username, name, description, token, avatar_url, is_verified, created_at
         FROM bots WHERE id = ?"
    )
    .bind(&bot_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(bot))
}

/// Удалить бота
pub async fn delete_bot(
    State(state): State<AppState>,
    Path(bot_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM bots WHERE id = ?")
        .bind(&bot_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Пересоздать токен бота
pub async fn regenerate_token(
    State(state): State<AppState>,
    Path(bot_id): Path<String>,
) -> Result<Json<Bot>, StatusCode> {
    let new_token = generate_bot_token();

    sqlx::query("UPDATE bots SET token = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(&new_token)
        .bind(&bot_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    get_bot(State(state), Path(bot_id)).await
}

// ==================== Bot Builder (ManyChat analog) ====================

/// Flow (сценарий)
#[derive(Serialize, Deserialize, Clone)]
pub struct Flow {
    pub id: String,
    pub bot_id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub trigger_type: String,
    pub trigger_value: Option<String>,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct CreateFlowRequest {
    pub name: String,
    pub description: Option<String>,
    pub trigger_type: Option<String>,
    pub trigger_value: Option<String>,
}

/// Список flow бота
pub async fn list_flows(
    State(state): State<AppState>,
    Path(bot_id): Path<String>,
) -> Result<Json<Vec<Flow>>, StatusCode> {
    let flows = sqlx::query_as(
        "SELECT id, bot_id, name, description, is_active, trigger_type, trigger_value, created_at
         FROM bot_flows WHERE bot_id = ?"
    )
    .bind(&bot_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(flows))
}

/// Создать flow
pub async fn create_flow(
    State(state): State<AppState>,
    Path(bot_id): Path<String>,
    Json(req): Json<CreateFlowRequest>,
) -> Result<Json<Flow>, StatusCode> {
    let flow_id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO bot_flows (id, bot_id, name, description, trigger_type, trigger_value, created_at)
         VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)"
    )
    .bind(&flow_id)
    .bind(&bot_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(req.trigger_type.as_deref().unwrap_or("keyword"))
    .bind(&req.trigger_value)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(Json(Flow {
        id: flow_id,
        bot_id,
        name: req.name,
        description: req.description,
        is_active: true,
        trigger_type: req.trigger_type.unwrap_or_else(|| "keyword".to_string()),
        trigger_value: req.trigger_value,
        created_at: chrono::Utc::now().to_rfc3339(),
    }))
}

/// Получить flow
pub async fn get_flow(
    State(state): State<AppState>,
    Path((bot_id, flow_id)): Path<(String, String)>,
) -> Result<Json<Flow>, StatusCode> {
    let flow = sqlx::query_as(
        "SELECT id, bot_id, name, description, is_active, trigger_type, trigger_value, created_at
         FROM bot_flows WHERE bot_id = ? AND id = ?"
    )
    .bind(&bot_id)
    .bind(&flow_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(flow))
}

/// Обновить flow
pub async fn update_flow(
    State(state): State<AppState>,
    Path((bot_id, flow_id)): Path<(String, String)>,
    Json(req): Json<CreateFlowRequest>,
) -> Result<Json<Flow>, StatusCode> {
    sqlx::query(
        "UPDATE bot_flows SET name = ?, description = ?, trigger_type = ?, trigger_value = ?, 
         updated_at = CURRENT_TIMESTAMP WHERE bot_id = ? AND id = ?"
    )
    .bind(&req.name)
    .bind(&req.description)
    .bind(req.trigger_type.as_deref().unwrap_or("keyword"))
    .bind(&req.trigger_value)
    .bind(&bot_id)
    .bind(&flow_id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    get_flow(State(state), Path((bot_id, flow_id))).await
}

/// Удалить flow
pub async fn delete_flow(
    State(state): State<AppState>,
    Path((bot_id, flow_id)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM bot_flows WHERE bot_id = ? AND id = ?")
        .bind(&bot_id)
        .bind(&flow_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

// ==================== Блоки (шаги flow) ====================

/// Блок
#[derive(Serialize, Deserialize, Clone)]
pub struct Block {
    pub id: String,
    pub flow_id: String,
    pub block_type: String,
    pub content: Option<String>,
    pub position: i32,
    pub next_block_id: Option<String>,
    pub conditions: Option<String>,
    pub created_at: String,
}

#[derive(Deserialize)]
pub struct CreateBlockRequest {
    pub block_type: String,
    pub content: Option<String>,
    pub position: Option<i32>,
    pub next_block_id: Option<String>,
    pub conditions: Option<String>,
}

/// Список блоков flow
pub async fn list_blocks(
    State(state): State<AppState>,
    Path(bot_id): Path<String>,
) -> Result<Json<Vec<Block>>, StatusCode> {
    let blocks = sqlx::query_as(
        "SELECT b.id, b.flow_id, b.block_type, b.content, b.position, b.next_block_id, b.conditions, b.created_at
         FROM bot_blocks b
         JOIN bot_flows f ON b.flow_id = f.id
         WHERE f.bot_id = ?
         ORDER BY b.position"
    )
    .bind(&bot_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(blocks))
}

/// Создать блок
pub async fn create_block(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
    Json(req): Json<CreateBlockRequest>,
) -> Result<Json<Block>, StatusCode> {
    let block_id = Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO bot_blocks (id, flow_id, block_type, content, position, next_block_id, conditions, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)"
    )
    .bind(&block_id)
    .bind(&flow_id)
    .bind(&req.block_type)
    .bind(&req.content)
    .bind(req.position.unwrap_or(0))
    .bind(&req.next_block_id)
    .bind(&req.conditions)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(Json(Block {
        id: block_id,
        flow_id,
        block_type: req.block_type,
        content: req.content,
        position: req.position.unwrap_or(0),
        next_block_id: req.next_block_id,
        conditions: req.conditions,
        created_at: chrono::Utc::now().to_rfc3339(),
    }))
}

/// Обновить блок
pub async fn update_block(
    State(state): State<AppState>,
    Path(block_id): Path<String>,
    Json(req): Json<CreateBlockRequest>,
) -> Result<Json<Block>, StatusCode> {
    // Получаем текущий блок
    let block = sqlx::query_as(
        "SELECT id, flow_id, block_type, content, position, next_block_id, conditions, created_at
         FROM bot_blocks WHERE id = ?"
    )
    .bind(&block_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let (flow_id, block_type): (String, String) = block;

    sqlx::query(
        "UPDATE bot_blocks SET block_type = ?, content = ?, position = ?, next_block_id = ?, 
         conditions = ? WHERE id = ?"
    )
    .bind(&req.block_type)
    .bind(&req.content)
    .bind(req.position.unwrap_or(0))
    .bind(&req.next_block_id)
    .bind(&req.conditions)
    .bind(&block_id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Возвращаем обновлённый блок
    get_block(State(state), Path(block_id)).await
}

/// Удалить блок
pub async fn delete_block(
    State(state): State<AppState>,
    Path(block_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM bot_blocks WHERE id = ?")
        .bind(&block_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn get_block(
    State(state): State<AppState>,
    Path(block_id): Path<String>,
) -> Result<Json<Block>, StatusCode> {
    let block = sqlx::query_as(
        "SELECT id, flow_id, block_type, content, position, next_block_id, conditions, created_at
         FROM bot_blocks WHERE id = ?"
    )
    .bind(&block_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(block))
}

// ==================== Webhooks ====================

#[derive(Serialize)]
pub struct Webhook {
    pub id: String,
    pub bot_id: String,
    pub url: String,
    pub secret: String,
    pub events: String,
    pub is_active: bool,
}

#[derive(Deserialize)]
pub struct SetWebhookRequest {
    pub url: String,
    pub events: Option<String>,
}

pub async fn get_webhook(
    State(state): State<AppState>,
    Path(bot_id): Path<String>,
) -> Result<Json<Vec<Webhook>>, StatusCode> {
    let webhooks = sqlx::query_as(
        "SELECT id, bot_id, url, secret, events, is_active FROM bot_webhooks WHERE bot_id = ?"
    )
    .bind(&bot_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(webhooks))
}

pub async fn set_webhook(
    State(state): State<AppState>,
    Path(bot_id): Path<String>,
    Json(req): Json<SetWebhookRequest>,
) -> Result<Json<Webhook>, StatusCode> {
    let webhook_id = Uuid::new_v4().to_string();
    let secret = generate_webhook_secret();

    sqlx::query(
        "INSERT INTO bot_webhooks (id, bot_id, url, secret, events, is_active, created_at)
         VALUES (?, ?, ?, ?, ?, TRUE, CURRENT_TIMESTAMP)"
    )
    .bind(&webhook_id)
    .bind(&bot_id)
    .bind(&req.url)
    .bind(&secret)
    .bind(req.events.as_deref().unwrap_or("message,subscribe,unsubscribe"))
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(Json(Webhook {
        id: webhook_id,
        bot_id,
        url: req.url,
        secret,
        events: req.events.unwrap_or_else(|| "message,subscribe,unsubscribe".to_string()),
        is_active: true,
    }))
}

pub async fn delete_webhook(
    State(state): State<AppState>,
    Path((bot_id, webhook_id)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM bot_webhooks WHERE bot_id = ? AND id = ?")
        .bind(&bot_id)
        .bind(&webhook_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

// ==================== Статистика бота ====================

#[derive(Serialize)]
pub struct BotStats {
    pub bot_id: String,
    pub total_subscribers: i64,
    pub total_messages: i64,
    pub active_flows: i64,
    pub total_blocks: i64,
}

pub async fn get_bot_stats(
    State(state): State<AppState>,
    Path(bot_id): Path<String>,
) -> Result<Json<BotStats>, StatusCode> {
    let total_subscribers: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM bot_subscribers WHERE bot_id = ?"
    )
    .bind(&bot_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let total_messages: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM bot_messages WHERE bot_id = ?"
    )
    .bind(&bot_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let active_flows: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM bot_flows WHERE bot_id = ? AND is_active = TRUE"
    )
    .bind(&bot_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let total_blocks: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM bot_blocks b
         JOIN bot_flows f ON b.flow_id = f.id
         WHERE f.bot_id = ?"
    )
    .bind(&bot_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    Ok(Json(BotStats {
        bot_id,
        total_subscribers,
        total_messages,
        active_flows,
        total_blocks,
    }))
}

// ==================== Helper functions ====================

fn generate_bot_token() -> String {
    use rand::{Rng, distributions::Alphanumeric};
    format!("bot_{}", rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect::<String>())
}

fn generate_webhook_secret() -> String {
    use rand::{Rng, distributions::Alphanumeric};
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect::<String>()
}
