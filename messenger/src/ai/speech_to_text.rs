//! Speech-to-Text — transcribe audio to text
//!
//! Online: OpenRouter Whisper-compatible endpoint
//! Local: Vosk offline (via CLI wrapper)

use crate::ai::client::{AiClient, AiError, AiResult};
use std::path::Path;

const OPENROUTER_WHISPER_MODEL: &str = "openai/whisper-large-v3";

/// Transcribe audio bytes to text
pub async fn transcribe(client: &AiClient, audio: &[u8], lang: &str) -> AiResult<String> {
    // Try OpenRouter Whisper first
    match transcribe_openrouter(client, audio, lang).await {
        Ok(text) => {
            tracing::info!("STT: used OpenRouter Whisper");
            return Ok(text);
        }
        Err(e) => {
            tracing::warn!("OpenRouter Whisper failed: {}", e);
        }
    }

    // Fallback to local Vosk
    match transcribe_vosk_local(audio, lang).await {
        Ok(text) => {
            tracing::info!("STT: used local Vosk");
            return Ok(text);
        }
        Err(e) => {
            tracing::warn!("Local Vosk failed: {}", e);
        }
    }

    Err(AiError::BothFailed {
        openrouter: "OpenRouter Whisper not available".into(),
        ollama: "Local Vosk not available".into(),
    })
}

/// Transcribe via OpenRouter Whisper-compatible endpoint
async fn transcribe_openrouter(client: &AiClient, audio: &[u8], lang: &str) -> AiResult<String> {
    let api_key = client
        .config
        .openrouter_key
        .as_ref()
        .ok_or_else(|| AiError::NoApiKey("OpenRouter".to_string()))?;

    // Create a temporary file for the audio
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("stt_{}.ogg", uuid::Uuid::new_v4()));

    std::fs::write(&temp_path, audio)
        .map_err(|e| AiError::Http(format!("Failed to write temp file: {}", e)))?;

    let file_bytes = std::fs::read(&temp_path)
        .map_err(|e| AiError::Http(format!("Failed to read temp file: {}", e)))?;

    let lang_owned = lang.to_string();

    let form = reqwest::multipart::Form::new()
        .text("model", OPENROUTER_WHISPER_MODEL)
        .text("language", lang_owned)
        .part(
            "file",
            reqwest::multipart::Part::bytes(file_bytes)
                .file_name("audio.ogg")
                .mime_str("audio/ogg")
                .map_err(|e| AiError::Http(format!("Invalid MIME: {}", e)))?,
        );

    let response = client
        .http_client
        .post("https://openrouter.ai/api/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .await
        .map_err(|e| AiError::Http(e.to_string()))?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(AiError::OpenRouter(format!("HTTP {}: {}", status, body)));
    }

    #[derive(serde::Deserialize)]
    struct WhisperResponse {
        text: String,
    }

    let data: WhisperResponse = response
        .json::<WhisperResponse>()
        .await
        .map_err(|e| AiError::OpenRouter(format!("JSON parse: {}", e)))?;

    Ok(data.text.trim().to_string())
}

/// Transcribe via local Vosk (offline)
async fn transcribe_vosk_local(audio: &[u8], _lang: &str) -> AiResult<String> {
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("vosk_{}.wav", uuid::Uuid::new_v4()));

    std::fs::write(&temp_path, audio)
        .map_err(|e| AiError::Ollama(format!("Failed to write temp file: {}", e)))?;

    // Check if vosk CLI is available
    let vosk_available = std::process::Command::new("vosk-cli")
        .arg("--version")
        .output()
        .is_ok();

    if !vosk_available {
        let _ = std::fs::remove_file(&temp_path);
        return Err(AiError::Ollama(
            "Vosk CLI not installed. Install with: pip install vosk".into(),
        ));
    }

    // Call vosk CLI
    let output = std::process::Command::new("vosk-cli")
        .arg("--model")
        .arg("vosk-model-small-ru-0.22")
        .arg("--file")
        .arg(&temp_path)
        .output()
        .map_err(|e| AiError::Ollama(format!("Vosk process failed: {}", e)))?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AiError::Ollama(format!("Vosk error: {}", stderr)));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text.trim().to_string())
}

/// Transcribe from a file path
pub async fn transcribe_file(client: &AiClient, file_path: &Path, lang: &str) -> AiResult<String> {
    let audio = std::fs::read(file_path)
        .map_err(|e| AiError::Http(format!("Failed to read file: {}", e)))?;

    transcribe(client, &audio, lang).await
}

/// Check if local STT is available
pub fn is_local_stt_available() -> bool {
    std::process::Command::new("vosk-cli")
        .arg("--version")
        .output()
        .is_ok()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::client::AiConfig;

    fn make_client() -> AiClient {
        AiClient::new(AiConfig::default())
    }

    #[test]
    fn test_vosk_availability_check() {
        let available = is_local_stt_available();
        // Just verify the function runs
        let _ = available;
    }

    #[tokio::test]
    #[ignore] // Requires real audio and backend
    async fn test_transcribe_audio() {
        let client = make_client();
        // Dummy audio bytes — real test needs actual audio
        let audio = vec![0u8; 1024];

        let result = transcribe(&client, &audio, "ru").await;
        // May fail without real audio, but shouldn't panic
        let _ = result;
    }
}
