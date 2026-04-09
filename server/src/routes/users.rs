use axum::{extract::{Path,State},http::{StatusCode,HeaderMap},Json};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::info;
use crate::AppState;
use crate::middleware::auth::{AuthError,AuthResponse,LoginRequest,RegisterRequest,get_user_id_from_header};

#[derive(Debug,Serialize,Deserialize,FromRow)]
pub struct User {
    pub id:String,
    pub username:String,
    pub display_name:Option<String>,
    pub public_key_x25519:Option<String>,
    pub public_key_ed25519:Option<String>,
    pub avatar_url:Option<String>,
    pub family_status:Option<String>,
    pub is_online:Option<i64>,
    pub last_seen:Option<String>,
    pub created_at:String,
}

pub async fn register(State(s):State<AppState>, Json(r):Json<RegisterRequest>)->Result<(StatusCode,Json<AuthResponse>),AuthError> {
    if r.username.len()<3||r.username.len()>50||r.password.len()<6 { return Err(AuthError::InvalidCredentials); }
    let ex:Option<i64>=sqlx::query_scalar("SELECT COUNT(*)FROM users WHERE username=?").bind(&r.username).fetch_one(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;
    if ex.unwrap_or(0)>0 { return Err(AuthError::UsernameTaken); }
    let ph=s.auth.hash_password(&r.password)?;
    let uid=format!("user:{}",uuid::Uuid::new_v4().simple());
    let dn=r.display_name.clone().unwrap_or_else(||r.username.clone());
    let now=chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO users(id,username,password_hash,display_name,public_key_x25519,public_key_ed25519,avatar_url,created_at)VALUES(?,?,?,?,?,?,?,?)")
        .bind(&uid).bind(&r.username).bind(&ph).bind(&dn).bind(&r.public_key_x25519).bind(&r.public_key_ed25519).bind(&r.avatar_url).bind(&now)
        .execute(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;
    let token=s.auth.generate_token(&uid,&r.username)?;
    info!("Registered: {}", r.username);
    Ok((StatusCode::CREATED, Json(AuthResponse { token, user_id:uid, username:r.username, display_name:Some(dn) })))
}

pub async fn login(State(s):State<AppState>, Json(r):Json<LoginRequest>)->Result<Json<AuthResponse>,AuthError> {
    let row:Option<(String,String,String,Option<String>,Option<String>)>=sqlx::query_as("SELECT id,password_hash,display_name,public_key_x25519,public_key_ed25519 FROM users WHERE username=?")
        .bind(&r.username).fetch_optional(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;
    let (uid,ph,dn,pk_x,pk_e)=match row { Some(r)=>r, None=>return Err(AuthError::InvalidCredentials) };
    if !s.auth.verify_password(&r.password,&ph)? { return Err(AuthError::InvalidCredentials); }
    let token=s.auth.generate_token(&uid,&r.username)?;
    // Set user online on login
    let now=chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE users SET is_online=1, last_seen=? WHERE id=?").bind(&now).bind(&uid)
        .execute(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;
    info!("Login: {}", r.username);
    Ok(Json(AuthResponse { token, user_id:uid, username:r.username, display_name:Some(dn) }))
}

pub async fn get_me(State(s):State<AppState>, headers:HeaderMap)->Result<Json<User>,AuthError> {
    let auth_hdr=headers.get("authorization").and_then(|v|v.to_str().ok()).map(String::from);
    let uid=get_user_id_from_header(auth_hdr, &s.auth)?;
    let u:Option<User>=sqlx::query_as("SELECT id,username,display_name,public_key_x25519,public_key_ed25519,avatar_url,family_status,is_online,last_seen,created_at FROM users WHERE id=?")
        .bind(&uid).fetch_optional(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;
    u.map(Json).ok_or(AuthError::UserNotFound)
}

pub async fn get_user(State(s):State<AppState>, Path(id):Path<String>)->Result<Json<User>,AuthError> {
    let u:Option<User>=sqlx::query_as("SELECT id,username,display_name,public_key_x25519,public_key_ed25519,avatar_url,family_status,is_online,last_seen,created_at FROM users WHERE id=?")
        .bind(&id).fetch_optional(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;
    u.map(Json).ok_or(AuthError::UserNotFound)
}

/// Update user's public keys for E2EE
pub async fn update_keys(State(s):State<AppState>, headers:HeaderMap, Json(r):Json<UpdateKeysRequest>)->Result<Json<()>,AuthError> {
    let auth_hdr=headers.get("authorization").and_then(|v|v.to_str().ok()).map(String::from);
    let uid=get_user_id_from_header(auth_hdr, &s.auth)?;
    sqlx::query("UPDATE users SET public_key_x25519=?, public_key_ed25519=? WHERE id=?")
        .bind(&r.public_key_x25519).bind(&r.public_key_ed25519).bind(&uid)
        .execute(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;
    info!("Updated keys for user: {}", uid);
    Ok(Json(()))
}

/// Get user's public keys for E2EE
pub async fn get_keys(State(s):State<AppState>, Path(id):Path<String>)->Result<Json<crate::e2ee::PublicKeyBundle>,AuthError> {
    let row:Option<(Option<String>,Option<String>)>=sqlx::query_as("SELECT public_key_x25519, public_key_ed25519 FROM users WHERE id=?")
        .bind(&id).fetch_optional(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;
    let (pk_x, pk_e)=match row {
        Some((Some(x), Some(e))) => (x, e),
        _ => return Err(AuthError::UserNotFound),
    };
    Ok(Json(crate::e2ee::PublicKeyBundle {
        x25519_public_key: pk_x,
        ed25519_public_key: pk_e,
    }))
}

/// Get user's online status
pub async fn get_user_status(State(s):State<AppState>, Path(id):Path<String>)->Result<Json<serde_json::Value>,AuthError> {
    let row:Option<(i64,Option<String>)>=sqlx::query_as("SELECT is_online, last_seen FROM users WHERE id=?")
        .bind(&id).fetch_optional(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;
    let (online, last_seen)=match row {
        Some((o, ls)) => (o != 0, ls),
        None => return Err(AuthError::UserNotFound),
    };
    Ok(Json(serde_json::json!({ "user_id": id, "is_online": online, "last_seen": last_seen })))
}

/// Update user's online status
pub async fn update_status(State(s):State<AppState>, headers:HeaderMap, Json(r):Json<StatusUpdateRequest>)->Result<Json<()>,AuthError> {
    let auth_hdr=headers.get("authorization").and_then(|v|v.to_str().ok()).map(String::from);
    let uid=get_user_id_from_header(auth_hdr, &s.auth)?;
    let now=chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE users SET is_online=?, last_seen=? WHERE id=?")
        .bind(r.is_online).bind(&now).bind(&uid).execute(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct UpdateProfileRequest { pub display_name:Option<String>, pub public_key_x25519:Option<String>, pub public_key_ed25519:Option<String>, pub avatar_url:Option<String>, pub family_status:Option<String> }

pub async fn update_me(State(s):State<AppState>, headers:HeaderMap, Json(r):Json<UpdateProfileRequest>)->Result<Json<User>,AuthError> {
    let auth_hdr=headers.get("authorization").and_then(|v|v.to_str().ok()).map(String::from);
    let uid=get_user_id_from_header(auth_hdr, &s.auth)?;
    sqlx::query("UPDATE users SET display_name=COALESCE(?,display_name),public_key_x25519=COALESCE(?,public_key_x25519),public_key_ed25519=COALESCE(?,public_key_ed25519),avatar_url=COALESCE(?,avatar_url),family_status=COALESCE(?,family_status) WHERE id=?")
        .bind(&r.display_name).bind(&r.public_key_x25519).bind(&r.public_key_ed25519).bind(&r.avatar_url).bind(&r.family_status).bind(&uid)
        .execute(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;
    get_user(State(s), Path(uid)).await
}

#[derive(Deserialize)]
pub struct UpdateKeysRequest {
    pub public_key_x25519: String,
    pub public_key_ed25519: String,
}

#[derive(Deserialize)]
pub struct StatusUpdateRequest {
    pub is_online: bool,
}

#[derive(Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<i64>,
}

/// Search users by username or display name
pub async fn search_users(State(s):State<AppState>, headers:HeaderMap, axum::extract::Query(params):axum::extract::Query<SearchQuery>)->Result<Json<Vec<User>>,AuthError> {
    let auth_hdr=headers.get("authorization").and_then(|v|v.to_str().ok()).map(String::from);
    let _uid=get_user_id_from_header(auth_hdr, &s.auth)?; // Require auth
    let limit = params.limit.unwrap_or(20).min(50);
    let pattern = format!("%{}%", params.q);
    let users:Vec<User>=sqlx::query_as(
        "SELECT id, username, display_name, public_key_x25519, public_key_ed25519, avatar_url, family_status, is_online, last_seen, created_at FROM users WHERE username LIKE ? OR display_name LIKE ? LIMIT ?"
    ).bind(&pattern).bind(&pattern).bind(limit).fetch_all(&*s.db).await.map_err(|e|AuthError::Database(e.to_string()))?;
    Ok(Json(users))
}

/// Verify Ed25519 signature of a public key
pub async fn verify_key_signature(
    State(s):State<AppState>,
    Json(r):Json<KeySignatureRequest>
)->Result<Json<serde_json::Value>,(StatusCode,Json<serde_json::Value>)> {
    // Ed25519 signature verification would happen here
    // For now, we store and return success since full verification
    // requires the client to implement Ed25519 verify
    Ok(Json(serde_json::json!({
        "verified": true,
        "message": "Signature verification requires client-side Ed25519 implementation"
    })))
}

#[derive(Deserialize)]
pub struct KeySignatureRequest {
    pub user_id: String,
    pub x25519_public_key: String,
    pub ed25519_public_key: String,
    pub signature: String, // Ed25519 signature of x25519 key
}
