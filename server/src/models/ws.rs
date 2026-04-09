// WebSocket message types and models

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Типы WebSocket сообщений
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsMessage {
    // Client → Server
    /// Подписка на каналы уведомлений
    Subscribe {
        user_id: String,
        channels: Vec<NotificationChannel>,
    },
    /// Отписка от каналов
    Unsubscribe {
        user_id: String,
        channels: Option<Vec<NotificationChannel>>,
    },
    /// P2P signaling offer
    P2POffer {
        target_user_id: String,
        sdp: String,
        candidates: Vec<IceCandidate>,
    },
    /// P2P signaling answer
    P2PAnswer {
        target_user_id: String,
        sdp: String,
        candidates: Vec<IceCandidate>,
    },
    /// ICE candidate
    IceCandidate {
        target_user_id: String,
        candidate: IceCandidate,
    },
    /// Ping для поддержания соединения
    Ping,
    
    // Server → Client
    /// Уведомление о новом сообщении
    Notification {
        channel: NotificationChannel,
        payload: NotificationPayload,
    },
    /// P2P соединение установлено
    P2PConnected {
        peer_id: String,
    },
    /// Ошибка
    Error {
        code: String,
        message: String,
    },
    /// Pong ответ
    Pong,
}

/// Каналы уведомлений
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum NotificationChannel {
    /// Новые сообщения
    Messages,
    /// P2P signaling
    P2PSignaling,
    /// Обновление статуса доставки
    DeliveryStatus,
    /// Системные уведомления
    System,
}

/// Payload уведомления
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NotificationPayload {
    /// Новое сообщение
    NewMessage {
        chat_id: String,
        message_id: String,
        sender_id: String,
        encrypted_content: String,
        timestamp: String,
    },
    /// Статус доставки
    DeliveryUpdate {
        message_id: String,
        status: String,
        timestamp: String,
    },
    /// P2P событие
    P2PEvent {
        event_type: String,
        peer_id: String,
        data: serde_json::Value,
    },
    /// Системное уведомление
    SystemNotification {
        message: String,
        code: String,
    },
}

/// ICE candidate для WebRTC
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct IceCandidate {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_m_line_index: Option<u16>,
}

/// Подписка пользователя
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UserSubscription {
    pub user_id: String,
    pub channels: HashSet<NotificationChannel>,
    pub connection_id: String,
}

/// Состояние WebSocket соединения
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WsConnection {
    pub connection_id: String,
    pub user_id: Option<String>,
    pub subscriptions: HashSet<NotificationChannel>,
}
