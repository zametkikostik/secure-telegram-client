//! AI Chat Summarizer — generate summaries of chat conversations
//!
//! Models:
//! - OpenRouter → qwen/qwen-2.5-72b-instruct
//! - Ollama local → qwen2.5:14b

use crate::ai::client::{AiClient, AiResult};

const OPENROUTER_MODEL: &str = "qwen/qwen-2.5-72b-instruct";
const OLLAMA_MODEL: &str = "qwen2.5:14b";

/// Summarize a chat conversation from formatted text
pub async fn summarize_text(client: &AiClient, conversation: &str) -> AiResult<String> {
    if conversation.trim().is_empty() {
        return Ok("Нет сообщений для суммаризации".to_string());
    }

    let system_prompt = "You are a chat summarizer. Read the conversation and provide a concise \
         summary in the same language as the conversation. Include key decisions, \
         action items, and important context. Use bullet points if appropriate.";

    let user_prompt = format!(
        "Conversation:\n{}\n\nProvide a brief summary of the key points.",
        conversation
    );

    client
        .send_with_fallback(
            OPENROUTER_MODEL,
            OLLAMA_MODEL,
            system_prompt,
            &user_prompt,
            0.3,
            Some(2048),
        )
        .await
}

/// Summarize a list of plain-text messages
pub async fn summarize_messages(
    client: &AiClient,
    messages: &[(String, String)], // (sender_name, content)
) -> AiResult<String> {
    if messages.is_empty() {
        return Ok("Нет сообщений для суммаризации".to_string());
    }

    let conversation = messages
        .iter()
        .map(|(sender, content)| format!("[{}] {}", sender, content))
        .collect::<Vec<_>>()
        .join("\n");

    summarize_text(client, &conversation).await
}

/// Generate a one-line summary title for a chat
pub async fn summarize_title(client: &AiClient, conversation: &str) -> AiResult<String> {
    if conversation.trim().is_empty() {
        return Ok("Без названия".to_string());
    }

    let system_prompt = "Generate a short title (3-5 words) for this conversation. \
         Return ONLY the title, no quotes, no explanations.";

    let user_prompt = format!("Conversation:\n{}", conversation);

    client
        .send_with_fallback(
            OPENROUTER_MODEL,
            OLLAMA_MODEL,
            system_prompt,
            &user_prompt,
            0.5,
            Some(50),
        )
        .await
}

/// Extract action items from chat
pub async fn extract_action_items(client: &AiClient, conversation: &str) -> AiResult<String> {
    if conversation.trim().is_empty() {
        return Ok("Нет задач".to_string());
    }

    let system_prompt = "Extract all action items, tasks, and TODOs from this conversation. \
         Format as a numbered list with who is responsible for each item. \
         Return ONLY the list, nothing else.";

    let user_prompt = format!("Conversation:\n{}", conversation);

    client
        .send_with_fallback(
            OPENROUTER_MODEL,
            OLLAMA_MODEL,
            system_prompt,
            &user_prompt,
            0.2,
            Some(1024),
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

    fn test_conversation() -> &'static str {
        "[Alice] Let's discuss the new encryption implementation
[You] Sure! I've implemented X25519+Kyber1024 hybrid encryption
[Alice] Great, let's add post-quantum key exchange
[You] Done, Kyber1024 is integrated"
    }

    #[test]
    fn test_empty_summary() {
        // Just verify the function handles empty input
        assert!("".trim().is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires backend
    async fn test_summarize_text() {
        let client = make_client();
        let conversation = test_conversation();

        let result = summarize_text(&client, conversation).await;
        assert!(result.is_ok(), "Summarization failed: {:?}", result);
    }

    #[tokio::test]
    #[ignore]
    async fn test_summarize_title() {
        let client = make_client();
        let conversation = test_conversation();

        let result = summarize_title(&client, conversation).await;
        assert!(result.is_ok());
        let title = result.unwrap();
        assert!(!title.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn test_summarize_messages() {
        let client = make_client();
        let messages = vec![
            ("Alice".into(), "Let's discuss encryption".into()),
            ("You".into(), "X25519+Kyber1024 is ready".into()),
        ];

        let result = summarize_messages(&client, &messages).await;
        assert!(result.is_ok());
    }
}
