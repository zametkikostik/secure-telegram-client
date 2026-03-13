//! Модуль групповых чатов
//!
//! Реализует:
//! - Структура Group с участниками и правами
//! - Динамические Gossipsub топики: liberty-group-[ID]
//! - Роли: Admin, Moderator, Member
//! - Шифрование сообщений для группы

use anyhow::{Result, Context};
use libp2p::gossipsub::{IdentTopic, TopicHash};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::{Aead, Key};
use rand::RngCore;

/// Роль участника группы
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GroupRole {
    /// Создатель/владелец группы
    Owner,
    /// Администратор
    Admin,
    /// Модератор
    Moderator,
    /// Обычный участник
    Member,
}

impl GroupRole {
    /// Проверка прав на добавление участников
    pub fn can_invite(&self) -> bool {
        matches!(self, GroupRole::Owner | GroupRole::Admin | GroupRole::Moderator)
    }

    /// Проверка прав на удаление участников
    pub fn can_kick(&self) -> bool {
        matches!(self, GroupRole::Owner | GroupRole::Admin | GroupRole::Moderator)
    }

    /// Проверка прав на изменение настроек группы
    pub fn can_edit_settings(&self) -> bool {
        matches!(self, GroupRole::Owner | GroupRole::Admin)
    }
}

/// Участник группы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMember {
    /// Peer ID участника
    pub peer_id: String,
    /// Роль
    pub role: GroupRole,
    /// Дата вступления
    pub joined_at: u64,
    /// Публичный ключ для шифрования
    pub public_key: Vec<u8>,
}

/// Настройки группы
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSettings {
    /// Название группы
    pub name: String,
    /// Описание
    pub description: Option<String>,
    /// Аватар (CID в IPFS)
    pub avatar_cid: Option<String>,
    /// Максимальное количество участников
    pub max_members: u32,
    /// Только админы могут писать (режим объявлений)
    pub admins_only: bool,
    /// История чата видна новым участникам
    pub history_visible: bool,
}

impl Default for GroupSettings {
    fn default() -> Self {
        Self {
            name: "Новая группа".to_string(),
            description: None,
            avatar_cid: None,
            max_members: 100,
            admins_only: false,
            history_visible: false,
        }
    }
}

/// Групповой чат
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    /// Уникальный ID группы
    pub id: String,
    /// ID создателя
    pub creator_peer_id: String,
    /// Участники
    pub members: HashMap<String, GroupMember>,
    /// Настройки
    pub settings: GroupSettings,
    /// Дата создания
    pub created_at: u64,
    /// Ключ шифрования группы (AES-256)
    /// В реальном использовании должен храниться зашифрованным
    pub encryption_key: Option<Vec<u8>>,
}

impl Group {
    /// Создание новой группы
    pub fn new(creator_peer_id: &str, name: &str) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Генерация ключа шифрования для группы
        let mut key_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key_bytes);

        let mut members = HashMap::new();
        members.insert(
            creator_peer_id.to_string(),
            GroupMember {
                peer_id: creator_peer_id.to_string(),
                role: GroupRole::Owner,
                joined_at: timestamp,
                public_key: vec![],
            },
        );

        Self {
            id,
            creator_peer_id: creator_peer_id.to_string(),
            members,
            settings: GroupSettings {
                name: name.to_string(),
                ..Default::default()
            },
            created_at: timestamp,
            encryption_key: Some(key_bytes.to_vec()),
        }
    }

    /// Получение Gossipsub топика для группы
    pub fn topic(&self) -> IdentTopic {
        IdentTopic::new(format!("liberty-group-{}", self.id))
    }

    /// Добавление участника
    pub fn add_member(&mut self, peer_id: &str, role: GroupRole, public_key: Vec<u8>) -> Result<()> {
        if self.members.len() >= self.settings.max_members as usize {
            anyhow::bail!("Достигнуто максимальное количество участников");
        }

        if self.members.contains_key(peer_id) {
            anyhow::bail!("Участник уже в группе");
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.members.insert(
            peer_id.to_string(),
            GroupMember {
                peer_id: peer_id.to_string(),
                role,
                joined_at: timestamp,
                public_key,
            },
        );

        Ok(())
    }

    /// Удаление участника
    pub fn remove_member(&mut self, peer_id: &str) -> Result<()> {
        if !self.members.contains_key(peer_id) {
            anyhow::bail!("Участник не найден");
        }

        // Нельзя удалить владельца
        if let Some(member) = self.members.get(peer_id) {
            if member.role == GroupRole::Owner {
                anyhow::bail!("Нельзя удалить создателя группы");
            }
        }

        self.members.remove(peer_id);
        Ok(())
    }

    /// Изменение роли участника
    pub fn change_role(&mut self, peer_id: &str, new_role: GroupRole) -> Result<()> {
        if let Some(member) = self.members.get_mut(peer_id) {
            // Нельзя изменить роль владельца
            if member.role == GroupRole::Owner {
                anyhow::bail!("Нельзя изменить роль создателя группы");
            }
            member.role = new_role;
            Ok(())
        } else {
            anyhow::bail!("Участник не найден")
        }
    }

    /// Проверка прав участника
    pub fn check_permission(&self, peer_id: &str, permission: &GroupRole) -> bool {
        if let Some(member) = self.members.get(peer_id) {
            match (&member.role, permission) {
                // Owner имеет все права
                (GroupRole::Owner, _) => true,
                // Admin может всё кроме передачи Owner
                (GroupRole::Admin, GroupRole::Owner) => false,
                (GroupRole::Admin, _) => true,
                // Moderator может приглашать и кикать
                (GroupRole::Moderator, GroupRole::Member) => true,
                (GroupRole::Moderator, _) => false,
                // Member не имеет специальных прав
                (GroupRole::Member, _) => false,
            }
        } else {
            false
        }
    }

    /// Получение списка участников
    pub fn get_members(&self) -> Vec<&GroupMember> {
        self.members.values().collect()
    }

    /// Получение количества участников
    pub fn member_count(&self) -> usize {
        self.members.len()
    }

    /// Проверка, является ли peer участником группы
    pub fn is_member(&self, peer_id: &str) -> bool {
        self.members.contains_key(peer_id)
    }
}

/// Менеджер групп
pub struct GroupManager {
    /// Локальный Peer ID
    local_peer_id: String,
    /// Группы
    groups: HashMap<String, Group>,
    /// Кэш ключей шифрования для групп
    group_keys: HashMap<String, Aes256Gcm>,
}

impl GroupManager {
    /// Создание нового менеджера групп
    pub fn new(local_peer_id: &str) -> Self {
        Self {
            local_peer_id: local_peer_id.to_string(),
            groups: HashMap::new(),
            group_keys: HashMap::new(),
        }
    }

    /// Создание новой группы
    pub fn create_group(&mut self, name: &str) -> &Group {
        let mut group = Group::new(&self.local_peer_id, name);
        let group_id = group.id.clone();

        // Сохранение ключа шифрования
        if let Some(key_bytes) = &group.encryption_key {
            let cipher = Aes256Gcm::new_from_slice(key_bytes)
                .expect("Invalid key length");
            self.group_keys.insert(group_id.clone(), cipher);
        }

        self.groups.insert(group_id.clone(), group);
        self.groups.get(&group_id).unwrap()
    }

    /// Получение группы по ID
    pub fn get_group(&self, group_id: &str) -> Option<&Group> {
        self.groups.get(group_id)
    }

    /// Получение группы для записи
    pub fn get_group_mut(&mut self, group_id: &str) -> Option<&mut Group> {
        self.groups.get_mut(group_id)
    }

    /// Присоединение к группе
    pub fn join_group(&mut self, group: Group) -> Result<()> {
        if !group.is_member(&self.local_peer_id) {
            anyhow::bail!("Вы не являетесь участником этой группы");
        }

        let group_id = group.id.clone();

        // Сохранение ключа шифрования
        if let Some(key_bytes) = &group.encryption_key {
            let cipher = Aes256Gcm::new_from_slice(key_bytes)
                .expect("Invalid key length");
            self.group_keys.insert(group_id.clone(), cipher);
        }

        self.groups.insert(group_id, group);
        Ok(())
    }

    /// Выход из группы
    pub fn leave_group(&mut self, group_id: &str) -> Result<()> {
        if let Some(group) = self.groups.get(group_id) {
            if group.members.get(&self.local_peer_id).map(|m| m.role.clone()) == Some(GroupRole::Owner) {
                anyhow::bail!("Владелец не может покинуть группу. Передайте права другому участнику.");
            }
        }

        self.groups.remove(group_id);
        self.group_keys.remove(group_id);
        Ok(())
    }

    /// Шифрование сообщения для группы
    pub fn encrypt_group_message(&self, group_id: &str, message: &[u8]) -> Result<Vec<u8>> {
        let cipher = self.group_keys.get(group_id)
            .ok_or_else(|| anyhow::anyhow!("Ключ шифрования для группы не найден"))?;

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let encrypted = cipher.encrypt(nonce, message)
            .map_err(|e| anyhow::anyhow!("Ошибка шифрования: {}", e))?;

        // nonce + encrypted data
        let mut result = Vec::new();
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&encrypted);

        Ok(result)
    }

    /// Дешифрование сообщения группы
    pub fn decrypt_group_message(&self, group_id: &str, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        if encrypted_data.len() < 12 {
            anyhow::bail!("Слишком короткие данные для дешифрования");
        }

        let cipher = self.group_keys.get(group_id)
            .ok_or_else(|| anyhow::anyhow!("Ключ шифрования для группы не найден"))?;

        let nonce_bytes = &encrypted_data[0..12];
        let data = &encrypted_data[12..];

        let nonce = Nonce::from_slice(nonce_bytes);
        let decrypted = cipher.decrypt(nonce, data)
            .map_err(|e| anyhow::anyhow!("Ошибка дешифрования: {}", e))?;

        Ok(decrypted)
    }

    /// Получение списка всех групп
    pub fn get_all_groups(&self) -> Vec<&Group> {
        self.groups.values().collect()
    }

    /// Получение списка ID групп
    pub fn get_group_ids(&self) -> Vec<String> {
        self.groups.keys().cloned().collect()
    }

    /// Получение топиков для всех групп
    pub fn get_all_topics(&self) -> Vec<IdentTopic> {
        self.groups.values().map(|g| g.topic()).collect()
    }
}

/// Сообщение группового чата
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMessage {
    /// ID группы
    pub group_id: String,
    /// Отправитель
    pub sender_peer_id: String,
    /// Текст сообщения
    pub text: String,
    /// Временная метка
    pub timestamp: u64,
    /// Тип сообщения
    pub message_type: GroupMessageType,
}

/// Тип сообщения группы
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GroupMessageType {
    /// Текстовое сообщение
    Text,
    /// Голосовое сообщение
    Voice,
    /// Файл
    File,
    /// Системное (вступление, выход)
    System,
}

/// События группы
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GroupEvent {
    /// Новый участник
    MemberJoined {
        group_id: String,
        peer_id: String,
    },
    /// Участник покинул
    MemberLeft {
        group_id: String,
        peer_id: String,
    },
    /// Участник удалён
    MemberKicked {
        group_id: String,
        peer_id: String,
        kicked_by: String,
    },
    /// Настройки изменены
    SettingsChanged {
        group_id: String,
        changed_by: String,
    },
    /// Новое сообщение
    NewMessage(GroupMessage),
}

/// Команды менеджера групп
pub const GROUP_COMMANDS: &[(&str, &str)] = &[
    ("/group create [name]", "Создать новую группу"),
    ("/group join [id]", "Присоединиться к группе"),
    ("/group leave [id]", "Покинуть группу"),
    ("/group invite [peer_id] [group_id]", "Пригласить участника"),
    ("/group kick [peer_id] [group_id]", "Удалить участника"),
    ("/group list", "Показать все группы"),
    ("/group info [id]", "Информация о группе"),
    ("/group settings [id] [key] [value]", "Изменить настройки"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_creation() {
        let mut manager = GroupManager::new("creator_123");
        let group = manager.create_group("Test Group");

        assert_eq!(group.settings.name, "Test Group");
        assert_eq!(group.creator_peer_id, "creator_123");
        assert_eq!(group.member_count(), 1);
        assert!(group.is_member("creator_123"));
    }

    #[test]
    fn test_group_add_member() {
        let mut manager = GroupManager::new("creator_123");
        let group = manager.create_group("Test Group");
        let group_id = group.id.clone();

        let group_mut = manager.get_group_mut(&group_id).unwrap();
        let result = group_mut.add_member("member_456", GroupRole::Member, vec![]);

        assert!(result.is_ok());
        assert_eq!(group_mut.member_count(), 2);
        assert!(group_mut.is_member("member_456"));
    }

    #[test]
    fn test_group_permissions() {
        let mut manager = GroupManager::new("creator_123");
        let group = manager.create_group("Test Group");
        let group_id = group.id.clone();

        // Добавляем модератора
        {
            let group_mut = manager.get_group_mut(&group_id).unwrap();
            group_mut.add_member("mod_789", GroupRole::Moderator, vec![]).unwrap();
        }

        let group = manager.get_group(&group_id).unwrap();

        // Проверка прав
        assert!(group.check_permission("creator_123", &GroupRole::Admin));
        assert!(group.check_permission("mod_789", &GroupRole::Member));
        assert!(!group.check_permission("unknown", &GroupRole::Member));
    }

    #[test]
    fn test_group_topic() {
        let mut manager = GroupManager::new("creator_123");
        let group = manager.create_group("Test Group");

        let topic = group.topic();
        assert!(topic.hash().as_str().starts_with("liberty-group-"));
    }

    #[test]
    fn test_group_encryption() {
        let mut manager = GroupManager::new("creator_123");
        let group = manager.create_group("Test Group");
        let group_id = group.id.clone();

        let original_message = b"Hello, group!";

        // Шифрование
        let encrypted = manager.encrypt_group_message(&group_id, original_message).unwrap();
        assert_ne!(encrypted, original_message.as_slice());

        // Дешифрование
        let decrypted = manager.decrypt_group_message(&group_id, &encrypted).unwrap();
        assert_eq!(decrypted, original_message.as_slice());
    }

    #[test]
    fn test_group_role_permissions() {
        assert!(GroupRole::Owner.can_invite());
        assert!(GroupRole::Owner.can_kick());
        assert!(GroupRole::Owner.can_edit_settings());

        assert!(GroupRole::Admin.can_invite());
        assert!(GroupRole::Admin.can_kick());
        assert!(GroupRole::Admin.can_edit_settings());

        assert!(GroupRole::Moderator.can_invite());
        assert!(GroupRole::Moderator.can_kick());
        assert!(!GroupRole::Moderator.can_edit_settings());

        assert!(!GroupRole::Member.can_invite());
        assert!(!GroupRole::Member.can_kick());
        assert!(!GroupRole::Member.can_edit_settings());
    }
}
