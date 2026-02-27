//! TDLib клиент
//!
//! Обёртка над TDLib для работы с Telegram API.

use crate::config::Config;
use anyhow::{anyhow, Context, Result};
use log::{debug, error, info, trace, warn};
use std::ffi::CString;
use std::os::raw::{c_char, c_int};
use std::sync::Arc;
use std::time::Duration;
use tdlib::{create_client, receive, types};
use tokio::sync::{mpsc, Mutex, RwLock};

/// Отправка запроса в TDLib (без ожидания ответа)
fn td_send_request(client_id: i32, request: &str) {
    let cstring = CString::new(request).unwrap();
    unsafe {
        tdjson_send(client_id, cstring.as_ptr());
    }
}

/// Отправка запроса в TDLib с ожиданием ответа через receive
fn td_send_return(client_id: i32, request: &str) -> String {
    td_send_request(client_id, request);

    // Ждём ответ
    for _ in 0..100 {
        std::thread::sleep(Duration::from_millis(10));
        if let Some((update, _)) = receive() {
            // Конвертируем update обратно в JSON строку
            return serde_json::to_string(&update).unwrap_or_default();
        }
    }

    String::new()
}

#[link(name = "tdjson")]
extern "C" {
    fn td_send(client_id: c_int, request: *const c_char);
}

fn tdjson_send(client_id: c_int, request: *const c_char) {
    unsafe {
        td_send(client_id, request);
    }
}

/// Размер канала событий
const EVENT_CHANNEL_SIZE: usize = 1000;

/// Типы событий TDLib
#[derive(Debug, Clone)]
pub enum TdEvent {
    /// Авторизация успешна
    AuthorizationSuccessful { user_id: i64 },
    /// Требуется код подтверждения
    AuthorizationStateWaitCode { code_info: String },
    /// Требуется пароль 2FA
    AuthorizationStateWaitPassword { hint: Option<String> },
    /// Новое сообщение
    NewMessage {
        chat_id: i64,
        message_id: i64,
        text: String,
    },
    /// Обновление соединения
    ConnectionState { connected: bool },
    /// Ошибка
    Error { code: i32, message: String },
    /// Событие получения чатов
    Chats { chats: Vec<ChatInfo> },
    /// Событие получения истории
    Messages { messages: Vec<Message> },
    /// Другие события
    Other { event_type: String, data: String },
}

/// TDLib клиент
pub struct TdClient {
    /// Конфигурация
    config: Config,
    /// TDLib клиент ID
    client_id: i32,
    /// Флаг авторизации
    authorized: Arc<RwLock<bool>>,
    /// Канал для получения событий
    event_receiver: Arc<Mutex<mpsc::Receiver<TdEvent>>>,
    /// Отправитель событий
    event_sender: mpsc::Sender<TdEvent>,
    /// Текущее состояние соединения
    connected: Arc<RwLock<bool>>,
}

impl TdClient {
    /// Создание нового клиента
    pub async fn new(config: &Config) -> Result<Self> {
        info!("Инициализация TDLib клиента...");

        // Создаём канал для событий
        let (event_sender, event_receiver) = mpsc::channel(EVENT_CHANNEL_SIZE);

        // Инициализация TDLib
        let client_id = create_client();

        if client_id < 0 {
            return Err(anyhow!(
                "Не удалось создать TDLib клиент (код ошибки: {})",
                client_id
            ));
        }

        info!("TDLib клиент создан (client_id: {})", client_id);

        // Определение пути к базе данных
        let db_path = config
            .database_path
            .clone()
            .unwrap_or_else(|| "./tdlib_data".to_string());

        // Настройка параметров - база данных
        let extra = serde_json::json!({
            "@type": "setOption",
            "name": "database_directory",
            "value": {
                "@type": "optionValueString",
                "value": db_path
            }
        });

        let result = td_send_return(client_id, &extra.to_string());
        trace!("Настройка database_directory: {}", result);

        // Настройка API ID
        let init_params = serde_json::json!({
            "@type": "setOption",
            "name": "api_id",
            "value": {
                "@type": "optionValueInt32",
                "value": config.api_id as i32
            }
        });
        let result = td_send_return(client_id, &init_params.to_string());
        trace!("Настройка api_id: {}", result);

        // Настройка API Hash
        let init_hash = serde_json::json!({
            "@type": "setOption",
            "name": "api_hash",
            "value": {
                "@type": "optionValueString",
                "value": config.api_hash
            }
        });
        let result = td_send_return(client_id, &init_hash.to_string());
        trace!("Настройка api_hash: {}", result);

        // Настройка уровня логирования
        let log_verbosity = serde_json::json!({
            "@type": "setOption",
            "name": "log_verbosity_level",
            "value": {
                "@type": "optionValueInt32",
                "value": 2 // 0-4, где 4 = максимальный уровень
            }
        });
        let _ = td_send_request(client_id, &log_verbosity.to_string());

        // Запуск фонового потока для обработки событий TDLib
        let tx_clone = event_sender.clone();
        let authorized_clone = Arc::new(RwLock::new(false));
        let connected_clone = Arc::new(RwLock::new(false));

        let authorized_inner = authorized_clone.clone();
        let connected_inner = connected_clone.clone();

        tokio::spawn(async move {
            loop {
                // Получение событий от TDLib (в реальном режиме)
                // Здесь используется заглушка - в реальности нужно использовать Client::receive
                tokio::time::sleep(Duration::from_millis(100)).await;

                // Проверка флага завершения
                if tx_clone.is_closed() {
                    break;
                }
            }
        });

        Ok(Self {
            config: config.clone(),
            client_id,
            authorized: authorized_clone,
            event_receiver: Arc::new(Mutex::new(event_receiver)),
            event_sender,
            connected: connected_clone,
        })
    }

    /// Отправка запроса в TDLib
    pub fn send_query(&self, query: &serde_json::Value) -> Result<String> {
        let query_str = query.to_string();
        trace!("TDLib запрос: {}", query_str);

        let result = td_send_return(self.client_id, &query_str);

        if result.is_empty() {
            return Err(anyhow!("Пустой ответ от TDLib"));
        }

        trace!("TDLib ответ: {}", result);
        Ok(result)
    }

    /// Отправка запроса с дополнительными параметрами
    pub fn send_query_extra(&self, query: &serde_json::Value, extra: &str) -> Result<String> {
        let query_str = query.to_string();
        trace!("TDLib запрос (extra={}): {}", extra, query_str);

        // В tdlib crate нет send_query_extra, используем обычный send
        let result = td_send_return(self.client_id, &query_str);

        if result.is_empty() {
            return Err(anyhow!("Пустой ответ от TDLib"));
        }

        Ok(result)
    }

    /// Начало авторизации по номеру телефона
    pub async fn start_auth(&self, phone: &str) -> Result<String> {
        info!("Начало авторизации: {}", phone);

        let query = serde_json::json!({
            "@type": "setAuthenticationPhoneNumber",
            "phone_number": phone,
            "settings": {
                "@type": "authenticationSettings",
                "allow_flash_call": false,
                "is_current_phone_number": true,
                "allow_firebase_auth": false,
                "show_dialog": false,
                "allow_tokenless_auth": false
            }
        });

        self.send_query(&query)
            .with_context(|| format!("Ошибка отправки номера {}", phone))?;

        Ok("Код отправлен на указанный номер".to_string())
    }

    /// Проверка кода подтверждения
    pub async fn check_code(&self, code: &str) -> Result<String> {
        info!("Проверка кода: {}", code);

        let query = serde_json::json!({
            "@type": "checkAuthenticationCode",
            "code": code
        });

        self.send_query(&query).context("Ошибка проверки кода")?;

        Ok("Код принят".to_string())
    }

    /// Проверка пароля 2FA
    pub async fn check_password(&self, password: &str) -> Result<String> {
        info!("Проверка пароля 2FA");

        let query = serde_json::json!({
            "@type": "checkAuthenticationPassword",
            "password": password
        });

        self.send_query(&query)
            .context("Ошибка проверки пароля 2FA")?;

        Ok("Пароль принят".to_string())
    }

    /// Проверка авторизации
    pub async fn is_authorized(&self) -> bool {
        *self.authorized.read().await
    }

    /// Установка флага авторизации
    pub async fn set_authorized(&mut self, authorized: bool) {
        *self.authorized.write().await = authorized;
    }

    /// Получение отправителя событий
    pub fn get_event_sender(&self) -> mpsc::Sender<TdEvent> {
        self.event_sender.clone()
    }

    /// Получение получателя событий
    pub async fn get_event_receiver(&self) -> Arc<Mutex<mpsc::Receiver<TdEvent>>> {
        self.event_receiver.clone()
    }

    /// Проверка состояния соединения
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    /// Отправка сообщения
    pub async fn send_message(&self, chat_id: i64, text: &str) -> Result<()> {
        if !*self.authorized.read().await {
            return Err(anyhow!("Клиент не авторизован"));
        }

        debug!("Отправка сообщения в чат {}: {}", chat_id, text);

        let query = serde_json::json!({
            "@type": "sendMessage",
            "chat_id": chat_id,
            "reply_to_message_id": 0,
            "options": null,
            "reply_markup": null,
            "input_message_content": {
                "@type": "inputMessageText",
                "text": {
                    "@type": "formattedText",
                    "text": text,
                    "entities": []
                },
                "clear_draft": true
            }
        });

        self.send_query(&query)
            .with_context(|| format!("Ошибка отправки сообщения в чат {}", chat_id))?;

        info!("Сообщение отправлено в чат {}", chat_id);
        Ok(())
    }

    /// Отправка зашифрованного сообщения
    pub async fn send_encrypted_message(&self, chat_id: i64, text: &str) -> Result<()> {
        if !*self.authorized.read().await {
            return Err(anyhow!("Клиент не авторизован"));
        }

        // 1. Шифрование текста с помощью XChaCha20-Poly1305
        let encrypted = self.encrypt_message(text)?;

        // 2. Обфускация (если включена)
        let obfuscated = if self.config.encryption.obfuscation_enabled {
            self.obfuscate_message(&encrypted)?
        } else {
            encrypted
        };

        // 3. Отправка
        self.send_message(chat_id, &obfuscated).await?;

        Ok(())
    }

    /// Шифрование сообщения с помощью XChaCha20-Poly1305
    fn encrypt_message(&self, text: &str) -> Result<String> {
        use crate::crypto::xchacha::XChaChaCipher;
        use base64::{engine::general_purpose, Engine as _};

        // Генерация ключа из мастер-ключа (в реальности ключ должен храниться securely)
        let key = XChaChaCipher::generate_key();
        let cipher = XChaChaCipher::new(key);

        // Шифрование
        let (nonce, ciphertext) = cipher
            .encrypt(text.as_bytes())
            .context("Ошибка шифрования сообщения")?;

        // Кодирование в base64 для передачи
        let mut combined = nonce;
        combined.extend_from_slice(&ciphertext);
        let encoded = general_purpose::STANDARD.encode(&combined);

        // Добавление префикса для идентификации зашифрованного сообщения
        Ok(format!("[ENC:{}]", encoded))
    }

    /// Обфускация сообщения
    fn obfuscate_message(&self, data: &str) -> Result<String> {
        use crate::obfs::obfs4::Obfs4Transformer;

        let mut transformer = Obfs4Transformer::new();
        let obfuscated = transformer.obfuscate(data.as_bytes());

        // Base64 кодирование
        use base64::{engine::general_purpose, Engine as _};
        let encoded = general_purpose::STANDARD.encode(&obfuscated);

        Ok(format!("[OBF:{}]", encoded))
    }

    /// Расшифровка сообщения
    pub fn decrypt_message(&self, encoded: &str) -> Result<String> {
        use crate::crypto::xchacha::XChaChaCipher;
        use base64::{engine::general_purpose, Engine as _};

        // Декодирование base64
        let combined = general_purpose::STANDARD
            .decode(encoded)
            .context("Ошибка декодирования base64")?;

        // Извлечение nonce и ciphertext
        if combined.len() <= 24 {
            return Err(anyhow!("Слишком короткие данные"));
        }

        let nonce = &combined[..24];
        let ciphertext = &combined[24..];

        // Для расшифровки нужен тот же ключ (в реальности ключ должен храниться securely)
        // Здесь заглушка - в реальной реализации ключи хранятся в secure storage
        let key = XChaChaCipher::generate_key();
        let cipher = XChaChaCipher::new(key);

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .context("Ошибка расшифровки сообщения")?;

        String::from_utf8(plaintext).context("Ошибка конвертации в UTF-8")
    }

    /// Получение списка чатов
    pub async fn get_chats(&self, limit: i32) -> Result<Vec<ChatInfo>> {
        if !*self.authorized.read().await {
            return Err(anyhow!("Клиент не авторизован"));
        }

        debug!("Получение списка чатов (limit: {})", limit);

        let query = serde_json::json!({
            "@type": "getChats",
            "chat_list": {
                "@type": "chatListMain"
            },
            "limit": limit
        });

        let response = self
            .send_query(&query)
            .context("Ошибка получения списка чатов")?;

        // Парсинг ответа
        let chats = self.parse_chats_response(&response)?;

        Ok(chats)
    }

    /// Получение истории сообщений
    pub async fn get_messages(&self, chat_id: i64, limit: i32) -> Result<Vec<Message>> {
        if !*self.authorized.read().await {
            return Err(anyhow!("Клиент не авторизован"));
        }

        debug!("Получение истории чата {} (limit: {})", chat_id, limit);

        let query = serde_json::json!({
            "@type": "getChatHistory",
            "chat_id": chat_id,
            "from_message_id": 0,
            "offset": 0,
            "limit": limit,
            "only_local": false
        });

        let response = self
            .send_query(&query)
            .with_context(|| format!("Ошибка получения истории чата {}", chat_id))?;

        // Парсинг ответа
        let messages = self.parse_messages_response(&response, chat_id)?;

        Ok(messages)
    }

    /// Парсинг ответа с чатами
    fn parse_chats_response(&self, response: &str) -> Result<Vec<ChatInfo>> {
        let json: serde_json::Value =
            serde_json::from_str(response).context("Ошибка парсинга JSON ответа")?;

        let mut chats = Vec::new();

        if let Some(chat_ids) = json["chat_ids"].as_array() {
            for chat_id in chat_ids {
                if let Some(id) = chat_id.as_i64() {
                    // Заглушка - в реальности нужно делать getChat для каждого ID
                    chats.push(ChatInfo {
                        id,
                        title: format!("Chat {}", id),
                        last_message: None,
                    });
                }
            }
        }

        Ok(chats)
    }

    /// Парсинг ответа с сообщениями
    fn parse_messages_response(&self, response: &str, chat_id: i64) -> Result<Vec<Message>> {
        let json: serde_json::Value =
            serde_json::from_str(response).context("Ошибка парсинга JSON ответа")?;

        let mut messages = Vec::new();

        if let Some(msg_array) = json["@type"].as_str() {
            if msg_array == "messages" {
                if let Some(messages_array) = json["messages"].as_array() {
                    for msg in messages_array {
                        let id = msg["id"].as_i64().unwrap_or(0);
                        let from_user_id = msg["sender_id"]["user_id"].as_i64().unwrap_or(0);
                        let text = msg["content"]["text"]["text"]
                            .as_str()
                            .unwrap_or("")
                            .to_string();
                        let date = msg["date"].as_u64().unwrap_or(0);

                        messages.push(Message::new(id, chat_id, from_user_id, text, date));
                    }
                }
            }
        }

        Ok(messages)
    }

    /// Закрытие клиента
    pub async fn close(&mut self) -> Result<()> {
        info!("Закрытие TDLib клиента...");

        let query = serde_json::json!({
            "@type": "close"
        });

        self.send_query(&query)
            .context("Ошибка закрытия TDLib клиента")?;

        *self.authorized.write().await = false;
        *self.connected.write().await = false;

        Ok(())
    }

    /// Обработка события от TDLib
    pub async fn handle_event(&self, event: &str) -> Option<TdEvent> {
        trace!("Событие TDLib: {}", event);

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(event) {
            let event_type = json["@type"].as_str()?;

            match event_type {
                "authorizationStateClosed" => {
                    info!("TDLib закрыт");
                    *self.authorized.write().await = false;
                    Some(TdEvent::AuthorizationSuccessful { user_id: 0 })
                }
                "authorizationStateReady" => {
                    info!("Авторизация успешна!");
                    *self.authorized.write().await = true;
                    Some(TdEvent::AuthorizationSuccessful {
                        user_id: json["user_id"].as_i64().unwrap_or(0),
                    })
                }
                "authorizationStateWaitCode" => {
                    let code_info = json["code_info"].to_string();
                    warn!("Требуется код подтверждения: {:?}", code_info);
                    Some(TdEvent::AuthorizationStateWaitCode { code_info })
                }
                "authorizationStateWaitPassword" => {
                    let hint = json["password_hint"].as_str().map(|s| s.to_string());
                    warn!("Требуется пароль 2FA: {:?}", hint);
                    Some(TdEvent::AuthorizationStateWaitPassword { hint })
                }
                "updateNewMessage" => {
                    // Извлечение сообщения
                    let message = &json["message"];
                    let chat_id = message["chat_id"].as_i64()?;
                    let message_id = message["id"].as_i64()?;
                    let text = message["content"]["text"]["text"]
                        .as_str()
                        .unwrap_or("")
                        .to_string();

                    Some(TdEvent::NewMessage {
                        chat_id,
                        message_id,
                        text,
                    })
                }
                "updateConnectionState" => {
                    let state = json["state"]["@type"].as_str()?;
                    let connected = state == "connectionStateReady";
                    *self.connected.write().await = connected;
                    Some(TdEvent::ConnectionState { connected })
                }
                "error" => {
                    let code = json["code"].as_i64().unwrap_or(0) as i32;
                    let message = json["message"].as_str().unwrap_or("").to_string();
                    error!("Ошибка TDLib {}: {}", code, message);
                    Some(TdEvent::Error { code, message })
                }
                "updateChatLastMessage" => {
                    // Обновление последнего сообщения в чате
                    let chat_id = json["chat_id"].as_i64()?;
                    let last_message = json["last_message"]["content"]["text"]["text"]
                        .as_str()
                        .map(|s| s.to_string());

                    Some(TdEvent::Other {
                        event_type: "updateChatLastMessage".to_string(),
                        data: format!("chat_id: {}, message: {:?}", chat_id, last_message),
                    })
                }
                _ => {
                    trace!("Необработанное событие: {}", event_type);
                    Some(TdEvent::Other {
                        event_type: event_type.to_string(),
                        data: json.to_string(),
                    })
                }
            }
        } else {
            None
        }
    }
}

/// Информация о чате
#[derive(Debug, Clone)]
pub struct ChatInfo {
    pub id: i64,
    pub title: String,
    pub last_message: Option<String>,
}

/// Сообщение
#[derive(Debug, Clone)]
pub struct Message {
    pub id: i64,
    pub chat_id: i64,
    pub from_user_id: i64,
    pub text: String,
    pub date: u64,
}

impl Message {
    pub fn new(id: i64, chat_id: i64, from_user_id: i64, text: String, date: u64) -> Self {
        Self {
            id,
            chat_id,
            from_user_id,
            text,
            date,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_client_creation() {
        let config = Config::default();
        let client = TdClient::new(&config).await;

        assert!(client.is_ok());
    }
}
