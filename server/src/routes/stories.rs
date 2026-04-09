// Stories routes
// CRUD for stories with media support

use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

use crate::middleware::auth::{get_user_id_from_header, AuthError};
use crate::AppState;

const STORIES_DIR: &str = "uploads/stories";
const STORY_TTL: i64 = 24 * 60 * 60; // 24 hours in seconds

#[derive(Debug, Serialize, Deserialize)]
pub struct Story {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub media_url: String,
    pub media_type: String, // image, video
    pub caption: Option<String>,
    pub created_at: String,
    pub expires_at: String,
    pub view_count: i64,
    pub is_viewed: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateStoryRequest {
    pub caption: Option<String>,
    pub media_type: String,
}

/// Create story with media upload
pub async fn create_story(
    State(s): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Story>), AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let uid = get_user_id_from_header(auth_hdr, &s.auth)?;

    // Get user info
    let user_row: Option<(String, Option<String>)> =
        sqlx::query_as("SELECT username, display_name FROM users WHERE id=?")
            .bind(&uid)
            .fetch_optional(&*s.db)
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;

    let (username, display_name) = user_row.ok_or(AuthError::UserNotFound)?;

    // Create stories directory
    let stories_dir = PathBuf::from(STORIES_DIR);
    if !stories_dir.exists() {
        fs::create_dir_all(&stories_dir)
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;
    }

    let mut media_url = String::new();
    let mut media_type = String::from("image");
    let mut caption: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "caption" {
            if let Ok(text) = field.text().await {
                caption = Some(text);
            }
        } else if name == "media_type" {
            if let Ok(mt) = field.text().await {
                media_type = mt;
            }
        } else if name == "media" {
            if let Some(file_name) = field.file_name() {
                let fname = file_name.to_string();
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AuthError::Database(e.to_string()))?;
                let extension = std::path::Path::new(&fname)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("jpg");
                let storage_name = format!("story_{}.{}", Uuid::new_v4().simple(), extension);
                let file_path = stories_dir.join(&storage_name);
                fs::write(&file_path, &data)
                    .await
                    .map_err(|e| AuthError::Database(e.to_string()))?;
                media_url = format!("/api/v1/stories/media/{}", storage_name);
            }
        }
    }

    if media_url.is_empty() {
        return Err(AuthError::Database("No media provided".into()));
    }

    let now = Utc::now();
    let created_at = now.to_rfc3339();
    let expires_at = (now + chrono::Duration::seconds(STORY_TTL)).to_rfc3339();
    let story_id = Uuid::new_v4().simple().to_string();

    sqlx::query(
        "INSERT INTO stories(id, user_id, media_url, media_type, caption, created_at, expires_at) VALUES(?,?,?,?,?,?,?)"
    ).bind(&story_id).bind(&uid).bind(&media_url).bind(&media_type).bind(&caption).bind(&created_at).bind(&expires_at)
    .execute(&*s.db).await.map_err(|e| AuthError::Database(e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(Story {
            id: story_id,
            user_id: uid,
            username,
            display_name,
            avatar_url: None,
            media_url,
            media_type,
            caption,
            created_at,
            expires_at,
            view_count: 0,
            is_viewed: false,
        }),
    ))
}

/// Get all active stories (last 24h)
pub async fn get_stories(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Story>>, AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let uid = get_user_id_from_header(auth_hdr, &s.auth)?;

    let now = Utc::now().to_rfc3339();
    let stories: Vec<(String, String, String, String, Option<String>, String, String, String, String, String)> = sqlx::query_as(
        "SELECT s.id, s.user_id, u.username, COALESCE(u.display_name, u.username), u.avatar_url, s.media_url, s.media_type, s.caption, s.created_at, s.expires_at FROM stories s JOIN users u ON s.user_id=u.id WHERE s.expires_at > ? ORDER BY s.created_at DESC"
    ).bind(&now).fetch_all(&*s.db).await.map_err(|e| AuthError::Database(e.to_string()))?;

    let result: Vec<Story> = stories
        .into_iter()
        .map(
            |(
                id,
                user_id,
                username,
                display_name,
                _avatar_url,
                media_url,
                media_type,
                caption,
                created_at,
                expires_at,
            )| {
                Story {
                    id,
                    user_id,
                    username,
                    display_name: Some(display_name),
                    avatar_url: None,
                    media_url,
                    media_type,
                    caption: if caption.is_empty() {
                        None
                    } else {
                        Some(caption)
                    },
                    created_at,
                    expires_at,
                    view_count: 0,
                    is_viewed: false,
                }
            },
        )
        .collect();

    Ok(Json(result))
}

/// Get stories for specific user
pub async fn get_user_stories(
    State(s): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<Vec<Story>>, (StatusCode, Json<serde_json::Value>)> {
    let now = Utc::now().to_rfc3339();
    let stories: Vec<(String, String, String, String, Option<String>, String, String, String, String, String)> = sqlx::query_as(
        "SELECT s.id, s.user_id, u.username, COALESCE(u.display_name, u.username), u.avatar_url, s.media_url, s.media_type, s.caption, s.created_at, s.expires_at FROM stories s JOIN users u ON s.user_id=u.id WHERE s.user_id=? AND s.expires_at > ? ORDER BY s.created_at DESC"
    ).bind(&user_id).bind(&now).fetch_all(&*s.db).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

    let result: Vec<Story> = stories
        .into_iter()
        .map(
            |(
                id,
                uid,
                username,
                display_name,
                _avatar_url,
                media_url,
                media_type,
                caption,
                created_at,
                expires_at,
            )| {
                Story {
                    id,
                    user_id: uid,
                    username,
                    display_name: Some(display_name),
                    avatar_url: None,
                    media_url,
                    media_type,
                    caption: if caption.is_empty() {
                        None
                    } else {
                        Some(caption)
                    },
                    created_at,
                    expires_at,
                    view_count: 0,
                    is_viewed: false,
                }
            },
        )
        .collect();

    Ok(Json(result))
}

/// Get story media
pub async fn get_story_media(
    State(_s): State<AppState>,
    Path(storage_name): Path<String>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let file_path = PathBuf::from(STORIES_DIR).join(&storage_name);
    if !file_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Media not found"})),
        ));
    }

    let data = fs::read(&file_path).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;
    let mime_type = mime_guess::from_path(&storage_name).first_or_octet_stream();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type.to_string())
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .body(Body::from(data))
        .unwrap())
}

/// View story (increment view counter)
pub async fn view_story(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(story_id): Path<String>,
) -> Result<Json<serde_json::Value>, AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let uid = get_user_id_from_header(auth_hdr, &s.auth)?;

    // Record view (ignore duplicates)
    let _ = sqlx::query(
        "INSERT OR IGNORE INTO story_views(story_id, user_id, viewed_at) VALUES(?,?,?)",
    )
    .bind(&story_id)
    .bind(&uid)
    .bind(&Utc::now().to_rfc3339())
    .execute(&*s.db)
    .await;

    // Increment counter
    let _ = sqlx::query("UPDATE stories SET view_count = view_count + 1 WHERE id=?")
        .bind(&story_id)
        .execute(&*s.db)
        .await;

    Ok(Json(serde_json::json!({"viewed": true})))
}

/// Delete story (owner only)
pub async fn delete_story(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(story_id): Path<String>,
) -> Result<Json<serde_json::Value>, AuthError> {
    let auth_hdr = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let uid = get_user_id_from_header(auth_hdr, &s.auth)?;

    // Verify ownership
    let owner: Option<String> = sqlx::query_scalar("SELECT user_id FROM stories WHERE id=?")
        .bind(&story_id)
        .fetch_optional(&*s.db)
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;
    if owner.as_deref() != Some(&uid) {
        return Err(AuthError::UserNotFound);
    }

    sqlx::query("DELETE FROM stories WHERE id=?")
        .bind(&story_id)
        .execute(&*s.db)
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;
    sqlx::query("DELETE FROM story_views WHERE story_id=?")
        .bind(&story_id)
        .execute(&*s.db)
        .await
        .ok();

    Ok(Json(serde_json::json!({"deleted": true})))
}
