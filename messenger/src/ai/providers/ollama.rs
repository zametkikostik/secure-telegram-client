//! Ollama Provider — local models running on user's machine
//!
//! API: http://localhost:11434
//! User has (RTX 3050 8GB):
//! - qwen2.5:14b (translation, summarization)
//! - qwen2.5-coder:14b (code generation)
//!
//! Endpoints:
//! - GET /api/tags — list available models
//! - POST /api/chat — chat completion (preferred)
//! - POST /api/generate — text generation (legacy)

use super::{AiError, AiResult, ModelInfo, AiProvider, ProviderConfig};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ============================================================================
// Ollama Request/Response Types
// ============================================================================

/// Chat completion request (new-style API)
#[derive(Debug, Clone, Serialize)]
pub struct OllamaChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OllamaOptions>,
}

/// Legacy generate request
#[derive(Debug, Clone, Serialize)]
pub struct OllamaGenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<OllamaOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<usize>,
}

/// Chat completion response (streaming: one per chunk, last has done=true)
#[derive(Debug, Clone, Deserialize)]
pub struct OllamaChatResponse {
    pub model: String,
    pub message: Option<ChatMessage>,
    pub done: bool,
    pub total_duration: Option<u64>,
    pub eval_count: Option<usize>,
    pub done_reason: Option<String>,
}

/// Generate response
#[derive(Debug, Clone, Deserialize)]
pub struct OllamaGenerateResponse {
    pub model: String,
    pub response: String,
    pub done: bool,
    pub total_duration: Option<u64>,
    pub eval_count: Option<usize>,
}

/// Tags response from /api/tags
#[derive(Debug, Clone, Deserialize)]
pub struct OllamaTagsResponse {
    pub models: Vec<OllamaModelTag>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OllamaModelTag {
    pub name: String,
    pub model: String,
    pub modified_at: Option<String>,
    pub size: Option<u64>,
    pub digest: Option<String>,
}

// ============================================================================
// Ollama Provider
// ============================================================================

pub struct OllamaProvider {
    pub config: ProviderConfig,
    pub http_client: Client,
}

impl OllamaProvider {
    pub fn new(config: ProviderConfig) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to build HTTP client for Ollama");

        Self {
            config,
            http_client,
        }
    }

    /// Check if Ollama is reachable
    pub async fn is_available(&self) -> bool {
        self.http_client
            .get(&self.config.base_url)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }

    /// Check if a specific model is available locally
    pub async fn has_model(&self, model_name: &str) -> bool {
        match self.list_local_models().await {
            Ok(models) => models.iter().any(|m| {
                m.name == model_name
                    || m.name.starts_with(&format!("{}:", model_name))
                    || m.model == model_name
            }),
            Err(_) => false,
        }
    }

    /// List locally available models
    pub async fn list_local_models(&self) -> AiResult<Vec<OllamaModelTag>> {
        let response = self
            .http_client
            .get(format!("{}/api/tags", self.config.base_url))
            .send()
            .await
            .map_err(|e| AiError::ProviderError(
                "Ollama".to_string(),
                format!("Failed to connect: {}", e),
            ))?;

        if !response.status().is_success() {
            return Err(AiError::ProviderError(
                "Ollama".to_string(),
                format!("HTTP {}", response.status()),
            ));
        }

        let data: OllamaTagsResponse = response
            .json()
            .await
            .map_err(|e| AiError::ProviderError(
                "Ollama".to_string(),
                format!("JSON parse: {}", e),
            ))?;

        Ok(data.models)
    }

    /// List models in ModelInfo format
    pub async fn list_models(&self) -> AiResult<Vec<ModelInfo>> {
        let models = self.list_local_models().await?;
        Ok(models
            .into_iter()
            .map(|m| ModelInfo {
                provider: AiProvider::Ollama,
                model_id: m.name.clone(),
                name: m.name.clone(),
                is_free: true, // All local models are "free"
                context_length: None,
                max_tokens: None,
            })
            .collect())
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
        let request = OllamaChatRequest {
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
            stream: false,
            options: Some(OllamaOptions {
                temperature: Some(temperature),
                num_predict: max_tokens,
            }),
        };

        let response = self
            .http_client
            .post(format!("{}/api/chat", self.config.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                AiError::ProviderError(
                    "Ollama".to_string(),
                    format!(
                        "Failed to connect to Ollama at {}. Is Ollama running? ({})",
                        self.config.base_url, e
                    ),
                )
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AiError::ProviderError(
                "Ollama".to_string(),
                format!("HTTP {}: {}", status, body),
            ));
        }

        let data: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| AiError::ProviderError(
                "Ollama".to_string(),
                format!("JSON parse: {}", e),
            ))?;

        let content = data
            .message
            .as_ref()
            .map(|m| m.content.clone())
            .ok_or_else(|| AiError::ProviderError(
                "Ollama".to_string(),
                "No message in response".into(),
            ))?;

        Ok(content.trim().to_string())
    }

    /// Send a generate request (legacy API)
    pub async fn generate(
        &self,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
        temperature: f32,
        max_tokens: Option<usize>,
    ) -> AiResult<String> {
        let prompt = if system_prompt.is_empty() {
            user_prompt.to_string()
        } else {
            format!("{}\n\n{}", system_prompt, user_prompt)
        };

        let request = OllamaGenerateRequest {
            model: model.to_string(),
            prompt,
            stream: false,
            system: if system_prompt.is_empty() {
                None
            } else {
                Some(system_prompt.to_string())
            },
            options: Some(OllamaOptions {
                temperature: Some(temperature),
                num_predict: max_tokens,
            }),
        };

        let response = self
            .http_client
            .post(format!("{}/api/generate", self.config.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                AiError::ProviderError(
                    "Ollama".to_string(),
                    format!(
                        "Failed to connect to Ollama at {}. Is Ollama running? ({})",
                        self.config.base_url, e
                    ),
                )
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AiError::ProviderError(
                "Ollama".to_string(),
                format!("HTTP {}: {}", status, body),
            ));
        }

        let data: OllamaGenerateResponse = response
            .json()
            .await
            .map_err(|e| AiError::ProviderError(
                "Ollama".to_string(),
                format!("JSON parse: {}", e),
            ))?;

        if data.response.is_empty() {
            return Err(AiError::ProviderError(
                "Ollama".to_string(),
                "Empty response from Ollama".into(),
            ));
        }

        Ok(data.response.trim().to_string())
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
            api_key: None,
            base_url: "http://localhost:11434".to_string(),
            model: "qwen2.5:14b".to_string(),
            timeout_secs: 120,
        }
    }

    #[tokio::test]
    #[ignore] // Requires Ollama running locally
    async fn test_ollama_availability_check() {
        let config = make_config();
        let provider = OllamaProvider::new(config);

        let available = provider.is_available().await;
        if !available {
            println!("Ollama is not running at localhost:11434");
            return;
        }

        let models = provider.list_local_models().await;
        assert!(models.is_ok(), "Failed to list models: {:?}", models);

        let models = models.unwrap();
        println!("Local Ollama models:");
        for m in &models {
            println!("  - {} (size: {:.1} GB)", m.name, (m.size.unwrap_or(0) as f64) / 1e9);
        }
    }

    #[tokio::test]
    #[ignore] // Requires Ollama running locally
    async fn test_ollama_chat() {
        let config = make_config();
        let provider = OllamaProvider::new(config);

        if !provider.is_available().await {
            println!("Skipping: Ollama is not running");
            return;
        }

        let result = provider
            .chat(
                "qwen2.5:14b",
                "You are a helpful assistant.",
                "Say 'Hello, Ollama works!' in one sentence.",
                0.5,
                Some(50),
            )
            .await;

        assert!(result.is_ok(), "Ollama chat failed: {:?}", result);
        let text = result.unwrap();
        assert!(!text.is_empty(), "Response was empty");
        println!("Ollama response: {}", text);
    }

    #[tokio::test]
    #[ignore] // Requires Ollama running locally
    async fn test_ollama_has_model() {
        let config = make_config();
        let provider = OllamaProvider::new(config);

        if !provider.is_available().await {
            println!("Skipping: Ollama is not running");
            return;
        }

        // Check for common models
        let has_qwen = provider.has_model("qwen2.5").await;
        println!("Has qwen2.5: {}", has_qwen);
        // Don't assert — depends on user's local setup
    }

    #[test]
    fn test_ollama_request_serialization() {
        let request = OllamaChatRequest {
            model: "qwen2.5:14b".to_string(),
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
            stream: false,
            options: Some(OllamaOptions {
                temperature: Some(0.7),
                num_predict: Some(1024),
            }),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\":\"qwen2.5:14b\""));
        assert!(json.contains("\"stream\":false"));
        assert!(json.contains("\"temperature\":0.7"));
    }
}
