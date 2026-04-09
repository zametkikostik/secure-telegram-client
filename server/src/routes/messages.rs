use crate::e2ee::E2EEPayload;
use crate::middleware::auth::{get_user_id_from_header, AuthError};
use crate::ws::WsMessage as WsMsg;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: String,
    pub chat_id: String,
    pub sender_id: String,
    pub sender_name: Option<String>,
    pub content: String,
    pub msg_type: String,
    pub created_at: String,
    pub edited_at: Option<String>,
    pub reply_to: Option<String>,
    pub is_encrypted: bool,
    pub ephemeral_key: Option<String>,
    pub nonce: Option<String>,
}

/// Send plaintext message
pub async fn send_message(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(cid): Path<String>,
    Json(r): Json<SendMessageRequest>,
) -> Result<(StatusCode, Json<Message>), AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let uid = get_user_id_from_header(auth_hdr, &s.auth)?;

    // Verify user is in chat
    let p: Option<i64> =
        sqlx::query_scalar("SELECT COUNT(*) FROM chat_participants WHERE chat_id=? AND user_id=?")
            .bind(&cid)
            .bind(&uid)
            .fetch_optional(&*s.db)
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;
    if p.unwrap_or(0) == 0 {
        return Err(AuthError::UserNotFound);
    }

    let mid = format!("msg:{}", uuid::Uuid::new_v4().simple());
    let now = chrono::Utc::now().to_rfc3339();
    let mt = r.msg_type.clone().unwrap_or_else(|| "text".into());

    // Calculate self-destruct time if specified
    let destroy_at = r
        .destroy_after_seconds
        .map(|secs| (chrono::Utc::now() + chrono::Duration::seconds(secs as i64)).to_rfc3339());

    // Calculate scheduled time if specified
    let scheduled_for = r.scheduled_for.clone();

    sqlx::query("INSERT INTO messages(id,chat_id,sender_id,content,msg_type,reply_to,created_at,is_encrypted,ephemeral_key,nonce,destroy_at,scheduled_for)VALUES(?,?,?,?,?,?,?,?,?,?,?,?)")
        .bind(&mid).bind(&cid).bind(&uid).bind(&r.content).bind(&mt).bind(&r.reply_to).bind(&now)
        .bind(false).bind::<Option<String>>(None).bind::<Option<String>>(None).bind(&destroy_at).bind(&scheduled_for)
        .execute(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;

    // Get sender name and broadcast via WebSocket
    let sender_name: Option<String> =
        sqlx::query_scalar("SELECT COALESCE(display_name, username) FROM users WHERE id=?")
            .bind(&uid)
            .fetch_optional(&*s.db)
            .await
            .ok()
            .flatten();

    let ws_msg = WsMsg::MessageReceived {
        id: mid.clone(),
        chat_id: cid.clone(),
        sender_id: uid.clone(),
        sender_name: sender_name.clone(),
        content: r.content.clone(),
        msg_type: mt.clone(),
        created_at: now.clone(),
    };

    // Broadcast to all chat participants
    let participants: Vec<String> =
        sqlx::query_scalar("SELECT user_id FROM chat_participants WHERE chat_id=?")
            .bind(&cid)
            .fetch_all(&*s.db)
            .await
            .unwrap_or_default();

    for participant_id in participants {
        if participant_id != uid {
            // Don't send to sender
            s.ws_manager
                .send_to_user(&participant_id, ws_msg.clone())
                .await;
        }
    }

    Ok((
        StatusCode::CREATED,
        Json(Message {
            id: mid,
            chat_id: cid,
            sender_id: uid,
            sender_name: None,
            content: r.content,
            msg_type: mt,
            created_at: now,
            edited_at: None,
            reply_to: r.reply_to,
            is_encrypted: false,
            ephemeral_key: None,
            nonce: None,
        }),
    ))
}

/// Send E2EE encrypted message
pub async fn send_message_e2ee(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(cid): Path<String>,
    Json(r): Json<SendE2EERequest>,
) -> Result<(StatusCode, Json<Message>), AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let uid = get_user_id_from_header(auth_hdr, &s.auth)?;

    let p: Option<i64> =
        sqlx::query_scalar("SELECT COUNT(*) FROM chat_participants WHERE chat_id=? AND user_id=?")
            .bind(&cid)
            .bind(&uid)
            .fetch_optional(&*s.db)
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;
    if p.unwrap_or(0) == 0 {
        return Err(AuthError::UserNotFound);
    }

    let mid = format!("msg:{}", uuid::Uuid::new_v4().simple());
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query("INSERT INTO messages(id,chat_id,sender_id,content,msg_type,reply_to,created_at,is_encrypted,ephemeral_key,nonce)VALUES(?,?,?,?,?,?,?,?,?,?)")
        .bind(&mid).bind(&cid).bind(&uid).bind(&r.payload.ciphertext).bind("e2ee").bind(&r.reply_to).bind(&now)
        .bind(true).bind(&r.payload.ephemeral_public_key).bind(&r.payload.nonce)
        .execute(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;

    // Broadcast E2EE message via WebSocket
    let ws_msg = WsMsg::MessageReceived {
        id: mid.clone(),
        chat_id: cid.clone(),
        sender_id: uid.clone(),
        sender_name: None,
        content: "[E2EE Encrypted]".into(),
        msg_type: "e2ee".into(),
        created_at: now.clone(),
    };

    let participants: Vec<String> =
        sqlx::query_scalar("SELECT user_id FROM chat_participants WHERE chat_id=?")
            .bind(&cid)
            .fetch_all(&*s.db)
            .await
            .unwrap_or_default();

    for participant_id in participants {
        if participant_id != uid {
            s.ws_manager
                .send_to_user(&participant_id, ws_msg.clone())
                .await;
        }
    }

    Ok((
        StatusCode::CREATED,
        Json(Message {
            id: mid,
            chat_id: cid,
            sender_id: uid,
            sender_name: None,
            content: r.payload.ciphertext,
            msg_type: "e2ee".into(),
            created_at: now,
            edited_at: None,
            reply_to: r.reply_to,
            is_encrypted: true,
            ephemeral_key: Some(r.payload.ephemeral_public_key),
            nonce: Some(r.payload.nonce),
        }),
    ))
}

pub async fn list_messages(
    State(s): State<AppState>,
    Path(cid): Path<String>,
    Query(p): Query<ListMessagesParams>,
) -> Result<Json<Vec<Message>>, AuthError> {
    let lim = p.limit.unwrap_or(50).min(100);
    let off = p.offset.unwrap_or(0);
    let msgs:Vec<Message>=sqlx::query_as(
        "SELECT m.id,m.chat_id,m.sender_id,u.display_name as sender_name,m.content,m.msg_type,m.created_at,m.edited_at,m.reply_to,m.is_encrypted,m.ephemeral_key,m.nonce FROM messages m LEFT JOIN users u ON m.sender_id=u.id WHERE m.chat_id=? ORDER BY m.created_at DESC LIMIT ? OFFSET ?"
    ).bind(&cid).bind(lim as i64).bind(off as i64).fetch_all(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;
    Ok(Json(msgs))
}

pub async fn get_message(
    State(s): State<AppState>,
    Path(mid): Path<String>,
) -> Result<Json<Message>, AuthError> {
    let m:Option<Message>=sqlx::query_as("SELECT m.id,m.chat_id,m.sender_id,u.display_name as sender_name,m.content,m.msg_type,m.created_at,m.edited_at,m.reply_to,m.is_encrypted,m.ephemeral_key,m.nonce FROM messages m LEFT JOIN users u ON m.sender_id=u.id WHERE m.id=?")
        .bind(&mid).fetch_optional(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;
    m.map(Json).ok_or(AuthError::UserNotFound)
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ReadReceipt {
    pub message_id: String,
    pub user_id: String,
    pub username: Option<String>,
    pub read_at: String,
}

/// Mark message as read
pub async fn mark_read(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(mid): Path<String>,
) -> Result<Json<()>, AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let uid = get_user_id_from_header(auth_hdr, &s.auth)?;
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT OR IGNORE INTO read_receipts(message_id,user_id,read_at)VALUES(?,?,?)")
        .bind(&mid)
        .bind(&uid)
        .bind(&now)
        .execute(&*s.db)
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;
    Ok(Json(()))
}

/// Get read receipts for a message
pub async fn get_read_receipts(
    State(s): State<AppState>,
    Path(mid): Path<String>,
) -> Result<Json<Vec<ReadReceipt>>, AuthError> {
    let receipts:Vec<ReadReceipt>=sqlx::query_as(
        "SELECT rr.message_id, rr.user_id, u.username, rr.read_at FROM read_receipts rr LEFT JOIN users u ON rr.user_id=u.id WHERE rr.message_id=?"
    ).bind(&mid).fetch_all(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;
    Ok(Json(receipts))
}

/// Get unread message count for a chat
pub async fn get_unread_count(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(cid): Path<String>,
) -> Result<Json<serde_json::Value>, AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let uid = get_user_id_from_header(auth_hdr, &s.auth)?;
    let count:Option<i64>=sqlx::query_scalar(
        "SELECT COUNT(*) FROM messages m WHERE m.chat_id=? AND m.sender_id!=? AND m.id NOT IN (SELECT rr.message_id FROM read_receipts rr WHERE rr.user_id=?)"
    ).bind(&cid).bind(&uid).bind(&uid).fetch_optional(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;
    Ok(Json(
        serde_json::json!({ "chat_id": cid, "unread_count": count.unwrap_or(0) }),
    ))
}

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub msg_type: Option<String>,
    pub reply_to: Option<String>,
    pub destroy_after_seconds: Option<u64>, // Self-destruct timer
    pub scheduled_for: Option<String>,      // Scheduled delivery ISO timestamp
}

#[derive(Deserialize)]
pub struct SendE2EERequest {
    pub payload: E2EEPayload,
    pub reply_to: Option<String>,
}

#[derive(Deserialize)]
pub struct ListMessagesParams {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Deserialize)]
pub struct AddReactionRequest {
    pub emoji: String,
}

/// Add reaction to message
pub async fn add_reaction(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(mid): Path<String>,
    Json(r): Json<AddReactionRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let uid = get_user_id_from_header(auth_hdr, &s.auth)?;
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT OR REPLACE INTO reactions(message_id,user_id,emoji,created_at)VALUES(?,?,?,?)",
    )
    .bind(&mid)
    .bind(&uid)
    .bind(&r.emoji)
    .bind(&now)
    .execute(&*s.db)
    .await
    .map_err(|e| AuthError::Database(e.to_string()))?;

    let count: Option<i64> =
        sqlx::query_scalar("SELECT COUNT(*) FROM reactions WHERE message_id=? AND emoji=?")
            .bind(&mid)
            .bind(&r.emoji)
            .fetch_optional(&*s.db)
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message_id": mid,
            "emoji": r.emoji,
            "count": count.unwrap_or(0),
            "user_reacted": true,
        })),
    ))
}

/// Remove reaction
pub async fn remove_reaction(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(mid): Path<String>,
    axum::extract::Query(q): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<(StatusCode, Json<serde_json::Value>), AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let uid = get_user_id_from_header(auth_hdr, &s.auth)?;
    let emoji = q.get("emoji").cloned().unwrap_or_default();

    sqlx::query("DELETE FROM reactions WHERE message_id=? AND user_id=? AND emoji=?")
        .bind(&mid)
        .bind(&uid)
        .bind(&emoji)
        .execute(&*s.db)
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message_id": mid,
            "emoji": emoji,
            "user_reacted": false,
        })),
    ))
}

/// Get reactions for message
pub async fn get_message_reactions(
    State(s): State<AppState>,
    Path(mid): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let reactions:Vec<(String,Option<String>)>=sqlx::query_as(
        "SELECT r.emoji, u.display_name FROM reactions r LEFT JOIN users u ON r.user_id=u.id WHERE r.message_id=?"
    ).bind(&mid).fetch_all(&*s.db).await
    .map_err(|e|(StatusCode::INTERNAL_SERVER_ERROR,Json(serde_json::json!({"error":e.to_string()}))))?;

    let mut grouped: std::collections::HashMap<String, (i64, Vec<String>)> =
        std::collections::HashMap::new();
    for (emoji, user_name) in reactions {
        let entry = grouped.entry(emoji.clone()).or_insert((0, Vec::new()));
        entry.0 += 1;
        if let Some(name) = user_name {
            entry.1.push(name);
        }
    }

    let reactions_vec: Vec<_> = grouped
        .into_iter()
        .map(|(emoji, (count, users))| {
            serde_json::json!({
                "emoji": emoji,
                "count": count,
                "users": users,
            })
        })
        .collect();
    Ok(Json(serde_json::json!({"reactions":reactions_vec})))
}

/// Broadcast typing status
pub async fn typing_indicator(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(cid): Path<String>,
    Json(r): Json<TypingRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let uid = get_user_id_from_header(auth_hdr, &s.auth)?;

    let username: Option<String> =
        sqlx::query_scalar("SELECT COALESCE(display_name,username) FROM users WHERE id=?")
            .bind(&uid)
            .fetch_optional(&*s.db)
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;

    let ws_msg = WsMsg::TypingUpdate {
        chat_id: cid.clone(),
        user_id: uid.clone(),
        username: username.unwrap_or_default(),
        is_typing: r.is_typing,
    };

    let participants: Vec<String> =
        sqlx::query_scalar("SELECT user_id FROM chat_participants WHERE chat_id=?")
            .bind(&cid)
            .fetch_all(&*s.db)
            .await
            .unwrap_or_default();

    for participant_id in participants {
        if participant_id != uid {
            s.ws_manager
                .send_to_user(&participant_id, ws_msg.clone())
                .await;
        }
    }

    Ok((StatusCode::OK, Json(serde_json::json!({"sent": true}))))
}

// ============================================================================
// Edit and Delete Messages
// ============================================================================

/// Edit a message (only sender can edit)
pub async fn edit_message(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(mid): Path<String>,
    Json(r): Json<EditMessageRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let uid = get_user_id_from_header(auth_hdr, &s.auth)?;

    // Verify user is the sender
    let sender_id: Option<String> =
        sqlx::query_scalar("SELECT sender_id FROM messages WHERE id = ?")
            .bind(&mid)
            .fetch_optional(&*s.db)
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;

    match sender_id {
        Some(sid) if sid == uid => {
            // Update message content
            let now = chrono::Utc::now().to_rfc3339();
            sqlx::query("UPDATE messages SET content = ?, edited_at = ? WHERE id = ?")
                .bind(&r.content)
                .bind(&now)
                .bind(&mid)
                .execute(&*s.db)
                .await
                .map_err(|e| AuthError::Database(e.to_string()))?;

            // Get updated message for WebSocket broadcast
            let msg: Option<Message> = sqlx::query_as(
                "SELECT m.id,m.chat_id,m.sender_id,u.display_name as sender_name,m.content,m.msg_type,m.created_at,m.edited_at,m.reply_to,m.is_encrypted,m.ephemeral_key,m.nonce FROM messages m LEFT JOIN users u ON m.sender_id=u.id WHERE m.id=?"
            )
            .bind(&mid)
            .fetch_optional(&*s.db)
            .await
            .ok()
            .flatten();

            // Broadcast edit to chat participants
            if let Some(m) = msg {
                let chat_id = m.chat_id.clone();
                let ws_msg = WsMsg::MessageEdited {
                    id: m.id,
                    chat_id: m.chat_id,
                    sender_id: m.sender_id,
                    content: m.content,
                    edited_at: m.edited_at,
                };

                let participants: Vec<String> =
                    sqlx::query_scalar("SELECT user_id FROM chat_participants WHERE chat_id = ?")
                        .bind(&chat_id)
                        .fetch_all(&*s.db)
                        .await
                        .unwrap_or_default();

                for participant_id in participants {
                    if participant_id != uid {
                        s.ws_manager
                            .send_to_user(&participant_id, ws_msg.clone())
                            .await;
                    }
                }
            }

            Ok((
                StatusCode::OK,
                Json(serde_json::json!({ "edited": true, "edited_at": now })),
            ))
        }
        Some(_) => Err(AuthError::PermissionDenied),
        None => Err(AuthError::UserNotFound),
    }
}

/// Delete a message (only sender or admin can delete)
pub async fn delete_message(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(mid): Path<String>,
) -> Result<(StatusCode, Json<serde_json::Value>), AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let uid = get_user_id_from_header(auth_hdr, &s.auth)?;

    // Verify user is the sender or admin
    let sender_id: Option<String> =
        sqlx::query_scalar("SELECT sender_id FROM messages WHERE id = ?")
            .bind(&mid)
            .fetch_optional(&*s.db)
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;

    match sender_id {
        Some(sid) if sid == uid => {
            // Get chat_id before delete for WebSocket broadcast
            let chat_id: Option<String> =
                sqlx::query_scalar("SELECT chat_id FROM messages WHERE id = ?")
                    .bind(&mid)
                    .fetch_optional(&*s.db)
                    .await
                    .map_err(|e| AuthError::Database(e.to_string()))?;

            sqlx::query("DELETE FROM messages WHERE id = ?")
                .bind(&mid)
                .execute(&*s.db)
                .await
                .map_err(|e| AuthError::Database(e.to_string()))?;

            // Also delete reactions
            sqlx::query("DELETE FROM reactions WHERE message_id = ?")
                .bind(&mid)
                .execute(&*s.db)
                .await
                .ok();

            // Broadcast delete to chat participants
            if let Some(cid) = chat_id {
                let ws_msg = WsMsg::MessageDeleted {
                    id: mid.clone(),
                    chat_id: cid.clone(),
                };

                let participants: Vec<String> =
                    sqlx::query_scalar("SELECT user_id FROM chat_participants WHERE chat_id = ?")
                        .bind(&cid)
                        .fetch_all(&*s.db)
                        .await
                        .unwrap_or_default();

                for participant_id in participants {
                    if participant_id != uid {
                        s.ws_manager
                            .send_to_user(&participant_id, ws_msg.clone())
                            .await;
                    }
                }
            }

            Ok((StatusCode::OK, Json(serde_json::json!({ "deleted": true }))))
        }
        Some(_) => Err(AuthError::PermissionDenied),
        None => Err(AuthError::UserNotFound),
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct EditMessageRequest {
    pub content: String,
}

#[derive(Deserialize)]
pub struct TypingRequest {
    pub is_typing: bool,
}

/// Pin a message
pub async fn pin_message(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(mid): Path<String>,
) -> Result<(StatusCode, Json<serde_json::Value>), AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let uid = get_user_id_from_header(auth_hdr, &s.auth)?;

    let chat_id: Option<String> = sqlx::query_scalar("SELECT chat_id FROM messages WHERE id = ?")
        .bind(&mid)
        .fetch_optional(&*s.db)
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

    let chat_id = match chat_id {
        Some(c) => c,
        None => return Err(AuthError::UserNotFound),
    };

    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT OR REPLACE INTO pinned_messages(chat_id, message_id, pinned_by, pinned_at) VALUES(?,?,?,?)"
    ).bind(&chat_id).bind(&mid).bind(&uid).bind(&now)
    .execute(&*s.db).await.map_err(|e| AuthError::Database(e.to_string()))?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({ "success": true, "chat_id": chat_id })),
    ))
}

/// Unpin a message
pub async fn unpin_message(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(mid): Path<String>,
) -> Result<(StatusCode, Json<serde_json::Value>), AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let _uid = get_user_id_from_header(auth_hdr, &s.auth)?;

    sqlx::query("DELETE FROM pinned_messages WHERE message_id = ?")
        .bind(&mid)
        .execute(&*s.db)
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;

    Ok((StatusCode::OK, Json(serde_json::json!({ "success": true }))))
}

/// List pinned messages for a chat
pub async fn get_pinned_messages(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(cid): Path<String>,
) -> Result<Json<Vec<Message>>, AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let _uid = get_user_id_from_header(auth_hdr, &s.auth)?;

    let msgs: Vec<Message> = sqlx::query_as(
        "SELECT m.id,m.chat_id,m.sender_id,u.display_name as sender_name,m.content,m.msg_type,m.created_at,m.edited_at,m.reply_to,m.is_encrypted,m.ephemeral_key,m.nonce
         FROM messages m JOIN pinned_messages p ON m.id=p.message_id LEFT JOIN users u ON m.sender_id=u.id
         WHERE p.chat_id=? ORDER BY p.pinned_at DESC"
    ).bind(&cid).fetch_all(&*s.db).await.map_err(|e| AuthError::Database(e.to_string()))?;

    Ok(Json(msgs))
}
