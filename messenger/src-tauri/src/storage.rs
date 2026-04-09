// Secure local storage
// SECURITY: все данные шифруются перед записью на диск
// TODO: реализовать encrypted SQLite хранение

use sqlx::SqlitePool;

/// Зашифрованное локальное хранилище
pub struct SecureStorage {
    #[allow(dead_code)]
    pool: SqlitePool,
}

impl SecureStorage {
    pub async fn new(db_path: &str) -> Result<Self, StorageError> {
        let pool = SqlitePool::connect(db_path).await
            .map_err(|e| StorageError::ConnectionFailed(e.to_string()))?;

        // Создаём таблицы если не существуют
        sqlx::query::<sqlx::Sqlite>(
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                encrypted_content BLOB NOT NULL,
                timestamp INTEGER NOT NULL,
                sender_id TEXT NOT NULL,
                chat_id TEXT NOT NULL
            )"
        )
        .execute(&pool)
        .await
        .map_err(|e| StorageError::QueryFailed(e.to_string()))?;

        sqlx::query::<sqlx::Sqlite>(
            "CREATE TABLE IF NOT EXISTS contacts (
                id TEXT PRIMARY KEY,
                public_key BLOB NOT NULL,
                display_name TEXT,
                added_at INTEGER NOT NULL
            )"
        )
        .execute(&pool)
        .await
        .map_err(|e| StorageError::QueryFailed(e.to_string()))?;

        tracing::info!("SecureStorage initialized at {}", db_path);

        Ok(Self { pool })
    }

    /// Сохранение зашифрованного сообщения
    // SECURITY: message уже должен быть зашифрован перед вызовом
    pub async fn save_message(&self, _id: &str, _encrypted: &[u8], _chat_id: &str) -> Result<(), StorageError> {
        // TODO: реализовать сохранение
        Ok(())
    }

    /// Получение зашифрованных сообщений
    pub async fn get_messages(&self, _chat_id: &str, _limit: usize) -> Result<Vec<Vec<u8>>, StorageError> {
        // TODO: реализовать получение
        Ok(vec![])
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Query failed: {0}")]
    QueryFailed(String),
}
