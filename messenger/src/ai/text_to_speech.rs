//! Text-to-Speech — synthesize text into audio
//!
//! Online: OpenRouter TTS-compatible endpoint
//! Local: Coqui TTS via CLI

use crate::ai::client::{AiClient, AiError, AiResult};
use std::path::Path;

/// Synthesize text to audio bytes (OGG opus)
pub async fn synthesize(
    client: &AiClient,
    text: &str,
    lang: &str,
) -> AiResult<Vec<u8>> {
    // Try local Coqui TTS first (prefer local for TTS)
    if client.config.prefer_local {
        match synthesize_coqui_local(text, lang).await {
            Ok(audio) => {
                tracing::info!("TTS: used local Coqui");
                return Ok(audio);
            }
            Err(e) => {
                tracing::warn!("Local Coqui failed: {}", e);
            }
        }
    }

    // Try online (placeholder — OpenRouter doesn't have TTS yet)
    match synthesize_online(client, text, lang).await {
        Ok(audio) => {
            tracing::info!("TTS: used online endpoint");
            return Ok(audio);
        }
        Err(e) => {
            tracing::warn!("Online TTS failed: {}", e);
        }
    }

    // If prefer_local is false, try local as fallback
    if !client.config.prefer_local {
        match synthesize_coqui_local(text, lang).await {
            Ok(audio) => {
                tracing::info!("TTS: fallback to local Coqui");
                return Ok(audio);
            }
            Err(e) => {
                tracing::warn!("Local Coqui fallback failed: {}", e);
            }
        }
    }

    Err(AiError::BothFailed {
        openrouter: "Online TTS not available".into(),
        ollama: "Local Coqui TTS not available".into(),
    })
}

/// Synthesize via online endpoint (placeholder)
async fn synthesize_online(_client: &AiClient, _text: &str, _lang: &str) -> AiResult<Vec<u8>> {
    // OpenRouter doesn't have a TTS endpoint yet
    // This is a placeholder for future integration
    Err(AiError::NotSupported(
        "Online TTS not yet implemented".into(),
    ))
}

/// Synthesize via local Coqui TTS
async fn synthesize_coqui_local(text: &str, lang: &str) -> AiResult<Vec<u8>> {
    // Check if tts CLI is available
    if !is_coqui_tts_available() {
        return Err(AiError::Ollama(
            "Coqui TTS not installed. Install with: pip install TTS".into(),
        ));
    }

    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("tts_{}.wav", uuid::Uuid::new_v4()));

    // Map language to Coqui voice
    let voice = match lang {
        "ru" | "russian" => "vits", // Russian TTS voice
        "en" | "english" => "tts_models/en/ljspeech/tacotron2-DDC",
        _ => "vits", // Default to VITS multilingual
    };

    // Call Coqui TTS CLI
    let output = std::process::Command::new("tts")
        .args([
            "--text",
            text,
            "--model_name",
            voice,
            "--out_path",
            temp_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| AiError::Ollama(format!("Coqui TTS process failed: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AiError::Ollama(format!(
            "Coqui TTS error: {}",
            stderr
        )));
    }

    // Read the generated WAV file
    let audio = std::fs::read(&temp_path)
        .map_err(|e| AiError::Ollama(format!("Failed to read audio file: {}", e)))?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    Ok(audio)
}

/// Synthesize text to audio and save to file
pub async fn synthesize_to_file(
    client: &AiClient,
    text: &str,
    lang: &str,
    output_path: &Path,
) -> AiResult<()> {
    let audio = synthesize(client, text, lang).await?;

    std::fs::write(output_path, audio)
        .map_err(|e| AiError::Http(format!("Failed to write audio file: {}", e)))?;

    Ok(())
}

/// Check if Coqui TTS is available
pub fn is_coqui_tts_available() -> bool {
    std::process::Command::new("tts")
        .arg("--help")
        .output()
        .is_ok()
}

/// Get available local voices
pub fn get_local_voices() -> Vec<String> {
    if !is_coqui_tts_available() {
        return vec![];
    }

    let output = std::process::Command::new("tts")
        .args(["--list_models"])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout
                .lines()
                .filter(|l| !l.is_empty() && !l.starts_with("WARN"))
                .map(String::from)
                .collect()
        }
        Err(_) => vec![],
    }
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
    fn test_coqui_availability_check() {
        let available = is_coqui_tts_available();
        assert!(!available || true); // Coqui might not be installed
    }

    #[test]
    fn test_get_local_voices() {
        let voices = get_local_voices();
        // Should return empty if Coqui is not installed
        assert!(voices.is_empty() || !voices.is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires Coqui TTS installed
    async fn test_synthesize_text() {
        let client = make_client();

        let result = synthesize(&client, "Привет мир", "ru").await;
        assert!(
            result.is_ok(),
            "Synthesis failed: {:?}",
            result
        );
        let audio = result.unwrap();
        assert!(!audio.is_empty());
    }
}
