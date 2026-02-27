//! Локальная очередь сообщений с шифрованием
//!
//! Использует SQLite + SQLCipher для безопасного хранения:
//! - Зашифрованная база данных
//! - Очередь исходящих сообщений
//! - Кэш входящих сообщений
//! - Синхронизация при появлении канала

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// Менеджер очереди сообщений
pub struct MessageQueue {
    /// SQLite соединение
    conn: Connection,
}

/// Сообщение в очереди
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedMessage {
    /// Уникальный ID
    pub id: i64,
    /// Отправитель
    pub from: String,
    /// Получатель
    pub to: String,
    /// Содержимое (зашифровано)
    pub encrypted_content: Vec<u8>,
    /// Временная метка создания
    pub created_at: i64,
    /// Статус отправки
    pub status: MessageStatus,
    /// Количество попыток отправки
    pub retry_count: u32,
}

/// Статус сообщения
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageStatus {
    /// Ожидает отправки
    Pending,
    /// В процессе отправки
    Sending,
    /// Отправлено
    Sent,
    /// Ошибка отправки
    Failed,
}

impl MessageQueue {
    /// Создание новой очереди с шифрованием
    pub fn new(db_path: &str, encryption_key: &str) -> Result<Self> {
        // Открытие базы с шифрованием
        let conn = Connection::open(db_path).context("Ошибка открытия базы данных")?;

        // Установка ключа шифрования
        conn.execute_batch(&format!("PRAGMA key = '{}';", encryption_key))
            .context("Ошибка установки ключа шифрования")?;

        // Создание таблиц
        Self::create_tables(&conn)?;

        Ok(Self { conn })
    }

    /// Создание таблиц
    fn create_tables(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                from_user TEXT NOT NULL,
                to_user TEXT NOT NULL,
                encrypted_content BLOB NOT NULL,
                created_at INTEGER NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                retry_count INTEGER NOT NULL DEFAULT 0,
                sent_at INTEGER,
                error_message TEXT
            )",
            [],
        )
        .context("Ошибка создания таблицы messages")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sync_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )
        .context("Ошибка создания таблицы sync_state")?;

        // Индексы для ускорения поиска
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_status ON messages(status)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_created ON messages(created_at)",
            [],
        )?;

        Ok(())
    }

    /// Добавление сообщения в очередь
    pub fn enqueue(&self, msg: &QueuedMessage) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO messages (from_user, to_user, encrypted_content, created_at, status, retry_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                msg.from,
                msg.to,
                msg.encrypted_content,
                msg.created_at,
                Self::status_to_string(&msg.status),
                msg.retry_count
            ],
        )
        .context("Ошибка добавления сообщения в очередь")?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Получение сообщений для отправки
    pub fn get_pending_messages(&self, limit: usize) -> Result<Vec<QueuedMessage>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, from_user, to_user, encrypted_content, created_at, status, retry_count
             FROM messages
             WHERE status = ?1
             ORDER BY created_at ASC
             LIMIT ?2",
        )?;

        let messages = stmt.query_map(
            params![
                Self::status_to_string(&MessageStatus::Pending),
                limit as i64
            ],
            |row| {
                Ok(QueuedMessage {
                    id: row.get(0)?,
                    from: row.get(1)?,
                    to: row.get(2)?,
                    encrypted_content: row.get(3)?,
                    created_at: row.get(4)?,
                    status: Self::string_to_status(&row.get::<_, String>(5)?),
                    retry_count: row.get(6)?,
                })
            },
        )?;

        let result: Vec<QueuedMessage> = messages.filter_map(|r| r.ok()).collect();

        Ok(result)
    }

    /// Обновление статуса сообщения
    pub fn update_status(&self, id: i64, status: MessageStatus, error: Option<&str>) -> Result<()> {
        let sent_at = if status == MessageStatus::Sent {
            Some(chrono::Utc::now().timestamp())
        } else {
            None
        };

        self.conn
            .execute(
                "UPDATE messages
             SET status = ?1,
                 sent_at = ?2,
                 error_message = ?3
             WHERE id = ?4",
                params![Self::status_to_string(&status), sent_at, error, id],
            )
            .context("Ошибка обновления статуса")?;

        Ok(())
    }

    /// Увеличение счётчика попыток
    pub fn increment_retry(&self, id: i64) -> Result<u32> {
        self.conn.execute(
            "UPDATE messages
             SET retry_count = retry_count + 1
             WHERE id = ?1",
            params![id],
        )?;

        // Получение нового значения
        let retry_count: u32 = self.conn.query_row(
            "SELECT retry_count FROM messages WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;

        Ok(retry_count)
    }

    /// Удаление старых сообщений
    pub fn cleanup_old_messages(&self, max_age_days: i64) -> Result<usize> {
        let cutoff_timestamp = chrono::Utc::now().timestamp() - (max_age_days * 24 * 60 * 60);

        let affected = self.conn.execute(
            "DELETE FROM messages
             WHERE created_at < ?1
             AND status IN (?2, ?3)",
            params![
                cutoff_timestamp,
                Self::status_to_string(&MessageStatus::Sent),
                Self::status_to_string(&MessageStatus::Failed),
            ],
        )?;

        Ok(affected as usize)
    }

    /// Получение статистики очереди
    pub fn get_queue_stats(&self) -> Result<QueueStats> {
        let stats = self.conn.query_row(
            "SELECT
                COUNT(*) as total,
                SUM(CASE WHEN status = 'pending' THEN 1 ELSE 0 END) as pending,
                SUM(CASE WHEN status = 'sent' THEN 1 ELSE 0 END) as sent,
                SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) as failed,
                AVG(retry_count) as avg_retries
             FROM messages",
            [],
            |row| {
                Ok(QueueStats {
                    total: row.get(0)?,
                    pending: row.get(1)?,
                    sent: row.get(2)?,
                    failed: row.get(3)?,
                    avg_retries: row.get(4)?,
                })
            },
        )?;

        Ok(stats)
    }

    /// Сохранение состояния синхронизации
    pub fn save_sync_state(&self, key: &str, value: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        self.conn.execute(
            "INSERT OR REPLACE INTO sync_state (key, value, updated_at)
             VALUES (?1, ?2, ?3)",
            params![key, value, now],
        )?;

        Ok(())
    }

    /// Получение состояния синхронизации
    pub fn get_sync_state(&self, key: &str) -> Result<Option<String>> {
        let value: Result<String, rusqlite::Error> = self.conn.query_row(
            "SELECT value FROM sync_state WHERE key = ?1",
            params![key],
            |row| row.get(0),
        );

        match value {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Ошибка получения состояния синхронизации"),
        }
    }

    /// Обновление состояния синхронизации
    pub fn update_sync_state(&self, key: &str, value: &str) -> Result<bool> {
        let now = chrono::Utc::now().timestamp();

        let rows_changed = self.conn.execute(
            "UPDATE sync_state SET value = ?1, updated_at = ?2 WHERE key = ?3",
            params![value, now, key],
        )?;

        Ok(rows_changed > 0)
    }

    /// Удаление состояния синхронизации
    pub fn delete_sync_state(&self, key: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM sync_state WHERE key = ?1", params![key])?;
        Ok(())
    }

    /// Получение всех состояний синхронизации
    pub fn get_all_sync_states(&self) -> Result<Vec<(String, String, i64)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT key, value, updated_at FROM sync_state ORDER BY updated_at DESC")?;

        let states = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })?;

        let result: Vec<(String, String, i64)> = states.filter_map(|r| r.ok()).collect();
        Ok(result)
    }

    fn status_to_string(status: &MessageStatus) -> &'static str {
        match status {
            MessageStatus::Pending => "pending",
            MessageStatus::Sending => "sending",
            MessageStatus::Sent => "sent",
            MessageStatus::Failed => "failed",
        }
    }

    fn string_to_status(s: &str) -> MessageStatus {
        match s {
            "pending" => MessageStatus::Pending,
            "sending" => MessageStatus::Sending,
            "sent" => MessageStatus::Sent,
            "failed" => MessageStatus::Failed,
            _ => MessageStatus::Pending,
        }
    }
}

/// Статистика очереди
#[derive(Debug, Clone)]
pub struct QueueStats {
    pub total: i64,
    pub pending: i64,
    pub sent: i64,
    pub failed: i64,
    pub avg_retries: f64,
}

/// Конфигурация очереди сообщений
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageQueueConfig {
    /// Путь к базе данных
    pub db_path: String,
    /// Ключ шифрования
    pub encryption_key: String,
    /// Максимальный возраст сообщений (дни)
    pub max_age_days: i64,
    /// Максимальное количество попыток
    pub max_retries: u32,
}

impl Default for MessageQueueConfig {
    fn default() -> Self {
        Self {
            db_path: "./data/messages.db".to_string(),
            encryption_key: "change-me-in-production".to_string(),
            max_age_days: 30,
            max_retries: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_queue_operations() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();

        let queue = MessageQueue::new(db_path, "test-key").unwrap();

        // Добавление сообщения
        let msg = QueuedMessage {
            id: 0,
            from: "user1".to_string(),
            to: "user2".to_string(),
            encrypted_content: vec![1, 2, 3],
            created_at: chrono::Utc::now().timestamp(),
            status: MessageStatus::Pending,
            retry_count: 0,
        };

        let id = queue.enqueue(&msg).unwrap();
        assert!(id > 0);

        // Получение pending сообщений
        let pending = queue.get_pending_messages(10).unwrap();
        assert_eq!(pending.len(), 1);

        // Обновление статуса
        queue.update_status(id, MessageStatus::Sent, None).unwrap();

        // Проверка статистики
        let stats = queue.get_queue_stats().unwrap();
        assert_eq!(stats.pending, 0);
        assert_eq!(stats.sent, 1);
    }
}
