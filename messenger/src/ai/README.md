# 🤖 AI Module

AI-powered features with **OpenRouter** cloud + **Ollama** local fallback.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                   User Request                       │
└──────────────────────┬──────────────────────────────┘
                       │
              ┌────────▼────────┐
              │   AiClient      │
              │ send_with_fallback│
              └───┬─────────┬───┘
                  │         │
          ┌───────▼──┐  ┌──▼────────┐
          │OpenRouter│  │  Ollama   │
          │  (cloud) │  │  (local)  │
          └───────┬──┘  └──┬────────┘
                  │         │
              ┌───▼─────────▼───┐
              │     Result      │
              └─────────────────┘
```

## Setup

### 1. OpenRouter (Cloud)

1. Register at https://openrouter.ai/
2. Create API key at https://openrouter.ai/keys
3. Set env: `OPENROUTER_API_KEY=sk-or-v1-...`

### 2. Ollama (Local)

```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Pull models (user has these on RTX 3050 8GB)
ollama pull qwen2.5:14b          # Translation, summary
ollama pull qwen2.5-coder:14b    # Code generation

# Set env (optional, default is http://localhost:11434)
export OLLAMA_URL=http://localhost:11434
```

### 3. Environment

```bash
cp messenger/.env.example messenger/.env
# Edit .env with your keys
```

## Models

| Feature | OpenRouter | Ollama Local |
|---------|-----------|--------------|
| Translation | `qwen/qwen-2.5-72b-instruct` | `qwen2.5:14b` |
| Summarization | `qwen/qwen-2.5-72b-instruct` | `qwen2.5:14b` |
| Code Generation | `qwen/qwen-2.5-coder-32b-instruct` | `qwen2.5-coder:14b` |
| Speech-to-Text | `openai/whisper-large-v3` | Vosk CLI (offline) |
| Text-to-Speech | Not yet | Coqui TTS CLI |

## Usage

```rust
use secure_messenger_lib::ai::{AiClient, AiConfig};
use secure_messenger_lib::ai::{translator, summarizer, code_generator};

// Create client
let config = AiConfig::default(); // reads from env
let client = AiClient::new(config);

// Translate
let translated = translator::translate(
    &client, "Привет мир", "Russian", "English"
).await?;
// "Hello world"

// Summarize
let messages = vec![
    ("Alice".into(), "Let's implement AI".into()),
    ("You".into(), "Done, using OpenRouter + Ollama".into()),
];
let summary = summarizer::summarize_messages(&client, &messages).await?;

// Generate code
let code = code_generator::generate_code(
    &client,
    "Write a Rust fibonacci function",
    "Rust"
).await?;

// Generate tests
let tests = code_generator::generate_tests(&client, &code, "Rust").await?;
```

## Fallback Strategy

1. **Default** — Try OpenRouter first, fallback to Ollama on failure
2. **`AI_PREFER_LOCAL=true`** — Try Ollama first, fallback to OpenRouter
3. **Both fail** — Return `AiError::BothFailed` with both error messages

## File Structure

```
messenger/src/ai/
├── mod.rs              # Module declaration + re-exports
├── client.rs           # AiClient, AiConfig, OpenRouter + Ollama
├── translator.rs       # Language translation + detection
├── summarizer.rs       # Chat summarization + action items
├── speech_to_text.rs   # Audio transcription (Whisper + Vosk)
├── text_to_speech.rs   # Text-to-speech (Coqui TTS)
└── code_generator.rs   # Code generation + refactoring + testing
```

## Testing

```bash
# Run unit tests (non-ignored only)
cargo test ai::

# Run all tests including integration (requires backends)
cargo test ai:: -- --ignored

# Run specific test
cargo test ai::client::tests::test_config_from_env
```

## Hardware Notes

**User's setup:** RTX 3050 8GB
- `qwen2.5:14b` — fits in VRAM, ~4-6 tokens/sec
- `qwen2.5-coder:14b` — fits in VRAM, optimized for code
- For larger models (32B+), use OpenRouter cloud

**Recommendation:**
- Code generation → prefer local (`qwen2.5-coder:14b` is excellent)
- Translation of 100+ languages → prefer OpenRouter (72B model)
- Summarization → either works well locally
