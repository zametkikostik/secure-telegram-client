//! AI Provider System — multi-provider support with user-switchable config
//!
//! Supported providers:
//! - OpenRouter (aggregates 200+ models, free tier available)
//! - Ollama (local models on user's machine)
//! - Anthropic (direct Claude API)
//! - OpenAI (direct GPT API)
//! - Groq (fast inference)
//! - Mistral (Mistral AI API)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::time::Duration;

pub mod anthropic;
pub mod ollama;
pub mod openrouter;

// Re-export common types
pub use anthropic::AnthropicProvider;
pub use ollama::OllamaProvider;
pub use openrouter::OpenRouterProvider;

// ============================================================================
// Provider Enum
// ============================================================================

/// Supported AI providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum AiProvider {
    #[default]
    OpenRouter,
    Ollama,
    Anthropic,
    OpenAI,
    Groq,
    Mistral,
}

impl AiProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            AiProvider::OpenRouter => "openrouter",
            AiProvider::Ollama => "ollama",
            AiProvider::Anthropic => "anthropic",
            AiProvider::OpenAI => "openai",
            AiProvider::Groq => "groq",
            AiProvider::Mistral => "mistral",
        }
    }
}

impl std::fmt::Display for AiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for AiProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openrouter" => Ok(AiProvider::OpenRouter),
            "ollama" => Ok(AiProvider::Ollama),
            "anthropic" => Ok(AiProvider::Anthropic),
            "openai" => Ok(AiProvider::OpenAI),
            "groq" => Ok(AiProvider::Groq),
            "mistral" => Ok(AiProvider::Mistral),
            _ => Err(format!("Unknown provider: {}", s)),
        }
    }
}

// ============================================================================
// Provider Configuration
// ============================================================================

/// Configuration for a single provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API key (None for local providers like Ollama)
    pub api_key: Option<String>,
    /// Base URL (e.g., https://openrouter.ai/api/v1, http://localhost:11434)
    pub base_url: String,
    /// Default model for this provider
    pub model: String,
    /// Request timeout in seconds
    pub timeout_secs: u64,
}

impl ProviderConfig {
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }
}

// ============================================================================
// Provider Registry
// ============================================================================

/// Registry of all configured providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderRegistry {
    pub providers: HashMap<AiProvider, ProviderConfig>,
    #[serde(default)]
    pub active_provider: AiProvider,
}

impl Default for AiProviderRegistry {
    fn default() -> Self {
        let mut registry = Self {
            providers: HashMap::new(),
            active_provider: AiProvider::OpenRouter,
        };

        // Auto-register providers from environment
        registry.register_from_env();
        registry
    }
}

impl AiProviderRegistry {
    /// Create empty registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            active_provider: AiProvider::OpenRouter,
        }
    }

    /// Register a provider configuration
    pub fn register(&mut self, provider: AiProvider, config: ProviderConfig) {
        self.providers.insert(provider, config);
    }

    /// Get config for a provider
    pub fn get(&self, provider: &AiProvider) -> Option<&ProviderConfig> {
        self.providers.get(provider)
    }

    /// Check if a provider is configured and available
    pub fn is_available(&self, provider: &AiProvider) -> bool {
        match provider {
            AiProvider::Ollama => {
                // Ollama is available if configured (no API key needed)
                self.providers.contains_key(provider)
            }
            AiProvider::OpenRouter
            | AiProvider::Anthropic
            | AiProvider::OpenAI
            | AiProvider::Groq
            | AiProvider::Mistral => {
                // Cloud providers need an API key
                self.providers
                    .get(provider)
                    .and_then(|c| c.api_key.as_ref())
                    .map(|k| !k.is_empty())
                    .unwrap_or(false)
            }
        }
    }

    /// List all available providers
    pub fn list_available(&self) -> Vec<AiProvider> {
        self.providers
            .keys()
            .filter(|p| self.is_available(p))
            .cloned()
            .collect()
    }

    /// Set the active provider
    pub fn set_active(&mut self, provider: AiProvider) -> Result<(), String> {
        if self.providers.contains_key(&provider) {
            self.active_provider = provider;
            Ok(())
        } else {
            Err(format!("Provider {} not registered", provider))
        }
    }

    /// Get the active provider
    pub fn active(&self) -> AiProvider {
        self.active_provider
    }

    /// Register providers from environment variables
    pub fn register_from_env(&mut self) {
        // OpenRouter
        if let Some(key) = env::var("OPENROUTER_API_KEY").ok() {
            if !key.is_empty() {
                self.register(
                    AiProvider::OpenRouter,
                    ProviderConfig {
                        api_key: Some(key),
                        base_url: "https://openrouter.ai/api/v1".to_string(),
                        model: "qwen/qwen-2.5-72b-instruct".to_string(),
                        timeout_secs: 120,
                    },
                );
            }
        }

        // Ollama
        let ollama_url =
            env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
        self.register(
            AiProvider::Ollama,
            ProviderConfig {
                api_key: None,
                base_url: ollama_url,
                model: "qwen2.5:14b".to_string(),
                timeout_secs: 120,
            },
        );

        // Anthropic
        if let Some(key) = env::var("ANTHROPIC_API_KEY").ok() {
            if !key.is_empty() {
                self.register(
                    AiProvider::Anthropic,
                    ProviderConfig {
                        api_key: Some(key),
                        base_url: "https://api.anthropic.com/v1".to_string(),
                        model: "claude-3-5-sonnet-20241022".to_string(),
                        timeout_secs: 120,
                    },
                );
            }
        }

        // OpenAI
        if let Some(key) = env::var("OPENAI_API_KEY").ok() {
            if !key.is_empty() {
                self.register(
                    AiProvider::OpenAI,
                    ProviderConfig {
                        api_key: Some(key),
                        base_url: "https://api.openai.com/v1".to_string(),
                        model: "gpt-4o-mini".to_string(),
                        timeout_secs: 60,
                    },
                );
            }
        }

        // Groq
        if let Some(key) = env::var("GROQ_API_KEY").ok() {
            if !key.is_empty() {
                self.register(
                    AiProvider::Groq,
                    ProviderConfig {
                        api_key: Some(key),
                        base_url: "https://api.groq.com/openai/v1".to_string(),
                        model: "llama-3.1-8b-instant".to_string(),
                        timeout_secs: 30,
                    },
                );
            }
        }

        // Mistral
        if let Some(key) = env::var("MISTRAL_API_KEY").ok() {
            if !key.is_empty() {
                self.register(
                    AiProvider::Mistral,
                    ProviderConfig {
                        api_key: Some(key),
                        base_url: "https://api.mistral.ai/v1".to_string(),
                        model: "mistral-large-latest".to_string(),
                        timeout_secs: 60,
                    },
                );
            }
        }

        // Set default active provider
        let default_provider = env::var("AI_DEFAULT_PROVIDER")
            .ok()
            .and_then(|s| s.parse::<AiProvider>().ok())
            .unwrap_or(AiProvider::OpenRouter);

        if self.providers.contains_key(&default_provider) {
            self.active_provider = default_provider;
        } else if self.is_available(&AiProvider::OpenRouter) {
            self.active_provider = AiProvider::OpenRouter;
        } else if self.is_available(&AiProvider::Ollama) {
            self.active_provider = AiProvider::Ollama;
        }
    }
}

// ============================================================================
// Model Hint — task-based model routing
// ============================================================================

/// Hint for what type of task the model should be good at
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelHint {
    /// Translation → qwen-2.5-72b or llama-3.1-8b:free
    Translation,
    /// Code generation → qwen-2.5-coder-32b or qwen2.5-coder:14b local
    CodeGen,
    /// Summarization → mistral-7b:free or qwen-2.5-72b
    Summarization,
    /// General chat → whatever is cheapest/fastest
    General,
    /// Speech-to-text → whisper-large-v3
    SpeechToText,
    /// Text-to-speech → local Coqui or cloud TTS
    TextToSpeech,
}

impl ModelHint {
    /// Get the best OpenRouter model for this task
    pub fn openrouter_model(&self) -> &'static str {
        match self {
            ModelHint::Translation => "qwen/qwen-2.5-72b-instruct",
            ModelHint::CodeGen => "qwen/qwen-2.5-coder-32b-instruct",
            ModelHint::Summarization => "mistralai/mistral-7b-instruct:free",
            ModelHint::General => "openai/gpt-4o-mini",
            ModelHint::SpeechToText => "openai/whisper-large-v3",
            ModelHint::TextToSpeech => "openai/tts-1",
        }
    }

    /// Get the best free OpenRouter model for this task
    pub fn openrouter_free_model(&self) -> &'static str {
        match self {
            ModelHint::Translation => "meta-llama/llama-3.1-8b-instruct:free",
            ModelHint::CodeGen => "google/gemma-2-9b-it:free",
            ModelHint::Summarization => "mistralai/mistral-7b-instruct:free",
            ModelHint::General => "meta-llama/llama-3.1-8b-instruct:free",
            ModelHint::SpeechToText => "openai/whisper-large-v3",
            ModelHint::TextToSpeech => "openai/tts-1",
        }
    }

    /// Get the best local Ollama model for this task
    pub fn ollama_model(&self) -> &'static str {
        match self {
            ModelHint::Translation => "qwen2.5:14b",
            ModelHint::CodeGen => "qwen2.5-coder:14b",
            ModelHint::Summarization => "qwen2.5:14b",
            ModelHint::General => "qwen2.5:7b",
            ModelHint::SpeechToText => "whisper-large-v3",
            ModelHint::TextToSpeech => "piper",
        }
    }

    /// Get the best Anthropic model for this task
    pub fn anthropic_model(&self) -> &'static str {
        match self {
            ModelHint::Translation => "claude-3-5-sonnet-20241022",
            ModelHint::CodeGen => "claude-3-5-sonnet-20241022",
            ModelHint::Summarization => "claude-3-haiku-20240307",
            ModelHint::General => "claude-3-5-sonnet-20241022",
            ModelHint::SpeechToText => "claude-3-5-sonnet-20241022",
            ModelHint::TextToSpeech => "claude-3-5-sonnet-20241022",
        }
    }

    /// Get the best OpenAI model for this task
    pub fn openai_model(&self) -> &'static str {
        match self {
            ModelHint::Translation => "gpt-4o",
            ModelHint::CodeGen => "gpt-4o",
            ModelHint::Summarization => "gpt-4o-mini",
            ModelHint::General => "gpt-4o-mini",
            ModelHint::SpeechToText => "whisper-1",
            ModelHint::TextToSpeech => "tts-1",
        }
    }
}

// ============================================================================
// Common Error & Result types
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("{0} API error: {1}")]
    ProviderError(String, String),

    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("No API key configured for {0}")]
    NoApiKey(String),

    #[error("All providers failed: {errors:?}")]
    AllFailed { errors: Vec<(String, String)> },

    #[error("No providers configured")]
    NoProviders,

    #[error("Task not supported: {0}")]
    NotSupported(String),

    #[error("Provider not available: {0}")]
    ProviderNotAvailable(String),

    // Backwards-compatible variants (for existing code)
    #[deprecated(since = "0.2.0", note = "Use ProviderError or NoApiKey instead")]
    #[error("OpenRouter API error: {0}")]
    OpenRouter(String),

    #[deprecated(since = "0.2.0", note = "Use ProviderError instead")]
    #[error("Ollama local error: {0}")]
    Ollama(String),

    #[deprecated(since = "0.2.0", note = "Use AllFailed instead")]
    #[error("Both backends failed: openrouter={openrouter}, ollama={ollama}")]
    BothFailed { openrouter: String, ollama: String },
}

pub type AiResult<T> = Result<T, AiError>;

// ============================================================================
// Model Info
// ============================================================================

/// Information about an available model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub provider: AiProvider,
    pub model_id: String,
    pub name: String,
    pub is_free: bool,
    pub context_length: Option<usize>,
    pub max_tokens: Option<usize>,
}

/// Provider status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub provider: String,
    pub available: bool,
    pub configured: bool,
    pub models: Vec<String>,
    pub error: Option<String>,
}
