//! Chat Module
//!
//! Типы чатов для мессенджера:
//! - **PrivateChat**: 1-on-1 зашифрованный чат
//! - **GroupChat**: Групповой чат с E2EE для всех участников
//! - **Channel**: Односторонний канал (broadcast)
//!
//! SECURITY: требует аудита перед production
//! TODO: pentest перед release

pub mod integration;

pub use integration::{ChatTransport, IntegrationError, UserKeysRegistry};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, info};
use uuid::Uuid;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum ChatError {
    #[error("Chat not found: {0}")]
    ChatNotFound(String),

    #[error("User not a member of chat: {0}")]
    NotMember(String),

    #[error("User already a member")]
    AlreadyMember,

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Message error: {0}")]
    MessageError(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

// ============================================================================
// Message Types
// ============================================================================

/// Сообщение в чате
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Уникальный ID сообщения
    pub id: String,
    /// ID чата
    pub chat_id: String,
    /// ID отправителя
    pub sender_id: String,
    /// Зашифрованный контент (ChaCha20-Poly1305)
    pub encrypted_content: Vec<u8>,
    /// Ed25519 подпись
    pub signature: Vec<u8>,
    /// Timestamp создания
    pub created_at: DateTime<Utc>,
    /// Тип сообщения
    pub msg_type: MessageContentType,
    /// Статус доставки
    pub delivery_status: DeliveryState,
    /// ID ответа (для reply)
    pub reply_to: Option<String>,
    /// Прикреплённые файлы (R2 keys)
    pub attachments: Vec<String>,
}

/// Тип контента сообщения
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageContentType {
    /// Зашифрованный текст
    Text,
    /// Зашифрованное изображение
    Image,
    /// Зашифрованный файл
    File,
    /// Зашифрованное видео
    Video,
    /// Зашифрованное аудио
    Audio,
    /// Системное сообщение (join/leave)
    System,
    /// Steganography (скрытое сообщение в изображении)
    Steganography,
}

/// Статус доставки сообщения
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeliveryState {
    /// Создаётся
    Draft,
    /// Отправляется
    Sending,
    /// Отправлено на сервер
    Sent,
    /// Доставлено получателю
    Delivered,
    /// Прочитано
    Read,
    /// Ошибка отправки
    Failed { error: String, retry_count: u32 },
}

// ============================================================================
// Private Chat (1-on-1)
// ============================================================================

/// Приватный чат между двумя пользователями
///
/// # Security Properties
/// - E2EE: X25519 + Kyber1024 + ChaCha20-Poly1305
/// - Аутентификация: Ed25519 подписи
/// - Perfect forward secrecy: ephemeral keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateChat {
    /// Уникальный ID чата
    pub id: String,
    /// ID первого участника
    pub user_id: String,
    /// ID второго участника
    pub peer_id: String,
    /// Сообщения (отсортированы по created_at)
    pub messages: Vec<ChatMessage>,
    /// Timestamp создания чата
    pub created_at: DateTime<Utc>,
    /// Timestamp последнего сообщения
    pub last_activity: DateTime<Utc>,
    /// Зашифрованный ключ чата (для локального хранения)
    pub encrypted_chat_key: Option<Vec<u8>>,
    /// Статус чата
    pub status: ChatStatus,
    /// Счётчик непрочитанных сообщений
    pub unread_count: u64,
    /// Последний прочитанный message_id
    pub last_read_message_id: Option<String>,
}

impl PrivateChat {
    /// Создать новый приватный чат
    pub fn new(user_id: String, peer_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            peer_id,
            messages: Vec::new(),
            created_at: now,
            last_activity: now,
            encrypted_chat_key: None,
            status: ChatStatus::Active,
            unread_count: 0,
            last_read_message_id: None,
        }
    }

    /// Добавить сообщение в чат
    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message.clone());
        self.last_activity = message.created_at;

        // Сортировать по timestamp
        self.messages
            .sort_by(|a, b| a.created_at.cmp(&b.created_at));

        debug!(
            "Added message to private chat {} (total: {})",
            self.id,
            self.messages.len()
        );
    }

    /// Получить последние N сообщений
    pub fn get_recent_messages(&self, limit: usize) -> Vec<&ChatMessage> {
        self.messages
            .iter()
            .rev()
            .take(limit)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Отметить сообщения как прочитанные
    pub fn mark_as_read(&mut self) {
        self.unread_count = 0;
        if let Some(last_msg) = self.messages.last() {
            self.last_read_message_id = Some(last_msg.id.clone());
        }
    }

    /// Получить ID собеседника
    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }

    /// Проверить, активен ли чат
    pub fn is_active(&self) -> bool {
        self.status == ChatStatus::Active
    }

    /// Удалить все сообщения (локально)
    pub fn clear_messages(&mut self) {
        self.messages.clear();
        info!("Cleared all messages from private chat {}", self.id);
    }
}

// ============================================================================
// Group Chat
// ============================================================================

/// Роль участника группового чата
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemberRole {
    /// Создатель/владелец группы
    Owner,
    /// Администратор (может добавлять/удалять)
    Admin,
    /// Обычный участник
    Member,
    /// Только чтение (забанен)
    ReadOnly,
}

/// Участник группового чата
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMember {
    /// ID участника
    pub user_id: String,
    /// Роль
    pub role: MemberRole,
    /// Timestamp присоединения
    pub joined_at: DateTime<Utc>,
    /// Публичные ключи (для E2EE)
    pub public_keys: Option<crate::crypto::PublicBundle>,
    /// Статус участника
    pub status: MemberStatus,
}

/// Статус участника
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemberStatus {
    /// Активный участник
    Active,
    /// Ожидает подтверждения
    Pending,
    /// Заблокирован
    Banned,
    /// Покинул группу
    Left,
}

/// Групповой чат с E2EE
///
/// # Security Properties
/// - Каждое сообщение шифруется для каждого участника
/// - Admin keys для управления доступом
/// - Member list verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupChat {
    /// Уникальный ID группы
    pub id: String,
    /// Название группы (зашифрованное)
    pub encrypted_name: Vec<u8>,
    /// Описание (зашифрованное)
    pub encrypted_description: Vec<u8>,
    /// Участники
    pub members: HashMap<String, GroupMember>,
    /// Сообщения
    pub messages: Vec<ChatMessage>,
    /// ID создателя
    pub creator_id: String,
    /// Timestamp создания
    pub created_at: DateTime<Utc>,
    /// Timestamp последней активности
    pub last_activity: DateTime<Utc>,
    /// Максимальное количество участников
    pub max_members: u32,
    /// Настройки группы
    pub settings: GroupSettings,
    /// Статус группы
    pub status: ChatStatus,
}

/// Настройки группового чата
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSettings {
    /// Только админы могут отправлять сообщения
    pub admins_only_messaging: bool,
    /// Требуется одобрение для вступления
    pub approval_required: bool,
    /// Разрешить вложения
    pub allow_attachments: bool,
    /// TTL сообщений (None = бессрочно)
    pub message_ttl: Option<u64>,
    /// Включить steganography
    pub enable_steganography: bool,
}

impl Default for GroupSettings {
    fn default() -> Self {
        Self {
            admins_only_messaging: false,
            approval_required: false,
            allow_attachments: true,
            message_ttl: None,
            enable_steganography: false,
        }
    }
}

impl GroupChat {
    /// Создать новую группу
    pub fn new(
        creator_id: String,
        encrypted_name: Vec<u8>,
        encrypted_description: Vec<u8>,
    ) -> Self {
        let now = Utc::now();
        let mut members = HashMap::new();

        // Создатель становится Owner
        members.insert(
            creator_id.clone(),
            GroupMember {
                user_id: creator_id.clone(),
                role: MemberRole::Owner,
                joined_at: now,
                public_keys: None,
                status: MemberStatus::Active,
            },
        );

        Self {
            id: Uuid::new_v4().to_string(),
            encrypted_name,
            encrypted_description,
            members,
            messages: Vec::new(),
            creator_id,
            created_at: now,
            last_activity: now,
            max_members: 256,
            settings: GroupSettings::default(),
            status: ChatStatus::Active,
        }
    }

    /// Добавить участника
    pub fn add_member(
        &mut self,
        user_id: String,
        role: MemberRole,
        public_keys: Option<crate::crypto::PublicBundle>,
    ) -> Result<(), ChatError> {
        if self.members.contains_key(&user_id) {
            return Err(ChatError::AlreadyMember);
        }

        if self.members.len() >= self.max_members as usize {
            return Err(ChatError::MessageError("Group is full".to_string()));
        }

        self.members.insert(
            user_id.clone(),
            GroupMember {
                user_id,
                role,
                joined_at: Utc::now(),
                public_keys,
                status: MemberStatus::Active,
            },
        );

        info!(
            "Added member to group {} (total: {})",
            self.id,
            self.members.len()
        );

        Ok(())
    }

    /// Удалить участника
    pub fn remove_member(&mut self, user_id: &str) -> Result<(), ChatError> {
        if !self.members.contains_key(user_id) {
            return Err(ChatError::NotMember(user_id.to_string()));
        }

        // Нельзя удалить Owner
        if let Some(member) = self.members.get(user_id) {
            if member.role == MemberRole::Owner {
                return Err(ChatError::PermissionDenied(
                    "Cannot remove group owner".to_string(),
                ));
            }
        }

        self.members.remove(user_id);
        info!(
            "Removed member from group {} (total: {})",
            self.id,
            self.members.len()
        );

        Ok(())
    }

    /// Обновить роль участника
    pub fn update_member_role(
        &mut self,
        user_id: &str,
        new_role: MemberRole,
    ) -> Result<(), ChatError> {
        let member = self
            .members
            .get_mut(user_id)
            .ok_or_else(|| ChatError::NotMember(user_id.to_string()))?;

        // Owner нельзя понизить
        if member.role == MemberRole::Owner && new_role != MemberRole::Owner {
            return Err(ChatError::PermissionDenied(
                "Cannot change owner role".to_string(),
            ));
        }

        member.role = new_role;

        Ok(())
    }

    /// Добавить сообщение
    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message.clone());
        self.last_activity = message.created_at;
        self.messages
            .sort_by(|a, b| a.created_at.cmp(&b.created_at));
    }

    /// Получить список активныхных участников
    pub fn active_members(&self) -> Vec<&GroupMember> {
        self.members
            .values()
            .filter(|m| m.status == MemberStatus::Active)
            .collect()
    }

    /// Получить администраторов
    pub fn admins(&self) -> Vec<&GroupMember> {
        self.members
            .values()
            .filter(|m| matches!(m.role, MemberRole::Owner | MemberRole::Admin))
            .collect()
    }

    /// Проверить, является ли пользователь администратором
    pub fn is_admin(&self, user_id: &str) -> bool {
        self.members
            .get(user_id)
            .map(|m| matches!(m.role, MemberRole::Owner | MemberRole::Admin))
            .unwrap_or(false)
    }

    /// Проверить, может ли пользователь отправлять сообщения
    pub fn can_send_message(&self, user_id: &str) -> bool {
        if let Some(member) = self.members.get(user_id) {
            if member.status != MemberStatus::Active {
                return false;
            }

            if member.status == MemberStatus::Banned {
                return false;
            }

            if self.settings.admins_only_messaging && !self.is_admin(user_id) {
                return false;
            }

            true
        } else {
            false
        }
    }

    /// Получить количество участников
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// Очистить старые сообщения (по TTL)
    pub fn cleanup_expired_messages(&mut self) -> usize {
        if let Some(ttl_seconds) = self.settings.message_ttl {
            let cutoff = Utc::now() - chrono::Duration::seconds(ttl_seconds as i64);
            let before = self.messages.len();
            self.messages.retain(|m| m.created_at >= cutoff);
            let removed = before - self.messages.len();
            if removed > 0 {
                info!(
                    "Cleaned up {} expired messages from group {}",
                    removed, self.id
                );
            }
            removed
        } else {
            0
        }
    }
}

// ============================================================================
// Channel (Broadcast)
// ============================================================================

/// Односторонний канал (broadcast)
///
/// Только админы/создатель могут публиковать сообщения.
/// Подписчики только читают.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    /// Уникальный ID канала
    pub id: String,
    /// Название канала (зашифрованное)
    pub encrypted_name: Vec<u8>,
    /// Описание (зашифрованное)
    pub encrypted_description: Vec<u8>,
    /// Администраторы (могут публиковать)
    pub admins: Vec<String>,
    /// Подписчики
    pub subscribers: Vec<String>,
    /// Сообщения
    pub messages: Vec<ChatMessage>,
    /// ID создателя
    pub creator_id: String,
    /// Timestamp создания
    pub created_at: DateTime<Utc>,
    /// Timestamp последней публикации
    pub last_activity: DateTime<Utc>,
    /// Статус канала
    pub status: ChatStatus,
    /// Настройки канала
    pub settings: ChannelSettings,
}

/// Настройки канала
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSettings {
    /// Разрешить комментарии подписчиков
    pub allow_comments: bool,
    /// Уведомлять подписчиков о новых сообщениях
    pub notify_on_publish: bool,
    /// Публичный канал (виден всем)
    pub is_public: bool,
    /// Максимальное количество подписчиков (None = без лимита)
    pub max_subscribers: Option<u32>,
}

impl Default for ChannelSettings {
    fn default() -> Self {
        Self {
            allow_comments: false,
            notify_on_publish: true,
            is_public: true,
            max_subscribers: None,
        }
    }
}

impl Channel {
    /// Создать новый канал
    pub fn new(
        creator_id: String,
        encrypted_name: Vec<u8>,
        encrypted_description: Vec<u8>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            encrypted_name,
            encrypted_description,
            admins: vec![creator_id.clone()],
            subscribers: Vec::new(),
            messages: Vec::new(),
            creator_id,
            created_at: now,
            last_activity: now,
            status: ChatStatus::Active,
            settings: ChannelSettings::default(),
        }
    }

    /// Опубликовать сообщение (только для админов)
    pub fn publish_message(
        &mut self,
        sender_id: &str,
        message: ChatMessage,
    ) -> Result<(), ChatError> {
        // Проверить права
        if !self.admins.contains(&sender_id.to_string()) {
            return Err(ChatError::PermissionDenied(
                "Only channel admins can publish messages".to_string(),
            ));
        }

        self.messages.push(message.clone());
        self.last_activity = message.created_at;
        self.messages
            .sort_by(|a, b| a.created_at.cmp(&b.created_at));

        info!(
            "Published message to channel {} (total: {})",
            self.id,
            self.messages.len()
        );

        Ok(())
    }

    /// Подписаться на канал
    pub fn subscribe(&mut self, user_id: String) -> Result<(), ChatError> {
        if self.subscribers.contains(&user_id) {
            return Err(ChatError::MessageError("Already subscribed".to_string()));
        }

        // Проверить лимит
        if let Some(max) = self.settings.max_subscribers {
            if self.subscribers.len() >= max as usize {
                return Err(ChatError::MessageError(
                    "Channel subscriber limit reached".to_string(),
                ));
            }
        }

        self.subscribers.push(user_id.clone());
        info!(
            "User {} subscribed to channel {} (total: {})",
            user_id,
            self.id,
            self.subscribers.len()
        );

        Ok(())
    }

    /// Отписаться от канала
    pub fn unsubscribe(&mut self, user_id: &str) -> Result<(), ChatError> {
        let pos = self
            .subscribers
            .iter()
            .position(|s| s == user_id)
            .ok_or_else(|| ChatError::MessageError("Not a subscriber".to_string()))?;

        self.subscribers.remove(pos);
        info!(
            "User {} unsubscribed from channel {} (total: {})",
            user_id,
            self.id,
            self.subscribers.len()
        );

        Ok(())
    }

    /// Добавить администратора
    pub fn add_admin(&mut self, user_id: String) -> Result<(), ChatError> {
        if self.admins.contains(&user_id) {
            return Err(ChatError::MessageError("Already an admin".to_string()));
        }

        self.admins.push(user_id.clone());
        info!("User {} added as channel admin", user_id);

        Ok(())
    }

    /// Удалить администратора
    pub fn remove_admin(&mut self, user_id: &str) -> Result<(), ChatError> {
        // Нельзя удалить создателя
        if user_id == self.creator_id {
            return Err(ChatError::PermissionDenied(
                "Cannot remove channel creator".to_string(),
            ));
        }

        let pos = self
            .admins
            .iter()
            .position(|a| a == user_id)
            .ok_or_else(|| ChatError::MessageError("Not an admin".to_string()))?;

        self.admins.remove(pos);
        Ok(())
    }

    /// Получить количество подписчиков
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.len()
    }

    /// Проверить, подписан ли пользователь
    pub fn is_subscriber(&self, user_id: &str) -> bool {
        self.subscribers.contains(&user_id.to_string())
    }

    /// Получить последние публикации
    pub fn get_recent_posts(&self, limit: usize) -> Vec<&ChatMessage> {
        self.messages
            .iter()
            .rev()
            .take(limit)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }
}

// ============================================================================
// Chat Status
// ============================================================================

/// Общий статус чата
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatStatus {
    /// Активный чат
    Active,
    /// Заархивирован
    Archived,
    /// Удалён (soft delete)
    Deleted,
    /// Заблокирован
    Banned,
}

// ============================================================================
// Chat Manager
// ============================================================================

/// Менеджер чатов — управляет всеми типами чатов
pub struct ChatManager {
    /// Приватные чаты (chat_id -> PrivateChat)
    private_chats: HashMap<String, PrivateChat>,
    /// Групповые чаты (chat_id -> GroupChat)
    group_chats: HashMap<String, GroupChat>,
    /// Каналы (channel_id -> Channel)
    channels: HashMap<String, Channel>,
    /// Индекс: user_id -> list of chat_ids
    user_chat_index: HashMap<String, Vec<String>>,
}

impl ChatManager {
    /// Создать новый менеджер чатов
    pub fn new() -> Self {
        Self {
            private_chats: HashMap::new(),
            group_chats: HashMap::new(),
            channels: HashMap::new(),
            user_chat_index: HashMap::new(),
        }
    }

    // ========================================================================
    // Private Chat Operations
    // ========================================================================

    /// Создать или получить приватный чат
    pub fn get_or_create_private_chat(&mut self, user_id: &str, peer_id: &str) -> &PrivateChat {
        // Поиск существующего чата
        let chat_key = Self::private_chat_key(user_id, peer_id);

        if !self.private_chats.contains_key(&chat_key) {
            let chat = PrivateChat::new(user_id.to_string(), peer_id.to_string());
            let chat_id = chat.id.clone();
            self.private_chats.insert(chat_key.clone(), chat);

            // Обновить индекс
            self.user_chat_index
                .entry(user_id.to_string())
                .or_default()
                .push(chat_id);
        }

        self.private_chats.get(&chat_key).unwrap()
    }

    /// Получить приватный чат по ID
    pub fn get_private_chat(&self, chat_id: &str) -> Option<&PrivateChat> {
        self.private_chats.values().find(|c| c.id == chat_id)
    }

    /// Получить приватный чат mutable
    pub fn get_private_chat_mut(&mut self, chat_id: &str) -> Option<&mut PrivateChat> {
        self.private_chats.values_mut().find(|c| c.id == chat_id)
    }

    // ========================================================================
    // Group Chat Operations
    // ========================================================================

    /// Создать групповой чат
    pub fn create_group_chat(
        &mut self,
        creator_id: String,
        encrypted_name: Vec<u8>,
        encrypted_description: Vec<u8>,
    ) -> Result<&GroupChat, ChatError> {
        let group = GroupChat::new(creator_id.clone(), encrypted_name, encrypted_description);
        let chat_id = group.id.clone();

        self.group_chats.insert(chat_id.clone(), group);

        // Обновить индекс
        self.user_chat_index
            .entry(creator_id)
            .or_default()
            .push(chat_id.clone());

        Ok(self.group_chats.values().find(|c| c.id == chat_id).unwrap())
    }

    /// Получить групповой чат
    pub fn get_group_chat(&self, chat_id: &str) -> Option<&GroupChat> {
        self.group_chats.values().find(|c| c.id == chat_id)
    }

    /// Получить групповой чат mutable
    pub fn get_group_chat_mut(&mut self, chat_id: &str) -> Option<&mut GroupChat> {
        self.group_chats.values_mut().find(|c| c.id == chat_id)
    }

    // ========================================================================
    // Channel Operations
    // ========================================================================

    /// Создать канал
    pub fn create_channel(
        &mut self,
        creator_id: String,
        encrypted_name: Vec<u8>,
        encrypted_description: Vec<u8>,
    ) -> Result<&Channel, ChatError> {
        let channel = Channel::new(creator_id.clone(), encrypted_name, encrypted_description);
        let channel_id = channel.id.clone();

        self.channels.insert(channel_id.clone(), channel);

        // Обновить индекс
        self.user_chat_index
            .entry(creator_id)
            .or_default()
            .push(channel_id.clone());

        Ok(self.channels.values().find(|c| c.id == channel_id).unwrap())
    }

    /// Получить канал
    pub fn get_channel(&self, channel_id: &str) -> Option<&Channel> {
        self.channels.values().find(|c| c.id == channel_id)
    }

    /// Получить канал mutable
    pub fn get_channel_mut(&mut self, channel_id: &str) -> Option<&mut Channel> {
        self.channels.values_mut().find(|c| c.id == channel_id)
    }

    // ========================================================================
    // User Operations
    // ========================================================================

    /// Получить все чаты пользователя
    pub fn get_user_chats(&self, user_id: &str) -> Vec<&str> {
        self.user_chat_index
            .get(user_id)
            .map(|ids| ids.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Удалить чат (soft delete)
    pub fn delete_chat(&mut self, chat_id: &str) -> Result<(), ChatError> {
        // Попробовать удалить из каждого типа
        if let Some(chat) = self.private_chats.values_mut().find(|c| c.id == chat_id) {
            chat.status = ChatStatus::Deleted;
            return Ok(());
        }

        if let Some(group) = self.group_chats.values_mut().find(|c| c.id == chat_id) {
            group.status = ChatStatus::Deleted;
            return Ok(());
        }

        if let Some(channel) = self.channels.values_mut().find(|c| c.id == chat_id) {
            channel.status = ChatStatus::Deleted;
            return Ok(());
        }

        Err(ChatError::ChatNotFound(chat_id.to_string()))
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    /// Сгенерировать ключ для приватного чата (сортированный)
    fn private_chat_key(user_id: &str, peer_id: &str) -> String {
        let mut ids = vec![user_id, peer_id];
        ids.sort();
        format!("private_{}_{}", ids[0], ids[1])
    }

    /// Получить общую статистику
    pub fn get_stats(&self) -> ChatManagerStats {
        ChatManagerStats {
            private_chats: self.private_chats.len(),
            group_chats: self.group_chats.len(),
            channels: self.channels.len(),
            total_messages: self
                .private_chats
                .values()
                .map(|c| c.messages.len())
                .sum::<usize>()
                + self
                    .group_chats
                    .values()
                    .map(|c| c.messages.len())
                    .sum::<usize>()
                + self
                    .channels
                    .values()
                    .map(|c| c.messages.len())
                    .sum::<usize>(),
        }
    }
}

impl Default for ChatManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Статистика менеджера чатов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatManagerStats {
    pub private_chats: usize,
    pub group_chats: usize,
    pub channels: usize,
    pub total_messages: usize,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_chat_creation() {
        let chat = PrivateChat::new("user-1".to_string(), "user-2".to_string());

        assert_eq!(chat.user_id, "user-1");
        assert_eq!(chat.peer_id, "user-2");
        assert!(chat.messages.is_empty());
        assert!(chat.is_active());
    }

    #[test]
    fn test_private_chat_add_message() {
        let mut chat = PrivateChat::new("user-1".to_string(), "user-2".to_string());

        let msg = ChatMessage {
            id: "msg-1".to_string(),
            chat_id: chat.id.clone(),
            sender_id: "user-1".to_string(),
            encrypted_content: vec![1, 2, 3],
            signature: vec![4, 5, 6],
            created_at: Utc::now(),
            msg_type: MessageContentType::Text,
            delivery_status: DeliveryState::Sent,
            reply_to: None,
            attachments: Vec::new(),
        };

        chat.add_message(msg);
        assert_eq!(chat.messages.len(), 1);
        assert_eq!(chat.last_read_message_id, None);
    }

    #[test]
    fn test_private_chat_mark_as_read() {
        let mut chat = PrivateChat::new("user-1".to_string(), "user-2".to_string());
        chat.unread_count = 5;

        chat.mark_as_read();
        assert_eq!(chat.unread_count, 0);
    }

    #[test]
    fn test_group_chat_creation() {
        let group = GroupChat::new("creator".to_string(), vec![1, 2, 3], vec![4, 5, 6]);

        assert_eq!(group.members.len(), 1);
        assert!(group.is_admin("creator"));
        assert_eq!(group.member_count(), 1);
    }

    #[test]
    fn test_group_chat_add_member() {
        let mut group = GroupChat::new("creator".to_string(), vec![1, 2, 3], vec![4, 5, 6]);

        group
            .add_member("user-2".to_string(), MemberRole::Member, None)
            .unwrap();

        assert_eq!(group.member_count(), 2);
        assert!(!group.is_admin("user-2"));
    }

    #[test]
    fn test_group_chat_duplicate_member() {
        let mut group = GroupChat::new("creator".to_string(), vec![1, 2, 3], vec![4, 5, 6]);

        group
            .add_member("user-2".to_string(), MemberRole::Member, None)
            .unwrap();

        let result = group.add_member("user-2".to_string(), MemberRole::Member, None);

        assert!(result.is_err());
    }

    #[test]
    fn test_group_chat_remove_member() {
        let mut group = GroupChat::new("creator".to_string(), vec![1, 2, 3], vec![4, 5, 6]);

        group
            .add_member("user-2".to_string(), MemberRole::Member, None)
            .unwrap();

        group.remove_member("user-2").unwrap();
        assert_eq!(group.member_count(), 1);
    }

    #[test]
    fn test_group_chat_cannot_remove_owner() {
        let mut group = GroupChat::new("creator".to_string(), vec![1, 2, 3], vec![4, 5, 6]);

        let result = group.remove_member("creator");
        assert!(result.is_err());
    }

    #[test]
    fn test_group_chat_can_send_messages() {
        let mut group = GroupChat::new("creator".to_string(), vec![1, 2, 3], vec![4, 5, 6]);

        group
            .add_member("user-2".to_string(), MemberRole::Member, None)
            .unwrap();

        assert!(group.can_send_message("creator"));
        assert!(group.can_send_message("user-2"));

        // Включить admins_only_messaging
        group.settings.admins_only_messaging = true;
        assert!(group.can_send_message("creator"));
        assert!(!group.can_send_message("user-2"));
    }

    #[test]
    fn test_channel_creation() {
        let channel = Channel::new("creator".to_string(), vec![1, 2, 3], vec![4, 5, 6]);

        assert_eq!(channel.admins.len(), 1);
        assert_eq!(channel.subscriber_count(), 0);
        assert!(channel.admins.contains(&"creator".to_string()));
    }

    #[test]
    fn test_channel_publish_message() {
        let mut channel = Channel::new("creator".to_string(), vec![1, 2, 3], vec![4, 5, 6]);

        let msg = ChatMessage {
            id: "msg-1".to_string(),
            chat_id: channel.id.clone(),
            sender_id: "creator".to_string(),
            encrypted_content: vec![1, 2, 3],
            signature: vec![4, 5, 6],
            created_at: Utc::now(),
            msg_type: MessageContentType::Text,
            delivery_status: DeliveryState::Sent,
            reply_to: None,
            attachments: Vec::new(),
        };

        channel.publish_message("creator", msg).unwrap();
        assert_eq!(channel.messages.len(), 1);
    }

    #[test]
    fn test_channel_non_admin_cannot_publish() {
        let mut channel = Channel::new("creator".to_string(), vec![1, 2, 3], vec![4, 5, 6]);

        let msg = ChatMessage {
            id: "msg-1".to_string(),
            chat_id: channel.id.clone(),
            sender_id: "user-2".to_string(),
            encrypted_content: vec![1, 2, 3],
            signature: vec![4, 5, 6],
            created_at: Utc::now(),
            msg_type: MessageContentType::Text,
            delivery_status: DeliveryState::Sent,
            reply_to: None,
            attachments: Vec::new(),
        };

        let result = channel.publish_message("user-2", msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_channel_subscribe() {
        let mut channel = Channel::new("creator".to_string(), vec![1, 2, 3], vec![4, 5, 6]);

        channel.subscribe("user-2".to_string()).unwrap();
        assert_eq!(channel.subscriber_count(), 1);
        assert!(channel.is_subscriber("user-2"));
    }

    #[test]
    fn test_channel_unsubscribe() {
        let mut channel = Channel::new("creator".to_string(), vec![1, 2, 3], vec![4, 5, 6]);

        channel.subscribe("user-2".to_string()).unwrap();
        channel.unsubscribe("user-2").unwrap();
        assert_eq!(channel.subscriber_count(), 0);
    }

    #[test]
    fn test_chat_manager_stats() {
        let mut manager = ChatManager::new();

        manager.get_or_create_private_chat("user-1", "user-2");
        manager
            .create_group_chat("user-1".to_string(), vec![1, 2, 3], vec![4, 5, 6])
            .unwrap();

        let stats = manager.get_stats();
        assert_eq!(stats.private_chats, 1);
        assert_eq!(stats.group_chats, 1);
        assert_eq!(stats.channels, 0);
    }

    #[test]
    fn test_chat_manager_delete_chat() {
        let mut manager = ChatManager::new();
        let chat = manager.get_or_create_private_chat("user-1", "user-2");
        let chat_id = chat.id.clone();

        manager.delete_chat(&chat_id).unwrap();

        let chat = manager.get_private_chat(&chat_id).unwrap();
        assert_eq!(chat.status, ChatStatus::Deleted);
    }

    #[test]
    fn test_message_content_type_serialization() {
        let types = vec![
            MessageContentType::Text,
            MessageContentType::Image,
            MessageContentType::File,
            MessageContentType::Video,
            MessageContentType::Audio,
            MessageContentType::System,
            MessageContentType::Steganography,
        ];

        for msg_type in types {
            let json = serde_json::to_string(&msg_type).unwrap();
            let deserialized: MessageContentType = serde_json::from_str(&json).unwrap();
            assert_eq!(msg_type, deserialized);
        }
    }

    #[test]
    fn test_delivery_state_serialization() {
        let states = vec![
            DeliveryState::Draft,
            DeliveryState::Sending,
            DeliveryState::Sent,
            DeliveryState::Delivered,
            DeliveryState::Read,
            DeliveryState::Failed {
                error: "timeout".to_string(),
                retry_count: 2,
            },
        ];

        for state in states {
            let json = serde_json::to_string(&state).unwrap();
            let deserialized: DeliveryState = serde_json::from_str(&json).unwrap();
            assert_eq!(format!("{:?}", state), format!("{:?}", deserialized));
        }
    }
}
