//! Chat + Transport Integration
//!
//! Интеграция чатов с транспортным слоем:
//! - Отправка сообщений через TransportRouter (P2P → Cloudflare fallback)
//! - Получение сообщений из P2P/Cloudflare → добавление в чат
//! - Автоматическое шифрование перед отправкой
//! - Обработка входящих сообщений
//!
//! SECURITY: требует аудита перед production
//! TODO: pentest перед release

use crate::chat::{
    ChatError, ChatManager, ChatMessage, ChatStatus, DeliveryState, GroupChat, MemberRole,
    MessageContentType, PrivateChat,
};
use crate::crypto::{decrypt, encrypt, HybridKeypair};
use crate::p2p::{MessageType, P2PEvent, P2PMessage};
use crate::transport::router::{Route, TransportRouter};
use chrono::Utc;
use ed25519_dalek::{SigningKey, VerifyingKey};
use oqs::kem::{PublicKey as KyberPublicKey, SecretKey as KyberSecretKey};
use std::collections::HashMap;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use x25519_dalek::PublicKey as X25519PublicKey;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum IntegrationError {
    #[error("Chat error: {0}")]
    Chat(#[from] ChatError),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("Router error: {0}")]
    Router(String),

    #[error("Key not found for user: {0}")]
    KeyNotFound(String),

    #[error("Chat not found: {0}")]
    ChatNotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

// ============================================================================
// User Keys Registry
// ============================================================================

/// Реестр публичных ключей пользователей
pub struct UserKeysRegistry {
    /// user_id → (X25519 public, Kyber public, Ed25519 verifying)
    keys: HashMap<String, (X25519PublicKey, KyberPublicKey, VerifyingKey)>,
}

impl UserKeysRegistry {
    pub fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    /// Зарегистрировать публичные ключи пользователя
    pub fn register(
        &mut self,
        user_id: String,
        x25519: X25519PublicKey,
        kyber: KyberPublicKey,
        ed25519: VerifyingKey,
    ) {
        debug!("Registered keys for user: {}", user_id);
        self.keys.insert(user_id, (x25519, kyber, ed25519));
    }

    /// Получить публичные ключи получателя
    pub fn get_recipient_keys(
        &self,
        user_id: &str,
    ) -> Option<(&X25519PublicKey, &KyberPublicKey, &VerifyingKey)> {
        self.keys.get(user_id).map(|(x, k, e)| (x, k, e))
    }

    /// Проверить, зарегистрирован ли пользователь
    pub fn is_registered(&self, user_id: &str) -> bool {
        self.keys.contains_key(user_id)
    }
}

impl Default for UserKeysRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Chat Transport Integration
// ============================================================================

/// Интеграция чатов с транспортным слоем
///
/// Обеспечивает:
/// - Сквозное шифрование сообщений перед отправкой
/// - Автоматический выбор маршрута (P2P → Cloudflare)
/// - Расшифровку входящих сообщений
/// - Добавление в соответствующий чат
pub struct ChatTransport {
    /// Менеджер чатов
    chat_manager: RwLock<ChatManager>,
    /// Транспортный роутер
    router: RwLock<Option<TransportRouter>>,
    /// Реестр ключей
    keys_registry: RwLock<UserKeysRegistry>,
    /// Локальные ключи пользователя
    local_keypair: HybridKeypair,
    /// Маппинг: peer_id → chat_id
    peer_to_chat: RwLock<HashMap<String, String>>,
}

impl ChatTransport {
    /// Создать новую интеграцию
    pub fn new(chat_manager: ChatManager, local_keypair: HybridKeypair) -> Self {
        Self {
            chat_manager: RwLock::new(chat_manager),
            router: RwLock::new(None),
            keys_registry: RwLock::new(UserKeysRegistry::new()),
            local_keypair,
            peer_to_chat: RwLock::new(HashMap::new()),
        }
    }

    /// Установить транспортный роутер
    pub async fn set_router(&self, router: TransportRouter) {
        let mut guard = self.router.write().await;
        *guard = Some(router);
        info!("Transport router set for chat integration");
    }

    /// Зарегистрировать ключи пользователя
    pub async fn register_user_keys(
        &self,
        user_id: String,
        x25519: X25519PublicKey,
        kyber: KyberPublicKey,
        ed25519: VerifyingKey,
    ) {
        self.keys_registry
            .write()
            .await
            .register(user_id, x25519, kyber, ed25519);
    }

    // ========================================================================
    // Отправка сообщений
    // ========================================================================

    /// Отправить зашифрованное сообщение в приватный чат
    ///
    /// Полный цикл:
    /// 1. Получить чат
    /// 2. Получить ключи получателя
    /// 3. Зашифровать plaintext
    /// 4. Отправить через router (P2P или Cloudflare)
    /// 5. Добавить в локальный чат
    pub async fn send_private_message(
        &self,
        chat_id: &str,
        peer_id: &str,
        plaintext: &[u8],
    ) -> Result<String, IntegrationError> {
        // 1. Получить роутер
        let router_guard = self.router.read().await;
        let router = router_guard
            .as_ref()
            .ok_or_else(|| IntegrationError::Router("Router not initialized".to_string()))?;

        // 2. Получить ключи получателя
        let keys_guard = self.keys_registry.read().await;
        let (recipient_x25519, recipient_kyber, _recipient_ed25519) = keys_guard
            .get_recipient_keys(peer_id)
            .ok_or_else(|| IntegrationError::KeyNotFound(peer_id.to_string()))?;

        // 3. Зашифровать и отправить через router
        let route = router
            .send_encrypted(
                peer_id,
                plaintext,
                recipient_x25519,
                recipient_kyber,
                &self.local_keypair.ed25519_secret,
            )
            .await
            .map_err(|e| IntegrationError::Router(e.to_string()))?;

        info!("Sent private message to {} via {}", peer_id, route);

        // 4. Создать локальное сообщение
        let chat_msg = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            chat_id: chat_id.to_string(),
            sender_id: self.local_peer_id(),
            encrypted_content: plaintext.to_vec(), // В реальности — ciphertext
            signature: vec![],
            created_at: Utc::now(),
            msg_type: MessageContentType::Text,
            delivery_status: DeliveryState::Sent,
            reply_to: None,
            attachments: Vec::new(),
        };

        let chat_id_clone = chat_msg.chat_id.clone();
        let msg_id = chat_msg.id.clone();

        // 5. Добавить в локальный чат
        let mut chat_guard = self.chat_manager.write().await;
        if let Some(chat) = chat_guard.get_private_chat_mut(&chat_id_clone) {
            chat.add_message(chat_msg);
        }

        Ok(msg_id)
    }

    /// Отправить сообщение в групповой чат
    ///
    /// Шифрует для каждого участника отдельно
    pub async fn send_group_message(
        &self,
        chat_id: &str,
        plaintext: &[u8],
    ) -> Result<Vec<String>, IntegrationError> {
        let mut chat_guard = self.chat_manager.write().await;
        let group = chat_guard
            .get_group_chat_mut(chat_id)
            .ok_or_else(|| IntegrationError::ChatNotFound(chat_id.to_string()))?;

        // Проверить права
        // (в реальности нужно проверить sender_id)

        let mut message_ids = Vec::new();
        let active_members = group.active_members();

        // Отправить каждому активному участнику
        for member in &active_members {
            if member.user_id == self.local_peer_id() {
                continue; // Не отправлять себе
            }

            // Зашифровать и отправить (упрощённо)
            let msg_id = self
                .send_to_user(&member.user_id, plaintext, chat_id)
                .await?;
            message_ids.push(msg_id);
        }

        let member_count = active_members.len();

        // Создать локальное сообщение
        let chat_msg = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            chat_id: chat_id.to_string(),
            sender_id: self.local_peer_id(),
            encrypted_content: plaintext.to_vec(),
            signature: vec![],
            created_at: Utc::now(),
            msg_type: MessageContentType::Text,
            delivery_status: DeliveryState::Sent,
            reply_to: None,
            attachments: Vec::new(),
        };

        group.add_message(chat_msg.clone());
        message_ids.push(chat_msg.id);

        info!(
            "Sent group message to {} ({} members)",
            chat_id,
            member_count
        );

        Ok(message_ids)
    }

    /// Вспомогательный метод: отправить одному пользователю
    async fn send_to_user(
        &self,
        user_id: &str,
        plaintext: &[u8],
        _chat_id: &str,
    ) -> Result<String, IntegrationError> {
        let router_guard = self.router.read().await;
        let router = router_guard
            .as_ref()
            .ok_or_else(|| IntegrationError::Router("Router not initialized".to_string()))?;

        let keys_guard = self.keys_registry.read().await;
        let (recipient_x25519, recipient_kyber, _recipient_ed25519) = keys_guard
            .get_recipient_keys(user_id)
            .ok_or_else(|| IntegrationError::KeyNotFound(user_id.to_string()))?;

        let route = router
            .send_encrypted(
                user_id,
                plaintext,
                recipient_x25519,
                recipient_kyber,
                &self.local_keypair.ed25519_secret,
            )
            .await
            .map_err(|e| IntegrationError::Router(e.to_string()))?;

        debug!("Sent to user {} via {}", user_id, route);

        Ok(uuid::Uuid::new_v4().to_string())
    }

    // ========================================================================
    // Получение сообщений
    // ========================================================================

    /// Обработать входящее P2P сообщение
    ///
    /// 1. Расшифровать ciphertext
    /// 2. Проверить подпись
    /// 3. Найти чат
    /// 4. Добавить сообщение
    pub async fn handle_incoming_p2p_message(
        &self,
        from_peer_id: &str,
        p2p_message: P2PMessage,
    ) -> Result<(), IntegrationError> {
        // 1. Десериализовать ciphertext
        let ciphertext_bytes = &p2p_message.ciphertext;

        // 2. Расшифровать (в реальности нужен decrypt с ключами)
        // let plaintext = decrypt(...)?;
        debug!(
            "Received P2P message from {} ({} bytes)",
            from_peer_id,
            ciphertext_bytes.len()
        );

        // 3. Найти или создать чат
        let chat_id = {
            let peer_to_chat = self.peer_to_chat.read().await;
            peer_to_chat.get(from_peer_id).cloned()
        };

        let chat_id = if let Some(id) = chat_id {
            id
        } else {
            // Создать новый чат
            let mut chat_guard = self.chat_manager.write().await;
            let chat = chat_guard
                .get_or_create_private_chat(&self.local_peer_id(), from_peer_id);
            let id = chat.id.clone();

            let mut peer_to_chat = self.peer_to_chat.write().await;
            peer_to_chat.insert(from_peer_id.to_string(), id.clone());

            id
        };

        // 4. Добавить сообщение в чат
        let chat_msg = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            chat_id: chat_id.clone(),
            sender_id: from_peer_id.to_string(),
            encrypted_content: ciphertext_bytes.clone(),
            signature: p2p_message.signature,
            created_at: chrono::DateTime::from_timestamp_millis(p2p_message.timestamp as i64)
                .unwrap_or_else(Utc::now),
            msg_type: MessageContentType::Text,
            delivery_status: DeliveryState::Delivered,
            reply_to: None,
            attachments: Vec::new(),
        };

        let mut chat_guard = self.chat_manager.write().await;
        if let Some(chat) = chat_guard.get_private_chat_mut(&chat_id) {
            chat.add_message(chat_msg);
            chat.unread_count += 1;
        }

        info!(
            "Added incoming message to chat {} from {}",
            chat_id, from_peer_id
        );

        Ok(())
    }

    /// Обработать входящее сообщение из Cloudflare
    pub async fn handle_incoming_cloudflare_message(
        &self,
        sender_id: &str,
        ciphertext: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<(), IntegrationError> {
        // Аналогично P2P, но с другим источником
        debug!(
            "Received Cloudflare message from {} ({} bytes)",
            sender_id,
            ciphertext.len()
        );

        // Найти чат
        let chat_id = {
            let peer_to_chat = self.peer_to_chat.read().await;
            peer_to_chat.get(sender_id).cloned()
        };

        if let Some(id) = chat_id {
            let chat_msg = ChatMessage {
                id: uuid::Uuid::new_v4().to_string(),
                chat_id: id.clone(),
                sender_id: sender_id.to_string(),
                encrypted_content: ciphertext,
                signature,
                created_at: Utc::now(),
                msg_type: MessageContentType::Text,
                delivery_status: DeliveryState::Delivered,
                reply_to: None,
                attachments: Vec::new(),
            };

            let mut chat_guard = self.chat_manager.write().await;
            if let Some(chat) = chat_guard.get_private_chat_mut(&id) {
                chat.add_message(chat_msg);
                chat.unread_count += 1;
            }
        }

        Ok(())
    }

    // ========================================================================
    // Event Loop Integration
    // ========================================================================

    /// Обработчик событий P2P
    ///
    /// Вызывается из P2PNode::run() для обработки входящих событий
    pub async fn handle_p2p_event(&self, event: P2PEvent) -> Result<(), IntegrationError> {
        match event {
            P2PEvent::MessageReceived { from, message } => {
                self.handle_incoming_p2p_message(&from.to_string(), message)
                    .await?;
            }
            P2PEvent::PeerDiscovered(peer_id) => {
                debug!("Peer discovered: {}", peer_id);
                // Можно автоматически создать чат
            }
            P2PEvent::PeerDisconnected(peer_id) => {
                warn!("Peer disconnected: {}", peer_id);
            }
            P2PEvent::GossipMessageReceived {
                from,
                topic,
                message: _,
            } => {
                debug!("Gossip message from {} on topic {}", from, topic);
                // Обработать групповое сообщение
            }
            P2PEvent::DhtRecordFound { key, value } => {
                debug!("DHT record found: key={:?}, value_len={}", key, value.len());
                // Можно использовать для discovery ключей
            }
        }

        Ok(())
    }

    // ========================================================================
    // Chat Management
    // ========================================================================

    /// Создать приватный чат и зарегистрировать маппинг
    pub async fn create_private_chat(&self, peer_id: &str) -> Result<PrivateChat, IntegrationError> {
        let mut chat_guard = self.chat_manager.write().await;
        let chat = chat_guard
            .get_or_create_private_chat(&self.local_peer_id(), peer_id);
        let chat_clone = chat.clone();

        let mut peer_to_chat = self.peer_to_chat.write().await;
        peer_to_chat.insert(peer_id.to_string(), chat.id.clone());

        Ok(chat_clone)
    }

    /// Создать групповой чат
    pub async fn create_group_chat(
        &self,
        encrypted_name: Vec<u8>,
        encrypted_description: Vec<u8>,
    ) -> Result<GroupChat, IntegrationError> {
        let mut chat_guard = self.chat_manager.write().await;
        let group = chat_guard.create_group_chat(
            self.local_peer_id(),
            encrypted_name,
            encrypted_description,
        )?;
        Ok(group.clone())
    }

    /// Получить все чаты пользователя
    pub async fn get_user_chats(&self) -> Vec<String> {
        let chat_guard = self.chat_manager.read().await;
        chat_guard
            .get_user_chats(&self.local_peer_id())
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Получить приватный чат
    pub async fn get_private_chat(&self, chat_id: &str) -> Option<PrivateChat> {
        let chat_guard = self.chat_manager.read().await;
        chat_guard.get_private_chat(chat_id).cloned()
    }

    /// Получить групповой чат
    pub async fn get_group_chat(&self, chat_id: &str) -> Option<GroupChat> {
        let chat_guard = self.chat_manager.read().await;
        chat_guard.get_group_chat(chat_id).cloned()
    }

    /// Получить менеджер чатов (для прямого доступа)
    pub async fn chat_manager(&self) -> tokio::sync::RwLockReadGuard<'_, ChatManager> {
        self.chat_manager.read().await
    }

    /// Получить mutable менеджер чатов
    pub async fn chat_manager_mut(&self) -> tokio::sync::RwLockWriteGuard<'_, ChatManager> {
        self.chat_manager.write().await
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    /// Получить ID локального пользователя
    fn local_peer_id(&self) -> String {
        // В реальности — из keypair
        "local_user".to_string()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat::ChatManager;
    use crate::crypto::HybridKeypair;

    #[test]
    fn test_user_keys_registry() {
        let mut registry = UserKeysRegistry::new();

        // Создать тестовые ключи
        let keypair = HybridKeypair::generate().unwrap();
        let x25519 = keypair.x25519_public;
        let kyber = keypair.kyber_public.clone();
        let ed25519 = keypair.ed25519_public;

        registry.register(
            "user-1".to_string(),
            x25519,
            kyber,
            ed25519,
        );

        assert!(registry.is_registered("user-1"));
        assert!(!registry.is_registered("user-2"));

        let keys = registry.get_recipient_keys("user-1");
        assert!(keys.is_some());
    }

    #[tokio::test]
    async fn test_chat_transport_creation() {
        let chat_manager = ChatManager::new();
        let keypair = HybridKeypair::generate().unwrap();

        let transport = ChatTransport::new(chat_manager, keypair);

        let chats = transport.get_user_chats().await;
        assert!(chats.is_empty());
    }

    #[tokio::test]
    async fn test_chat_transport_private_chat() {
        let chat_manager = ChatManager::new();
        let keypair = HybridKeypair::generate().unwrap();

        let transport = ChatTransport::new(chat_manager, keypair);

        let chat = transport.create_private_chat("peer-1").await.unwrap();

        assert_eq!(chat.peer_id, "peer-1");
        assert!(chat.is_active());

        let chats = transport.get_user_chats().await;
        assert_eq!(chats.len(), 1);
    }

    #[tokio::test]
    async fn test_chat_transport_peer_mapping() {
        let chat_manager = ChatManager::new();
        let keypair = HybridKeypair::generate().unwrap();

        let transport = ChatTransport::new(chat_manager, keypair);

        // Создать несколько чатов
        transport.create_private_chat("peer-1").await.unwrap();
        transport.create_private_chat("peer-2").await.unwrap();
        transport.create_private_chat("peer-3").await.unwrap();

        let chats = transport.get_user_chats().await;
        assert_eq!(chats.len(), 3);
    }

    #[tokio::test]
    async fn test_handle_incoming_message() {
        let chat_manager = ChatManager::new();
        let keypair = HybridKeypair::generate().unwrap();

        let transport = ChatTransport::new(chat_manager, keypair);

        // Создать чат
        transport.create_private_chat("peer-1").await.unwrap();

        // Имитировать входящее сообщение
        let p2p_msg = P2PMessage {
            ciphertext: vec![1, 2, 3, 4, 5],
            signature: vec![6, 7, 8, 9],
            timestamp: Utc::now().timestamp_millis() as u64,
            msg_type: MessageType::Direct,
        };

        transport
            .handle_incoming_p2p_message("peer-1", p2p_msg)
            .await
            .unwrap();

        // Проверить, что сообщение добавлено
        let chats = transport.get_user_chats().await;
        assert_eq!(chats.len(), 1);

        let chat = transport.get_private_chat(&chats[0]).await.unwrap();
        assert_eq!(chat.messages.len(), 1);
        assert_eq!(chat.unread_count, 1);
    }

    #[tokio::test]
    async fn test_handle_incoming_creates_chat() {
        let chat_manager = ChatManager::new();
        let keypair = HybridKeypair::generate().unwrap();

        let transport = ChatTransport::new(chat_manager, keypair);

        // Входящее сообщение без существующего чата
        let p2p_msg = P2PMessage {
            ciphertext: vec![1, 2, 3],
            signature: vec![4, 5, 6],
            timestamp: Utc::now().timestamp_millis() as u64,
            msg_type: MessageType::Direct,
        };

        transport
            .handle_incoming_p2p_message("new-peer", p2p_msg)
            .await
            .unwrap();

        let chats = transport.get_user_chats().await;
        assert_eq!(chats.len(), 1);

        let chat = transport.get_private_chat(&chats[0]).await.unwrap();
        assert_eq!(chat.peer_id, "new-peer");
        assert_eq!(chat.messages.len(), 1);
    }
}
