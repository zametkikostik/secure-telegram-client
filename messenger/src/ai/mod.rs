// AI Module — Multi-Provider AI System (PHASE 4.5)
//
// Architecture:
// - Provider registry manages configured AI backends
// - Routing logic selects best provider for each task
// - Automatic fallback chain: local → cloud → error
//
// Providers:
// - OpenRouter (aggregates 200+ models, free tier available)
// - Ollama (local models, user has qwen2.5:14b + qwen2.5-coder:14b on RTX 3050 8GB)
// - Anthropic (direct Claude API)
// - OpenAI, Groq, Mistral (optional, extendable)
//
// Models:
// - OpenRouter → qwen/qwen-2.5-72b-instruct (translation)
// - OpenRouter → qwen/qwen-2.5-coder-32b-instruct (code gen)
// - OpenRouter → meta-llama/llama-3.1-8b-instruct:free (free tier)
// - Ollama local → qwen2.5:14b (translation, summary)
// - Ollama local → qwen2.5-coder:14b (code generation)
// - Anthropic → claude-3-5-sonnet-20241022 (flagship)
// - Anthropic → claude-3-haiku-20240307 (fast/cheap)

pub mod client;
pub mod code_generator;
pub mod providers;
pub mod speech_to_text;
pub mod summarizer;
pub mod text_to_speech;
pub mod translator;

// Re-export main types
pub use client::{AiClient, AiConfig, AiError, AiResult};
pub use providers::{
    AiProvider, AiProviderRegistry, ModelHint, ModelInfo, ProviderConfig, ProviderStatus,
};
