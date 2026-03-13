//! Модуль управления идентичностью и профилями пользователей
//! 
//! Реализует:
//! - Ed25519 ключи для Peer ID
//! - UserProfile с семейным статусом
//! - Двустороннее подтверждение отношений

use libp2p::identity::Keypair;
use libp2p::PeerId;
use std::fs;
use std::path::Path;
use std::io::Write;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Типы отношений между пирами
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationshipType {
    /// Одинокий
    Single,
    /// В поиске
    Looking,
    /// Пара (требует подтверждения)
    Partner,
    /// Семья
    Family,
    /// Помолвлен
    Engaged,
    /// Влюблен (одностороннее)
    InLove,
}

impl RelationshipType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationshipType::Single => "Одинокий",
            RelationshipType::Looking => "В поиске",
            RelationshipType::Partner => "Пара",
            RelationshipType::Family => "Семья",
            RelationshipType::Engaged => "Помолвлен",
            RelationshipType::InLove => "Влюблен",
        }
    }
}

/// Запрос на подтверждение отношений
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipRequest {
    /// Кто отправил запрос (как строка)
    pub from_peer: String,
    /// Кому отправлен запрос (как строка)
    pub to_peer: String,
    /// Тип отношений
    pub relationship_type: RelationshipType,
    /// Timestamp запроса
    pub timestamp: u64,
    /// Статус запроса
    pub status: RequestStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RequestStatus {
    Pending,
    Accepted,
    Declined,
}

/// Профиль пользователя
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// Peer ID пользователя
    pub peer_id: String,
    /// Отображаемое имя
    pub name: String,
    /// IPFS хэш аватара
    pub avatar_ipfs_hash: Option<String>,
    /// Текущий статус (текст)
    pub status_text: String,
    /// Тип отношений
    pub relationship_status: RelationshipType,
    /// Peer ID партнера (если есть)
    pub partner_peer_id: Option<String>,
    /// Подтверждено ли партнерство
    pub relationship_confirmed: bool,
}

impl UserProfile {
    pub fn new(peer_id: &PeerId) -> Self {
        Self {
            peer_id: peer_id.to_string(),
            name: format!("User_{}", peer_id.to_string().chars().take(4).collect::<String>()),
            avatar_ipfs_hash: None,
            status_text: String::new(),
            relationship_status: RelationshipType::Single,
            partner_peer_id: None,
            relationship_confirmed: false,
        }
    }

    /// Создание запроса на изменение статуса отношений
    pub fn create_relationship_request(
        &self,
        target_peer: PeerId,
        relationship_type: RelationshipType,
    ) -> RelationshipRequest {
        RelationshipRequest {
            from_peer: self.peer_id.clone(),
            to_peer: target_peer.to_string(),
            relationship_type,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            status: RequestStatus::Pending,
        }
    }

    /// Проверка, есть ли активный партнер
    pub fn has_partner(&self) -> bool {
        self.relationship_confirmed && self.partner_peer_id.is_some()
    }

    /// Получение краткого статуса
    pub fn get_status_display(&self) -> String {
        if self.has_partner() {
            format!("{} 💞", self.relationship_status.as_str())
        } else {
            self.relationship_status.as_str().to_string()
        }
    }
}

/// Менеджер профилей и отношений
pub struct ProfileManager {
    /// Наш профиль
    pub our_profile: UserProfile,
    /// Кэшированные профили других пиров
    pub peer_profiles: HashMap<PeerId, UserProfile>,
    /// Входящие запросы отношений
    pub incoming_requests: Vec<RelationshipRequest>,
    /// Исходящие запросы отношений
    pub outgoing_requests: Vec<RelationshipRequest>,
}

impl ProfileManager {
    pub fn new(peer_id: &PeerId) -> Self {
        Self {
            our_profile: UserProfile::new(peer_id),
            peer_profiles: HashMap::new(),
            incoming_requests: Vec::new(),
            outgoing_requests: Vec::new(),
        }
    }

    /// Обновление нашего статуса
    pub fn update_status(&mut self, status_text: String) {
        self.our_profile.status_text = status_text;
    }

    /// Обновление аватара (IPFS hash)
    pub fn update_avatar(&mut self, ipfs_hash: String) {
        self.our_profile.avatar_ipfs_hash = Some(ipfs_hash);
    }

    /// Отправка запроса на отношения
    pub fn send_relationship_request(
        &mut self,
        target_peer: PeerId,
        relationship_type: RelationshipType,
    ) -> RelationshipRequest {
        let request = self.our_profile.create_relationship_request(target_peer, relationship_type);
        self.outgoing_requests.push(request.clone());
        request
    }

    /// Обработка входящего запроса
    pub fn receive_relationship_request(&mut self, request: RelationshipRequest) {
        // Проверяем, нет ли уже такого запроса
        if !self.incoming_requests.iter().any(|r| r.from_peer == request.from_peer) {
            self.incoming_requests.push(request);
        }
    }

    /// Принятие запроса на отношения
    pub fn accept_relationship_request(&mut self, from_peer: &str) -> bool {
        if let Some(pos) = self.incoming_requests.iter().position(|r| r.from_peer == from_peer) {
            let request = &mut self.incoming_requests[pos];
            request.status = RequestStatus::Accepted;

            // Обновляем наш профиль
            self.our_profile.relationship_status = request.relationship_type.clone();
            self.our_profile.partner_peer_id = Some(from_peer.to_string());
            self.our_profile.relationship_confirmed = true;

            // Удаляем из входящих
            self.incoming_requests.remove(pos);
            true
        } else {
            false
        }
    }

    /// Отклонение запроса
    pub fn decline_relationship_request(&mut self, from_peer: &str) -> bool {
        if let Some(pos) = self.incoming_requests.iter().position(|r| r.from_peer == from_peer) {
            self.incoming_requests[pos].status = RequestStatus::Declined;
            self.incoming_requests.remove(pos);
            true
        } else {
            false
        }
    }

    /// Синхронизация статуса партнера через Kademlia
    pub fn sync_partner_status(&mut self, partner_peer_id: &str) -> bool {
        if let Ok(partner_id) = partner_peer_id.parse::<PeerId>() {
            if let Some(partner_profile) = self.peer_profiles.get(&partner_id) {
                // Проверяем взаимность
                if partner_profile.partner_peer_id.as_ref() == Some(&self.our_profile.peer_id) {
                    self.our_profile.relationship_confirmed = true;
                    return true;
                }
            }
        }
        false
    }

    /// Кэширование профиля пира
    pub fn cache_peer_profile(&mut self, profile: UserProfile) {
        if let Ok(peer_id) = profile.peer_id.parse::<PeerId>() {
            self.peer_profiles.insert(peer_id, profile);
        }
    }

    /// Получение количества входящих запросов
    pub fn pending_requests_count(&self) -> usize {
        self.incoming_requests.iter()
            .filter(|r| r.status == RequestStatus::Pending)
            .count()
    }

    /// Получение ключа шифрования (производный от identity key)
    pub fn get_cipher_key(&self) -> [u8; 32] {
        // В продакшене здесь был бы HKDF или PBKDF2
        // Для примера используем хэш от peer_id
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(self.our_profile.peer_id.as_bytes());
        let result = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&result);
        key
    }

    /// Получение private key для подписи сообщений
    pub fn get_private_key(&self) -> [u8; 32] {
        // В продакшене здесь был бы настоящий private key от Ed25519
        // Для примера используем другой хэш
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(self.our_profile.peer_id.as_bytes());
        hasher.update(b"signing-key");
        let result = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&result);
        key
    }
}

/// Загружает или генерирует ключи идентичности
pub fn load_or_generate_identity(path: &str) -> Keypair {
    if Path::new(path).exists() {
        match load_identity(path) {
            Ok(keypair) => {
                println!("✓ Идентичность загружена из {}", path);
                return keypair;
            }
            Err(e) => {
                eprintln!("⚠ Ошибка чтения ключа: {}. Генерируем новый...", e);
            }
        }
    } else {
        println!("ℹ Файл идентичности не найден, генерируем новый...");
    }
    
    generate_and_save_identity(path)
}

/// Загрузка ключа из файла
fn load_identity(path: &str) -> Result<Keypair, anyhow::Error> {
    let mut data = fs::read(path)
        .map_err(|e| anyhow::anyhow!("Не удалось прочитать файл: {}", e))?;
    
    if data.len() == 64 {
        match Keypair::ed25519_from_bytes(&mut data) {
            Ok(keypair) => return Ok(keypair),
            Err(_) => {}
        }
    }
    
    match Keypair::from_protobuf_encoding(&data) {
        Ok(keypair) => Ok(keypair),
        Err(_) => Err(anyhow::anyhow!("Неверный формат ключа")),
    }
}

/// Генерация и сохранение нового ключа
fn generate_and_save_identity(path: &str) -> Keypair {
    let keypair = Keypair::generate_ed25519();
    
    if let Ok(bytes) = keypair.to_protobuf_encoding() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .mode(0o600)
                .open(path)
            {
                let _ = file.write_all(&bytes);
                println!("✓ Новая идентичность сохранена в {} (права: 600)", path);
                return keypair;
            }
        }
        
        if fs::write(path, &bytes).is_ok() {
            println!("✓ Новая идентичность сохранена в {}", path);
        }
    }
    
    keypair
}

/// Получает PeerId из ключей
pub fn get_peer_id(keys: &Keypair) -> PeerId {
    keys.public().to_peer_id()
}

#[allow(dead_code)]
pub fn get_short_peer_id(keys: &Keypair) -> String {
    let peer_id = get_peer_id(keys);
    peer_id.to_string().chars().take(8).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_request() {
        let peer_id = PeerId::random();
        let mut manager = ProfileManager::new(&peer_id);
        
        let target = PeerId::random();
        let request = manager.send_relationship_request(target, RelationshipType::Partner);
        
        assert_eq!(request.status, RequestStatus::Pending);
        assert_eq!(manager.outgoing_requests.len(), 1);
    }

    #[test]
    fn test_accept_relationship() {
        let peer_id = PeerId::random();
        let mut manager = ProfileManager::new(&peer_id);
        
        let from_peer = PeerId::random();
        let request = RelationshipRequest {
            from_peer: from_peer.to_string(),
            to_peer: peer_id.to_string(),
            relationship_type: RelationshipType::Partner,
            timestamp: 0,
            status: RequestStatus::Pending,
        };
        
        manager.receive_relationship_request(request);
        assert_eq!(manager.pending_requests_count(), 1);
        
        let accepted = manager.accept_relationship_request(&from_peer.to_string());
        assert!(accepted);
        assert!(manager.our_profile.relationship_confirmed);
        assert_eq!(manager.pending_requests_count(), 0);
    }
}
