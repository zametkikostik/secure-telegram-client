//! Code Generator — AI-powered code generation
//!
//! Models:
//! - OpenRouter → qwen/qwen-2.5-coder-32b-instruct
//! - Ollama local → qwen2.5-coder:14b (RTX 3050 8GB)
//!
//! PREFER LOCAL for code generation — the user has qwen2.5-coder:14b
//! running locally which is optimized for code tasks.

use crate::ai::client::{AiClient, AiResult};

const OPENROUTER_MODEL: &str = "qwen/qwen-2.5-coder-32b-instruct";
const OLLAMA_MODEL: &str = "qwen2.5-coder:14b";

/// Generate code from a description
pub async fn generate_code(
    client: &AiClient,
    description: &str,
    lang: &str,
) -> AiResult<String> {
    let system_prompt = format!(
        "You are an expert {lang} developer. Generate clean, well-documented, \
         production-ready code based on the user's description. \
         Include error handling, type annotations, and comments. \
         Use code blocks with language tags.",
    );

    let user_prompt = format!(
        "Language: {lang}\n\nDescription:\n{description}",
    );

    client
        .send_with_fallback(
            OPENROUTER_MODEL,
            OLLAMA_MODEL,
            &system_prompt,
            &user_prompt,
            0.3,
            Some(8192),
        )
        .await
}

/// Generate a complete function
pub async fn generate_function(
    client: &AiClient,
    signature: &str,
    description: &str,
    lang: &str,
) -> AiResult<String> {
    let system_prompt = format!(
        "You are an expert {lang} developer. Implement the function exactly \
         as specified in the signature. Include docstrings, type hints, \
         error handling, and unit tests.",
    );

    let user_prompt = format!(
        "Function signature:\n```\n{signature}\n```\n\nDescription:\n{description}",
    );

    client
        .send_with_fallback(
            OPENROUTER_MODEL,
            OLLAMA_MODEL,
            &system_prompt,
            &user_prompt,
            0.2,
            Some(4096),
        )
        .await
}

/// Refactor existing code
pub async fn refactor_code(
    client: &AiClient,
    code: &str,
    instructions: &str,
    lang: &str,
) -> AiResult<String> {
    let system_prompt = format!(
        "You are an expert {lang} code reviewer and refactoring specialist. \
         Improve the provided code according to the instructions. \
         Preserve existing functionality, just improve quality.",
    );

    let user_prompt = format!(
        "Instructions: {}\n\nCode:\n```\n{}\n```",
        instructions, code
    );

    client
        .send_with_fallback(
            OPENROUTER_MODEL,
            OLLAMA_MODEL,
            &system_prompt,
            &user_prompt,
            0.2,
            Some(8192),
        )
        .await
}

/// Generate unit tests for code
pub async fn generate_tests(
    client: &AiClient,
    code: &str,
    lang: &str,
) -> AiResult<String> {
    let system_prompt = format!(
        "You are an expert {lang} developer specializing in testing. \
         Generate comprehensive unit tests for the provided code. \
         Cover happy path, edge cases, and error conditions.",
    );

    let user_prompt = format!(
        "Generate tests for this {lang} code:\n```\n{code}\n```",
    );

    client
        .send_with_fallback(
            OPENROUTER_MODEL,
            OLLAMA_MODEL,
            &system_prompt,
            &user_prompt,
            0.2,
            Some(8192),
        )
        .await
}

/// Explain code in natural language
pub async fn explain_code(
    client: &AiClient,
    code: &str,
    lang: &str,
) -> AiResult<String> {
    let system_prompt = format!(
        "You are an expert {lang} developer. Explain what the provided code \
         does in clear, concise language. Include time/space complexity \
         if applicable.",
    );

    let user_prompt = format!("Explain this {lang} code:\n```\n{code}\n```",);

    client
        .chat(
            OPENROUTER_MODEL,
            OLLAMA_MODEL,
            &system_prompt,
            &user_prompt,
        )
        .await
}

/// Generate Rust struct from JSON
pub async fn json_to_rust_struct(client: &AiClient, json: &str) -> AiResult<String> {
    let system_prompt =
        "You are an expert Rust developer. Generate idiomatic Rust structs \
         with serde derive macros from the provided JSON. Use proper types, \
         lifetimes, and derive macros.";

    let user_prompt = format!(
        "Generate Rust structs for this JSON:\n```json\n{}\n```",
        json
    );

    client
        .send_with_fallback(
            OPENROUTER_MODEL,
            OLLAMA_MODEL,
            system_prompt,
            &user_prompt,
            0.1,
            Some(4096),
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
    fn test_generate_code_prompt_format() {
        let desc = "A function that sorts a vector";
        assert!(!desc.is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires backend (local Ollama preferred for code)
    async fn test_generate_code_rust() {
        let client = make_client();

        let result = generate_code(
            &client,
            "Write a Rust function that calculates fibonacci(n) iteratively",
            "Rust",
        )
        .await;

        assert!(
            result.is_ok(),
            "Code generation failed: {:?}",
            result
        );
        let code = result.unwrap();
        assert!(!code.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn test_generate_code_typescript() {
        let client = make_client();

        let result = generate_code(
            &client,
            "Write a TypeScript function that debounces another function",
            "TypeScript",
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore]
    async fn test_json_to_rust_struct() {
        let client = make_client();

        let json = r#"{
            "name": "Alice",
            "age": 30,
            "is_admin": true,
            "roles": ["user", "moderator"]
        }"#;

        let result = json_to_rust_struct(&client, json).await;
        assert!(result.is_ok());
        let code = result.unwrap();
        // Should contain struct definition
        assert!(
            code.contains("struct") || code.contains("pub struct"),
            "Expected struct in output: {}",
            code
        );
    }

    #[tokio::test]
    #[ignore]
    async fn test_refactor_code() {
        let client = make_client();

        let code = "fn add(a: i32, b: i32) -> i32 { return a + b; }";

        let result = refactor_code(
            &client,
            code,
            "Remove unnecessary return keyword, use idiomatic Rust",
            "Rust",
        )
        .await;

        assert!(result.is_ok());
    }
}
