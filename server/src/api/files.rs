// server/src/api/files.rs
//! API загрузки файлов

use axum::{
    extract::{State, Multipart, Path},
    Json,
    http::StatusCode,
};
use serde::Serialize;
use uuid::Uuid;
use std::path::PathBuf;
use crate::api::AppState;

#[derive(Serialize)]
pub struct FileResponse {
    pub id: String,
    pub filename: String,
    pub original_name: String,
    pub mime_type: String,
    pub size: u64,
    pub url: String,
    pub created_at: String,
}

/// Загрузка файла
pub async fn upload_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
    // claims: Claims,
) -> Result<Json<FileResponse>, StatusCode> {
    let field = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .ok_or(StatusCode::BAD_REQUEST)?;

    let filename = field.file_name().unwrap_or("unknown").to_string();
    let mime_type = field.content_type().map(|m| m.to_string()).unwrap_or_default();
    let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
    let size = data.len() as u64;

    // Генерация уникального имени
    let file_id = Uuid::new_v4().to_string();
    let extension = std::path::Path::new(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    let new_filename = format!("{}.{}", file_id, extension);

    // Определение типа файла и пути
    let (file_type, subdir) = if mime_type.starts_with("image/") {
        ("image", "images")
    } else if mime_type.starts_with("video/") {
        ("video", "videos")
    } else if mime_type.starts_with("audio/") {
        ("audio", "audio")
    } else {
        ("file", "files")
    };

    // Сохранение файла
    let file_path = PathBuf::from(&state.uploads_dir).join(subdir).join(&new_filename);
    
    // Создание директории если не существует
    if let Some(parent) = file_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    tokio::fs::write(&file_path, &data)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Сохранение в базу данных
    let url = format!("/uploads/{}/{}", subdir, new_filename);
    
    sqlx::query(
        "INSERT INTO files (id, owner_id, filename, original_name, mime_type, size, url) 
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&file_id)
    .bind("owner-id") // claims.sub
    .bind(&new_filename)
    .bind(&filename)
    .bind(&mime_type)
    .bind(size as i64)
    .bind(&url)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Ошибка сохранения файла: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(FileResponse {
        id: file_id,
        filename: new_filename,
        original_name: filename,
        mime_type,
        size,
        url,
        created_at: chrono::Utc::now().to_rfc3339(),
    }))
}

/// Получить файл
pub async fn get_file(
    State(state): State<AppState>,
    Path(file_id): Path<String>,
) -> Result<Json<FileResponse>, StatusCode> {
    let file: (String, String, String, String, i64, String, String) = sqlx::query_as(
        "SELECT id, filename, original_name, mime_type, size, url, created_at FROM files WHERE id = ?"
    )
    .bind(&file_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(FileResponse {
        id: file.0,
        filename: file.1,
        original_name: file.2,
        mime_type: file.3,
        size: file.4 as u64,
        url: file.5,
        created_at: file.6,
    }))
}
