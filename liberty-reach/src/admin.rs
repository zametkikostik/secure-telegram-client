//! Модуль админ-панели
//!
//! Реализует:
//! - Проверку прав администратора (с хэшированием PeerID для безопасности)
//! - Верификацию пользователей (галочка)
//! - Premium статус
//! - Geo-Trace (Emergency функция)
//! - Zeroize для безопасной очистки чувствительных данных

use anyhow::{Result, Context};
use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use reqwest::Client;
use sha2::{Sha256, Digest};
use zeroize::Zeroize;

/// Конфигурация админ-панели
pub struct AdminConfig {
    /// Email администратора (основной идентификатор)
    pub admin_email: String,
    /// Хэш Peer ID администратора (SHA-256) для безопасности
    /// Оригинальный PeerID не хранится в бинарнике
    pub admin_peer_id_hash: String,
    /// Cloudflare API ключ для KV
    pub cloudflare_api_key: String,
    /// Cloudflare Account ID
    pub cloudflare_account_id: String,
    /// KV Namespace ID для метаданных
    pub kv_namespace_id: String,
}

impl AdminConfig {
    pub fn new() -> Self {
        Self {
            admin_email: std::env::var("ADMIN_EMAIL")
                .unwrap_or_else(|_| "zametkikostik@gmail.com".to_string()),
            admin_peer_id_hash: std::env::var("ADMIN_PEER_ID_HASH").unwrap_or_default(),
            cloudflare_api_key: std::env::var("CLOUDFLARE_API_KEY").unwrap_or_default(),
            cloudflare_account_id: std::env::var("CLOUDFLARE_ACCOUNT_ID").unwrap_or_default(),
            kv_namespace_id: std::env::var("KV_NAMESPACE_ID").unwrap_or_default(),
        }
    }

    /// Хэширование PeerID через SHA-256
    pub fn hash_peer_id(peer_id: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(peer_id.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Проверка, является ли Peer ID администратором (сравнение хэшей)
    pub fn is_admin(&self, peer_id: &str) -> bool {
        if self.admin_peer_id_hash.is_empty() {
            return false;
        }
        let peer_id_hash = Self::hash_peer_id(peer_id);
        peer_id_hash == self.admin_peer_id_hash
    }

    /// Установка Admin Peer ID через хэширование
    pub fn set_admin_peer_id(&mut self, peer_id: &str) {
        self.admin_peer_id_hash = Self::hash_peer_id(peer_id);
    }

    pub fn is_configured(&self) -> bool {
        !self.admin_peer_id_hash.is_empty()
    }
}

impl Default for AdminConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Статус верификации пользователя
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VerificationStatus {
    /// Обычный пользователь
    Unverified,
    /// Заявка на верификацию
    Pending,
    /// Верифицирован (галочка)
    Verified,
    /// Premium статус
    Premium,
    /// Заблокирован
    Banned,
}

impl VerificationStatus {
    pub fn as_emoji(&self) -> &'static str {
        match self {
            VerificationStatus::Unverified => "",
            VerificationStatus::Pending => "⏳",
            VerificationStatus::Verified => "✓",
            VerificationStatus::Premium => "★",
            VerificationStatus::Banned => "✗",
        }
    }
}

/// Заявка на верификацию
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRequest {
    pub peer_id: String,
    pub name: String,
    pub reason: String,
    pub timestamp: u64,
    pub status: VerificationStatus,
    pub reviewed_by: Option<String>,
}

/// Метаданные для Geo-Trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerMetadata {
    pub peer_id: String,
    pub last_seen: u64,
    pub last_ip: Option<String>,
    pub last_cell_tower: Option<String>, // ID вышки (если доступно)
    pub country: Option<String>,
    pub city: Option<String>,
}

/// Менеджер админ-панели
pub struct AdminManager {
    pub config: AdminConfig,
    client: Client,
    /// Кэш верификаций
    verification_requests: HashMap<String, VerificationRequest>,
    /// Кэш статусов
    peer_statuses: HashMap<String, VerificationStatus>,
    /// Кэш метаданных для Geo-Trace
    peer_metadata: HashMap<String, PeerMetadata>,
}

impl AdminManager {
    pub fn new(config: AdminConfig) -> Self {
        Self {
            config,
            client: Client::new(),
            verification_requests: HashMap::new(),
            peer_statuses: HashMap::new(),
            peer_metadata: HashMap::new(),
        }
    }

    /// Проверка прав администратора
    pub fn check_admin(&self, peer_id: &str) -> bool {
        self.config.is_admin(peer_id)
    }

    /// Требование прав администратора
    pub fn require_admin(&self, peer_id: &str) -> Result<()> {
        if self.check_admin(peer_id) {
            Ok(())
        } else {
            anyhow::bail!("Доступ запрещен: требуется права администратора")
        }
    }

    /// Создание заявки на верификацию
    pub fn submit_verification_request(
        &mut self,
        peer_id: &str,
        name: &str,
        reason: &str,
    ) -> Result<&VerificationRequest> {
        let request = VerificationRequest {
            peer_id: peer_id.to_string(),
            name: name.to_string(),
            reason: reason.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            status: VerificationStatus::Pending,
            reviewed_by: None,
        };

        self.verification_requests.insert(peer_id.to_string(), request);
        Ok(self.verification_requests.get(peer_id).unwrap())
    }

    /// Обработка заявки (принять/отклонить)
    pub fn review_verification_request(
        &mut self,
        admin_peer_id: &str,
        target_peer_id: &str,
        approve: bool,
    ) -> Result<VerificationStatus> {
        // Проверка прав администратора
        self.require_admin(admin_peer_id)?;

        let request = self.verification_requests.get_mut(target_peer_id)
            .ok_or_else(|| anyhow::anyhow!("Заявка не найдена"))?;

        if request.status != VerificationStatus::Pending {
            anyhow::bail!("Заявка уже обработана");
        }

        let new_status = if approve {
            VerificationStatus::Verified
        } else {
            VerificationStatus::Unverified
        };

        request.status = new_status.clone();
        request.reviewed_by = Some(admin_peer_id.to_string());
        self.peer_statuses.insert(target_peer_id.to_string(), new_status.clone());

        Ok(new_status)
    }

    /// Выдача Premium статуса
    pub fn grant_premium(
        &mut self,
        admin_peer_id: &str,
        target_peer_id: &str,
        duration_days: u32,
    ) -> Result<()> {
        // Проверка прав администратора
        self.require_admin(admin_peer_id)?;

        // В реальной реализации здесь была бы логика оплаты и срока действия
        self.peer_statuses.insert(target_peer_id.to_string(), VerificationStatus::Premium);

        tracing::info!(
            "Premium выдан: {} (срок: {} дней)",
            target_peer_id,
            duration_days
        );

        Ok(())
    }

    /// Получение статуса пира
    pub fn get_status(&self, peer_id: &str) -> &VerificationStatus {
        self.peer_statuses
            .get(peer_id)
            .unwrap_or(&VerificationStatus::Unverified)
    }

    /// Geo-Trace: Поиск последнего местоположения пира
    pub async fn trace_peer_location(&self, target_peer_id: &str) -> Result<Option<PeerMetadata>> {
        // Проверка прав администратора
        self.require_admin(&AdminConfig::hash_peer_id(target_peer_id))?;

        // 1. Проверка локального кэша
        if let Some(metadata) = self.peer_metadata.get(target_peer_id) {
            // Проверка актуальности (не старше 1 часа)
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if now - metadata.last_seen < 3600 {
                return Ok(Some(metadata.clone()));
            }
        }

        // 2. Запрос к Cloudflare KV
        if self.config.is_kv_configured() {
            match self.fetch_from_cloudflare_kv(target_peer_id).await {
                Ok(Some(metadata)) => return Ok(Some(metadata)),
                Ok(None) => {},
                Err(e) => tracing::warn!("Ошибка запроса к Cloudflare KV: {}", e),
            }
        }

        // 3. Пинг через соседние узлы (заглушка)
        // В реальной реализации здесь был бы запрос к узлам в той же подсети
        tracing::warn!("Пир {} офлайн, данные устарели", target_peer_id);

        Ok(None)
    }

    /// Запрос метаданных из Cloudflare KV
    async fn fetch_from_cloudflare_kv(&self, peer_id: &str) -> Result<Option<PeerMetadata>> {
        if !self.config.is_kv_configured() {
            return Ok(None);
        }

        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
            self.config.cloudflare_account_id,
            self.config.kv_namespace_id,
            peer_id
        );

        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.config.cloudflare_api_key))
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    let metadata: PeerMetadata = resp.json().await
                        .context("Ошибка парсинга метаданных")?;
                    Ok(Some(metadata))
                } else {
                    // 404 = нет данных
                    Ok(None)
                }
            }
            Err(e) => Err(anyhow::anyhow!("Ошибка запроса к KV: {}", e)),
        }
    }

    /// Сохранение метаданных пира (вызывается при подключении)
    pub fn update_peer_metadata(
        &mut self,
        peer_id: &str,
        ip: Option<String>,
        country: Option<String>,
        city: Option<String>,
    ) {
        let metadata = PeerMetadata {
            peer_id: peer_id.to_string(),
            last_seen: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            last_ip: ip,
            last_cell_tower: None, // Требует доступа к телеком данным
            country,
            city,
        };

        self.peer_metadata.insert(peer_id.to_string(), metadata);
    }

    /// Получение списка всех заявок на верификацию
    pub fn get_pending_requests(&self) -> Vec<&VerificationRequest> {
        self.verification_requests
            .values()
            .filter(|r| r.status == VerificationStatus::Pending)
            .collect()
    }

    /// Бан пользователя
    pub fn ban_peer(&mut self, admin_peer_id: &str, target_peer_id: &str, reason: &str) -> Result<()> {
        self.require_admin(admin_peer_id)?;
        
        self.peer_statuses.insert(target_peer_id.to_string(), VerificationStatus::Banned);
        tracing::warn!("Пир {} забанен: {}", target_peer_id, reason);
        
        Ok(())
    }

    /// Разбан пользователя
    pub fn unban_peer(&mut self, admin_peer_id: &str, target_peer_id: &str) -> Result<()> {
        self.require_admin(admin_peer_id)?;

        self.peer_statuses.insert(target_peer_id.to_string(), VerificationStatus::Unverified);
        tracing::info!("Пир {} разбанен", target_peer_id);

        Ok(())
    }

    /// ZEROIZE: Мгновенная очистка всех чувствительных данных
    /// Вызывается при команде "Clear all local metadata"
    pub fn zeroize(&mut self) {
        // Очистка кэша верификаций
        self.verification_requests.clear();

        // Очистка кэша статусов
        self.peer_statuses.clear();

        // Очистка кэша метаданных (Geo-Trace)
        self.peer_metadata.clear();

        // Очистка API ключей в конфиге
        self.config.cloudflare_api_key.zeroize();
        self.config.cloudflare_account_id.zeroize();
        self.config.kv_namespace_id.zeroize();
        self.config.admin_peer_id_hash.zeroize();
        self.config.admin_email.zeroize();

        tracing::warn!("AdminManager: все чувствительные данные уничтожены (zeroize)");
    }

    /// Очистка Cloudflare KV кэша (удаление всех ключей)
    pub async fn clear_cloudflare_kv_cache(&self) -> Result<()> {
        if !self.config.is_kv_configured() {
            return Ok(());
        }

        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/keys",
            self.config.cloudflare_account_id,
            self.config.kv_namespace_id
        );

        // Получение списка всех ключей
        let response = self.client.get(&url)
            .header("Authorization", format!("Bearer {}", self.config.cloudflare_api_key))
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    let data: serde_json::Value = resp.json().await
                        .context("Ошибка парсинга списка ключей")?;

                    if let Some(keys) = data.get("result").and_then(|r| r.as_array()) {
                        // Удаление каждого ключа
                        for key in keys {
                            if let Some(key_str) = key.get("name").and_then(|k| k.as_str()) {
                                let delete_url = format!(
                                    "https://api.cloudflare.com/client/v4/accounts/{}/storage/kv/namespaces/{}/values/{}",
                                    self.config.cloudflare_account_id,
                                    self.config.kv_namespace_id,
                                    key_str
                                );

                                let _ = self.client.delete(&delete_url)
                                    .header("Authorization", format!("Bearer {}", self.config.cloudflare_api_key))
                                    .send()
                                    .await;
                            }
                        }
                        tracing::info!("Cloudflare KV: удалено {} ключей", keys.len());
                    }
                }
            }
            Err(e) => tracing::warn!("Ошибка очистки KV: {}", e),
        }

        Ok(())
    }
}

impl AdminConfig {
    fn is_kv_configured(&self) -> bool {
        !self.cloudflare_api_key.is_empty() &&
        !self.cloudflare_account_id.is_empty() &&
        !self.kv_namespace_id.is_empty()
    }
}

/// Команды админ-панели
pub const ADMIN_COMMANDS: &[(&str, &str)] = &[
    ("admin_verify [peer] [approve|decline]", "Обработать заявку на верификацию"),
    ("admin_premium [peer] [days]", "Выдать Premium статус"),
    ("admin_trace [peer]", "Geo-Trace: найти местоположение пира"),
    ("admin_ban [peer] [reason]", "Забанить пользователя"),
    ("admin_unban [peer]", "Разбанить пользователя"),
    ("admin_requests", "Показать заявки на верификацию"),
    ("admin_status [peer]", "Показать статус пира"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_check() {
        let mut config = AdminConfig::new();
        // Используем хэш вместо открытого PeerID
        config.admin_peer_id_hash = AdminConfig::hash_peer_id("admin123");

        let manager = AdminManager::new(config);

        assert!(manager.check_admin("admin123"));
        assert!(!manager.check_admin("user456"));
    }

    #[test]
    fn test_hash_peer_id() {
        // Проверка, что хэширование работает детерминировано
        let hash1 = AdminConfig::hash_peer_id("test_peer");
        let hash2 = AdminConfig::hash_peer_id("test_peer");
        assert_eq!(hash1, hash2);

        // Разные PeerID дают разные хэши
        let hash3 = AdminConfig::hash_peer_id("different_peer");
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_zeroize() {
        let config = AdminConfig::new();
        let mut manager = AdminManager::new(config);

        // Добавляем тестовые данные
        manager.submit_verification_request("user123", "Test", "Reason").ok();
        manager.update_peer_metadata("user123", Some("1.2.3.4".to_string()), None, None);

        // Проверяем, что данные есть
        assert!(!manager.peer_statuses.is_empty() || !manager.verification_requests.is_empty());

        // Вызываем zeroize
        manager.zeroize();

        // Проверяем, что всё очищено
        assert!(manager.verification_requests.is_empty());
        assert!(manager.peer_statuses.is_empty());
        assert!(manager.peer_metadata.is_empty());
    }

    #[test]
    fn test_verification_request() {
        let config = AdminConfig::new();
        let mut manager = AdminManager::new(config);

        let request = manager.submit_verification_request(
            "user123",
            "Test User",
            "Прошу верифицировать"
        ).unwrap();

        assert_eq!(request.status, VerificationStatus::Pending);
        assert_eq!(manager.get_pending_requests().len(), 1);
    }
}
