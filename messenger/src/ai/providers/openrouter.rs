//! OpenRouter Provider — aggregates 200+ models including free tier
//!
//! API: https://openrouter.ai/api/v1
//! Free models (no billing needed):
//! - meta-llama/llama-3.1-8b-instruct:free
//! - mistralai/mistral-7b-instruct:free
//! - google/gemma-2-9b-it:free
//!
//! Paid models:
//! - qwen/qwen-2.5-72b-instruct
//! - qwen/qwen-2.5-coder-32b-instruct
//! - anthropic/claude-3-5-sonnet
//! - openai/gpt-4o-mini

use super::{AiError, AiProvider, AiResult, ModelInfo, ProviderConfig};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ============================================================================
// OpenRouter Request/Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct OpenRouterRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenRouterResponse {
    pub id: String,
    pub model: String,
    pub choices: Vec<Choice>,
    pub error: Option<ApiError>,
    pub usage: Option<UsageInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    pub index: usize,
    pub message: Option<ResponseMessage>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResponseMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    pub message: String,
    pub code: Option<u16>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsageInfo {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

// ============================================================================
// Models List Response
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct OpenRouterModelsResponse {
    pub data: Vec<OpenRouterModel>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenRouterModel {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub context_length: Option<usize>,
    pub architecture: Option<ModelArchitecture>,
    pub pricing: Option<PricingInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelArchitecture {
    pub modality: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PricingInfo {
    pub prompt: Option<String>,
    pub completion: Option<String>,
}

// ============================================================================
// OpenRouter Provider
// ============================================================================

pub struct OpenRouterProvider {
    pub config: ProviderConfig,
    pub http_client: Client,
}

impl OpenRouterProvider {
    pub fn new(config: ProviderConfig) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to build HTTP client for OpenRouter");

        Self {
            config,
            http_client,
        }
    }

    /// Check if OpenRouter is properly configured
    pub fn is_configured(&self) -> bool {
        self.config
            .api_key
            .as_ref()
            .map(|k| !k.is_empty())
            .unwrap_or(false)
    }

    /// Send a chat completion request
    pub async fn chat(
        &self,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
        temperature: f32,
        max_tokens: Option<usize>,
    ) -> AiResult<String> {
        if !self.is_configured() {
            return Err(AiError::NoApiKey("OpenRouter".to_string()));
        }

        let request = OpenRouterRequest {
            model: model.to_string(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                },
            ],
            temperature: Some(temperature),
            max_tokens,
            stream: Some(false),
        };

        let api_key = self.config.api_key.as_ref().unwrap();

        let response = self
            .http_client
            .post(format!("{}/chat/completions", self.config.base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://secure-messenger.app")
            .header("X-Title", "SecureMessenger")
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
                "OpenRouter".to_string(),
                format!("HTTP {}: {}", status, body),
            ));
        }

        let data: OpenRouterResponse = response.json().await.map_err(|e| {
            AiError::ProviderError("OpenRouter".to_string(), format!("JSON parse: {}", e))
        })?;

        if let Some(err) = data.error {
            return Err(AiError::ProviderError(
                "OpenRouter".to_string(),
                err.message,
            ));
        }

        let choice = data.choices.first().ok_or_else(|| {
            AiError::ProviderError("OpenRouter".to_string(), "No choices in response".into())
        })?;

        let content = choice
            .message
            .as_ref()
            .map(|m| m.content.clone())
            .ok_or_else(|| {
                AiError::ProviderError("OpenRouter".to_string(), "No content in response".into())
            })?;

        Ok(content.trim().to_string())
    }

    /// List available models from OpenRouter
    pub async fn list_models(&self) -> AiResult<Vec<ModelInfo>> {
        if !self.is_configured() {
            return Err(AiError::NoApiKey("OpenRouter".to_string()));
        }

        let api_key = self.config.api_key.as_ref().unwrap();

        let response = self
            .http_client
            .get(format!("{}/models", self.config.base_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://secure-messenger.app")
            .header("X-Title", "SecureMessenger")
            .send()
            .await
            .map_err(|e| AiError::Http(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AiError::ProviderError(
                "OpenRouter".to_string(),
                format!("Failed to fetch models: HTTP {}", response.status()),
            ));
        }

        let data: OpenRouterModelsResponse = response.json().await.map_err(|e| {
            AiError::ProviderError("OpenRouter".to_string(), format!("JSON parse: {}", e))
        })?;

        let models: Vec<ModelInfo> = data
            .data
            .into_iter()
            .map(|m| {
                let is_free = m.id.ends_with(":free")
                    || m.pricing
                        .as_ref()
                        .and_then(|p| p.prompt.as_ref())
                        .map(|p| p == "0")
                        .unwrap_or(false);

                ModelInfo {
                    provider: AiProvider::OpenRouter,
                    model_id: m.id,
                    name: m.name,
                    is_free,
                    context_length: m.context_length,
                    max_tokens: None,
                }
            })
            .collect();

        Ok(models)
    }

    /// Check if a specific model is available
    pub async fn check_model(&self, model_id: &str) -> bool {
        match self.list_models().await {
            Ok(models) => models.iter().any(|m| m.model_id == model_id),
            Err(_) => false,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use super::{AiProvider, ProviderConfig};

    fn make_config() -> ProviderConfig {
        ProviderConfig {
            api_key: std::env::var("OPENROUTER_API_KEY").ok(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            model: "meta-llama/llama-3.1-8b-instruct:free".to_string(),
            timeout_secs: 60,
        }
    }

    #[test]
    fn test_openrouter_request_serialization() {
        let request = OpenRouterRequest {
            model: "test-model".to_string(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: "You are helpful".to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                },
            ],
            temperature: Some(0.7),
            max_tokens: Some(1024),
            stream: Some(false),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\":\"test-model\""));
        assert!(json.contains("\"role\":\"system\""));
        assert!(json.contains("\"role\":\"user\""));
    }

    #[test]
    fn test_is_configured_with_key() {
        let config = ProviderConfig {
            api_key: Some("sk-or-test-key".to_string()),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            model: "test".to_string(),
            timeout_secs: 60,
        };
        let provider = OpenRouterProvider::new(config);
        assert!(provider.is_configured());
    }

    #[test]
    fn test_is_configured_without_key() {
        let config = ProviderConfig {
            api_key: None,
            base_url: "https://openrouter.ai/api/v1".to_string(),
            model: "test".to_string(),
            timeout_secs: 60,
        };
        let provider = OpenRouterProvider::new(config);
        assert!(!provider.is_configured());
    }

    #[test]
    fn test_is_configured_with_empty_key() {
        let config = ProviderConfig {
            api_key: Some("".to_string()),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            model: "test".to_string(),
            timeout_secs: 60,
        };
        let provider = OpenRouterProvider::new(config);
        assert!(!provider.is_configured());
    }

    #[tokio::test]
    #[ignore] // Requires OPENROUTER_API_KEY
    async fn test_openrouter_free_model() {
        let config = make_config();
        if !config
            .api_key
            .as_ref()
            .map(|k| !k.is_empty())
            .unwrap_or(false)
        {
            println!("Skipping: no OPENROUTER_API_KEY set");
            return;
        }

        let provider = OpenRouterProvider::new(config);

        let result = provider
            .chat(
                "meta-llama/llama-3.1-8b-instruct:free",
                "You are a helpful assistant.",
                "Say 'Hello, OpenRouter works!' in one sentence.",
                0.5,
                Some(50),
            )
            .await;

        assert!(result.is_ok(), "OpenRouter free model failed: {:?}", result);
        let text = result.unwrap();
        assert!(!text.is_empty(), "Response was empty");
        println!("OpenRouter response: {}", text);
    }

    #[tokio::test]
    #[ignore] // Requires OPENROUTER_API_KEY
    async fn test_openrouter_list_models() {
        let config = make_config();
        if !config
            .api_key
            .as_ref()
            .map(|k| !k.is_empty())
            .unwrap_or(false)
        {
            println!("Skipping: no OPENROUTER_API_KEY set");
            return;
        }

        let provider = OpenRouterProvider::new(config);
        let models = provider.list_models().await;
        assert!(models.is_ok(), "Failed to list models: {:?}", models);

        let models = models.unwrap();
        assert!(!models.is_empty(), "No models returned");

        // Check for known free models
        let free_models: Vec<_> = models.iter().filter(|m| m.is_free).collect();
        println!("Found {} free models", free_models.len());
        assert!(!free_models.is_empty(), "No free models found");
    }
}
