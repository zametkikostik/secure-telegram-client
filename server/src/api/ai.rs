// server/src/api/ai.rs
//! AI API (Qwen API для перевода, саммаризации, чата)

use axum::{
    extract::State,
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use crate::api::AppState;

#[derive(Deserialize)]
pub struct TranslateRequest {
    pub text: String,
    pub from: String,
    pub to: String,
}

#[derive(Serialize)]
pub struct TranslateResponse {
    pub translated_text: String,
    pub detected_language: Option<String>,
}

#[derive(Deserialize)]
pub struct SummarizeRequest {
    pub text: String,
    pub max_length: Option<usize>,
}

#[derive(Serialize)]
pub struct SummarizeResponse {
    pub summary: String,
}

#[derive(Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub context: Option<Vec<ChatMessage>>,
}

#[derive(Deserialize, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub reply: String,
}

/// Перевод текста через Qwen API
pub async fn translate(
    State(state): State<AppState>,
    Json(req): Json<TranslateRequest>,
) -> Result<Json<TranslateResponse>, StatusCode> {
    let api_key = std::env::var("QWEN_API_KEY").unwrap_or_default();
    let client = reqwest::Client::new();

    let response = client
        .post("https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": "qwen-turbo",
            "messages": [{
                "role": "user",
                "content": format!("Translate from {} to {}: {}", req.from, req.to, req.text)
            }],
            "max_tokens": 2048
        }))
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Ошибка Qwen API: {}", e);
            StatusCode::BAD_GATEWAY
        })?
        .json::<serde_json::Value>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let translated = response["output"]["text"]
        .as_str()
        .unwrap_or(&req.text)
        .to_string();

    Ok(Json(TranslateResponse {
        translated_text: translated,
        detected_language: None,
    }))
}

/// Саммаризация текста
pub async fn summarize(
    State(state): State<AppState>,
    Json(req): Json<SummarizeRequest>,
) -> Result<Json<SummarizeResponse>, StatusCode> {
    let api_key = std::env::var("QWEN_API_KEY").unwrap_or_default();
    let client = reqwest::Client::new();
    let max_length = req.max_length.unwrap_or(500);

    let response = client
        .post("https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": "qwen-turbo",
            "messages": [{
                "role": "user",
                "content": format!("Саммаризируй следующий текст (максимум {} слов):\n{}", max_length, req.text)
            }],
            "max_tokens": max_length
        }))
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Ошибка Qwen API: {}", e);
            StatusCode::BAD_GATEWAY
        })?
        .json::<serde_json::Value>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let summary = response["output"]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(Json(SummarizeResponse { summary }))
}

/// AI чат с ассистентом
pub async fn chat(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, StatusCode> {
    let api_key = std::env::var("QWEN_API_KEY").unwrap_or_default();
    let client = reqwest::Client::new();

    let mut messages = vec![serde_json::json!({
        "role": "system",
        "content": "Ты Liberty Reach AI Assistant - полезный помощник для мессенджера."
    })];

    if let Some(context) = req.context {
        for msg in context {
            messages.push(serde_json::json!({
                "role": msg.role,
                "content": msg.content
            }));
        }
    }

    messages.push(serde_json::json!({
        "role": "user",
        "content": req.message
    }));

    let response = client
        .post("https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": "qwen-turbo",
            "messages": messages,
            "max_tokens": 2048
        }))
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Ошибка Qwen API: {}", e);
            StatusCode::BAD_GATEWAY
        })?
        .json::<serde_json::Value>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let reply = response["output"]["text"]
        .as_str()
        .unwrap_or("Извините, произошла ошибка.")
        .to_string();

    Ok(Json(ChatResponse { reply }))
}
