//! IPFS интеграция через Pinata.cloud

use axum::{
    extract::{State, Multipart},
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use crate::AppState;

#[derive(Serialize)]
pub struct IpfsUploadResponse {
    pub id: String,
    pub cid: String,
    pub filename: String,
    pub size: u64,
    pub ipfs_url: String,
    pub gateway_url: String,
}

#[derive(Serialize)]
pub struct IpfsFile {
    pub cid: String,
    pub filename: String,
    pub size: u64,
    pub uploaded_at: String,
}

/// Загрузить файл на IPFS через Pinata
pub async fn upload_to_ipfs(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<IpfsUploadResponse>, StatusCode> {
    // Получаем файл из multipart
    let file = multipart.next_field().await.map_err(|_| {
        tracing::error!("Ошибка получения файла");
        StatusCode::BAD_REQUEST
    })?.ok_or(StatusCode::BAD_REQUEST)?;

    let filename = file.file_name().unwrap_or("unnamed").to_string();
    let data = file.bytes().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let size = data.len() as u64;

    // Загрузка на Pinata
    let client = reqwest::Client::new();
    let form = reqwest::multipart::Form::new()
        .part("file", reqwest::multipart::Part::bytes(data.to_vec())
            .file_name(filename.clone()));

    let response = client
        .post("https://api.pinata.cloud/pinning/pinFileToIPFS")
        .header("pinata_api_key", &state.pinata_api_key)
        .header("pinata_secret_api_key", &state.pinata_secret_key)
        .multipart(form)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Ошибка загрузки на Pinata: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let pinata_response: PinataResponse = response.json().await.map_err(|_| {
        tracing::error!("Ошибка парсинга ответа Pinata");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let cid = pinata_response.ipfs_hash;
    let id = uuid::Uuid::new_v4().to_string();

    // Сохранение в БД
    sqlx::query(
        "INSERT INTO ipfs_files (id, cid, filename, owner_id, size, uploaded_at)
         VALUES (?, ?, ?, NULL, ?, CURRENT_TIMESTAMP)"
    )
    .bind(&id)
    .bind(&cid)
    .bind(&filename)
    .bind(size as i64)
    .execute(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(IpfsUploadResponse {
        id,
        cid: cid.clone(),
        filename,
        size,
        ipfs_url: format!("ipfs://{}", cid),
        gateway_url: format!("https://gateway.pinata.cloud/ipfs/{}", cid),
    }))
}

/// Получить файл с IPFS
pub async fn get_from_ipfs(
    State(state): State<AppState>,
    Path(cid): Path<String>,
) -> Result<Json<IpfsFile>, StatusCode> {
    let file = sqlx::query_as(
        "SELECT cid, filename, size, uploaded_at FROM ipfs_files WHERE cid = ?"
    )
    .bind(&cid)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(file))
}

#[derive(Deserialize)]
struct PinataResponse {
    #[serde(rename = "IpfsHash")]
    ipfs_hash: String,
    #[serde(rename = "PinSize")]
    pin_size: u64,
    #[serde(rename = "Timestamp")]
    timestamp: String,
}

/// Загрузить JSON метаданные на IPFS
pub async fn upload_json_to_ipfs(
    state: &AppState,
    json_data: serde_json::Value,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let response = client
        .post("https://api.pinata.cloud/pinning/pinJSONToIPFS")
        .header("pinata_api_key", &state.pinata_api_key)
        .header("pinata_secret_api_key", &state.pinata_secret_key)
        .json(&json_data)
        .send()
        .await?;

    let pinata_response: PinataResponse = response.json().await?;

    Ok(pinata_response.ipfs_hash)
}
