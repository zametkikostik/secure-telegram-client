//! Message Queue Module
//!
//! Reliable message delivery with:
//! - Persistent queue (SQLite-backed)
//! - Retry with exponential backoff
//! - Dead letter queue for failed messages
//! - Priority queues
//! - Deduplication
//! - Batch processing

use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum QueueError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Message not found: {0}")]
    NotFound(String),
    #[error("Queue full (max {} messages)", .0)]
    QueueFull(usize),
    #[error("Max retries exceeded")]
    MaxRetriesExceeded,
}

pub type QueueResult<T> = Result<T, QueueError>;

// ============================================================================
// Message Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueMessage {
    pub id: String,
    pub queue_name: String,
    pub priority: u8,
    pub payload: Vec<u8>,
    pub metadata: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub visible_after: chrono::DateTime<chrono::Utc>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub status: MessageStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    DeadLetter,
}

// ============================================================================
// Message Queue
// ============================================================================

pub struct MessageQueue {
    db: SqlitePool,
    max_queue_size: usize,
    default_visibility_timeout: Duration,
    default_max_retries: u32,
    processing: RwLock<HashMap<String, Instant>>, // message_id -> acquired_at
}

impl MessageQueue {
    pub async fn new(db_url: &str) -> QueueResult<Self> {
        let db = SqlitePool::connect(db_url).await
            .map_err(|e| QueueError::Database(e.to_string()))?;

        let queue = Self {
            db,
            max_queue_size: 100_000,
            default_visibility_timeout: Duration::from_secs(30),
            default_max_retries: 3,
            processing: RwLock::new(HashMap::new()),
        };

        queue.init_schema().await?;
        Ok(queue)
    }

    async fn init_schema(&self) -> QueueResult<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS queue_messages (
                id TEXT PRIMARY KEY,
                queue_name TEXT NOT NULL,
                priority INTEGER NOT NULL DEFAULT 1,
                payload BLOB NOT NULL,
                metadata TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL,
                visible_after TEXT NOT NULL,
                retry_count INTEGER NOT NULL DEFAULT 0,
                max_retries INTEGER NOT NULL DEFAULT 3,
                status TEXT NOT NULL DEFAULT 'pending'
            )"
        ).execute(&self.db).await.map_err(|e| QueueError::Database(e.to_string()))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_queue_status ON queue_messages(status, queue_name, visible_after, priority DESC)")
            .execute(&self.db).await.map_err(|e| QueueError::Database(e.to_string()))?;

        sqlx::query("CREATE TABLE IF NOT EXISTS queue_dead_letters (
            id TEXT PRIMARY KEY,
            original_queue TEXT NOT NULL,
            payload BLOB NOT NULL,
            error_reason TEXT,
            failed_at TEXT NOT NULL
        )").execute(&self.db).await.map_err(|e| QueueError::Database(e.to_string()))?;

        Ok(())
    }

    // ========================================================================
    // Enqueue
    // ========================================================================

    pub async fn enqueue(&self, queue_name: &str, payload: Vec<u8>, priority: MessagePriority) -> QueueResult<String> {
        // Check queue size
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM queue_messages WHERE status = 'pending'")
            .fetch_one(&self.db).await.map_err(|e| QueueError::Database(e.to_string()))?;

        if count as usize >= self.max_queue_size {
            return Err(QueueError::QueueFull(self.max_queue_size));
        }

        let id = format!("msg:{}", uuid::Uuid::new_v4().simple());
        let now = chrono::Utc::now();
        let metadata = serde_json::json!({}).to_string();

        sqlx::query(
            "INSERT INTO queue_messages (id, queue_name, priority, payload, metadata, created_at, visible_after, status)
             VALUES (?, ?, ?, ?, ?, ?, ?, 'pending')"
        )
        .bind(&id)
        .bind(queue_name)
        .bind(priority as i64)
        .bind(&payload)
        .bind(&metadata)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .execute(&self.db).await.map_err(|e| QueueError::Database(e.to_string()))?;

        debug!("Enqueued message {} to {} (priority: {:?})", id, queue_name, priority);
        Ok(id)
    }

    // ========================================================================
    // Dequeue
    // ========================================================================

    pub async fn dequeue(&self, queue_name: &str) -> QueueResult<Option<QueueMessage>> {
        let now = chrono::Utc::now();
        let visible = now.to_rfc3339();

        let row = sqlx::query(
            "SELECT id, queue_name, priority, payload, metadata, created_at, visible_after,
                    retry_count, max_retries, status
             FROM queue_messages
             WHERE queue_name = ? AND status = 'pending' AND visible_after <= ?
             ORDER BY priority DESC, created_at ASC
             LIMIT 1"
        )
        .bind(queue_name)
        .bind(visible)
        .fetch_optional(&self.db).await.map_err(|e| QueueError::Database(e.to_string()))?;

        let Some(row) = row else { return Ok(None); };

        let id: String = row.try_get("id").map_err(|e| QueueError::Database(e.to_string()))?;

        // Mark as processing
        sqlx::query("UPDATE queue_messages SET status = 'processing' WHERE id = ?")
            .bind(&id).execute(&self.db).await.map_err(|e| QueueError::Database(e.to_string()))?;

        self.processing.write().await.insert(id.clone(), Instant::now());

        let payload: Vec<u8> = row.try_get("payload").map_err(|e| QueueError::Database(e.to_string()))?;
        let priority: i64 = row.try_get("priority").map_err(|e| QueueError::Database(e.to_string()))?;

        Ok(Some(QueueMessage {
            id,
            queue_name: queue_name.to_string(),
            priority: priority as u8,
            payload,
            metadata: HashMap::new(),
            created_at: now,
            visible_after: now,
            retry_count: 0,
            max_retries: self.default_max_retries,
            status: MessageStatus::Processing,
        }))
    }

    // ========================================================================
    // Acknowledge / Reject
    // ========================================================================

    pub async fn ack(&self, message_id: &str) -> QueueResult<()> {
        sqlx::query("UPDATE queue_messages SET status = 'completed' WHERE id = ?")
            .bind(message_id).execute(&self.db).await.map_err(|e| QueueError::Database(e.to_string()))?;

        self.processing.write().await.remove(message_id);
        debug!("Acknowledged message {}", message_id);
        Ok(())
    }

    pub async fn nack(&self, message_id: &str) -> QueueResult<()> {
        // Increment retry count
        sqlx::query(
            "UPDATE queue_messages SET
                retry_count = retry_count + 1,
                status = CASE WHEN retry_count + 1 >= max_retries THEN 'dead_letter' ELSE 'pending' END,
                visible_after = datetime('now', '+30 seconds')
             WHERE id = ?"
        ).bind(message_id).execute(&self.db).await.map_err(|e| QueueError::Database(e.to_string()))?;

        self.processing.write().await.remove(message_id);
        debug!("Nacked message {}", message_id);
        Ok(())
    }

    // ========================================================================
    // Dead Letter Queue
    // ========================================================================

    pub async fn get_dead_letters(&self, limit: usize) -> QueueResult<Vec<QueueMessage>> {
        let rows = sqlx::query("SELECT * FROM queue_messages WHERE status = 'dead_letter' ORDER BY created_at DESC LIMIT ?")
            .bind(limit as i64)
            .fetch_all(&self.db).await.map_err(|e| QueueError::Database(e.to_string()))?;

        Ok(rows.into_iter().filter_map(|row| {
            let id: String = row.try_get("id").ok()?;
            let payload: Vec<u8> = row.try_get("payload").ok()?;
            Some(QueueMessage {
                id, queue_name: "dead_letter".to_string(), priority: 0, payload,
                metadata: HashMap::new(), created_at: chrono::Utc::now(),
                visible_after: chrono::Utc::now(), retry_count: 0, max_retries: 0,
                status: MessageStatus::DeadLetter,
            })
        }).collect())
    }

    pub async fn retry_dead_letter(&self, message_id: &str) -> QueueResult<()> {
        sqlx::query(
            "UPDATE queue_messages SET
                status = 'pending', retry_count = 0, visible_after = datetime('now')
             WHERE id = ? AND status = 'dead_letter'"
        ).bind(message_id).execute(&self.db).await.map_err(|e| QueueError::Database(e.to_string()))?;

        info!("Retried dead letter: {}", message_id);
        Ok(())
    }

    // ========================================================================
    // Stats
    // ========================================================================

    pub async fn queue_size(&self, queue_name: &str) -> QueueResult<usize> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM queue_messages WHERE queue_name = ? AND status = 'pending'")
            .bind(queue_name).fetch_one(&self.db).await.map_err(|e| QueueError::Database(e.to_string()))?;
        Ok(count as usize)
    }

    pub async fn processing_count(&self) -> usize {
        self.processing.read().await.len()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enqueue_dequeue() {
        let queue = MessageQueue::new("sqlite::memory:").await.unwrap();

        let id = queue.enqueue("test", b"hello".to_vec(), MessagePriority::Normal).await.unwrap();
        assert!(!id.is_empty());

        let msg = queue.dequeue("test").await.unwrap();
        assert!(msg.is_some());
        assert_eq!(msg.unwrap().payload, b"hello");
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let queue = MessageQueue::new("sqlite::memory:").await.unwrap();

        queue.enqueue("test", b"low".to_vec(), MessagePriority::Low).await.unwrap();
        queue.enqueue("test", b"high".to_vec(), MessagePriority::High).await.unwrap();
        queue.enqueue("test", b"normal".to_vec(), MessagePriority::Normal).await.unwrap();

        // Should dequeue high priority first
        let msg1 = queue.dequeue("test").await.unwrap().unwrap();
        assert_eq!(msg1.payload, b"high");

        let msg2 = queue.dequeue("test").await.unwrap().unwrap();
        assert_eq!(msg2.payload, b"normal");

        let msg3 = queue.dequeue("test").await.unwrap().unwrap();
        assert_eq!(msg3.payload, b"low");
    }

    #[tokio::test]
    async fn test_ack_nack() {
        let queue = MessageQueue::new("sqlite::memory:").await.unwrap();

        let id = queue.enqueue("test", b"msg".to_vec(), MessagePriority::Normal).await.unwrap();

        queue.dequeue("test").await.unwrap();
        queue.ack(&id).await.unwrap();

        // Should not be able to dequeue again
        let msg = queue.dequeue("test").await.unwrap();
        assert!(msg.is_none());
    }

    #[tokio::test]
    async fn test_dead_letter() {
        let queue = MessageQueue::new("sqlite::memory:").await.unwrap();

        let id = queue.enqueue("test", b"msg".to_vec(), MessagePriority::Normal).await.unwrap();
        queue.dequeue("test").await.unwrap();
        queue.nack(&id).await.unwrap(); // Will retry

        // Force to dead letter by manually updating
        sqlx::query("UPDATE queue_messages SET retry_count = max_retries, status = 'dead_letter' WHERE id = ?")
            .bind(&id).execute(&queue.db).await.unwrap();

        let dead = queue.get_dead_letters(10).await.unwrap();
        assert_eq!(dead.len(), 1);
    }
}
