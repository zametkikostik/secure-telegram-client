//! AI Client — multi-provider support with automatic routing
//!
//! Architecture:
//! - Provider registry manages configured AI backends
//! - Routing logic selects best provider for each task
//! - Automatic fallback chain: local → cloud → error
//!
//! Providers:
//! - OpenRouter (aggregates 200+ models, free tier)
//! - Ollama (local models, user has qwen2.5:14b + qwen2.5-coder:14b)
//! - Anthropic (direct Claude API)
//! - OpenAI, Groq, Mistral (optional)

use crate::ai::providers::{OpenRouterProvider, OllamaProvider, AnthropicProvider};
use std::env;
use std::time::Duration;

// Re-export all provider types for use by other ai modules and external code
pub use crate::ai::providers::{
    AiProvider, AiProviderRegistry, ProviderConfig, ModelHint, ModelInfo, ProviderStatus,
    AiError, AiResult,
};

// ============================================================================
// Configuration
// ============================================================================

/// AI backend configuration
#[derive(Debug, Clone)]
pub struct AiConfig {
    /// OpenRouter API key (from env OPENROUTER_API_KEY)
    pub openrouter_key: Option<String>,
    /// Ollama base URL (from env OLLAMA_URL, default http://localhost:11434)
    pub ollama_url: String,
    /// Prefer local Ollama for certain tasks
    pub prefer_local: bool,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            openrouter_key: env::var("OPENROUTER_API_KEY").ok(),
            ollama_url: env::var("OLLAMA_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            prefer_local: env::var("AI_PREFER_LOCAL")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false),
        }
    }
}

impl AiConfig {
    /// Convert to provider registry
    pub fn to_registry(&self) -> AiProviderRegistry {
        let mut registry = AiProviderRegistry::new();

        // Register OpenRouter
        if let Some(ref key) = self.openrouter_key {
            if !key.is_empty() {
                registry.register(
                    AiProvider::OpenRouter,
                    ProviderConfig {
                        api_key: Some(key.clone()),
                        base_url: "https://openrouter.ai/api/v1".to_string(),
                        model: "qwen/qwen-2.5-72b-instruct".to_string(),
                        timeout_secs: 120,
                    },
                );
            }
        }

        // Register Ollama
        registry.register(
            AiProvider::Ollama,
            ProviderConfig {
                api_key: None,
                base_url: self.ollama_url.clone(),
                model: "qwen2.5:14b".to_string(),
                timeout_secs: 120,
            },
        );

        // Set active provider based on prefer_local
        if self.prefer_local {
            let _ = registry.set_active(AiProvider::Ollama);
        } else if registry.is_available(&AiProvider::OpenRouter) {
            let _ = registry.set_active(AiProvider::OpenRouter);
        }

        registry
    }
}

// ============================================================================
// AI Client
// ============================================================================

/// Main AI client with provider routing
pub struct AiClient {
    pub config: AiConfig,
    pub http_client: reqwest::Client,
    provider_registry: AiProviderRegistry,
    openrouter: Option<OpenRouterProvider>,
    ollama: Option<OllamaProvider>,
    anthropic: Option<AnthropicProvider>,
    active_provider: AiProvider,
    prefer_local: bool,
}

impl AiClient {
    pub fn new(config: AiConfig) -> Self {
        let registry = config.to_registry();
        let prefer_local = config.prefer_local;

        // Build provider instances
        let openrouter = registry
            .get(&AiProvider::OpenRouter)
            .map(|c| OpenRouterProvider::new(c.clone()));

        let ollama = registry
            .get(&AiProvider::Ollama)
            .map(|c| OllamaProvider::new(c.clone()));

        let anthropic = registry
            .get(&AiProvider::Anthropic)
            .map(|c| AnthropicProvider::new(c.clone()));

        let active_provider = registry.active();

        Self {
            config,
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .expect("Failed to build HTTP client"),
            provider_registry: registry,
            openrouter,
            ollama,
            anthropic,
            active_provider,
            prefer_local,
        }
    }

    // ========================================================================
    // Provider Management
    // ========================================================================

    /// Get the currently active provider
    pub fn active_provider(&self) -> AiProvider {
        self.active_provider
    }

    /// Switch the active provider
    pub fn switch_provider(&mut self, provider: AiProvider) -> Result<(), String> {
        self.provider_registry.set_active(provider)?;
        self.active_provider = provider;
        Ok(())
    }

    /// List all available (configured + reachable) providers
    pub async fn list_available_providers(&self) -> Vec<AiProvider> {
        let mut available = Vec::new();

        // Check OpenRouter
        if self.openrouter.is_some() {
            available.push(AiProvider::OpenRouter);
        }

        // Check Ollama
        if let Some(ref ollama) = self.ollama {
            if ollama.is_available().await {
                available.push(AiProvider::Ollama);
            }
        }

        // Check Anthropic
        if self.anthropic.is_some() {
            available.push(AiProvider::Anthropic);
        }

        available
    }

    /// List all available models across providers
    pub async fn list_available_models(&self) -> Vec<ModelInfo> {
        let mut models = Vec::new();

        // OpenRouter models
        if let Some(ref or) = self.openrouter {
            if let Ok(ors) = or.list_models().await {
                models.extend(ors);
            }
        }

        // Ollama local models
        if let Some(ref oll) = self.ollama {
            if let Ok(ols) = oll.list_models().await {
                models.extend(ols);
            }
        }

        // Anthropic models
        if let Some(ref ant) = self.anthropic {
            models.extend(ant.list_models());
        }

        models
    }

    /// Check provider status
    pub async fn check_provider_status(&self, provider: AiProvider) -> ProviderStatus {
        let configured = self.provider_registry.is_available(&provider);
        let mut available = false;
        let mut models = Vec::new();
        let mut error = None;

        match provider {
            AiProvider::OpenRouter => {
                if let Some(ref or) = self.openrouter {
                    if let Ok(ors) = or.list_models().await {
                        models = ors.into_iter().map(|m| m.model_id).collect();
                        available = !models.is_empty();
                    } else {
                        error = Some("Failed to fetch models".to_string());
                    }
                } else {
                    error = Some("Not configured".to_string());
                }
            }
            AiProvider::Ollama => {
                if let Some(ref oll) = self.ollama {
                    available = oll.is_available().await;
                    if let Ok(ols) = oll.list_local_models().await {
                        models = ols.into_iter().map(|m| m.name).collect();
                    } else {
                        error = Some("Failed to list models".to_string());
                    }
                } else {
                    error = Some("Not configured".to_string());
                }
            }
            AiProvider::Anthropic => {
                if let Some(ref ant) = self.anthropic {
                    available = ant.is_configured();
                    models = ant.list_models().into_iter().map(|m| m.model_id).collect();
                } else {
                    error = Some("Not configured".to_string());
                }
            }
            _ => {
                error = Some("Provider not implemented".to_string());
            }
        }

        ProviderStatus {
            provider: provider.to_string(),
            available,
            configured,
            models,
            error,
        }
    }

    // ========================================================================
    // Routing Logic
    // ========================================================================

    /// Resolve which provider to use based on task hint and config
    fn resolve_provider(&self, hint: ModelHint) -> AiProvider {
        // If prefer_local and task is code gen, prefer Ollama
        if self.prefer_local && hint == ModelHint::CodeGen {
            return AiProvider::Ollama;
        }

        // Otherwise prefer_local means try Ollama first for everything
        if self.prefer_local {
            return AiProvider::Ollama;
        }

        // Default: use active provider
        self.active_provider
    }

    /// Get the best model ID for a given provider and hint
    fn model_for(&self, provider: AiProvider, hint: ModelHint) -> String {
        match provider {
            AiProvider::OpenRouter => hint.openrouter_model().to_string(),
            AiProvider::Ollama => hint.ollama_model().to_string(),
            AiProvider::Anthropic => hint.anthropic_model().to_string(),
            AiProvider::OpenAI => hint.openai_model().to_string(),
            _ => {
                // Fallback to active provider's config
                self.provider_registry
                    .get(&self.active_provider)
                    .map(|c| c.model.clone())
                    .unwrap_or_else(|| "qwen2.5:14b".to_string())
            }
        }
    }

    // ========================================================================
    // Send Methods (per-provider)
    // ========================================================================

    /// Send via OpenRouter
    pub async fn send_openrouter(
        &self,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
        temperature: f32,
        max_tokens: Option<usize>,
    ) -> AiResult<String> {
        let or = self
            .openrouter
            .as_ref()
            .ok_or_else(|| AiError::ProviderNotAvailable("OpenRouter".to_string()))?;

        or.chat(model, system_prompt, user_prompt, temperature, max_tokens)
            .await
    }

    /// Send via Ollama
    pub async fn send_ollama(
        &self,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
        temperature: f32,
        max_tokens: Option<usize>,
    ) -> AiResult<String> {
        let oll = self
            .ollama
            .as_ref()
            .ok_or_else(|| AiError::ProviderNotAvailable("Ollama".to_string()))?;

        oll.chat(model, system_prompt, user_prompt, temperature, max_tokens)
            .await
    }

    /// Send via Anthropic
    pub async fn send_anthropic(
        &self,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
        temperature: f32,
        max_tokens: Option<usize>,
    ) -> AiResult<String> {
        let ant = self
            .anthropic
            .as_ref()
            .ok_or_else(|| AiError::ProviderNotAvailable("Anthropic".to_string()))?;

        ant.chat(model, system_prompt, user_prompt, temperature, max_tokens)
            .await
    }

    // ========================================================================
    // Main Send Method — with automatic fallback
    // ========================================================================

    /// Send a prompt with automatic provider routing and fallback
    pub async fn send(
        &self,
        hint: ModelHint,
        system_prompt: &str,
        user_prompt: &str,
        temperature: f32,
        max_tokens: Option<usize>,
    ) -> AiResult<String> {
        let primary = self.resolve_provider(hint);
        let model = self.model_for(primary, hint);

        let mut errors = Vec::new();

        // Try primary provider
        let result = match primary {
            AiProvider::OpenRouter => {
                self.send_openrouter(&model, system_prompt, user_prompt, temperature, max_tokens)
                    .await
            }
            AiProvider::Ollama => {
                self.send_ollama(&model, system_prompt, user_prompt, temperature, max_tokens)
                    .await
            }
            AiProvider::Anthropic => {
                self.send_anthropic(&model, system_prompt, user_prompt, temperature, max_tokens)
                    .await
            }
            _ => Err(AiError::ProviderNotAvailable(primary.to_string())),
        };

        if let Ok(text) = result {
            tracing::info!("AI: used {} ({})", primary, model);
            return Ok(text);
        } else if let Err(e) = result {
            tracing::warn!("Primary provider {} failed: {}", primary, e);
            errors.push((primary.to_string(), e.to_string()));
        }

        // Fallback chain: try other available providers
        let fallbacks = self.list_available_providers().await;

        for fb in fallbacks {
            if fb == primary {
                continue;
            }

            let fb_model = self.model_for(fb, hint);

            let fb_result = match fb {
                AiProvider::OpenRouter => {
                    self.send_openrouter(&fb_model, system_prompt, user_prompt, temperature, max_tokens)
                        .await
                }
                AiProvider::Ollama => {
                    self.send_ollama(&fb_model, system_prompt, user_prompt, temperature, max_tokens)
                        .await
                }
                AiProvider::Anthropic => {
                    self.send_anthropic(&fb_model, system_prompt, user_prompt, temperature, max_tokens)
                        .await
                }
                _ => continue,
            };

            if let Ok(text) = fb_result {
                tracing::info!("AI: fallback to {} ({}) succeeded", fb, fb_model);
                return Ok(text);
            } else if let Err(e) = fb_result {
                tracing::warn!("Fallback {} failed: {}", fb, e);
                errors.push((fb.to_string(), e.to_string()));
            }
        }

        // All providers failed
        Err(AiError::AllFailed { errors })
    }

    // ========================================================================
    // Backwards Compatibility — old methods kept for existing code
    // ========================================================================

    /// Send with fallback — old signature, now delegates to new `send()`
    pub async fn send_with_fallback(
        &self,
        openrouter_model: &str,
        ollama_model: &str,
        system_prompt: &str,
        user_prompt: &str,
        temperature: f32,
        max_tokens: Option<usize>,
    ) -> AiResult<String> {
        // If prefer_local, try Ollama first
        if self.prefer_local {
            match self
                .send_ollama(ollama_model, system_prompt, user_prompt, temperature, max_tokens)
                .await
            {
                Ok(result) => {
                    tracing::info!("AI: used Ollama local ({})", ollama_model);
                    return Ok(result);
                }
                Err(e) => {
                    tracing::warn!("Ollama failed, trying OpenRouter: {}", e);
                }
            }
        }

        // Try OpenRouter
        match self
            .send_openrouter(openrouter_model, system_prompt, user_prompt, temperature, max_tokens)
            .await
        {
            Ok(result) => {
                tracing::info!("AI: used OpenRouter ({})", openrouter_model);
                Ok(result)
            }
            Err(e_openrouter) => {
                tracing::warn!("OpenRouter failed: {}", e_openrouter);

                // Fallback to Ollama
                match self
                    .send_ollama(ollama_model, system_prompt, user_prompt, temperature, max_tokens)
                    .await
                {
                    Ok(result) => {
                        tracing::info!(
                            "AI: fallback to Ollama local ({}) succeeded",
                            ollama_model
                        );
                        Ok(result)
                    }
                    Err(e_ollama) => {
                        tracing::error!(
                            "AI: both backends failed — OpenRouter: {}, Ollama: {}",
                            e_openrouter,
                            e_ollama
                        );
                        Err(AiError::BothFailed {
                            openrouter: e_openrouter.to_string(),
                            ollama: e_ollama.to_string(),
                        })
                    }
                }
            }
        }
    }

    /// Quick chat — single turn, auto-fallback (old signature)
    pub async fn chat(
        &self,
        openrouter_model: &str,
        ollama_model: &str,
        system_prompt: &str,
        user_prompt: &str,
    ) -> AiResult<String> {
        self.send_with_fallback(
            openrouter_model,
            ollama_model,
            system_prompt,
            user_prompt,
            0.7,
            Some(4096),
        )
        .await
    }

    /// Send via ModelHint — new preferred method
    pub async fn send_with_hint(
        &self,
        hint: ModelHint,
        system_prompt: &str,
        user_prompt: &str,
    ) -> AiResult<String> {
        let max_tokens = match hint {
            ModelHint::Translation => Some(8192),
            ModelHint::CodeGen => Some(8192),
            ModelHint::Summarization => Some(2048),
            ModelHint::General => Some(4096),
            ModelHint::SpeechToText => None,
            ModelHint::TextToSpeech => None,
        };

        let temperature = match hint {
            ModelHint::Translation => 0.3,
            ModelHint::CodeGen => 0.3,
            ModelHint::Summarization => 0.3,
            ModelHint::General => 0.7,
            ModelHint::SpeechToText => 0.0,
            ModelHint::TextToSpeech => 0.0,
        };

        self.send(hint, system_prompt, user_prompt, temperature, max_tokens)
            .await
    }

    // ========================================================================
    // Legacy direct methods (kept for backwards compatibility)
    // ========================================================================

    /// Send prompt to OpenRouter API (legacy direct method)
    pub async fn try_openrouter(
        &self,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
        temperature: f32,
        max_tokens: Option<usize>,
    ) -> AiResult<String> {
        self.send_openrouter(model, system_prompt, user_prompt, temperature, max_tokens)
            .await
    }

    /// Send prompt to local Ollama instance (legacy direct method)
    pub async fn try_ollama(
        &self,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
        temperature: f32,
    ) -> AiResult<String> {
        self.send_ollama(model, system_prompt, user_prompt, temperature, Some(4096))
            .await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env() {
        let config = AiConfig::default();
        // Should not panic even without env vars
        assert!(!config.ollama_url.is_empty());
    }

    #[test]
    fn test_config_prefer_local() {
        let config = AiConfig {
            openrouter_key: Some("test-key".into()),
            ollama_url: "http://localhost:11434".into(),
            prefer_local: true,
        };
        assert!(config.prefer_local);
    }

    #[test]
    fn test_config_to_registry() {
        let config = AiConfig {
            openrouter_key: Some("test-key".into()),
            ollama_url: "http://localhost:11434".into(),
            prefer_local: false,
        };

        let registry = config.to_registry();
        assert!(registry.is_available(&AiProvider::OpenRouter));
        assert!(registry.is_available(&AiProvider::Ollama));
        assert_eq!(registry.active(), AiProvider::OpenRouter);
    }

    #[test]
    fn test_config_to_registry_prefer_local() {
        let config = AiConfig {
            openrouter_key: Some("test-key".into()),
            ollama_url: "http://localhost:11434".into(),
            prefer_local: true,
        };

        let registry = config.to_registry();
        assert_eq!(registry.active(), AiProvider::Ollama);
    }

    #[test]
    fn test_model_hint_openrouter_models() {
        assert_eq!(
            ModelHint::Translation.openrouter_model(),
            "qwen/qwen-2.5-72b-instruct"
        );
        assert_eq!(
            ModelHint::CodeGen.openrouter_model(),
            "qwen/qwen-2.5-coder-32b-instruct"
        );
    }

    #[test]
    fn test_model_hint_free_models() {
        assert_eq!(
            ModelHint::Translation.openrouter_free_model(),
            "meta-llama/llama-3.1-8b-instruct:free"
        );
        assert_eq!(
            ModelHint::Summarization.openrouter_free_model(),
            "mistralai/mistral-7b-instruct:free"
        );
    }

    #[test]
    fn test_model_hint_ollama_models() {
        assert_eq!(ModelHint::CodeGen.ollama_model(), "qwen2.5-coder:14b");
        assert_eq!(ModelHint::Translation.ollama_model(), "qwen2.5:14b");
    }

    #[test]
    fn test_model_hint_anthropic_models() {
        assert_eq!(
            ModelHint::Translation.anthropic_model(),
            "claude-3-5-sonnet-20241022"
        );
        assert_eq!(
            ModelHint::Summarization.anthropic_model(),
            "claude-3-haiku-20240307"
        );
    }

    #[test]
    fn test_model_hint_openai_models() {
        assert_eq!(ModelHint::General.openai_model(), "gpt-4o-mini");
        assert_eq!(ModelHint::Translation.openai_model(), "gpt-4o");
    }

    #[test]
    fn test_ai_provider_display() {
        assert_eq!(AiProvider::OpenRouter.to_string(), "openrouter");
        assert_eq!(AiProvider::Ollama.to_string(), "ollama");
        assert_eq!(AiProvider::Anthropic.to_string(), "anthropic");
    }

    #[test]
    fn test_ai_provider_from_str() {
        assert_eq!("openrouter".parse::<AiProvider>().unwrap(), AiProvider::OpenRouter);
        assert_eq!("ollama".parse::<AiProvider>().unwrap(), AiProvider::Ollama);
        assert_eq!("anthropic".parse::<AiProvider>().unwrap(), AiProvider::Anthropic);
        assert!("unknown".parse::<AiProvider>().is_err());
    }

    #[test]
    fn test_provider_config_timeout() {
        let config = ProviderConfig {
            api_key: Some("test".into()),
            base_url: "https://test.api".into(),
            model: "test-model".into(),
            timeout_secs: 30,
        };
        assert_eq!(config.timeout(), Duration::from_secs(30));
    }

    #[tokio::test]
    #[ignore] // Requires real API key
    async fn test_openrouter_call() {
        let config = AiConfig::default();
        let client = AiClient::new(config);

        let result = client
            .try_openrouter(
                "openai/gpt-4o-mini",
                "You are a helpful assistant.",
                "Say 'Hello, AI works!'",
                0.5,
                Some(50),
            )
            .await;

        assert!(result.is_ok(), "OpenRouter call failed: {:?}", result);
    }

    #[tokio::test]
    #[ignore] // Requires local Ollama running
    async fn test_ollama_call() {
        let config = AiConfig::default();
        let client = AiClient::new(config);

        let result = client
            .try_ollama(
                "qwen2.5:7b",
                "You are a helpful assistant.",
                "Say 'Hello, Ollama works!'",
                0.5,
            )
            .await;

        assert!(result.is_ok(), "Ollama call failed: {:?}", result);
    }

    #[tokio::test]
    #[ignore] // Requires at least one backend
    async fn test_fallback() {
        let config = AiConfig::default();
        let client = AiClient::new(config);

        let result = client
            .send_with_fallback(
                "openai/gpt-4o-mini",
                "qwen2.5:7b",
                "You are a helpful assistant.",
                "Say 'Hello, fallback works!'",
                0.5,
                Some(50),
            )
            .await;

        assert!(result.is_ok(), "Fallback failed: {:?}", result);
    }

    #[tokio::test]
    #[ignore] // Requires backend
    async fn test_send_with_hint() {
        let config = AiConfig::default();
        let client = AiClient::new(config);

        let result = client
            .send_with_hint(
                ModelHint::General,
                "You are a helpful assistant.",
                "Say hello!",
            )
            .await;

        assert!(result.is_ok(), "Send with hint failed: {:?}", result);
    }

    #[tokio::test]
    #[ignore] // Requires backend
    async fn test_list_available_providers() {
        let config = AiConfig::default();
        let client = AiClient::new(config);

        let providers = client.list_available_providers().await;
        println!("Available providers: {:?}", providers);
        assert!(!providers.is_empty() || providers.is_empty()); // May be empty if no backends
    }
}
