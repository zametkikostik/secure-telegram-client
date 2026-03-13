//! Telegram Bridge — Мост между Telegram и P2P сетью Liberty Reach
//!
//! Реализует:
//! - Получение сообщений из Telegram Bot API
//! - Пересылка сообщений в P2P сеть
//! - Синхронизация статусов

use anyhow::{Result, Context};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;

/// Конфигурация Telegram бота
pub struct TelegramConfig {
    pub bot_token: String,
    pub webhook_url: Option<String>,
}

impl TelegramConfig {
    pub fn new() -> Self {
        let bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
            .unwrap_or_else(|_| String::new());

        Self {
            bot_token,
            webhook_url: None,
        }
    }

    pub fn with_token(token: &str) -> Self {
        Self {
            bot_token: token.to_string(),
            webhook_url: None,
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.bot_token.is_empty()
    }

    fn api_url(&self) -> String {
        format!("https://api.telegram.org/bot{}/", self.bot_token)
    }
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Сообщение от Telegram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramMessage {
    pub chat_id: i64,
    pub message_id: i64,
    pub from_user: TelegramUser,
    pub text: Option<String>,
    pub timestamp: u64,
}

/// Пользователь Telegram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramUser {
    pub id: i64,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

/// Входящее событие от Telegram
#[derive(Debug, Clone)]
pub enum TelegramEvent {
    Message(TelegramMessage),
    Command { chat_id: i64, command: String, args: Vec<String> },
    Callback { chat_id: i64, data: String },
}

/// Менеджер Telegram моста
pub struct TelegramBridge {
    client: Client,
    config: TelegramConfig,
    /// Канал для отправки событий в P2P сеть
    p2p_sender: Option<mpsc::Sender<TelegramEvent>>,
    /// Маппинг Telegram chat_id -> PeerId
    chat_to_peer: HashMap<i64, String>,
    /// Маппинг PeerId -> Telegram chat_id
    peer_to_chat: HashMap<String, i64>,
}

impl TelegramBridge {
    pub fn new(config: TelegramConfig) -> Self {
        Self {
            client: Client::new(),
            config,
            p2p_sender: None,
            chat_to_peer: HashMap::new(),
            peer_to_chat: HashMap::new(),
        }
    }

    /// Установка канала для отправки в P2P сеть
    pub fn set_p2p_sender(&mut self, sender: mpsc::Sender<TelegramEvent>) {
        self.p2p_sender = Some(sender);
    }

    /// Привязка Telegram чата к PeerId
    pub fn link_chat_to_peer(&mut self, chat_id: i64, peer_id: String) {
        self.chat_to_peer.insert(chat_id, peer_id.clone());
        self.peer_to_chat.insert(peer_id, chat_id);
    }

    /// Отправка сообщения в Telegram
    pub async fn send_to_telegram(&self, chat_id: i64, text: &str) -> Result<()> {
        if !self.config.is_configured() {
            anyhow::bail!("Telegram не настроен: установите TELEGRAM_BOT_TOKEN");
        }

        let url = format!("{}sendMessage", self.config.api_url());

        let response = self.client.post(&url)
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "text": text,
                "parse_mode": "Markdown",
            }))
            .send()
            .await
            .context("Ошибка отправки сообщения в Telegram")?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Telegram вернул ошибку: {}", body);
        }

        Ok(())
    }

    /// Отправка файла в Telegram
    pub async fn send_file_to_telegram(
        &self,
        chat_id: i64,
        file_data: Vec<u8>,
        filename: &str,
        caption: Option<&str>,
    ) -> Result<()> {
        if !self.config.is_configured() {
            anyhow::bail!("Telegram не настроен");
        }

        let url = format!("{}sendDocument", self.config.api_url());

        let form = reqwest::multipart::Form::new()
            .text("chat_id", chat_id.to_string())
            .part("document", reqwest::multipart::Part::bytes(file_data)
                .file_name(filename.to_string()));

        let form = if let Some(cap) = caption {
            form.text("caption", cap.to_string())
        } else {
            form
        };

        let response = self.client.post(&url)
            .multipart(form)
            .send()
            .await
            .context("Ошибка отправки файла в Telegram")?;

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Telegram вернул ошибку: {}", body);
        }

        Ok(())
    }

    /// Получение обновлений (long polling)
    pub async fn get_updates(
        &self,
        offset: Option<i64>,
        timeout: u32,
    ) -> Result<Vec<serde_json::Value>> {
        if !self.config.is_configured() {
            anyhow::bail!("Telegram не настроен");
        }

        let url = format!("{}getUpdates", self.config.api_url());

        let mut params = serde_json::json!({
            "timeout": timeout,
        });

        if let Some(off) = offset {
            params["offset"] = serde_json::json!(off);
        }

        let response = self.client.post(&url)
            .json(&params)
            .send()
            .await
            .context("Ошибка получения обновлений Telegram")?;

        let result: serde_json::Value = response.json()
            .await
            .context("Ошибка парсинга ответа Telegram")?;

        if let Some(results) = result["result"].as_array() {
            Ok(results.clone())
        } else {
            Ok(Vec::new())
        }
    }

    /// Обработка входящего сообщения из Telegram
    pub fn process_incoming_message(
        &self,
        update: &serde_json::Value,
    ) -> Option<TelegramEvent> {
        // Проверяем наличие сообщения
        if let Some(message) = update.get("message") {
            let chat_id = message["chat"]["id"].as_i64()?;
            let message_id = message["message_id"].as_i64()?;
            let timestamp = message["date"].as_u64().unwrap_or(0);

            // Извлекаем пользователя
            let from_user = TelegramUser {
                id: message["from"]["id"].as_i64().unwrap_or(0),
                username: message["from"]["username"].as_str().map(String::from),
                first_name: message["from"]["first_name"].as_str().map(String::from),
                last_name: message["from"]["last_name"].as_str().map(String::from),
            };

            // Проверяем текст
            if let Some(text) = message["text"].as_str() {
                // Проверяем, не команда ли это
                if text.starts_with('/') {
                    let parts: Vec<&str> = text.split_whitespace().collect();
                    let command = parts[0].trim_start_matches('/').to_string();
                    let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

                    return Some(TelegramEvent::Command { chat_id, command, args });
                }

                return Some(TelegramEvent::Message(TelegramMessage {
                    chat_id,
                    message_id,
                    from_user,
                    text: Some(text.to_string()),
                    timestamp,
                }));
            }
        }

        // Проверяем callback query
        if let Some(callback) = update.get("callback_query") {
            let chat_id = callback["message"]["chat"]["id"].as_i64()?;
            let data = callback["data"].as_str()?.to_string();
            return Some(TelegramEvent::Callback { chat_id, data });
        }

        None
    }

    /// Запуск цикла получения сообщений
    pub async fn run_listener(
        &self,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<()> {
        if !self.config.is_configured() {
            tracing::warn!("Telegram не настроен, слушатель не запущен");
            return Ok(());
        }

        let mut offset: Option<i64> = None;
        let p2p_sender = self.p2p_sender.clone();

        tracing::info!("Telegram Bridge запущен");

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    tracing::info!("Telegram Bridge остановлен");
                    break;
                }

                result = self.get_updates(offset, 30) => {
                    match result {
                        Ok(updates) => {
                            for update in updates {
                                // Обновляем offset
                                if let Some(id) = update["update_id"].as_i64() {
                                    offset = Some(id + 1);
                                }

                                // Обрабатываем сообщение
                                if let Some(event) = self.process_incoming_message(&update) {
                                    // Отправляем в P2P сеть если есть канал
                                    if let Some(sender) = &p2p_sender {
                                        if let Err(e) = sender.send(event.clone()).await {
                                            tracing::error!("Ошибка отправки в P2P: {}", e);
                                        }
                                    }

                                    // Логируем
                                    match event {
                                        TelegramEvent::Message(msg) => {
                                            tracing::info!(
                                                "Telegram [{}]: {:?}",
                                                msg.from_user.username.unwrap_or_default(),
                                                msg.text
                                            );
                                        }
                                        TelegramEvent::Command { command, .. } => {
                                            tracing::info!("Telegram команда: /{}", command);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Ошибка получения обновлений: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Отправка уведомления о новом сообщении из P2P в Telegram
    pub async fn forward_p2p_message_to_telegram(
        &self,
        peer_id: &str,
        message_text: &str,
    ) -> Result<()> {
        if let Some(chat_id) = self.peer_to_chat.get(peer_id) {
            let formatted = format!(
                "📨 **Сообщение из Liberty Reach**\n\n\
                 От: `{}`\n\
                 {}\n",
                peer_id.chars().take(16).collect::<String>(),
                message_text
            );
            self.send_to_telegram(*chat_id, &formatted).await?;
        }
        Ok(())
    }
}

/// Команды Telegram бота
pub const TELEGRAM_COMMANDS: &[(&str, &str)] = &[
    ("start", "Запуск бота"),
    ("link", "Привязать Telegram к Liberty Reach"),
    ("status", "Показать статус"),
    ("help", "Помощь"),
];

/// Форматирование списка команд для BotFather
#[allow(dead_code)]
pub fn format_commands_for_botfather() -> String {
    TELEGRAM_COMMANDS
        .iter()
        .map(|(cmd, desc)| format!("/{} - {}", cmd, desc))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        let config = TelegramConfig::new();
        // Если переменная окружения не установлена, бот не настроен
        // Это ожидаемое поведение
        assert!(!config.is_configured() || config.bot_token.starts_with("0"));
    }

    #[test]
    fn test_commands_format() {
        let formatted = format_commands_for_botfather();
        assert!(formatted.contains("/start"));
        assert!(formatted.contains("/help"));
    }
}
