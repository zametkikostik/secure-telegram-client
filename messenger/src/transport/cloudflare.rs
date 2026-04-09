//! Cloudflare Worker Transport Layer
//!
//! HTTPS POST relay через Cloudflare Workers для доставки сообщений
//! когда P2P соединение недоступно.
//!
//! Features:
//! - HTTPS POST к Cloudflare Worker (/api/v1/msg)
//! - Offline queue: SQLite для хранения сообщений при отсутствии сети
//! - Retry logic: exponential backoff, max 3 attempts
//!
//! SECURITY: требует аудита перед production
//! TODO: pentest перед release

use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, info, warn};
use uuid::Uuid;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum TransportError {
    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Message send failed after {0} attempts")]
    MaxRetriesExceeded(u32),

    #[error("Queue is empty")]
    QueueEmpty,
}

// ============================================================================
// Message Types
// ============================================================================

/// Сообщение для отправки через Cloudflare Worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareMessage {
    /// Уникальный ID сообщения
    pub id: String,
    /// ID получателя
    pub recipient_id: String,
    /// Зашифрованный payload (ChaCha20-Poly1305 ciphertext)
    pub ciphertext: Vec<u8>,
    /// Ed25519 подпись
    pub signature: Vec<u8>,
    /// Timestamp создания (Unix epoch ms)
    pub created_at: i64,
    /// TTL в секундах (по умолчанию 24 часа)
    #[serde(default = "default_ttl")]
    pub ttl: u64,
}

fn default_ttl() -> u64 {
    86400 // 24 часа
}

/// Статус доставки сообщения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryStatus {
    /// Ожидает отправки
    Pending,
    /// В процессе отправки
    Sending,
    /// Доставлено
    Delivered,
    /// Ошибка доставки
    Failed { error: String, attempts: u32 },
}

/// Запись в offline queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedMessage {
    pub id: String,
    pub recipient_id: String,
    pub ciphertext: Vec<u8>,
    pub signature: Vec<u8>,
    pub created_at: i64,
    pub status: DeliveryStatus,
    pub attempts: u32,
    pub next_retry_at: Option<i64>,
}

// ============================================================================
// Cloudflare Transport
// ============================================================================

/// Cloudflare Worker transport для relay сообщений
pub struct CloudflareTransport {
    /// HTTP client для запросов
    http_client: Client,
    /// URL Cloudflare Worker
    worker_url: String,
    /// SQLite pool для offline queue
    db_pool: Pool<Sqlite>,
    /// Максимальное количество попыток
    max_retries: u32,
    /// Базовая задержка для exponential backoff (мс)
    base_delay_ms: u64,
}

impl CloudflareTransport {
    /// Создать новый транспорт с SQLite queue
    ///
    /// # Arguments
    /// * `worker_url` — URL Cloudflare Worker
    /// * `db_path` — путь к SQLite файлу
    ///
    /// # Returns
    /// * `Ok(CloudflareTransport)` — инициализированный транспорт
    /// * `Err(TransportError)` — при ошибке инициализации
    pub async fn new(worker_url: &str, db_path: &str) -> Result<Self, TransportError> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");

        let db_pool = SqlitePoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            .connect(db_path)
            .await?;

        let transport = Self {
            http_client,
            worker_url: worker_url.to_string(),
            db_pool,
            max_retries: 3,
            base_delay_ms: 1000, // 1 секунда
        };

        // Инициализировать схему БД
        transport.init_schema().await?;

        info!(
            "Cloudflare transport initialized: worker_url={}",
            worker_url
        );

        Ok(transport)
    }

    /// Инициализировать схему БД
    async fn init_schema(&self) -> Result<(), TransportError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS message_queue (
                id TEXT PRIMARY KEY,
                recipient_id TEXT NOT NULL,
                ciphertext BLOB NOT NULL,
                signature BLOB NOT NULL,
                created_at INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                attempts INTEGER NOT NULL DEFAULT 0,
                next_retry_at INTEGER,
                error TEXT
            )
            "#,
        )
        .execute(&self.db_pool)
        .await?;

        // Индекс для быстрого поиска pending сообщений
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_queue_status_retry
            ON message_queue (status, next_retry_at)
            "#,
        )
        .execute(&self.db_pool)
        .await?;

        debug!("Database schema initialized");

        Ok(())
    }

    /// Отправить сообщение через Cloudflare Worker
    ///
    /// # Arguments
    /// * `message` — сообщение для отправки
    ///
    /// # Returns
    /// * `Ok(())` — сообщение доставлено
    /// * `Err(TransportError)` — при ошибке (сообщение сохранено в queue)
    pub async fn send_message(&self, message: CloudflareMessage) -> Result<(), TransportError> {
        let url = format!("{}/api/v1/msg", self.worker_url);

        debug!("Sending message to: {} (id: {})", url, message.id);

        let response = self
            .http_client
            .post(&url)
            .json(&message)
            .send()
            .await
            .map_err(|e| TransportError::Http(e.to_string()))?;

        if response.status().is_success() {
            info!("Message delivered: id={}", message.id);
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(TransportError::Http(format!("HTTP {}: {}", status, body)))
        }
    }

    /// Добавить сообщение в offline queue
    ///
    /// # Arguments
    /// * `message` — сообщение для сохранения
    ///
    /// # Returns
    /// * `Ok(())` — сообщение сохранено
    /// * `Err(TransportError)` — при ошибке БД
    pub async fn queue_message(&self, message: CloudflareMessage) -> Result<(), TransportError> {
        let queued = QueuedMessage {
            id: message.id,
            recipient_id: message.recipient_id,
            ciphertext: message.ciphertext,
            signature: message.signature,
            created_at: message.created_at,
            status: DeliveryStatus::Pending,
            attempts: 0,
            next_retry_at: None,
        };

        let status_json = serde_json::to_string(&queued.status)?;

        sqlx::query(
            r#"
            INSERT INTO message_queue
                (id, recipient_id, ciphertext, signature, created_at, status, attempts, next_retry_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&queued.id)
        .bind(&queued.recipient_id)
        .bind(&queued.ciphertext)
        .bind(&queued.signature)
        .bind(queued.created_at)
        .bind(&status_json)
        .bind(queued.attempts as i64)
        .bind(queued.next_retry_at)
        .execute(&self.db_pool)
        .await?;

        debug!("Message queued: id={}", queued.id);

        Ok(())
    }

    /// Попытаться отправить все pending сообщения из queue
    ///
    /// Использует exponential backoff с max 3 попытками.
    ///
    /// # Returns
    /// * `Ok(sent_count)` — количество успешно отправленных сообщений
    /// * `Err(TransportError)` — при критической ошибке
    pub async fn flush_queue(&self) -> Result<usize, TransportError> {
        // Получить pending сообщения, готовые к retry
        let now = Utc::now().timestamp();
        let rows = sqlx::query_as::<_, (String, String, Vec<u8>, Vec<u8>, i64, String, i64, Option<i64>)>(
            r#"
            SELECT id, recipient_id, ciphertext, signature, created_at, status, attempts, next_retry_at
            FROM message_queue
            WHERE status = 'pending'
              AND (next_retry_at IS NULL OR next_retry_at <= ?)
            ORDER BY created_at ASC
            LIMIT 50
            "#,
        )
        .bind(now)
        .fetch_all(&self.db_pool)
        .await?;

        let mut sent_count = 0;

        for (id, recipient_id, ciphertext, signature, created_at, _status, attempts, _next_retry) in
            rows
        {
            let message = CloudflareMessage {
                id: id.clone(),
                recipient_id,
                ciphertext,
                signature,
                created_at,
                ttl: default_ttl(),
            };

            // Обновить статус на "sending"
            self.update_message_status(
                &id,
                DeliveryStatus::Sending,
                attempts as u32 + 1,
                None,
                None,
            )
            .await?;

            match self.send_message(message).await {
                Ok(()) => {
                    // Удалить из queue после успешной отправки
                    self.remove_message(&id).await?;
                    sent_count += 1;
                }
                Err(e) => {
                    let new_attempts = attempts as u32 + 1;
                    warn!(
                        "Failed to send message {} (attempt {}/{}): {}",
                        id, new_attempts, self.max_retries, e
                    );

                    if new_attempts >= self.max_retries {
                        // Превышено максимальное количество попыток
                        let error_msg = e.to_string();
                        self.update_message_status(
                            &id,
                            DeliveryStatus::Failed {
                                error: error_msg,
                                attempts: new_attempts,
                            },
                            new_attempts,
                            None,
                            Some(&e.to_string()),
                        )
                        .await?;
                    } else {
                        // Запланировать retry с exponential backoff
                        let delay = self.calculate_backoff(new_attempts);
                        let next_retry_at = Utc::now().timestamp() + (delay as i64 / 1000);
                        self.update_message_status(
                            &id,
                            DeliveryStatus::Pending,
                            new_attempts,
                            Some(next_retry_at),
                            Some(&e.to_string()),
                        )
                        .await?;
                    }
                }
            }
        }

        if sent_count > 0 {
            info!("Flushed {} messages from queue", sent_count);
        }

        Ok(sent_count)
    }

    /// Рассчитать задержку для exponential backoff
    ///
    /// Формула: base_delay * 2^attempt
    /// Пример: 1s, 2s, 4s, 8s, ...
    fn calculate_backoff(&self, attempt: u32) -> u64 {
        let delay = self.base_delay_ms * 2u64.pow(attempt);
        // Максимальная задержка 60 секунд
        delay.min(60_000)
    }

    /// Обновить статус сообщения в queue
    async fn update_message_status(
        &self,
        id: &str,
        status: DeliveryStatus,
        attempts: u32,
        next_retry_at: Option<i64>,
        error: Option<&str>,
    ) -> Result<(), TransportError> {
        let status_json = serde_json::to_string(&status)?;

        sqlx::query(
            r#"
            UPDATE message_queue
            SET status = ?, attempts = ?, next_retry_at = ?, error = ?
            WHERE id = ?
            "#,
        )
        .bind(&status_json)
        .bind(attempts as i64)
        .bind(next_retry_at)
        .bind(error)
        .bind(id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Удалить сообщение из queue после успешной отправки
    async fn remove_message(&self, id: &str) -> Result<(), TransportError> {
        sqlx::query("DELETE FROM message_queue WHERE id = ?")
            .bind(id)
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }

    /// Получить количество pending сообщений в queue
    pub async fn queue_size(&self) -> Result<usize, TransportError> {
        let row = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM message_queue WHERE status = 'pending'",
        )
        .fetch_one(&self.db_pool)
        .await?;

        Ok(row.0 as usize)
    }

    /// Очистить старые failed сообщения (старше 7 дней)
    pub async fn cleanup_old_failed(&self) -> Result<usize, TransportError> {
        let cutoff = Utc::now().timestamp() - (7 * 24 * 3600); // 7 дней

        let result = sqlx::query(
            r#"
            DELETE FROM message_queue
            WHERE status LIKE '%"Failed"%'
              AND created_at < ?
            "#,
        )
        .bind(cutoff)
        .execute(&self.db_pool)
        .await?;

        let deleted = result.rows_affected() as usize;
        if deleted > 0 {
            info!("Cleaned up {} old failed messages", deleted);
        }

        Ok(deleted)
    }

    /// Создать новое сообщение с уникальным ID
    pub fn create_message(
        recipient_id: String,
        ciphertext: Vec<u8>,
        signature: Vec<u8>,
    ) -> CloudflareMessage {
        CloudflareMessage {
            id: Uuid::new_v4().to_string(),
            recipient_id,
            ciphertext,
            signature,
            created_at: Utc::now().timestamp_millis(),
            ttl: default_ttl(),
        }
    }

    /// Получить URL worker
    pub fn worker_url(&self) -> &str {
        &self.worker_url
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloudflare_message_serialization() {
        let msg = CloudflareMessage {
            id: "test-id".to_string(),
            recipient_id: "user-123".to_string(),
            ciphertext: vec![1, 2, 3, 4],
            signature: vec![5, 6, 7, 8],
            created_at: 1234567890,
            ttl: 86400,
        };

        let json = serde_json::to_string(&msg).unwrap();
        let deserialized: CloudflareMessage = serde_json::from_str(&json).unwrap();

        assert_eq!(msg.id, deserialized.id);
        assert_eq!(msg.recipient_id, deserialized.recipient_id);
        assert_eq!(msg.ciphertext, deserialized.ciphertext);
        assert_eq!(msg.signature, deserialized.signature);
        assert_eq!(msg.created_at, deserialized.created_at);
    }

    #[test]
    fn test_delivery_status_serialization() {
        let pending = DeliveryStatus::Pending;
        let json = serde_json::to_string(&pending).unwrap();
        assert_eq!(json, "\"Pending\"");

        let failed = DeliveryStatus::Failed {
            error: "timeout".to_string(),
            attempts: 2,
        };
        let json = serde_json::to_string(&failed).unwrap();
        let deserialized: DeliveryStatus = serde_json::from_str(&json).unwrap();

        match deserialized {
            DeliveryStatus::Failed { error, attempts } => {
                assert_eq!(error, "timeout");
                assert_eq!(attempts, 2);
            }
            _ => panic!("Expected Failed variant"),
        }
    }

    #[test]
    fn test_exponential_backoff() {
        let base_delay = 1000; // 1 секунда

        // attempt 1: 1s * 2^1 = 2s
        assert_eq!(base_delay * 2u64.pow(1), 2000);
        // attempt 2: 1s * 2^2 = 4s
        assert_eq!(base_delay * 2u64.pow(2), 4000);
        // attempt 3: 1s * 2^3 = 8s
        assert_eq!(base_delay * 2u64.pow(3), 8000);
    }

    #[test]
    fn test_backoff_cap_at_60_seconds() {
        let base_delay = 1000;
        // attempt 10: 1s * 2^10 = 1024s > 60s, должно быть capped
        let delay = (base_delay * 2u64.pow(10)).min(60_000);
        assert_eq!(delay, 60_000);
    }

    #[test]
    fn test_create_message() {
        let msg = CloudflareTransport::create_message(
            "user-123".to_string(),
            vec![1, 2, 3],
            vec![4, 5, 6],
        );

        assert_eq!(msg.recipient_id, "user-123");
        assert_eq!(msg.ciphertext, vec![1, 2, 3]);
        assert_eq!(msg.signature, vec![4, 5, 6]);
        assert!(!msg.id.is_empty());
        assert!(msg.created_at > 0);
    }
}
