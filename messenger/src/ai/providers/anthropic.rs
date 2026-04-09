//! Anthropic Provider — direct Claude API access
//!
//! API: https://api.anthropic.com/v1
//! Models:
//! - claude-3-5-sonnet-20241022 (flagship, best for complex tasks)
//! - claude-3-haiku-20240307 (fast, cheap, good for summarization)
//!
//! Required headers:
//! - x-api-key: $ANTHROPIC_API_KEY
//! - anthropic-version: 2023-06-01
//! - content-type: application/json

use super::{AiError, AiResult, ModelInfo, AiProvider, ProviderConfig};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ============================================================================
// Anthropic Request/Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub max_tokens: usize,
    pub system: String,
    pub messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub role: String,
    pub model: String,
    pub content: Vec<ContentBlock>,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: Option<AnthropicUsage>,
    pub error: Option<AnthropicError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicUsage {
    pub input_tokens: usize,
    pub output_tokens: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

// ============================================================================
// Models
// ============================================================================

pub const CLAUDE_35_SONNET: &str = "claude-3-5-sonnet-20241022";
pub const CLAUDE_3_HAIKU: &str = "claude-3-haiku-20240307";
pub const ANTHROPIC_VERSION: &str = "2023-06-01";

// ============================================================================
// Anthropic Provider
// ============================================================================

pub struct AnthropicProvider {
    pub config: ProviderConfig,
    pub http_client: Client,
}

impl AnthropicProvider {
    pub fn new(config: ProviderConfig) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to build HTTP client for Anthropic");

        Self {
            config,
            http_client,
        }
    }

    /// Check if Anthropic is properly configured
    pub fn is_configured(&self) -> bool {
        self.config
            .api_key
            .as_ref()
            .map(|k| !k.is_empty())
            .unwrap_or(false)
    }

    /// Send a message to Claude
    pub async fn chat(
        &self,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
        temperature: f32,
        max_tokens: Option<usize>,
    ) -> AiResult<String> {
        if !self.is_configured() {
            return Err(AiError::NoApiKey("Anthropic".to_string()));
        }

        let max_tokens = max_tokens.unwrap_or(4096);

        let request = AnthropicRequest {
            model: model.to_string(),
            max_tokens,
            system: system_prompt.to_string(),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            }],
            temperature: Some(temperature),
        };

        let api_key = self.config.api_key.as_ref().unwrap();

        let response = self
            .http_client
            .post(format!("{}/messages", self.config.base_url))
            .header("x-api-key", api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AiError::Http(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AiError::ProviderError(
                "Anthropic".to_string(),
                format!("HTTP {}: {}", status, body),
            ));
        }

        let data: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| AiError::ProviderError(
                "Anthropic".to_string(),
                format!("JSON parse: {}", e),
            ))?;

        if let Some(err) = data.error {
            return Err(AiError::ProviderError(
                "Anthropic".to_string(),
                format!("{}: {}", err.error_type, err.message),
            ));
        }

        let content = data
            .content
            .first()
            .map(|b| b.text.clone())
            .ok_or_else(|| AiError::ProviderError(
                "Anthropic".to_string(),
                "No content blocks in response".into(),
            ))?;

        Ok(content.trim().to_string())
    }

    /// List available Anthropic models
    pub fn list_models(&self) -> Vec<ModelInfo> {
        vec![
            ModelInfo {
                provider: AiProvider::Anthropic,
                model_id: CLAUDE_35_SONNET.to_string(),
                name: "Claude 3.5 Sonnet".to_string(),
                is_free: false,
                context_length: Some(200_000),
                max_tokens: Some(8192),
            },
            ModelInfo {
                provider: AiProvider::Anthropic,
                model_id: CLAUDE_3_HAIKU.to_string(),
                name: "Claude 3 Haiku".to_string(),
                is_free: false,
                context_length: Some(200_000),
                max_tokens: Some(4096),
            },
        ]
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use super::ProviderConfig;

    fn make_config() -> ProviderConfig {
        ProviderConfig {
            api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            base_url: "https://api.anthropic.com/v1".to_string(),
            model: CLAUDE_35_SONNET.to_string(),
            timeout_secs: 120,
        }
    }

    #[test]
    fn test_is_configured_with_key() {
        let config = ProviderConfig {
            api_key: Some("sk-ant-test-key".to_string()),
            base_url: "https://api.anthropic.com/v1".to_string(),
            model: CLAUDE_35_SONNET.to_string(),
            timeout_secs: 120,
        };
        let provider = AnthropicProvider::new(config);
        assert!(provider.is_configured());
    }

    #[test]
    fn test_is_configured_without_key() {
        let config = ProviderConfig {
            api_key: None,
            base_url: "https://api.anthropic.com/v1".to_string(),
            model: CLAUDE_35_SONNET.to_string(),
            timeout_secs: 120,
        };
        let provider = AnthropicProvider::new(config);
        assert!(!provider.is_configured());
    }

    #[test]
    fn test_anthropic_request_serialization() {
        let request = AnthropicRequest {
            model: CLAUDE_35_SONNET.to_string(),
            max_tokens: 4096,
            system: "You are helpful".to_string(),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
            }],
            temperature: Some(0.7),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\":\"claude-3-5-sonnet-20241022\""));
        assert!(json.contains("\"max_tokens\":4096"));
        assert!(json.contains("\"role\":\"user\""));
    }

    #[test]
    fn test_list_models() {
        let config = make_config();
        let provider = AnthropicProvider::new(config);
        let models = provider.list_models();

        assert_eq!(models.len(), 2);
        assert!(models.iter().any(|m| m.model_id == CLAUDE_35_SONNET));
        assert!(models.iter().any(|m| m.model_id == CLAUDE_3_HAIKU));
    }

    #[tokio::test]
    #[ignore] // Requires ANTHROPIC_API_KEY
    async fn test_anthropic_chat() {
        let config = make_config();
        if !config.api_key.as_ref().map(|k| !k.is_empty()).unwrap_or(false) {
            println!("Skipping: no ANTHROPIC_API_KEY set");
            return;
        }

        let provider = AnthropicProvider::new(config);

        let result = provider
            .chat(
                CLAUDE_3_HAIKU,
                "You are a helpful assistant.",
                "Say 'Hello, Claude works!' in one sentence.",
                0.5,
                Some(50),
            )
            .await;

        assert!(result.is_ok(), "Anthropic chat failed: {:?}", result);
        let text = result.unwrap();
        assert!(!text.is_empty(), "Response was empty");
        println!("Anthropic response: {}", text);
    }
}
