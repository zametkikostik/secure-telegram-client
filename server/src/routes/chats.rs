use crate::middleware::auth::{get_user_id_from_header, AuthError};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Chat {
    pub id: String,
    pub name: String,
    pub chat_type: String,
    pub created_by: String,
    pub created_at: String,
    pub last_message_at: Option<String>,
    pub participant_count: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ChatListItem {
    pub id: String,
    pub name: String,
    pub chat_type: String,
    pub last_message: Option<String>,
    pub last_message_at: Option<String>,
    pub unread_count: i64,
}

pub async fn create_chat(
    State(s): State<AppState>,
    headers: HeaderMap,
    Json(r): Json<CreateChatRequest>,
) -> Result<(StatusCode, Json<Chat>), AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let uid = get_user_id_from_header(auth_hdr, &s.auth)?;
    let cid = format!("chat:{}", uuid::Uuid::new_v4().simple());
    let now = chrono::Utc::now().to_rfc3339();
    let ct = r.chat_type.clone().unwrap_or_else(|| "direct".into());
    sqlx::query("INSERT INTO chats(id,name,chat_type,created_by,created_at)VALUES(?,?,?,?,?)")
        .bind(&cid)
        .bind(&r.name)
        .bind(&ct)
        .bind(&uid)
        .bind(&now)
        .execute(&*s.db)
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;
    sqlx::query("INSERT INTO chat_participants(chat_id,user_id,role,joined_at)VALUES(?,?,?,?)")
        .bind(&cid)
        .bind(&uid)
        .bind("admin")
        .bind(&now)
        .execute(&*s.db)
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;
    if let Some(ps) = &r.participants {
        for p in ps {
            sqlx::query(
                "INSERT INTO chat_participants(chat_id,user_id,role,joined_at)VALUES(?,?,?,?)",
            )
            .bind(&cid)
            .bind(p)
            .bind("member")
            .bind(&now)
            .execute(&*s.db)
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;
        }
    }
    let count = 1 + r.participants.as_ref().map(|p| p.len() as i64).unwrap_or(0);
    Ok((
        StatusCode::CREATED,
        Json(Chat {
            id: cid,
            name: r.name.clone(),
            chat_type: ct,
            created_by: uid,
            created_at: now,
            last_message_at: None,
            participant_count: count,
        }),
    ))
}

pub async fn list_chats(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ChatListItem>>, AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let uid = get_user_id_from_header(auth_hdr, &s.auth)?;
    let chats:Vec<ChatListItem>=sqlx::query_as(
        "SELECT c.id,c.name,c.chat_type,(SELECT m.content FROM messages m WHERE m.chat_id=c.id ORDER BY m.created_at DESC LIMIT 1)as last_message,(SELECT m.created_at FROM messages m WHERE m.chat_id=c.id ORDER BY m.created_at DESC LIMIT 1)as last_message_at,0 as unread_count FROM chats c INNER JOIN chat_participants cp ON c.id=cp.chat_id WHERE cp.user_id=? ORDER BY last_message_at DESC"
    ).bind(&uid).fetch_all(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;
    Ok(Json(chats))
}

pub async fn get_chat(
    State(s): State<AppState>,
    Path(cid): Path<String>,
) -> Result<Json<Chat>, AuthError> {
    let c:Option<Chat>=sqlx::query_as(
        "SELECT c.id,c.name,c.chat_type,c.created_by,c.created_at,MAX(m.created_at)as last_message_at,COUNT(DISTINCT cp.user_id)as participant_count FROM chats c LEFT JOIN messages m ON c.id=m.chat_id LEFT JOIN chat_participants cp ON c.id=cp.chat_id WHERE c.id=? GROUP BY c.id"
    ).bind(&cid).fetch_optional(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;
    c.map(Json).ok_or(AuthError::UserNotFound)
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ChatParticipant {
    pub chat_id: String,
    pub user_id: String,
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub role: String,
    pub joined_at: String,
}

/// Get chat participants
pub async fn get_participants(
    State(s): State<AppState>,
    Path(cid): Path<String>,
) -> Result<Json<Vec<ChatParticipant>>, AuthError> {
    let parts:Vec<ChatParticipant>=sqlx::query_as(
        "SELECT cp.chat_id, cp.user_id, u.username, u.display_name, cp.role, cp.joined_at FROM chat_participants cp LEFT JOIN users u ON cp.user_id=u.id WHERE cp.chat_id=?"
    ).bind(&cid).fetch_all(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;
    Ok(Json(parts))
}

/// Add participant to chat (admin only)
pub async fn add_participant(
    state: State<AppState>,
    headers: HeaderMap,
    Path(cid): Path<String>,
    Json(r): Json<AddParticipantRequest>,
) -> Result<Json<()>, AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let admin_id = get_user_id_from_header(auth_hdr, &state.auth)?;
    let is_admin: Option<i64> = sqlx::query_scalar(
        "SELECT COUNT(*) FROM chat_participants WHERE chat_id=? AND user_id=? AND role='admin'",
    )
    .bind(&cid)
    .bind(&admin_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| AuthError::Database(e.to_string()))?;
    if is_admin.unwrap_or(0) == 0 {
        return Err(AuthError::Database("Not admin".into()));
    }
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT OR IGNORE INTO chat_participants(chat_id,user_id,role,joined_at)VALUES(?,?,?,?)",
    )
    .bind(&cid)
    .bind(&r.user_id)
    .bind("member")
    .bind(&now)
    .execute(&*state.db)
    .await
    .map_err(|e| AuthError::Database(e.to_string()))?;
    Ok(Json(()))
}

/// Remove participant from chat (admin only)
pub async fn remove_participant(
    state: State<AppState>,
    headers: HeaderMap,
    Path(cid): Path<String>,
    user_id_query: axum::extract::Query<UserIdQuery>,
) -> Result<Json<()>, AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let admin_id = get_user_id_from_header(auth_hdr, &state.auth)?;
    let user_id = &user_id_query.user_id;
    let is_admin: Option<i64> = sqlx::query_scalar(
        "SELECT COUNT(*) FROM chat_participants WHERE chat_id=? AND user_id=? AND role='admin'",
    )
    .bind(&cid)
    .bind(&admin_id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| AuthError::Database(e.to_string()))?;
    if is_admin.unwrap_or(0) == 0 {
        return Err(AuthError::Database("Not admin".into()));
    }
    sqlx::query("DELETE FROM chat_participants WHERE chat_id=? AND user_id=?")
        .bind(&cid)
        .bind(user_id)
        .execute(&*state.db)
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;
    Ok(Json(()))
}

/// Leave chat
pub async fn leave_chat(
    state: State<AppState>,
    headers: HeaderMap,
    Path(cid): Path<String>,
) -> Result<Json<()>, AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let uid = get_user_id_from_header(auth_hdr, &state.auth)?;
    sqlx::query("DELETE FROM chat_participants WHERE chat_id=? AND user_id=?")
        .bind(&cid)
        .bind(&uid)
        .execute(&*state.db)
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct CreateChatRequest {
    pub name: String,
    pub chat_type: Option<String>,
    pub participants: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct AddParticipantRequest {
    pub user_id: String,
}

#[derive(Deserialize)]
pub struct UserIdQuery {
    pub user_id: String,
}

#[derive(Deserialize)]
pub struct WallpaperRequest {
    pub color: Option<String>,
    pub pattern: Option<String>,
    pub custom_url: Option<String>,
}

/// Get chat wallpaper
pub async fn get_wallpaper(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(cid): Path<String>,
) -> Result<Json<serde_json::Value>, AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let uid = get_user_id_from_header(auth_hdr, &s.auth)?;
    let row: Option<(String, String, Option<String>)> = sqlx::query_as(
        "SELECT color, pattern, custom_url FROM chat_wallpapers WHERE chat_id=? AND user_id=?",
    )
    .bind(&cid)
    .bind(&uid)
    .fetch_optional(&*s.db)
    .await
    .map_err(|e| AuthError::Database(e.to_string()))?;
    match row {
        Some((color, pattern, url)) => Ok(Json(
            serde_json::json!({ "color": color, "pattern": pattern, "custom_url": url }),
        )),
        None => Ok(Json(
            serde_json::json!({ "color": "#1e293b", "pattern": "solid", "custom_url": null }),
        )),
    }
}

/// Set chat wallpaper
pub async fn set_wallpaper(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(cid): Path<String>,
    Json(r): Json<WallpaperRequest>,
) -> Result<Json<serde_json::Value>, AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let uid = get_user_id_from_header(auth_hdr, &s.auth)?;
    let color = r.color.unwrap_or_else(|| "#1e293b".into());
    let pattern = r.pattern.unwrap_or_else(|| "solid".into());
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO chat_wallpapers(chat_id,user_id,color,pattern,custom_url,updated_at) VALUES(?,?,?,?,?,?)
         ON CONFLICT(chat_id) DO UPDATE SET color=?, pattern=?, custom_url=?, updated_at=?"
    ).bind(&cid).bind(&uid).bind(&color).bind(&pattern).bind(&r.custom_url).bind(&now)
     .bind(&color).bind(&pattern).bind(&r.custom_url).bind(&now)
    .execute(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;
    Ok(Json(
        serde_json::json!({ "success": true, "color": color, "pattern": pattern }),
    ))
}
