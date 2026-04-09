// File upload/download routes

use axum::{
    extract::{State, Path, Multipart},
    http::{StatusCode, HeaderMap, header},
    Json,
    response::Response,
    body::Body,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;
use chrono::Utc;
use tokio::fs;

use crate::AppState;
use crate::middleware::auth::{AuthError, get_user_id_from_header};

const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50MB
const UPLOAD_DIR: &str = "uploads";

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUploadResponse {
    pub file_id: String,
    pub file_name: String,
    pub file_size: u64,
    pub mime_type: String,
    pub url: String,
    pub uploaded_at: String,
}

/// Upload file
pub async fn upload_file(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<FileUploadResponse>), AuthError> {
    let auth_hdr = headers.get("authorization").and_then(|v| v.to_str().ok()).map(String::from);
    let uid = get_user_id_from_header(auth_hdr, &state.auth)?;

    // Create uploads directory if not exists
    let upload_path = PathBuf::from(UPLOAD_DIR);
    if !upload_path.exists() {
        fs::create_dir_all(&upload_path).await.map_err(|e| AuthError::Database(e.to_string()))?;
    }

    // Process multipart upload
    while let Some(field) = multipart.next_field().await.map_err(|e| AuthError::Database(e.to_string()))? {
        if let Some(file_name) = field.file_name() {
            let file_name = file_name.to_string();
            let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
            let data = field.bytes().await.map_err(|e| AuthError::Database(e.to_string()))?;

            if data.len() as u64 > MAX_FILE_SIZE {
                return Err(AuthError::Database("File too large (max 50MB)".into()));
            }

            // Generate unique file ID
            let file_id = Uuid::new_v4().simple().to_string();
            let extension = std::path::Path::new(&file_name)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("bin");
            let storage_name = format!("{}.{}", file_id, extension);
            let file_path = upload_path.join(&storage_name);

            // Save file
            fs::write(&file_path, &data).await.map_err(|e| AuthError::Database(e.to_string()))?;

            // Save metadata to DB
            let now = Utc::now().to_rfc3339();
            sqlx::query(
                "INSERT INTO files(id, owner_id, file_name, storage_name, file_size, mime_type, created_at) VALUES(?,?,?,?,?,?,?)"
            )
            .bind(&file_id)
            .bind(&uid)
            .bind(&file_name)
            .bind(&storage_name)
            .bind(data.len() as i64)
            .bind(&content_type)
            .bind(&now)
            .execute(&*state.db).await.map_err(|e| AuthError::Database(e.to_string()))?;

            return Ok((StatusCode::CREATED, Json(FileUploadResponse {
                file_id: file_id.clone(),
                file_name,
                file_size: data.len() as u64,
                mime_type: content_type,
                url: format!("/api/v1/files/{}", file_id),
                uploaded_at: now,
            })));
        }
    }

    Err(AuthError::Database("No file found in upload".into()))
}

/// Download file
pub async fn download_file(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    // Get file metadata from DB
    let row: Option<(String, String, i64)> = sqlx::query_as(
        "SELECT file_name, storage_name, file_size FROM files WHERE id=?"
    )
    .bind(&id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

    let (file_name, storage_name, _file_size) = row.ok_or_else(|| {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "File not found"})))
    })?;

    let file_path = PathBuf::from(UPLOAD_DIR).join(&storage_name);

    if !file_path.exists() {
        return Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "File not found on disk"}))));
    }

    // Update download counter
    let _ = sqlx::query("UPDATE files SET download_count = download_count + 1 WHERE id=?")
        .bind(&id)
        .execute(&*state.db)
        .await;

    // Read file and return response
    let data = fs::read(&file_path).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

    let mime_type = mime_guess::from_path(&file_name).first_or_octet_stream();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime_type.to_string())
        .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", file_name))
        .body(Body::from(data))
        .unwrap())
}

/// Get file metadata
pub async fn get_file_info(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let row: Option<(String, String, i64, String, String, i64)> = sqlx::query_as(
        "SELECT id, file_name, file_size, mime_type, created_at, download_count FROM files WHERE id=?"
    )
    .bind(&id)
    .fetch_optional(&*state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

    let (id, file_name, file_size, mime_type, created_at, download_count) = row.ok_or_else(|| {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "File not found"})))
    })?;

    Ok(Json(serde_json::json!({
        "file_id": id,
        "file_name": file_name,
        "file_size": file_size,
        "mime_type": mime_type,
        "created_at": created_at,
        "download_count": download_count,
        "url": format!("/api/v1/files/{}", id),
    })))
}
