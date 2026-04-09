//! Telegram Bot Fallback Transport
//!
//! When P2P connection is unavailable and the recipient is offline in our network,
//! encrypted messages are delivered via Telegram Bot API as a fallback channel.
//!
//! Security:
//! - Messages are E2EE encrypted BEFORE being sent to Telegram Bot
//! - Telegram only sees ciphertext (ChaCha20-Poly1305)
//! - Bot cannot decrypt messages (no access to user keys)
//! - Telegram acts as an untrusted delivery channel
//!
//! Flow:
//! 1. Sender encrypts message with recipient's public key
//! 2. Encrypted payload sent to Telegram Bot API
//! 3. Bot sends notification to recipient via Telegram
//! 4. Recipient opens Secure Messenger → decrypts locally
//!
//! Usage:
//! 1. Create bot via @BotFather on Telegram
//! 2. Get bot token (e.g., "123456:ABC-DEF...")
//! 3. Set TELEGRAM_BOT_TOKEN in .env
//! 4. Users must start a conversation with the bot first

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, info, warn};

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum TelegramBotError {
    #[error("Bot token not configured")]
    BotTokenNotConfigured,

    #[error("HTTP error: {0}")]
    HttpError(String),

    #[error("Telegram API error: {code} - {description}")]
    TelegramApiError { code: i64, description: String },

    #[error("User has not started the bot")]
    UserNotStarted,

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),
}

pub type TelegramBotResult<T> = Result<T, TelegramBotError>;

// ============================================================================
// Telegram Bot Transport
// ============================================================================

/// Encrypted message payload for Telegram delivery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedTelegramPayload {
    /// E2EE encrypted content (base64)
    pub encrypted_content: String,
    /// Sender's public key (for verification)
    pub sender_public_key: String,
    /// Message timestamp
    pub timestamp: i64,
    /// Message type hint (text, photo, file)
    pub content_type: String,
}

/// Telegram Bot Transport for fallback message delivery
pub struct TelegramBotTransport {
    bot_token: String,
    http_client: Client,
    base_url: String,
}

impl TelegramBotTransport {
    /// Create new transport from environment variable
    pub fn from_env() -> TelegramBotResult<Self> {
        let bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
            .map_err(|_| TelegramBotError::BotTokenNotConfigured)?;

        Ok(Self::new(&bot_token))
    }

    /// Create new transport with explicit token
    pub fn new(bot_token: &str) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            bot_token: bot_token.to_string(),
            http_client,
            base_url: "https://api.telegram.org".to_string(),
        }
    }

    // ========================================================================
    // Core: Send Encrypted Message
    // ========================================================================

    /// Send encrypted message via Telegram Bot
    ///
    /// The message is already E2EE encrypted by the sender.
    /// Telegram only sees the ciphertext — it cannot decrypt.
    ///
    /// # Arguments
    /// * `telegram_user_id` — Telegram user ID (numeric)
    /// * `encrypted_payload` — E2EE encrypted message bytes
    /// * `sender_public_key` — Sender's public key for verification
    ///
    /// # Returns
    /// * `Ok(message_id)` — Telegram message ID
    /// * `Err(TelegramBotError)` — Delivery failed
    pub async fn send_encrypted_message(
        &self,
        telegram_user_id: i64,
        encrypted_payload: &[u8],
        sender_public_key: &str,
    ) -> TelegramBotResult<i64> {
        // Create payload wrapper
        let payload = EncryptedTelegramPayload {
            encrypted_content: base64::encode(encrypted_payload),
            sender_public_key: sender_public_key.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            content_type: "encrypted_message".to_string(),
        };

        // Serialize to JSON
        let payload_json = serde_json::to_string(&payload)
            .map_err(|e| TelegramBotError::SerializationError(e.to_string()))?;

        // Send as formatted message with "Open in Messenger" button
        let message_text = format!(
            "🔒 <b>Secure Messenger</b>\n\n\
             You have a new encrypted message.\n\
             <i>Message cannot be read here — open Secure Messenger to decrypt.</i>\n\n\
             <code>{}</code>",
            &payload
                .encrypted_content
                .chars()
                .take(50)
                .collect::<String>()
        );

        // Inline keyboard with "Open in Secure Messenger" button
        let keyboard = serde_json::json!({
            "inline_keyboard": [[{
                "text": "🔓 Открыть в Secure Messenger",
                "url": "https://messenger.your-domain.com"
            }]]
        });

        let response = self
            .http_client
            .post(&format!(
                "{}/bot{}/sendMessage",
                self.base_url, self.bot_token
            ))
            .json(&serde_json::json!({
                "chat_id": telegram_user_id,
                "text": message_text,
                "parse_mode": "HTML",
                "reply_markup": keyboard,
            }))
            .send()
            .await
            .map_err(|e| TelegramBotError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TelegramBotError::TelegramApiError {
                code: status.as_u16() as i64,
                description: body,
            });
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| TelegramBotError::SerializationError(e.to_string()))?;

        if result["ok"].as_bool() == Some(true) {
            let message_id = result["result"]["message_id"].as_i64().ok_or_else(|| {
                TelegramBotError::TelegramApiError {
                    code: 0,
                    description: "No message_id in response".into(),
                }
            })?;

            info!(
                "Sent encrypted message to telegram user {} (msg_id: {})",
                telegram_user_id, message_id
            );

            Ok(message_id)
        } else {
            let error_code = result["error_code"].as_i64().unwrap_or(0);
            let description = result["description"]
                .as_str()
                .unwrap_or("Unknown error")
                .to_string();

            if description.contains("bot was blocked") || description.contains("user not found") {
                return Err(TelegramBotError::UserNotStarted);
            }

            Err(TelegramBotError::TelegramApiError {
                code: error_code,
                description,
            })
        }
    }

    // ========================================================================
    // Notification Methods
    // ========================================================================

    /// Send a push notification (short, when full message too large)
    pub async fn send_notification(
        &self,
        telegram_user_id: i64,
        sender_name: &str,
        chat_name: &str,
    ) -> TelegramBotResult<i64> {
        let text = format!(
            "🔒 <b>Secure Messenger</b>\n\n\
             <b>{}</b> sent you a message in <b>{}</b>\n\n\
             Open the app to read.",
            sender_name, chat_name
        );

        let keyboard = serde_json::json!({
            "inline_keyboard": [[{
                "text": "🔓 Открыть Secure Messenger",
                "url": "https://messenger.your-domain.com"
            }]]
        });

        let response = self
            .http_client
            .post(&format!(
                "{}/bot{}/sendMessage",
                self.base_url, self.bot_token
            ))
            .json(&serde_json::json!({
                "chat_id": telegram_user_id,
                "text": text,
                "parse_mode": "HTML",
                "reply_markup": keyboard,
            }))
            .send()
            .await
            .map_err(|e| TelegramBotError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(TelegramBotError::HttpError(format!(
                "HTTP {}",
                response.status()
            )));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| TelegramBotError::SerializationError(e.to_string()))?;

        let message_id = result["result"]["message_id"].as_i64().unwrap_or(0);

        Ok(message_id)
    }

    // ========================================================================
    // Bot Status Checks
    // ========================================================================

    /// Check if user has started the bot
    pub async fn is_user_active(&self, telegram_user_id: i64) -> bool {
        let response = self
            .http_client
            .post(&format!("{}/bot{}/getChat", self.base_url, self.bot_token))
            .json(&serde_json::json!({
                "chat_id": telegram_user_id,
            }))
            .send()
            .await;

        match response {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// Get bot info (username, name, etc.)
    pub async fn get_bot_info(&self) -> TelegramBotResult<BotInfo> {
        let response = self
            .http_client
            .get(&format!("{}/bot{}/getMe", self.base_url, self.bot_token))
            .send()
            .await
            .map_err(|e| TelegramBotError::HttpError(e.to_string()))?;

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| TelegramBotError::SerializationError(e.to_string()))?;

        if result["ok"].as_bool() != Some(true) {
            return Err(TelegramBotError::TelegramApiError {
                code: result["error_code"].as_i64().unwrap_or(0),
                description: result["description"]
                    .as_str()
                    .unwrap_or("Unknown error")
                    .into(),
            });
        }

        let bot = &result["result"];
        Ok(BotInfo {
            id: bot["id"].as_i64().unwrap_or(0),
            username: bot["username"].as_str().unwrap_or("").to_string(),
            first_name: bot["first_name"].as_str().unwrap_or("").to_string(),
        })
    }
}

/// Bot information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotInfo {
    pub id: i64,
    pub username: String,
    pub first_name: String,
}

// ============================================================================
// Message Router with Telegram Fallback
// ============================================================================

/// Message delivery route with automatic fallback
#[derive(Debug, Clone)]
pub enum DeliveryRoute {
    /// Direct P2P connection (preferred)
    P2PDirect,
    /// Cloudflare Worker relay (fallback #1)
    CloudflareWorker,
    /// Telegram Bot (fallback #2 — last resort)
    TelegramBot(i64), // Telegram user ID
}

/// Router that tries multiple delivery routes
pub struct MessageRouter {
    telegram_transport: Option<TelegramBotTransport>,
}

impl MessageRouter {
    pub fn new() -> Self {
        let telegram_transport = TelegramBotTransport::from_env().ok();

        Self { telegram_transport }
    }

    /// Send message with automatic fallback
    pub async fn send_with_fallback(
        &self,
        telegram_user_id: i64,
        encrypted_payload: &[u8],
        sender_public_key: &str,
        preferred_route: DeliveryRoute,
    ) -> TelegramBotResult<String> {
        match preferred_route {
            DeliveryRoute::P2PDirect => {
                // TODO: Implement P2P direct send
                debug!("P2P direct route — falling back to Telegram Bot");
                self.send_via_telegram_bot(telegram_user_id, encrypted_payload, sender_public_key)
                    .await
            }
            DeliveryRoute::CloudflareWorker => {
                // TODO: Implement Cloudflare Worker relay
                debug!("Cloudflare Worker route — falling back to Telegram Bot");
                self.send_via_telegram_bot(telegram_user_id, encrypted_payload, sender_public_key)
                    .await
            }
            DeliveryRoute::TelegramBot(_) => {
                self.send_via_telegram_bot(telegram_user_id, encrypted_payload, sender_public_key)
                    .await
            }
        }
    }

    async fn send_via_telegram_bot(
        &self,
        telegram_user_id: i64,
        encrypted_payload: &[u8],
        sender_public_key: &str,
    ) -> TelegramBotResult<String> {
        match &self.telegram_transport {
            Some(transport) => {
                let msg_id = transport
                    .send_encrypted_message(telegram_user_id, encrypted_payload, sender_public_key)
                    .await?;

                Ok(format!("telegram:{}", msg_id))
            }
            None => Err(TelegramBotError::BotTokenNotConfigured),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypted_payload_serialization() {
        let payload = EncryptedTelegramPayload {
            encrypted_content: "base64data...".to_string(),
            sender_public_key: "pubkey123".to_string(),
            timestamp: 1234567890,
            content_type: "encrypted_message".to_string(),
        };

        let json = serde_json::to_string(&payload).unwrap();
        let deserialized: EncryptedTelegramPayload = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.encrypted_content, "base64data...");
        assert_eq!(deserialized.content_type, "encrypted_message");
    }

    #[test]
    fn test_bot_info_serialization() {
        let info = BotInfo {
            id: 123456789,
            username: "SecureMessengerBot".to_string(),
            first_name: "Secure Messenger".to_string(),
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: BotInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.username, "SecureMessengerBot");
    }

    #[test]
    fn test_delivery_route() {
        let p2p = DeliveryRoute::P2PDirect;
        let cf = DeliveryRoute::CloudflareWorker;
        let tg = DeliveryRoute::TelegramBot(12345);

        assert!(matches!(p2p, DeliveryRoute::P2PDirect));
        assert!(matches!(cf, DeliveryRoute::CloudflareWorker));
        assert!(matches!(tg, DeliveryRoute::TelegramBot(12345)));
    }

    #[test]
    fn test_message_router_creation() {
        // Without TELEGRAM_BOT_TOKEN, should still create router (with None transport)
        std::env::remove_var("TELEGRAM_BOT_TOKEN");
        let router = MessageRouter::new();
        assert!(router.telegram_transport.is_none());
    }
}
