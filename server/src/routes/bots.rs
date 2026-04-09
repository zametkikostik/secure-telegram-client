//! Bot Platform Routes
//!
//! REST API for managing bots, commands, webhooks, and bot store.

use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, HeaderMap},
    Json,
};
use serde::Deserialize;
use sqlx::SqlitePool;
use sha3::{Digest, Sha3_256};
use uuid::Uuid;
use chrono::Utc;

use crate::AppState;
use crate::models::bot::*;
#[allow(unused_imports)]
use crate::middleware::auth::{AuthError, get_user_id_from_header};

// ============================================================================
// Bot CRUD
// ============================================================================

// ============================================================================
// Helpers
// ============================================================================

fn require_auth(headers: &HeaderMap, state: &AppState) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let auth_header = headers.get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    get_user_id_from_header(auth_header, &state.auth)
        .map_err(|_| (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error": "Unauthorized"}))))
}

/// Create a new bot
pub async fn create_bot(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateBotRequest>,
) -> Result<(StatusCode, Json<BotWithToken>), (StatusCode, Json<serde_json::Value>)> {
    let user_id = require_auth(&headers, &state)?;
    // Validate handler type
    let handler_type = req.handler_type.unwrap_or_else(|| "internal".to_string());
    if !["internal", "webhook", "ai"].contains(&handler_type.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid handler type"})),
        ));
    }

    // Generate bot token (plain text — only shown once)
    let plain_token = format!("bot_{}", Uuid::new_v4().simple());
    let token_hash = format!("{:x}", Sha3_256::digest(plain_token.as_bytes()));

    let bot_id = Uuid::new_v4().simple().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO bots (id, name, username, description, token_hash, owner_id, handler_type, webhook_url, ai_prompt, is_active, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?)",
    )
    .bind(&bot_id)
    .bind(&req.name)
    .bind(&req.username)
    .bind(&req.description)
    .bind(&token_hash)
    .bind(&user_id)
    .bind(&handler_type)
    .bind(&req.webhook_url)
    .bind(&req.ai_prompt)
    .bind(&now)
    .bind(&now)
    .execute(&*state.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            (
                StatusCode::CONFLICT,
                Json(serde_json::json!({"error": "Username already taken"})),
            )
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        }
    })?;

    // Add default commands for new bots
    let default_commands = vec![
        ("/start", "Start interacting with the bot", "Hello! I'm {bot_name}. How can I help you?"),
        ("/help", "Show available commands", "Available commands:\n/start - Start\n/help - This help"),
    ];

    for (cmd, desc, tmpl) in default_commands {
        sqlx::query(
            "INSERT INTO bot_commands (id, bot_id, command, description, response_template, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().simple().to_string())
        .bind(&bot_id)
        .bind(cmd)
        .bind(desc)
        .bind(
            tmpl.replace("{bot_name}", &req.name)
        )
        .bind(&now)
        .execute(&*state.db)
        .await
        .ok();
    }

    let bot = get_bot_by_id(&*state.db, &bot_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

    Ok((StatusCode::CREATED, Json(BotWithToken { bot, token: plain_token })))
}

/// List user's bots
pub async fn list_my_bots(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Bot>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = require_auth(&headers, &state)?;
    let bots = sqlx::query_as::<_, Bot>(
        "SELECT id, name, username, description, avatar_url, owner_id, handler_type, webhook_url, ai_prompt, is_active = 1 as is_active, created_at
         FROM bots WHERE owner_id = ? ORDER BY created_at DESC",
    )
    .bind(&user_id)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;

    // Count commands for each bot
    let mut bots_with_commands = Vec::new();
    for mut bot in bots {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM bot_commands WHERE bot_id = ?")
            .bind(&bot.id)
            .fetch_one(&*state.db)
            .await
            .unwrap_or((0,));
        bot.command_count = count.0;
        bots_with_commands.push(bot);
    }

    Ok(Json(bots_with_commands))
}

/// Get a specific bot
pub async fn get_bot(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(bot_id): Path<String>,
) -> Result<Json<Bot>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = require_auth(&headers, &state)?;
    let bot = sqlx::query_as::<_, Bot>(
        "SELECT id, name, username, description, avatar_url, owner_id, handler_type, webhook_url, ai_prompt, is_active = 1 as is_active, created_at
         FROM bots WHERE id = ? AND owner_id = ?",
    )
    .bind(&bot_id)
    .bind(&user_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;

    match bot {
        Some(b) => Ok(Json(b)),
        None => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Bot not found"})))),
    }
}

/// Update a bot
pub async fn update_bot(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(bot_id): Path<String>,
    Json(req): Json<UpdateBotRequest>,
) -> Result<Json<Bot>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = require_auth(&headers, &state)?;
    let now = Utc::now().to_rfc3339();

    let result = sqlx::query(
        "UPDATE bots SET
            name = COALESCE(?, name),
            description = COALESCE(?, description),
            avatar_url = COALESCE(?, avatar_url),
            webhook_url = COALESCE(?, webhook_url),
            is_active = COALESCE(?, is_active),
            updated_at = ?
         WHERE id = ? AND owner_id = ?",
    )
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.avatar_url)
    .bind(&req.webhook_url)
    .bind(&req.is_active)
    .bind(&now)
    .bind(&bot_id)
    .bind(&user_id)
    .execute(&*state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Bot not found"}))));
    }

    get_bot_by_id(&*state.db, &bot_id).await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))
}

/// Delete a bot
pub async fn delete_bot(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(bot_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let user_id = require_auth(&headers, &state)?;
    let result = sqlx::query("DELETE FROM bots WHERE id = ? AND owner_id = ?")
        .bind(&bot_id)
        .bind(&user_id)
        .execute(&*state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Bot not found"}))));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Rotate bot token
pub async fn rotate_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(bot_id): Path<String>,
) -> Result<Json<BotTokenResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = require_auth(&headers, &state)?;
    let new_token = format!("bot_{}", Uuid::new_v4().simple());
    let token_hash = format!("{:x}", Sha3_256::digest(new_token.as_bytes()));

    let result = sqlx::query("UPDATE bots SET token_hash = ? WHERE id = ? AND owner_id = ?")
        .bind(&token_hash)
        .bind(&bot_id)
        .bind(&user_id)
        .execute(&*state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Bot not found"}))));
    }

    Ok(Json(BotTokenResponse { token: new_token }))
}

// ============================================================================
// Bot Commands
// ============================================================================

pub async fn list_commands(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(bot_id): Path<String>,
) -> Result<Json<Vec<BotCommand>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = require_auth(&headers, &state)?;
    // Verify ownership
    let owner = sqlx::query_scalar::<_, String>("SELECT owner_id FROM bots WHERE id = ?")
        .bind(&bot_id)
        .fetch_optional(&*state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;

    if owner.as_deref() != Some(&user_id) {
        return Err((StatusCode::FORBIDDEN, Json(serde_json::json!({"error": "Forbidden"}))));
    }

    let commands = sqlx::query_as::<_, BotCommand>(
        "SELECT * FROM bot_commands WHERE bot_id = ? ORDER BY command",
    )
    .bind(&bot_id)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;

    Ok(Json(commands))
}

pub async fn create_command(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(bot_id): Path<String>,
    Json(req): Json<CreateCommandRequest>,
) -> Result<(StatusCode, Json<BotCommand>), (StatusCode, Json<serde_json::Value>)> {
    let user_id = require_auth(&headers, &state)?;
    // Verify ownership
    let owner = sqlx::query_scalar::<_, String>("SELECT owner_id FROM bots WHERE id = ?")
        .bind(&bot_id)
        .fetch_optional(&*state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;

    if owner.as_deref() != Some(&user_id) {
        return Err((StatusCode::FORBIDDEN, Json(serde_json::json!({"error": "Forbidden"}))));
    }

    let cmd_id = Uuid::new_v4().simple().to_string();
    let now = Utc::now().to_rfc3339();
    let handler_type = req.handler_type.unwrap_or_else(|| "internal".to_string());

    let cmd = sqlx::query_as::<_, BotCommand>(
        "INSERT INTO bot_commands (id, bot_id, command, description, handler_type, handler_url, response_template, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)
         RETURNING *",
    )
    .bind(&cmd_id)
    .bind(&bot_id)
    .bind(&req.command)
    .bind(&req.description)
    .bind(&handler_type)
    .bind(&req.handler_url)
    .bind(&req.response_template)
    .bind(&now)
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            (
                StatusCode::CONFLICT,
                Json(serde_json::json!({"error": "Command already exists"})),
            )
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        }
    })?;

    Ok((StatusCode::CREATED, Json(cmd)))
}

pub async fn delete_command(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((bot_id, cmd_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let user_id = require_auth(&headers, &state)?;
    // Verify ownership
    let owner = sqlx::query_scalar::<_, String>("SELECT owner_id FROM bots WHERE id = ?")
        .bind(&bot_id)
        .fetch_optional(&*state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;

    if owner.as_deref() != Some(&user_id) {
        return Err((StatusCode::FORBIDDEN, Json(serde_json::json!({"error": "Forbidden"}))));
    }

    let result = sqlx::query("DELETE FROM bot_commands WHERE id = ? AND bot_id = ?")
        .bind(&cmd_id)
        .bind(&bot_id)
        .execute(&*state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Command not found"}))));
    }

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Bot Webhooks
// ============================================================================

pub async fn list_webhooks(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(bot_id): Path<String>,
) -> Result<Json<Vec<BotWebhook>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = require_auth(&headers, &state)?;
    let owner = sqlx::query_scalar::<_, String>("SELECT owner_id FROM bots WHERE id = ?")
        .bind(&bot_id)
        .fetch_optional(&*state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;

    if owner.as_deref() != Some(&user_id) {
        return Err((StatusCode::FORBIDDEN, Json(serde_json::json!({"error": "Forbidden"}))));
    }

    let webhooks = sqlx::query_as::<_, BotWebhook>(
        "SELECT * FROM bot_webhooks WHERE bot_id = ? ORDER BY created_at DESC",
    )
    .bind(&bot_id)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;

    Ok(Json(webhooks))
}

pub async fn create_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(bot_id): Path<String>,
    Json(req): Json<CreateWebhookRequest>,
) -> Result<(StatusCode, Json<BotWebhook>), (StatusCode, Json<serde_json::Value>)> {
    let user_id = require_auth(&headers, &state)?;
    let owner = sqlx::query_scalar::<_, String>("SELECT owner_id FROM bots WHERE id = ?")
        .bind(&bot_id)
        .fetch_optional(&*state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;

    if owner.as_deref() != Some(&user_id) {
        return Err((StatusCode::FORBIDDEN, Json(serde_json::json!({"error": "Forbidden"}))));
    }

    let wh_id = Uuid::new_v4().simple().to_string();
    let secret = req.secret.unwrap_or_else(|| Uuid::new_v4().simple().to_string());
    let events = serde_json::to_string(&req.events.unwrap_or_else(|| vec!["message".to_string()]))
        .unwrap_or_else(|_| r#"["message"]"#.to_string());
    let now = Utc::now().to_rfc3339();

    let webhook = sqlx::query_as::<_, BotWebhook>(
        "INSERT INTO bot_webhooks (id, bot_id, url, events, secret, active, created_at)
         VALUES (?, ?, ?, ?, ?, 1, ?)
         RETURNING *",
    )
    .bind(&wh_id)
    .bind(&bot_id)
    .bind(&req.url)
    .bind(&events)
    .bind(&secret)
    .bind(&now)
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;

    Ok((StatusCode::CREATED, Json(webhook)))
}

pub async fn delete_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((bot_id, wh_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let user_id = require_auth(&headers, &state)?;
    let owner = sqlx::query_scalar::<_, String>("SELECT owner_id FROM bots WHERE id = ?")
        .bind(&bot_id)
        .fetch_optional(&*state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;

    if owner.as_deref() != Some(&user_id) {
        return Err((StatusCode::FORBIDDEN, Json(serde_json::json!({"error": "Forbidden"}))));
    }

    let result = sqlx::query("DELETE FROM bot_webhooks WHERE id = ? AND bot_id = ?")
        .bind(&wh_id)
        .bind(&bot_id)
        .execute(&*state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Webhook not found"}))));
    }

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Bot Store
// ============================================================================

#[derive(Deserialize)]
pub struct StoreQuery {
    pub category: Option<String>,
}

pub async fn list_store(
    State(state): State<AppState>,
    Query(query): Query<StoreQuery>,
) -> Result<Json<Vec<BotStoreListing>>, (StatusCode, Json<serde_json::Value>)> {
    let query_str = match &query.category {
        Some(cat) => format!(
            "SELECT * FROM bot_store_listings WHERE category = '{}' ORDER BY install_count DESC",
            cat
        ),
        None => "SELECT * FROM bot_store_listings ORDER BY install_count DESC".to_string(),
    };

    let listings = sqlx::query_as::<_, BotStoreListing>(&query_str)
        .fetch_all(&*state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;

    Ok(Json(listings))
}

pub async fn install_from_store(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(listing_id): Path<String>,
) -> Result<Json<Bot>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = require_auth(&headers, &state)?;
    // Get listing
    let listing: Option<BotStoreListing> = sqlx::query_as(
        "SELECT * FROM bot_store_listings WHERE id = ?",
    )
    .bind(&listing_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;

    let listing = match listing {
        Some(l) => l,
        None => return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Bot not found in store"})))),
    };

    // If listing has a bot_id, install it. Otherwise create a new bot.
    let bot_id = if let Some(ref bot_id) = listing.bot_id {
        // Check if already installed
        let exists: Option<(String,)> = sqlx::query_as(
            "SELECT bot_id FROM bot_installations WHERE bot_id = ? AND user_id = ?",
        )
        .bind(bot_id)
        .bind(&user_id)
        .fetch_optional(&*state.db)
        .await
        .ok()
        .flatten();

        if exists.is_some() {
            return Err((StatusCode::CONFLICT, Json(serde_json::json!({"error": "Already installed"}))));
        }

        // Link to user
        sqlx::query(
            "UPDATE bots SET owner_id = ? WHERE id = ?",
        )
        .bind(&user_id)
        .bind(bot_id)
        .execute(&*state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;

        bot_id.clone()
    } else {
        // Create new bot from listing
        let bot_id = Uuid::new_v4().simple().to_string();
        let token_hash = format!("{:x}", Sha3_256::digest(Uuid::new_v4().as_bytes()));
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO bots (id, name, username, description, avatar_url, token_hash, owner_id, handler_type, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, 'internal', ?, ?)",
        )
        .bind(&bot_id)
        .bind(&listing.name)
        .bind(&listing.username)
        .bind(&listing.description)
        .bind(&listing.avatar_url)
        .bind(&token_hash)
        .bind(&user_id)
        .bind(&now)
        .bind(&now)
        .execute(&*state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;

        bot_id
    };

    // Record installation
    sqlx::query(
        "INSERT OR IGNORE INTO bot_installations (bot_id, user_id, installed_at) VALUES (?, ?, ?)",
    )
    .bind(&bot_id)
    .bind(&user_id)
    .bind(Utc::now().to_rfc3339())
    .execute(&*state.db)
    .await
    .ok();

    // Update install count
    sqlx::query(
        "UPDATE bot_store_listings SET install_count = install_count + 1 WHERE id = ?",
    )
    .bind(&listing_id)
    .execute(&*state.db)
    .await
    .ok();

    get_bot_by_id(&*state.db, &bot_id).await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))
}

/// Search bots in bot store
#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub category: Option<String>,
}

pub async fn search_bots(
    State(state): State<AppState>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<BotStoreListing>>, (StatusCode, Json<serde_json::Value>)> {
    let mut sql = "SELECT * FROM bot_store_listings WHERE is_active = 1".to_string();
    let mut binds: Vec<String> = vec![];

    if let Some(query) = &q.q {
        if !query.is_empty() {
            sql.push_str(" AND (name LIKE ? OR description LIKE ? OR tags LIKE ?)");
            let pattern = format!("%{}%", query);
            binds.push(pattern.clone());
            binds.push(pattern.clone());
            binds.push(pattern);
        }
    }

    if let Some(category) = &q.category {
        if !category.is_empty() {
            sql.push_str(" AND category = ?");
            binds.push(category.clone());
        }
    }

    sql.push_str(" ORDER BY install_count DESC, created_at DESC LIMIT 50");

    let mut query_builder = sqlx::query_as::<_, BotStoreListing>(&sql);
    for bind in &binds {
        query_builder = query_builder.bind(bind);
    }

    let bots = query_builder.fetch_all(&*state.db).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

    Ok(Json(bots))
}

/// Uninstall a bot from store
pub async fn uninstall_from_store(
    State(state): State<AppState>,
    Path(bot_id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = require_auth(&headers, &state)?;

    // Check if bot is installed by this user
    let installed: Option<(String,)> = sqlx::query_as(
        "SELECT bot_id FROM bot_installations WHERE bot_id = ? AND user_id = ?",
    )
    .bind(&bot_id)
    .bind(&user_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

    if installed.is_none() {
        return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Bot not installed by this user"}))));
    }

    // Remove installation record
    sqlx::query(
        "DELETE FROM bot_installations WHERE bot_id = ? AND user_id = ?",
    )
    .bind(&bot_id)
    .bind(&user_id)
    .execute(&*state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

    // Decrement install count
    sqlx::query(
        "UPDATE bot_store_listings SET install_count = MAX(0, install_count - 1) WHERE id = ?",
    )
    .bind(&bot_id)
    .execute(&*state.db)
    .await
    .ok();

    Ok(Json(serde_json::json!({ "success": true, "message": "Bot uninstalled successfully" })))
}

// ============================================================================
// Helpers
// ============================================================================

async fn get_bot_by_id(db: &SqlitePool, bot_id: &str) -> Result<Bot, String> {
    let bot: Bot = sqlx::query_as(
        "SELECT id, name, username, description, avatar_url, owner_id, handler_type, webhook_url, ai_prompt, is_active = 1 as is_active, 0 as command_count, created_at
         FROM bots WHERE id = ?",
    )
    .bind(bot_id)
    .fetch_one(db)
    .await
    .map_err(|e| e.to_string())?;

    Ok(bot)
}
