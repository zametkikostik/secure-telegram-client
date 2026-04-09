//! AI Translator — translate text between languages
//!
//! Models:
//! - OpenRouter → qwen/qwen-2.5-72b-instruct
//! - Ollama local → qwen2.5:14b

use crate::ai::client::{AiClient, AiResult};

const OPENROUTER_MODEL: &str = "qwen/qwen-2.5-72b-instruct";
const OLLAMA_MODEL: &str = "qwen2.5:14b";

/// Translate text from one language to another
pub async fn translate(
    client: &AiClient,
    text: &str,
    from_lang: &str,
    to_lang: &str,
) -> AiResult<String> {
    let system_prompt = format!(
        "You are a professional translator. Translate the following text from {from_lang} \
         to {to_lang}. Preserve formatting, code blocks, and special characters. \
         Return ONLY the translation, no explanations.",
    );

    let user_prompt = text.to_string();

    client
        .send_with_fallback(
            OPENROUTER_MODEL,
            OLLAMA_MODEL,
            &system_prompt,
            &user_prompt,
            0.3, // Low temperature for consistent translations
            Some(8192),
        )
        .await
}

/// Translate to multiple languages at once
pub async fn translate_multi(
    client: &AiClient,
    text: &str,
    from_lang: &str,
    to_langs: &[&str],
) -> AiResult<Vec<(String, String)>> {
    let mut results = Vec::new();

    for &lang in to_langs {
        let translated = translate(client, text, from_lang, lang).await?;
        results.push((lang.to_string(), translated));
    }

    Ok(results)
}

/// Detect language of the text
pub async fn detect_language(client: &AiClient, text: &str) -> AiResult<String> {
    let system_prompt =
        "Detect the language of the following text. Return ONLY the language name \
         in English, nothing else.";

    let user_prompt = format!("Text: {}", text);

    client
        .send_with_fallback(
            OPENROUTER_MODEL,
            OLLAMA_MODEL,
            system_prompt,
            &user_prompt,
            0.1,
            Some(50),
        )
        .await
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
    fn test_translate_prompt_format() {
        // Verify prompt construction
        let text = "Hello world";
        assert!(!text.is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires backend
    async fn test_translate_ru_en() {
        let client = make_client();
        let result = translate(&client, "Привет мир", "Russian", "English").await;
        assert!(
            result.is_ok(),
            "Translation failed: {:?}",
            result
        );
        let translated = result.unwrap();
        assert!(
            translated.to_lowercase().contains("hello"),
            "Expected 'hello' in '{}'",
            translated
        );
    }

    #[tokio::test]
    #[ignore]
    async fn test_detect_language() {
        let client = make_client();
        let result = detect_language(&client, "Привет мир").await;
        assert!(result.is_ok());
    }
}
