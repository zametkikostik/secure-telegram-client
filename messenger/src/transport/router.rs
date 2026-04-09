//! Transport Router
//!
//! Интеллектуальный выбор маршрута для доставки сообщений:
//! - Если peer в DHT → P2P (прямое соединение)
//! - Иначе → Cloudflare fallback (HTTPS relay)
//! - Логирование выбора маршрута (зашифрованное)
//!
//! SECURITY: требует аудита перед production
//! TODO: pentest перед release

use crate::crypto::encrypt;
use crate::p2p::{P2PError, P2PMessage, P2PNode};
use crate::transport::cloudflare::{CloudflareTransport, TransportError};
use ed25519_dalek::{Signer, SigningKey};
use oqs::kem::PublicKey as KyberPublicKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use x25519_dalek::PublicKey as X25519PublicKey;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum RouterError {
    #[error("P2P error: {0}")]
    P2P(#[from] P2PError),

    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),

    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    #[error("All routes failed")]
    AllRoutesFailed,

    #[error("Router not initialized")]
    NotInitialized,
}

// ============================================================================
// Route Types
// ============================================================================

/// Доступные маршруты
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Route {
    /// Прямое P2P соединение (предпочтительно)
    P2P,
    /// Cloudflare Worker fallback
    Cloudflare,
}

impl std::fmt::Display for Route {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Route::P2P => write!(f, "P2P"),
            Route::Cloudflare => write!(f, "Cloudflare"),
        }
    }
}

/// Зашифрованная запись в логе маршрутизации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedRouteLog {
    /// Зашифрованный ID отправителя
    pub sender_hash: String,
    /// Зашифрованный ID получателя
    pub recipient_hash: String,
    /// Выбранный маршрут
    pub route: Route,
    /// Timestamp (Unix epoch ms)
    pub timestamp: i64,
    /// Причина выбора (зашифрованная)
    pub reason_hash: String,
    /// Статус доставки
    pub success: bool,
}

/// Статистика маршрутизации
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RouteStats {
    pub p2p_attempts: u64,
    pub p2p_successes: u64,
    pub cloudflare_attempts: u64,
    pub cloudflare_successes: u64,
    pub total_messages: u64,
}

// ============================================================================
// Transport Router
// ============================================================================

/// Роутер транспорта с автоматическим выбором маршрута
pub struct TransportRouter {
    /// P2P узел
    p2p_node: RwLock<Option<P2PNode>>,
    /// Cloudflare транспорт
    cloudflare_transport: RwLock<Option<CloudflareTransport>>,
    /// Кэш доступности пиров (peer_id -> доступен ли по P2P)
    peer_cache: RwLock<HashMap<String, bool>>,
    /// Статистика маршрутизации
    stats: AtomicRouteStats,
    /// Зашифрованный лог маршрутизации
    route_log: RwLock<Vec<EncryptedRouteLog>>,
    /// Максимальный размер лога
    max_log_size: usize,
}

/// Атомарная статистика
struct AtomicRouteStats {
    p2p_attempts: AtomicU64,
    p2p_successes: AtomicU64,
    cloudflare_attempts: AtomicU64,
    cloudflare_successes: AtomicU64,
    total_messages: AtomicU64,
}

impl AtomicRouteStats {
    fn new() -> Self {
        Self {
            p2p_attempts: AtomicU64::new(0),
            p2p_successes: AtomicU64::new(0),
            cloudflare_attempts: AtomicU64::new(0),
            cloudflare_successes: AtomicU64::new(0),
            total_messages: AtomicU64::new(0),
        }
    }

    fn to_stats(&self) -> RouteStats {
        RouteStats {
            p2p_attempts: self.p2p_attempts.load(Ordering::Relaxed),
            p2p_successes: self.p2p_successes.load(Ordering::Relaxed),
            cloudflare_attempts: self.cloudflare_attempts.load(Ordering::Relaxed),
            cloudflare_successes: self.cloudflare_successes.load(Ordering::Relaxed),
            total_messages: self.total_messages.load(Ordering::Relaxed),
        }
    }

    fn record_attempt(&self, route: Route) {
        self.total_messages.fetch_add(1, Ordering::Relaxed);
        match route {
            Route::P2P => {
                self.p2p_attempts.fetch_add(1, Ordering::Relaxed);
            }
            Route::Cloudflare => {
                self.cloudflare_attempts.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    fn record_success(&self, route: Route) {
        match route {
            Route::P2P => {
                self.p2p_successes.fetch_add(1, Ordering::Relaxed);
            }
            Route::Cloudflare => {
                self.cloudflare_successes.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

impl TransportRouter {
    /// Создать новый роутер
    ///
    /// # Arguments
    /// * `p2p_node` — P2P узел (может быть None)
    /// * `cloudflare_transport` — Cloudflare транспорт (может быть None)
    /// * `max_log_size` — максимальный размер лога маршрутизации
    pub fn new(
        p2p_node: Option<P2PNode>,
        cloudflare_transport: Option<CloudflareTransport>,
        max_log_size: usize,
    ) -> Self {
        Self {
            p2p_node: RwLock::new(p2p_node),
            cloudflare_transport: RwLock::new(cloudflare_transport),
            peer_cache: RwLock::new(HashMap::new()),
            stats: AtomicRouteStats::new(),
            route_log: RwLock::new(Vec::new()),
            max_log_size,
        }
    }

    /// Выбрать маршрут для получателя
    ///
    /// Логика:
    /// 1. Если peer в кэше как P2P-доступный → P2P
    /// 2. Если peer в DHT → P2P
    /// 3. Иначе → Cloudflare fallback
    ///
    /// # Arguments
    /// * `peer_id` — ID получателя
    ///
    /// # Returns
    /// * `Route` — выбранный маршрут
    pub async fn select_route(&self, peer_id: &str) -> Route {
        // Проверить кэш
        {
            let cache = self.peer_cache.read().await;
            if let Some(&available) = cache.get(peer_id) {
                if available {
                    debug!("Route cache hit: {} -> P2P", peer_id);
                    return Route::P2P;
                }
            }
        }

        // Проверить DHT
        if self.is_peer_in_dht(peer_id).await {
            // Обновить кэш
            {
                let mut cache = self.peer_cache.write().await;
                cache.insert(peer_id.to_string(), true);
            }
            debug!("Peer {} found in DHT -> P2P", peer_id);
            return Route::P2P;
        }

        // Fallback на Cloudflare
        {
            let mut cache = self.peer_cache.write().await;
            cache.insert(peer_id.to_string(), false);
        }
        debug!("Peer {} not in DHT -> Cloudflare", peer_id);
        Route::Cloudflare
    }

    /// Проверить, доступен ли peer в DHT
    async fn is_peer_in_dht(&self, peer_id: &str) -> bool {
        let p2p_guard = self.p2p_node.read().await;
        if let Some(_p2p) = p2p_guard.as_ref() {
            // TODO: реальная проверка DHT
            // Сейчас заглушка — в реальности нужно:
            // 1. Найти peer в routing table
            // 2. Проверить freshness записи
            // 3. Вернуть true если peer найден
            debug!("Checking DHT for peer: {}", peer_id);
            false // Заглушка
        } else {
            false
        }
    }

    /// Отправить сообщение с автоматическим выбором маршрута
    ///
    /// # Arguments
    /// * `peer_id` — ID получателя
    /// * `message` — P2P сообщение
    /// * `ciphertext` — зашифрованный payload для Cloudflare
    /// * `signature` — подпись сообщения
    ///
    /// # Returns
    /// * `Ok(Route)` — использованный маршрут
    /// * `Err(RouterError)` — при ошибке
    pub async fn send_with_auto_route(
        &self,
        peer_id: &str,
        message: P2PMessage,
        ciphertext: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<Route, RouterError> {
        let route = self.select_route(peer_id).await;

        info!("Routing message to {} via {}", peer_id, route);

        let result = match route {
            Route::P2P => self.send_via_p2p(peer_id, message).await,
            Route::Cloudflare => {
                self.send_via_cloudflare(peer_id, ciphertext, signature)
                    .await
            }
        };

        // Записать в лог
        self.log_route(peer_id, route, result.is_ok(), "auto_select")
            .await;

        // Обновить статистику
        self.stats.record_attempt(route);
        if result.is_ok() {
            self.stats.record_success(route);
        }

        result.map(|_| route)
    }

    /// Зашифровать и отправить сообщение (полный цикл)
    ///
    /// Шифрует payload ДО отправки через crypto::hybrid::encrypt,
    /// затем выбирает оптимальный маршрут.
    ///
    /// # Arguments
    /// * `peer_id` — ID получателя
    /// * `plaintext` — исходное сообщение (будет зашифровано)
    /// * `recipient_x25519` — X25519 публичный ключ получателя
    /// * `recipient_kyber` — Kyber1024 публичный ключ получателя
    /// * `sender_signing_key` — Ed25519 ключ отправителя
    ///
    /// # Returns
    /// * `Ok(Route)` — использованный маршрут
    /// * `Err(RouterError)` — при ошибке
    pub async fn send_encrypted(
        &self,
        peer_id: &str,
        plaintext: &[u8],
        recipient_x25519: &X25519PublicKey,
        recipient_kyber: &KyberPublicKey,
        sender_signing_key: &SigningKey,
    ) -> Result<Route, RouterError> {
        // 1. Шифруем payload ДО отправки
        let ciphertext = encrypt(
            plaintext,
            recipient_x25519,
            recipient_kyber,
            sender_signing_key,
        )
        .map_err(|e| RouterError::P2P(P2PError::SendFailed(e.to_string())))?;

        debug!("Payload encrypted with hybrid X25519+Kyber1024");

        // 2. Сериализуем ciphertext для передачи
        let ciphertext_bytes = serde_json::to_vec(&ciphertext)
            .map_err(|e| RouterError::Transport(TransportError::Serialization(e)))?;

        // 3. Подписываем ciphertext
        let signature = sender_signing_key.sign(&ciphertext_bytes);

        // 4. Создаём P2P сообщение
        let p2p_message = P2PMessage {
            ciphertext: ciphertext_bytes.clone(),
            signature: signature.to_bytes().to_vec(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            msg_type: crate::p2p::MessageType::Direct,
        };

        // 5. Отправляем с авто-выбором маршрута
        self.send_with_auto_route(
            peer_id,
            p2p_message,
            ciphertext_bytes,
            signature.to_bytes().to_vec(),
        )
        .await
    }

    /// Отправить через P2P
    async fn send_via_p2p(&self, peer_id: &str, message: P2PMessage) -> Result<(), RouterError> {
        let mut p2p_guard = self.p2p_node.write().await;

        if let Some(p2p) = p2p_guard.as_mut() {
            // Попытаться отправить через P2P
            match p2p
                .send_message(
                    peer_id
                        .parse()
                        .map_err(|_| RouterError::PeerNotFound(peer_id.to_string()))?,
                    message,
                )
                .await
            {
                Ok(()) => {
                    info!("P2P send successful to {}", peer_id);
                    Ok(())
                }
                Err(e) => {
                    warn!("P2P send failed for {}: {}", peer_id, e);
                    // Обновить кэш — peer недоступен по P2P
                    {
                        let mut cache = self.peer_cache.write().await;
                        cache.insert(peer_id.to_string(), false);
                    }
                    Err(RouterError::P2P(e))
                }
            }
        } else {
            warn!("P2P node not initialized");
            Err(RouterError::NotInitialized)
        }
    }

    /// Отправить через Cloudflare
    async fn send_via_cloudflare(
        &self,
        peer_id: &str,
        ciphertext: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<(), RouterError> {
        let cf_guard = self.cloudflare_transport.read().await;

        if let Some(cf) = cf_guard.as_ref() {
            let message =
                CloudflareTransport::create_message(peer_id.to_string(), ciphertext, signature);

            match cf.send_message(message.clone()).await {
                Ok(()) => {
                    info!("Cloudflare send successful to {}", peer_id);
                    Ok(())
                }
                Err(e) => {
                    warn!("Cloudflare send failed for {}: {}", peer_id, e);
                    // Добавить в offline queue
                    cf.queue_message(message).await?;
                    Err(RouterError::Transport(e))
                }
            }
        } else {
            warn!("Cloudflare transport not initialized");
            Err(RouterError::NotInitialized)
        }
    }

    /// Записать выбор маршрута в зашифрованный лог
    async fn log_route(&self, peer_id: &str, route: Route, success: bool, reason: &str) {
        use sha3::{Digest, Sha3_256};

        // Хэшировать ID для приватности
        let sender_hash = format!("{:x}", Sha3_256::digest(b"local_peer"));
        let recipient_hash = format!("{:x}", Sha3_256::digest(peer_id.as_bytes()));
        let reason_hash = format!("{:x}", Sha3_256::digest(reason.as_bytes()));

        let log_entry = EncryptedRouteLog {
            sender_hash,
            recipient_hash,
            route,
            timestamp: chrono::Utc::now().timestamp_millis(),
            reason_hash,
            success,
        };

        let mut log = self.route_log.write().await;
        log.push(log_entry);

        // Ограничить размер лога
        if log.len() > self.max_log_size {
            let remove_count = log.len() - self.max_log_size;
            log.drain(..remove_count);
        }

        debug!(
            "Route logged: {} -> {} (success: {})",
            route, peer_id, success
        );
    }

    /// Получить статистику маршрутизации
    pub async fn get_stats(&self) -> RouteStats {
        self.stats.to_stats()
    }

    /// Получить лог маршрутизации
    pub async fn get_route_log(&self) -> Vec<EncryptedRouteLog> {
        self.route_log.read().await.clone()
    }

    /// Очистить кэш пиров
    pub async fn clear_peer_cache(&self) {
        let mut cache = self.peer_cache.write().await;
        cache.clear();
        info!("Peer cache cleared");
    }

    /// Обновить статус peer в кэше
    pub async fn update_peer_status(&self, peer_id: &str, available: bool) {
        let mut cache = self.peer_cache.write().await;
        cache.insert(peer_id.to_string(), available);
        debug!("Peer {} status updated: available={}", peer_id, available);
    }

    /// Получить размер очереди Cloudflare
    pub async fn get_cloudflare_queue_size(&self) -> Option<usize> {
        let cf_guard = self.cloudflare_transport.read().await;
        if let Some(cf) = cf_guard.as_ref() {
            cf.queue_size().await.ok()
        } else {
            None
        }
    }

    /// Принудительно отправить очередь Cloudflare
    pub async fn flush_cloudflare_queue(&self) -> Result<usize, RouterError> {
        let cf_guard = self.cloudflare_transport.read().await;
        if let Some(cf) = cf_guard.as_ref() {
            let sent = cf.flush_queue().await?;
            info!("Flushed {} messages from Cloudflare queue", sent);
            Ok(sent)
        } else {
            Err(RouterError::NotInitialized)
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
    fn test_route_display() {
        assert_eq!(format!("{}", Route::P2P), "P2P");
        assert_eq!(format!("{}", Route::Cloudflare), "Cloudflare");
    }

    #[test]
    fn test_route_serialization() {
        let p2p = Route::P2P;
        let json = serde_json::to_string(&p2p).unwrap();
        assert_eq!(json, "\"P2P\"");

        let cf = Route::Cloudflare;
        let json = serde_json::to_string(&cf).unwrap();
        assert_eq!(json, "\"Cloudflare\"");
    }

    #[test]
    fn test_route_stats() {
        let stats = AtomicRouteStats::new();

        stats.record_attempt(Route::P2P);
        stats.record_attempt(Route::P2P);
        stats.record_success(Route::P2P);

        stats.record_attempt(Route::Cloudflare);
        stats.record_success(Route::Cloudflare);

        let s = stats.to_stats();
        assert_eq!(s.p2p_attempts, 2);
        assert_eq!(s.p2p_successes, 1);
        assert_eq!(s.cloudflare_attempts, 1);
        assert_eq!(s.cloudflare_successes, 1);
        assert_eq!(s.total_messages, 3);
    }

    #[test]
    fn test_encrypted_route_log() {
        let log = EncryptedRouteLog {
            sender_hash: "abc123".to_string(),
            recipient_hash: "def456".to_string(),
            route: Route::P2P,
            timestamp: 1234567890,
            reason_hash: "ghi789".to_string(),
            success: true,
        };

        let json = serde_json::to_string(&log).unwrap();
        let deserialized: EncryptedRouteLog = serde_json::from_str(&json).unwrap();

        assert_eq!(log.sender_hash, deserialized.sender_hash);
        assert_eq!(log.recipient_hash, deserialized.recipient_hash);
        assert_eq!(log.route, deserialized.route);
        assert_eq!(log.timestamp, deserialized.timestamp);
        assert_eq!(log.success, deserialized.success);
    }

    #[tokio::test]
    async fn test_router_creation() {
        let router = TransportRouter::new(None, None, 100);
        let stats = router.get_stats().await;
        assert_eq!(stats.total_messages, 0);
    }

    #[tokio::test]
    async fn test_route_selection_no_p2p() {
        let router = TransportRouter::new(None, None, 100);

        // Без P2P node должен выбрать Cloudflare
        let route = router.select_route("test-peer").await;
        assert_eq!(route, Route::Cloudflare);
    }

    #[tokio::test]
    async fn test_peer_cache_update() {
        let router = TransportRouter::new(None, None, 100);

        router.update_peer_status("peer-1", true).await;
        router.update_peer_status("peer-2", false).await;

        {
            let cache = router.peer_cache.read().await;
            assert_eq!(cache.get("peer-1"), Some(&true));
            assert_eq!(cache.get("peer-2"), Some(&false));
        }

        router.clear_peer_cache().await;

        {
            let cache = router.peer_cache.read().await;
            assert!(cache.is_empty());
        }
    }
}
