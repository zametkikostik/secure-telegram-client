//! Admin Panel API

use axum::{
    extract::{State, Path, Query},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use crate::AppState;

// ==================== Dashboard ====================

#[derive(Serialize)]
pub struct DashboardStats {
    pub total_users: i64,
    pub total_bots: i64,
    pub verified_users: i64,
    pub pending_verifications: i64,
    pub active_reports: i64,
    pub banned_users: i64,
    pub messages_24h: i64,
}

pub async fn get_dashboard(
    State(state): State<AppState>,
) -> Result<Json<DashboardStats>, StatusCode> {
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await.unwrap_or(0);

    let total_bots: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bots")
        .fetch_one(&state.db)
        .await.unwrap_or(0);

    let verified_users: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT user_id) FROM user_badges WHERE badge_id = 'verified'"
    )
    .fetch_one(&state.db)
    .await.unwrap_or(0);

    let pending_verifications: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM verifications WHERE status = 'pending'"
    )
    .fetch_one(&state.db)
    .await.unwrap_or(0);

    let active_reports: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM reports WHERE status = 'pending'"
    )
    .fetch_one(&state.db)
    .await.unwrap_or(0);

    let banned_users: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE is_banned = TRUE"
    )
    .fetch_one(&state.db)
    .await.unwrap_or(0);

    let messages_24h: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM messages WHERE created_at > datetime('now', '-1 day')"
    )
    .fetch_one(&state.db)
    .await.unwrap_or(0);

    Ok(Json(DashboardStats {
        total_users,
        total_bots,
        verified_users,
        pending_verifications,
        active_reports,
        banned_users,
        messages_24h,
    }))
}

// ==================== Users ====================

#[derive(Serialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub is_verified: bool,
    pub is_banned: bool,
    pub created_at: String,
}

pub async fn list_users(
    State(state): State<AppState>,
    Query(params): Query<ListUsersParams>,
) -> Result<Json<Vec<User>>, StatusCode> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    let users = sqlx::query_as(
        "SELECT u.id, u.username, u.email, u.avatar_url, 
                CASE WHEN ub.user_id IS NOT NULL THEN 1 ELSE 0 END as is_verified,
                COALESCE(u.is_banned, 0) as is_banned,
                u.created_at
         FROM users u
         LEFT JOIN user_badges ub ON u.id = ub.user_id AND ub.badge_id = 'verified'
         ORDER BY u.created_at DESC
         LIMIT ? OFFSET ?"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(users))
}

#[derive(Deserialize)]
pub struct ListUsersParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<User>, StatusCode> {
    let user = sqlx::query_as(
        "SELECT u.id, u.username, u.email, u.avatar_url,
                CASE WHEN ub.user_id IS NOT NULL THEN 1 ELSE 0 END as is_verified,
                COALESCE(u.is_banned, 0) as is_banned,
                u.created_at
         FROM users u
         LEFT JOIN user_badges ub ON u.id = ub.user_id AND ub.badge_id = 'verified'
         WHERE u.id = ?"
    )
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(user))
}

pub async fn ban_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("UPDATE users SET is_banned = TRUE WHERE id = ?")
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

pub async fn unban_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("UPDATE users SET is_banned = FALSE WHERE id = ?")
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

// ==================== Verification ====================

#[derive(Serialize)]
pub struct VerificationRequest {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub verification_type: String,
    pub document_url: Option<String>,
    pub status: String,
    pub created_at: String,
}

pub async fn list_verification_requests(
    State(state): State<AppState>,
) -> Result<Json<Vec<VerificationRequest>>, StatusCode> {
    let requests = sqlx::query_as(
        "SELECT v.id, v.user_id, u.username, v.type as verification_type,
                v.document_url, v.status, v.created_at
         FROM verifications v
         JOIN users u ON v.user_id = u.id
         ORDER BY v.created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(requests))
}

pub async fn verify_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    admin_id: String, // TODO: Получить из токена
) -> Result<StatusCode, StatusCode> {
    // Добавляем бейдж верификации
    sqlx::query(
        "INSERT OR REPLACE INTO user_badges (user_id, badge_id, assigned_by, assigned_at)
         VALUES (?, 'verified', ?, CURRENT_TIMESTAMP)"
    )
    .bind(&user_id)
    .bind(&admin_id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Обновляем статус верификации
    sqlx::query(
        "UPDATE verifications SET status = 'approved', reviewed_by = ?, reviewed_at = CURRENT_TIMESTAMP
         WHERE user_id = ? AND status = 'pending'"
    )
    .bind(&admin_id)
    .bind(&user_id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

pub async fn revoke_verification(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    // Удаляем бейдж
    sqlx::query("DELETE FROM user_badges WHERE user_id = ? AND badge_id = 'verified'")
        .bind(&user_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

// ==================== Badges ====================

#[derive(Serialize)]
pub struct Badge {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub color: String,
    pub is_verified_badge: bool,
}

pub async fn list_badges(
    State(state): State<AppState>,
) -> Result<Json<Vec<Badge>>, StatusCode> {
    let badges = sqlx::query_as(
        "SELECT id, name, description, icon_url, color, is_verified_badge FROM badges"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(badges))
}

#[derive(Deserialize)]
pub struct CreateBadgeRequest {
    pub name: String,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub color: Option<String>,
}

pub async fn create_badge(
    State(state): State<AppState>,
    Json(req): Json<CreateBadgeRequest>,
) -> Result<Json<Badge>, StatusCode> {
    let badge_id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO badges (id, name, description, icon_url, color, created_at)
         VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)"
    )
    .bind(&badge_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.icon_url)
    .bind(req.color.as_deref().unwrap_or("#3390EC"))
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(Json(Badge {
        id: badge_id,
        name: req.name,
        description: req.description,
        icon_url: req.icon_url,
        color: req.color.unwrap_or_else(|| "#3390EC".to_string()),
        is_verified_badge: false,
    }))
}

pub async fn delete_badge(
    State(state): State<AppState>,
    Path(badge_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM badges WHERE id = ?")
        .bind(&badge_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn assign_badge(
    State(state): State<AppState>,
    Path((user_id, badge_id)): Path<(String, String)>,
    admin_id: String,
) -> Result<StatusCode, StatusCode> {
    sqlx::query(
        "INSERT OR REPLACE INTO user_badges (user_id, badge_id, assigned_by, assigned_at)
         VALUES (?, ?, ?, CURRENT_TIMESTAMP)"
    )
    .bind(&user_id)
    .bind(&badge_id)
    .bind(&admin_id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(StatusCode::OK)
}

// ==================== Reports ====================

#[derive(Serialize)]
pub struct Report {
    pub id: String,
    pub reporter_id: Option<String>,
    pub reported_user_id: Option<String>,
    pub reported_message_id: Option<String>,
    pub reason: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
}

pub async fn list_reports(
    State(state): State<AppState>,
) -> Result<Json<Vec<Report>>, StatusCode> {
    let reports = sqlx::query_as(
        "SELECT id, reporter_id, reported_user_id, reported_message_id, reason, 
                description, status, created_at
         FROM reports
         ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(reports))
}

#[derive(Deserialize)]
pub struct HandleReportRequest {
    pub status: String, // approved, rejected, ignored
    pub action: Option<String>,
}

pub async fn handle_report(
    State(state): State<AppState>,
    Path(report_id): Path<String>,
    admin_id: String,
    Json(req): Json<HandleReportRequest>,
) -> Result<StatusCode, StatusCode> {
    sqlx::query(
        "UPDATE reports SET status = ?, reviewed_by = ?, reviewed_at = CURRENT_TIMESTAMP
         WHERE id = ?"
    )
    .bind(&req.status)
    .bind(&admin_id)
    .bind(&report_id)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(StatusCode::OK)
}

// ==================== Bots ====================

use crate::api::Bot;

pub async fn list_bots(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::api::Bot>>, StatusCode> {
    let bots = sqlx::query_as(
        "SELECT id, owner_id, username, name, description, token, avatar_url, 
                is_verified, created_at
         FROM bots
         ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(bots))
}

pub async fn verify_bot(
    State(state): State<AppState>,
    Path(bot_id): Path<String>,
    admin_id: String,
) -> Result<StatusCode, StatusCode> {
    sqlx::query("UPDATE bots SET is_verified = TRUE WHERE id = ?")
        .bind(&bot_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

// ==================== Settings ====================

#[derive(Serialize)]
pub struct SystemSettings {
    pub registration_enabled: bool,
    pub verification_enabled: bool,
    pub max_bots_per_user: i64,
}

pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<SystemSettings>, StatusCode> {
    let settings = sqlx::query_as(
        "SELECT key, value FROM system_settings"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut result = SystemSettings {
        registration_enabled: true,
        verification_enabled: true,
        max_bots_per_user: 10,
    };

    for (key, value): (String, String) in settings {
        match key.as_str() {
            "registration_enabled" => result.registration_enabled = value == "true",
            "verification_enabled" => result.verification_enabled = value == "true",
            "max_bots_per_user" => result.max_bots_per_user = value.parse().unwrap_or(10),
            _ => {}
        }
    }

    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct UpdateSettingsRequest {
    pub registration_enabled: Option<bool>,
    pub verification_enabled: Option<bool>,
    pub max_bots_per_user: Option<i64>,
}

pub async fn update_settings(
    State(state): State<AppState>,
    admin_id: String,
    Json(req): Json<UpdateSettingsRequest>,
) -> Result<Json<SystemSettings>, StatusCode> {
    if let Some(value) = req.registration_enabled {
        sqlx::query(
            "INSERT OR REPLACE INTO system_settings (key, value, updated_by, updated_at)
             VALUES ('registration_enabled', ?, ?, CURRENT_TIMESTAMP)"
        )
        .bind(if value { "true" } else { "false" })
        .bind(&admin_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    }

    if let Some(value) = req.verification_enabled {
        sqlx::query(
            "INSERT OR REPLACE INTO system_settings (key, value, updated_by, updated_at)
             VALUES ('verification_enabled', ?, ?, CURRENT_TIMESTAMP)"
        )
        .bind(if value { "true" } else { "false" })
        .bind(&admin_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    }

    get_settings(State(state)).await
}
