// server/src/websocket.rs
//! WebSocket для реального времени общения

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tokio::sync::broadcast;
use std::collections::HashMap;
use uuid::Uuid;

/// Тип для отправки сообщений в канал
pub type Tx = broadcast::Sender<WsMessage>;

/// Тип для получения сообщений из канала
pub type Rx = broadcast::Receiver<WsMessage>;

/// Сообщение WebSocket
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "auth")]
    Auth { token: String },
    
    #[serde(rename = "subscribe")]
    Subscribe { chat_id: String },
    
    #[serde(rename = "unsubscribe")]
    Unsubscribe { chat_id: String },
    
    #[serde(rename = "message")]
    Message {
        chat_id: String,
        content: String,
        message_type: Option<String>,
        file_url: Option<String>,
    },
    
    #[serde(rename = "typing")]
    Typing { chat_id: String },
    
    #[serde(rename = "read")]
    Read { chat_id: String, message_ids: Vec<String> },
    
    #[serde(rename = "error")]
    Error { message: String },
    
    #[serde(rename = "success")]
    Success { message: String },
}

/// Состояние WebSocket менеджера
pub struct WebSocketManager {
    /// Канал для рассылки сообщений
    tx: Tx,
    /// Подключенные пользователи: user_id -> websocket sender
    users: HashMap<String, Tx>,
    /// Подписки на чаты: chat_id -> список user_id
    chat_subscriptions: HashMap<String, Vec<String>>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000);
        
        Self {
            tx,
            users: HashMap::new(),
            chat_subscriptions: HashMap::new(),
        }
    }

    pub fn subscribe_chat(&mut self, chat_id: String, user_id: String) {
        self.chat_subscriptions
            .entry(chat_id)
            .or_insert_with(Vec::new)
            .push(user_id);
    }

    pub fn unsubscribe_chat(&mut self, chat_id: &str, user_id: &str) {
        if let Some(subscribers) = self.chat_subscriptions.get_mut(chat_id) {
            subscribers.retain(|id| id != user_id);
        }
    }

    pub fn broadcast_to_chat(&self, chat_id: &str, message: WsMessage) {
        if let Some(subscribers) = self.chat_subscriptions.get(chat_id) {
            for user_id in subscribers {
                if let Some(tx) = self.users.get(user_id) {
                    let _ = tx.send(message.clone());
                }
            }
        }
    }
}

/// Обработка WebSocket подключения
pub async fn handle_socket(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();

    // Задача для получения сообщений от клиента
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                // Парсинг сообщения
                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                    tracing::info!("Получено WS сообщение: {:?}", ws_msg);
                    // Здесь должна быть логика обработки
                }
            }
        }
    });

    // Задача для отправки сообщений клиенту
    let send_task = tokio::spawn(async move {
        // В реальности здесь будет получение сообщений из канала
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            // Ping для поддержания соединения
            let _ = sender.send(Message::Ping(vec![])).await;
        }
    });

    // Ожидание завершения любой из задач
    tokio::select! {
        _ = recv_task => {},
        _ = send_task => {},
    }
}

/// Middleware для авторизации WebSocket
pub async fn authorize_ws(token: &str, jwt_secret: &str) -> Option<String> {
    use jsonwebtoken::{decode, Validation, Algorithm};
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Serialize, Deserialize)]
    struct WsClaims {
        sub: String,
    }
    
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = false;
    
    match decode::<WsClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    ) {
        Ok(token_data) => Some(token_data.claims.sub),
        Err(_) => None,
    }
}
