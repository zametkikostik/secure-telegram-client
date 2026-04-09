//! Bot Platform Models

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ============================================================================
// Database Models
// ============================================================================

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Bot {
    pub id: String,
    pub name: String,
    pub username: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub owner_id: String,
    pub handler_type: String,
    pub webhook_url: Option<String>,
    pub ai_prompt: Option<String>,
    pub is_active: bool,
    #[serde(default)]
    pub command_count: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct BotCommand {
    pub id: String,
    pub bot_id: String,
    pub command: String,
    pub description: Option<String>,
    pub handler_type: String,
    pub handler_url: Option<String>,
    pub response_template: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct BotWebhook {
    pub id: String,
    pub bot_id: String,
    pub url: String,
    pub events: String, // JSON array
    pub secret: String,
    pub active: bool,
    pub last_triggered_at: Option<String>,
    pub last_status: Option<i32>,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct BotSession {
    pub bot_id: String,
    pub user_id: String,
    pub chat_id: String,
    pub state: String,
    pub context: String, // JSON
    pub updated_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct BotStoreListing {
    pub id: String,
    pub name: String,
    pub username: String,
    pub description: String,
    pub avatar_url: Option<String>,
    pub category: String,
    pub rating: f64,
    pub install_count: i64,
    pub is_verified: bool,
    pub is_premium: bool,
    pub author: String,
    pub commands: String, // JSON array
    pub bot_id: Option<String>,
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateBotRequest {
    pub name: String,
    pub username: String,
    pub description: Option<String>,
    pub handler_type: Option<String>,
    pub webhook_url: Option<String>,
    pub ai_prompt: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBotRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub webhook_url: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommandRequest {
    pub command: String,
    pub description: Option<String>,
    pub handler_type: Option<String>,
    pub handler_url: Option<String>,
    pub response_template: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateWebhookRequest {
    pub url: String,
    pub events: Option<Vec<String>>,
    pub secret: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BotTokenResponse {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct BotWithToken {
    #[serde(flatten)]
    pub bot: Bot,
    pub token: String, // Plain text token (only sent once)
}

// ============================================================================
// Bot Event (dispatched to bots)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct BotEvent {
    pub event_type: String,
    pub bot_id: String,
    pub user_id: String,
    pub chat_id: String,
    pub message: Option<EventMessage>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct EventMessage {
    pub id: String,
    pub content: String,
    pub is_command: bool,
    pub command: Option<String>,
    pub args: Option<String>,
}

// ============================================================================
// Bot API Response (what external webhooks return)
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct BotApiResponse {
    pub text: Option<String>,
    pub sticker_id: Option<String>,
    pub inline_keyboard: Option<Vec<Vec<InlineKeyboardButton>>>,
    pub delete_message: Option<bool>,
    pub edit_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct InlineKeyboardButton {
    pub text: String,
    pub callback_data: Option<String>,
    pub url: Option<String>,
    pub switch_inline_query: Option<String>,
}
