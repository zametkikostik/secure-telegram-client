//! Local Storage Module
//!
//! SQLite-based local storage for:
//! - **Chat History**: Encrypted messages, chat metadata
//! - **Contacts**: User contacts with public keys
//! - **Settings**: App settings, user preferences
//! - **IPFS**: Distributed file storage via Pinata/IPFS
//!
//! SECURITY: все чувствительные данные шифруются перед сохранением
//! TODO: encryption at rest для SQLite файла

pub mod ipfs;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, info};

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),
}

// ============================================================================
// Contact
// ============================================================================

/// Контакт пользователя
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: String,
    pub user_id: String,
    pub encrypted_display_name: Vec<u8>,
    pub encrypted_avatar: Option<String>,
    pub public_key_x25519: Vec<u8>,
    pub public_key_kyber: Vec<u8>,
    pub public_key_ed25519: Vec<u8>,
    pub encrypted_notes: Option<Vec<u8>>,
    pub added_at: DateTime<Utc>,
    pub last_contacted: Option<DateTime<Utc>>,
    pub is_blocked: bool,
    pub is_favorite: bool,
}

// ============================================================================
// Settings
// ============================================================================

/// Настройки приложения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub user_id: String,
    pub display_name: String,
    pub language: String,
    pub theme: String,
    pub notifications_enabled: bool,
    pub sound_enabled: bool,
    pub image_preview_enabled: bool,
    pub auto_download_files: bool,
    pub max_file_size: u64,
    pub message_ttl_hours: u64,
    pub steganography_enabled: bool,
    pub log_level: String,
    pub db_path: String,
    pub cloudflare_worker_url: String,
    pub p2p_enabled: bool,
    pub p2p_listen_port: u16,
    pub auto_backup_enabled: bool,
    pub backup_interval_hours: u64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            user_id: String::new(),
            display_name: "User".to_string(),
            language: "ru".to_string(),
            theme: "dark".to_string(),
            notifications_enabled: true,
            sound_enabled: true,
            image_preview_enabled: true,
            auto_download_files: false,
            max_file_size: 100 * 1024 * 1024,
            message_ttl_hours: 0,
            steganography_enabled: false,
            log_level: "info".to_string(),
            db_path: "secure-messenger.db".to_string(),
            cloudflare_worker_url: "https://secure-messenger-push.kostik.workers.dev".to_string(),
            p2p_enabled: true,
            p2p_listen_port: 0,
            auto_backup_enabled: false,
            backup_interval_hours: 24,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingEntry {
    pub key: String,
    pub value: String,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Helper Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatHistoryEntry {
    pub id: String,
    pub chat_id: String,
    pub chat_type: String,
    pub peer_id: Option<String>,
    pub encrypted_name: Option<Vec<u8>>,
    pub last_message_id: Option<String>,
    pub last_message_at: Option<DateTime<Utc>>,
    pub unread_count: u64,
    pub is_archived: bool,
    pub is_muted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedMessage {
    pub id: String,
    pub chat_id: String,
    pub sender_id: String,
    pub encrypted_content: Vec<u8>,
    pub signature: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub msg_type: String,
    pub delivery_status: String,
    pub reply_to: Option<String>,
    pub attachments: Vec<String>,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub contacts: usize,
    pub settings: usize,
    pub chats: usize,
    pub messages: usize,
    pub db_size_bytes: u64,
}

// ============================================================================
// Local Storage
// ============================================================================

pub struct LocalStorage {
    db_pool: Pool<Sqlite>,
}

impl LocalStorage {
    pub async fn new(db_path: &str) -> Result<Self, StorageError> {
        let db_pool = SqlitePoolOptions::new()
            .max_connections(5)
            .acquire_timeout(Duration::from_secs(5))
            .connect(db_path)
            .await?;

        let storage = Self { db_pool };
        storage.run_migrations().await?;

        info!("Local storage initialized: {}", db_path);
        Ok(storage)
    }

    pub async fn in_memory() -> Result<Self, StorageError> {
        Self::new("sqlite::memory:").await
    }

    async fn run_migrations(&self) -> Result<(), StorageError> {
        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS contacts (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL UNIQUE,
                encrypted_display_name BLOB NOT NULL,
                encrypted_avatar TEXT,
                public_key_x25519 BLOB NOT NULL,
                public_key_kyber BLOB NOT NULL,
                public_key_ed25519 BLOB NOT NULL,
                encrypted_notes BLOB,
                added_at INTEGER NOT NULL,
                last_contacted INTEGER,
                is_blocked INTEGER NOT NULL DEFAULT 0,
                is_favorite INTEGER NOT NULL DEFAULT 0
            )"#,
        )
        .execute(&self.db_pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_contacts_user_id ON contacts(user_id)")
            .execute(&self.db_pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_contacts_favorite ON contacts(is_favorite)")
            .execute(&self.db_pool)
            .await?;

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            )"#,
        )
        .execute(&self.db_pool)
        .await?;

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS chat_history (
                id TEXT PRIMARY KEY,
                chat_id TEXT NOT NULL,
                chat_type TEXT NOT NULL,
                peer_id TEXT,
                encrypted_name BLOB,
                last_message_id TEXT,
                last_message_at INTEGER,
                unread_count INTEGER NOT NULL DEFAULT 0,
                is_archived INTEGER NOT NULL DEFAULT 0,
                is_muted INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )"#,
        )
        .execute(&self.db_pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_chat_history_updated ON chat_history(updated_at DESC)",
        )
        .execute(&self.db_pool)
        .await?;

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS messages_cache (
                id TEXT PRIMARY KEY,
                chat_id TEXT NOT NULL,
                sender_id TEXT NOT NULL,
                encrypted_content BLOB NOT NULL,
                signature BLOB NOT NULL,
                created_at INTEGER NOT NULL,
                msg_type TEXT NOT NULL,
                delivery_status TEXT NOT NULL,
                reply_to TEXT,
                attachments TEXT,
                is_deleted INTEGER NOT NULL DEFAULT 0
            )"#,
        )
        .execute(&self.db_pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_messages_cache_chat ON messages_cache(chat_id, created_at DESC)")
            .execute(&self.db_pool)
            .await?;

        debug!("Migrations applied");
        Ok(())
    }

    // ========================================================================
    // Contacts Operations
    // ========================================================================

    pub async fn add_contact(&self, contact: &Contact) -> Result<(), StorageError> {
        let added_at = contact.added_at.timestamp();
        let last_contacted = contact.last_contacted.map(|t| t.timestamp());

        sqlx::query(
            r#"INSERT INTO contacts
                (id, user_id, encrypted_display_name, encrypted_avatar,
                 public_key_x25519, public_key_kyber, public_key_ed25519,
                 encrypted_notes, added_at, last_contacted, is_blocked, is_favorite)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(&contact.id)
        .bind(&contact.user_id)
        .bind(&contact.encrypted_display_name)
        .bind(&contact.encrypted_avatar)
        .bind(&contact.public_key_x25519)
        .bind(&contact.public_key_kyber)
        .bind(&contact.public_key_ed25519)
        .bind(&contact.encrypted_notes)
        .bind(added_at)
        .bind(last_contacted)
        .bind(contact.is_blocked as i64)
        .bind(contact.is_favorite as i64)
        .execute(&self.db_pool)
        .await?;

        info!("Added contact: {} ({})", contact.user_id, contact.id);
        Ok(())
    }

    pub async fn get_contact(&self, contact_id: &str) -> Result<Contact, StorageError> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                Vec<u8>,
                Option<String>,
                Vec<u8>,
                Vec<u8>,
                Vec<u8>,
                Option<Vec<u8>>,
                i64,
                Option<i64>,
                i64,
                i64,
            ),
        >(
            "SELECT id, user_id, encrypted_display_name, encrypted_avatar,
             public_key_x25519, public_key_kyber, public_key_ed25519,
             encrypted_notes, added_at, last_contacted, is_blocked, is_favorite
             FROM contacts WHERE id = ?",
        )
        .bind(contact_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => StorageError::NotFound(format!("Contact: {}", contact_id)),
            _ => StorageError::Database(e),
        })?;

        Ok(Contact {
            id: row.0,
            user_id: row.1,
            encrypted_display_name: row.2,
            encrypted_avatar: row.3,
            public_key_x25519: row.4,
            public_key_kyber: row.5,
            public_key_ed25519: row.6,
            encrypted_notes: row.7,
            added_at: DateTime::from_timestamp(row.8, 0).unwrap_or_default(),
            last_contacted: row.9.and_then(|t| DateTime::from_timestamp(t, 0)),
            is_blocked: row.10 != 0,
            is_favorite: row.11 != 0,
        })
    }

    pub async fn get_contact_by_user_id(&self, user_id: &str) -> Result<Contact, StorageError> {
        let row = sqlx::query_as::<
            _,
            (
                String,
                String,
                Vec<u8>,
                Option<String>,
                Vec<u8>,
                Vec<u8>,
                Vec<u8>,
                Option<Vec<u8>>,
                i64,
                Option<i64>,
                i64,
                i64,
            ),
        >(
            "SELECT id, user_id, encrypted_display_name, encrypted_avatar,
             public_key_x25519, public_key_kyber, public_key_ed25519,
             encrypted_notes, added_at, last_contacted, is_blocked, is_favorite
             FROM contacts WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                StorageError::NotFound(format!("Contact by user_id: {}", user_id))
            }
            _ => StorageError::Database(e),
        })?;

        Ok(Contact {
            id: row.0,
            user_id: row.1,
            encrypted_display_name: row.2,
            encrypted_avatar: row.3,
            public_key_x25519: row.4,
            public_key_kyber: row.5,
            public_key_ed25519: row.6,
            encrypted_notes: row.7,
            added_at: DateTime::from_timestamp(row.8, 0).unwrap_or_default(),
            last_contacted: row.9.and_then(|t| DateTime::from_timestamp(t, 0)),
            is_blocked: row.10 != 0,
            is_favorite: row.11 != 0,
        })
    }

    pub async fn get_all_contacts(&self) -> Result<Vec<Contact>, StorageError> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                Vec<u8>,
                Option<String>,
                Vec<u8>,
                Vec<u8>,
                Vec<u8>,
                Option<Vec<u8>>,
                i64,
                Option<i64>,
                i64,
                i64,
            ),
        >(
            "SELECT id, user_id, encrypted_display_name, encrypted_avatar,
             public_key_x25519, public_key_kyber, public_key_ed25519,
             encrypted_notes, added_at, last_contacted, is_blocked, is_favorite
             FROM contacts ORDER BY is_favorite DESC, last_contacted DESC, added_at DESC",
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| Contact {
                id: row.0,
                user_id: row.1,
                encrypted_display_name: row.2,
                encrypted_avatar: row.3,
                public_key_x25519: row.4,
                public_key_kyber: row.5,
                public_key_ed25519: row.6,
                encrypted_notes: row.7,
                added_at: DateTime::from_timestamp(row.8, 0).unwrap_or_default(),
                last_contacted: row.9.and_then(|t| DateTime::from_timestamp(t, 0)),
                is_blocked: row.10 != 0,
                is_favorite: row.11 != 0,
            })
            .collect())
    }

    pub async fn get_favorite_contacts(&self) -> Result<Vec<Contact>, StorageError> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                Vec<u8>,
                Option<String>,
                Vec<u8>,
                Vec<u8>,
                Vec<u8>,
                Option<Vec<u8>>,
                i64,
                Option<i64>,
                i64,
                i64,
            ),
        >(
            "SELECT id, user_id, encrypted_display_name, encrypted_avatar,
             public_key_x25519, public_key_kyber, public_key_ed25519,
             encrypted_notes, added_at, last_contacted, is_blocked, is_favorite
             FROM contacts WHERE is_favorite = 1 ORDER BY last_contacted DESC",
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| Contact {
                id: row.0,
                user_id: row.1,
                encrypted_display_name: row.2,
                encrypted_avatar: row.3,
                public_key_x25519: row.4,
                public_key_kyber: row.5,
                public_key_ed25519: row.6,
                encrypted_notes: row.7,
                added_at: DateTime::from_timestamp(row.8, 0).unwrap_or_default(),
                last_contacted: row.9.and_then(|t| DateTime::from_timestamp(t, 0)),
                is_blocked: row.10 != 0,
                is_favorite: row.11 != 0,
            })
            .collect())
    }

    pub async fn update_contact(&self, contact: &Contact) -> Result<(), StorageError> {
        let last_contacted = contact.last_contacted.map(|t| t.timestamp());

        sqlx::query(
            r#"UPDATE contacts
            SET encrypted_display_name = ?, encrypted_avatar = ?,
                public_key_x25519 = ?, public_key_kyber = ?, public_key_ed25519 = ?,
                encrypted_notes = ?, last_contacted = ?,
                is_blocked = ?, is_favorite = ?
            WHERE id = ?"#,
        )
        .bind(&contact.encrypted_display_name)
        .bind(&contact.encrypted_avatar)
        .bind(&contact.public_key_x25519)
        .bind(&contact.public_key_kyber)
        .bind(&contact.public_key_ed25519)
        .bind(&contact.encrypted_notes)
        .bind(last_contacted)
        .bind(contact.is_blocked as i64)
        .bind(contact.is_favorite as i64)
        .bind(&contact.id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn delete_contact(&self, contact_id: &str) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM contacts WHERE id = ?")
            .bind(contact_id)
            .execute(&self.db_pool)
            .await?;

        info!("Deleted contact: {}", contact_id);
        Ok(())
    }

    pub async fn set_contact_blocked(
        &self,
        contact_id: &str,
        blocked: bool,
    ) -> Result<(), StorageError> {
        sqlx::query("UPDATE contacts SET is_blocked = ? WHERE id = ?")
            .bind(blocked as i64)
            .bind(contact_id)
            .execute(&self.db_pool)
            .await?;
        Ok(())
    }

    pub async fn set_contact_favorite(
        &self,
        contact_id: &str,
        favorite: bool,
    ) -> Result<(), StorageError> {
        sqlx::query("UPDATE contacts SET is_favorite = ? WHERE id = ?")
            .bind(favorite as i64)
            .bind(contact_id)
            .execute(&self.db_pool)
            .await?;
        Ok(())
    }

    // ========================================================================
    // Settings Operations
    // ========================================================================

    pub async fn save_setting(&self, key: &str, value: &str) -> Result<(), StorageError> {
        let now = Utc::now().timestamp();

        sqlx::query(
            r#"INSERT INTO settings (key, value, updated_at)
            VALUES (?, ?, ?)
            ON CONFLICT(key) DO UPDATE SET value = ?, updated_at = ?"#,
        )
        .bind(key)
        .bind(value)
        .bind(now)
        .bind(value)
        .bind(now)
        .execute(&self.db_pool)
        .await?;

        debug!("Saved setting: {} = {}", key, value);
        Ok(())
    }

    pub async fn get_setting(&self, key: &str) -> Result<Option<String>, StorageError> {
        let row = sqlx::query_as::<_, (String,)>("SELECT value FROM settings WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.db_pool)
            .await?;
        Ok(row.map(|r| r.0))
    }

    pub async fn save_settings(&self, settings: &AppSettings) -> Result<(), StorageError> {
        let json = serde_json::to_string(settings)?;
        self.save_setting("app_settings", &json).await
    }

    pub async fn load_settings(&self) -> Result<AppSettings, StorageError> {
        match self.get_setting("app_settings").await? {
            Some(json) => Ok(serde_json::from_str(&json)?),
            None => {
                let settings = AppSettings::default();
                self.save_settings(&settings).await?;
                Ok(settings)
            }
        }
    }

    pub async fn delete_setting(&self, key: &str) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM settings WHERE key = ?")
            .bind(key)
            .execute(&self.db_pool)
            .await?;
        Ok(())
    }

    pub async fn get_all_settings(&self) -> Result<Vec<SettingEntry>, StorageError> {
        let rows = sqlx::query_as::<_, (String, String, i64)>(
            "SELECT key, value, updated_at FROM settings ORDER BY key",
        )
        .fetch_all(&self.db_pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| SettingEntry {
                key: row.0,
                value: row.1,
                updated_at: DateTime::from_timestamp(row.2, 0).unwrap_or_default(),
            })
            .collect())
    }

    // ========================================================================
    // Chat History Operations
    // ========================================================================

    pub async fn save_chat_history_entry(
        &self,
        chat_id: &str,
        chat_type: &str,
        peer_id: Option<&str>,
        unread_count: u64,
    ) -> Result<(), StorageError> {
        let now = Utc::now().timestamp();

        sqlx::query(
            r#"INSERT INTO chat_history
                (id, chat_id, chat_type, peer_id, unread_count, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET unread_count = ?, updated_at = ?"#,
        )
        .bind(chat_id)
        .bind(chat_id)
        .bind(chat_type)
        .bind(peer_id)
        .bind(unread_count as i64)
        .bind(now)
        .bind(now)
        .bind(unread_count as i64)
        .bind(now)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn update_unread_count(
        &self,
        chat_id: &str,
        unread_count: u64,
    ) -> Result<(), StorageError> {
        sqlx::query("UPDATE chat_history SET unread_count = ?, updated_at = ? WHERE chat_id = ?")
            .bind(unread_count as i64)
            .bind(Utc::now().timestamp())
            .bind(chat_id)
            .execute(&self.db_pool)
            .await?;
        Ok(())
    }

    pub async fn set_chat_archived(
        &self,
        chat_id: &str,
        archived: bool,
    ) -> Result<(), StorageError> {
        sqlx::query("UPDATE chat_history SET is_archived = ?, updated_at = ? WHERE chat_id = ?")
            .bind(archived as i64)
            .bind(Utc::now().timestamp())
            .bind(chat_id)
            .execute(&self.db_pool)
            .await?;
        Ok(())
    }

    // ========================================================================
    // Message Cache Operations
    // ========================================================================

    pub async fn cache_message(&self, message: &CachedMessage) -> Result<(), StorageError> {
        let created_at = message.created_at.timestamp();
        let attachments_json = serde_json::to_string(&message.attachments)?;

        sqlx::query(
            r#"INSERT INTO messages_cache
                (id, chat_id, sender_id, encrypted_content, signature,
                 created_at, msg_type, delivery_status, reply_to, attachments)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(&message.id)
        .bind(&message.chat_id)
        .bind(&message.sender_id)
        .bind(&message.encrypted_content)
        .bind(&message.signature)
        .bind(created_at)
        .bind(&message.msg_type)
        .bind(&message.delivery_status)
        .bind(&message.reply_to)
        .bind(attachments_json)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn get_chat_messages(
        &self,
        chat_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<CachedMessage>, StorageError> {
        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                String,
                Vec<u8>,
                Vec<u8>,
                i64,
                String,
                String,
                Option<String>,
                String,
                i64,
            ),
        >(
            r#"SELECT id, chat_id, sender_id, encrypted_content, signature,
                   created_at, msg_type, delivery_status, reply_to, attachments,
                   is_deleted
            FROM messages_cache
            WHERE chat_id = ? AND is_deleted = 0
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?"#,
        )
        .bind(chat_id)
        .bind(limit as i64)
        .bind(offset as i64)
        .fetch_all(&self.db_pool)
        .await?;

        let mut messages = Vec::new();
        for row in rows {
            let attachments: Vec<String> = serde_json::from_str(&row.9).unwrap_or_default();
            messages.push(CachedMessage {
                id: row.0,
                chat_id: row.1,
                sender_id: row.2,
                encrypted_content: row.3,
                signature: row.4,
                created_at: DateTime::from_timestamp(row.5, 0).unwrap_or_default(),
                msg_type: row.6,
                delivery_status: row.7,
                reply_to: row.8,
                attachments,
                is_deleted: row.10 != 0,
            });
        }

        messages.reverse();
        Ok(messages)
    }

    pub async fn delete_message(&self, message_id: &str) -> Result<(), StorageError> {
        sqlx::query("UPDATE messages_cache SET is_deleted = 1 WHERE id = ?")
            .bind(message_id)
            .execute(&self.db_pool)
            .await?;
        Ok(())
    }

    pub async fn clear_chat_cache(&self, chat_id: &str) -> Result<usize, StorageError> {
        let result = sqlx::query("DELETE FROM messages_cache WHERE chat_id = ?")
            .bind(chat_id)
            .execute(&self.db_pool)
            .await?;
        Ok(result.rows_affected() as usize)
    }

    // ========================================================================
    // Maintenance
    // ========================================================================

    pub async fn get_db_size(&self) -> Result<u64, StorageError> {
        let row = sqlx::query_as::<_, (i64,)>(
            "SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()",
        )
        .fetch_one(&self.db_pool)
        .await?;
        Ok(row.0 as u64)
    }

    pub async fn vacuum(&self) -> Result<(), StorageError> {
        sqlx::query("VACUUM").execute(&self.db_pool).await?;
        info!("Database vacuumed");
        Ok(())
    }

    pub async fn get_stats(&self) -> Result<StorageStats, StorageError> {
        let contacts = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM contacts")
            .fetch_one(&self.db_pool)
            .await?;
        let settings = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM settings")
            .fetch_one(&self.db_pool)
            .await?;
        let chats = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM chat_history")
            .fetch_one(&self.db_pool)
            .await?;
        let messages =
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM messages_cache WHERE is_deleted = 0")
                .fetch_one(&self.db_pool)
                .await?;
        let db_size = self.get_db_size().await?;

        Ok(StorageStats {
            contacts: contacts.0 as usize,
            settings: settings.0 as usize,
            chats: chats.0 as usize,
            messages: messages.0 as usize,
            db_size_bytes: db_size,
        })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_creation() {
        let storage = LocalStorage::in_memory().await.unwrap();
        let stats = storage.get_stats().await.unwrap();
        assert_eq!(stats.contacts, 0);
        assert_eq!(stats.settings, 0);
    }

    #[tokio::test]
    async fn test_settings_roundtrip() {
        let storage = LocalStorage::in_memory().await.unwrap();

        let settings = AppSettings {
            display_name: "Test User".to_string(),
            language: "en".to_string(),
            theme: "light".to_string(),
            ..Default::default()
        };
        storage.save_settings(&settings).await.unwrap();

        let loaded = storage.load_settings().await.unwrap();
        assert_eq!(loaded.display_name, "Test User");
        assert_eq!(loaded.language, "en");
        assert_eq!(loaded.theme, "light");
    }

    #[tokio::test]
    async fn test_setting_individual() {
        let storage = LocalStorage::in_memory().await.unwrap();

        storage
            .save_setting("test_key", "test_value")
            .await
            .unwrap();
        let value = storage.get_setting("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        storage.delete_setting("test_key").await.unwrap();
        let value = storage.get_setting("test_key").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_contact_add_get() {
        let storage = LocalStorage::in_memory().await.unwrap();

        let contact = Contact {
            id: "contact-1".to_string(),
            user_id: "user-123".to_string(),
            encrypted_display_name: vec![1, 2, 3],
            encrypted_avatar: None,
            public_key_x25519: vec![4, 5, 6],
            public_key_kyber: vec![7, 8, 9],
            public_key_ed25519: vec![10, 11, 12],
            encrypted_notes: None,
            added_at: Utc::now(),
            last_contacted: None,
            is_blocked: false,
            is_favorite: false,
        };

        storage.add_contact(&contact).await.unwrap();
        let loaded = storage.get_contact("contact-1").await.unwrap();
        assert_eq!(loaded.user_id, "user-123");
        assert_eq!(loaded.public_key_x25519, vec![4, 5, 6]);
    }

    #[tokio::test]
    async fn test_contact_by_user_id() {
        let storage = LocalStorage::in_memory().await.unwrap();

        let contact = Contact {
            id: "contact-1".to_string(),
            user_id: "user-456".to_string(),
            encrypted_display_name: vec![1, 2, 3],
            encrypted_avatar: None,
            public_key_x25519: vec![4, 5, 6],
            public_key_kyber: vec![7, 8, 9],
            public_key_ed25519: vec![10, 11, 12],
            encrypted_notes: None,
            added_at: Utc::now(),
            last_contacted: None,
            is_blocked: false,
            is_favorite: false,
        };

        storage.add_contact(&contact).await.unwrap();
        let loaded = storage.get_contact_by_user_id("user-456").await.unwrap();
        assert_eq!(loaded.id, "contact-1");
    }

    #[tokio::test]
    async fn test_contact_not_found() {
        let storage = LocalStorage::in_memory().await.unwrap();
        let result = storage.get_contact("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_contact_delete() {
        let storage = LocalStorage::in_memory().await.unwrap();

        let contact = Contact {
            id: "contact-del".to_string(),
            user_id: "user-del".to_string(),
            encrypted_display_name: vec![1, 2, 3],
            encrypted_avatar: None,
            public_key_x25519: vec![4, 5, 6],
            public_key_kyber: vec![7, 8, 9],
            public_key_ed25519: vec![10, 11, 12],
            encrypted_notes: None,
            added_at: Utc::now(),
            last_contacted: None,
            is_blocked: false,
            is_favorite: false,
        };

        storage.add_contact(&contact).await.unwrap();
        storage.delete_contact("contact-del").await.unwrap();

        let result = storage.get_contact("contact-del").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_contact_favorite() {
        let storage = LocalStorage::in_memory().await.unwrap();

        let contact = Contact {
            id: "contact-fav".to_string(),
            user_id: "user-fav".to_string(),
            encrypted_display_name: vec![1, 2, 3],
            encrypted_avatar: None,
            public_key_x25519: vec![4, 5, 6],
            public_key_kyber: vec![7, 8, 9],
            public_key_ed25519: vec![10, 11, 12],
            encrypted_notes: None,
            added_at: Utc::now(),
            last_contacted: None,
            is_blocked: false,
            is_favorite: false,
        };

        storage.add_contact(&contact).await.unwrap();
        storage
            .set_contact_favorite("contact-fav", true)
            .await
            .unwrap();

        let favorites = storage.get_favorite_contacts().await.unwrap();
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].id, "contact-fav");
    }

    #[tokio::test]
    async fn test_contact_blocked() {
        let storage = LocalStorage::in_memory().await.unwrap();

        let contact = Contact {
            id: "contact-block".to_string(),
            user_id: "user-block".to_string(),
            encrypted_display_name: vec![1, 2, 3],
            encrypted_avatar: None,
            public_key_x25519: vec![4, 5, 6],
            public_key_kyber: vec![7, 8, 9],
            public_key_ed25519: vec![10, 11, 12],
            encrypted_notes: None,
            added_at: Utc::now(),
            last_contacted: None,
            is_blocked: false,
            is_favorite: false,
        };

        storage.add_contact(&contact).await.unwrap();
        storage
            .set_contact_blocked("contact-block", true)
            .await
            .unwrap();

        let loaded = storage.get_contact("contact-block").await.unwrap();
        assert!(loaded.is_blocked);
    }

    #[tokio::test]
    async fn test_message_cache() {
        let storage = LocalStorage::in_memory().await.unwrap();

        let msg = CachedMessage {
            id: "msg-1".to_string(),
            chat_id: "chat-1".to_string(),
            sender_id: "user-1".to_string(),
            encrypted_content: vec![1, 2, 3],
            signature: vec![4, 5, 6],
            created_at: Utc::now(),
            msg_type: "Text".to_string(),
            delivery_status: "Sent".to_string(),
            reply_to: None,
            attachments: Vec::new(),
            is_deleted: false,
        };

        storage.cache_message(&msg).await.unwrap();

        let messages = storage.get_chat_messages("chat-1", 10, 0).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, "msg-1");
    }

    #[tokio::test]
    async fn test_message_delete() {
        let storage = LocalStorage::in_memory().await.unwrap();

        let msg = CachedMessage {
            id: "msg-del".to_string(),
            chat_id: "chat-1".to_string(),
            sender_id: "user-1".to_string(),
            encrypted_content: vec![1, 2, 3],
            signature: vec![4, 5, 6],
            created_at: Utc::now(),
            msg_type: "Text".to_string(),
            delivery_status: "Sent".to_string(),
            reply_to: None,
            attachments: Vec::new(),
            is_deleted: false,
        };

        storage.cache_message(&msg).await.unwrap();
        storage.delete_message("msg-del").await.unwrap();

        let messages = storage.get_chat_messages("chat-1", 10, 0).await.unwrap();
        assert_eq!(messages.len(), 0);
    }

    #[tokio::test]
    async fn test_storage_stats() {
        let storage = LocalStorage::in_memory().await.unwrap();

        // Добавить контакт
        let contact = Contact {
            id: "c1".to_string(),
            user_id: "u1".to_string(),
            encrypted_display_name: vec![1],
            encrypted_avatar: None,
            public_key_x25519: vec![2],
            public_key_kyber: vec![3],
            public_key_ed25519: vec![4],
            encrypted_notes: None,
            added_at: Utc::now(),
            last_contacted: None,
            is_blocked: false,
            is_favorite: false,
        };
        storage.add_contact(&contact).await.unwrap();

        // Сохранить настройку
        storage.save_setting("test", "value").await.unwrap();

        let stats = storage.get_stats().await.unwrap();
        assert_eq!(stats.contacts, 1);
        assert_eq!(stats.settings, 1);
        assert!(stats.db_size_bytes > 0);
    }
}
