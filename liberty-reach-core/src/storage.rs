//! Encrypted Storage Module
//! 
//! DatabaseManager с полным шифрованием через SQLCipher.
//! Ключ AES-256 передается один раз при инициализации и хранится только в памяти.

use anyhow::{Context, Result, anyhow, bail};
use rusqlite::{Connection, OpenFlags, params};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn, error};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rand::{RngCore, thread_rng};

use crate::crypto::HashFunctions;

// ============================================================================
// КОНФИГУРАЦИЯ БАЗЫ ДАННЫХ
// ============================================================================

/// Конфигурация зашифрованной базы данных
pub struct DatabaseConfig {
    /// Путь к файлу базы данных
    pub path: PathBuf,
    /// Ключ шифрования (AES-256)
    pub encryption_key: Vec<u8>,
    /// Флаг создания новой базы
    pub create_if_missing: bool,
}

// ============================================================================
// DATABASE MANAGER
// ============================================================================

/// Менеджер зашифрованной базы данных
pub struct DatabaseManager {
    conn: Arc<Mutex<Connection>>,
    config: DatabaseConfig,
    is_initialized: bool,
}

impl DatabaseManager {
    /// Инициализировать базу данных
    pub async fn new(config: DatabaseConfig) -> Result<Self> {
        // Проверяем размер ключа (должен быть 32 байта для AES-256)
        if config.encryption_key.len() != 32 {
            bail!("Encryption key must be exactly 32 bytes (AES-256)");
        }
        
        // Создаем директорию если нужно
        if let Some(parent) = config.path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .context("Failed to create database directory")?;
            }
        }
        
        let create_if_missing = config.create_if_missing;
        let path = config.path.clone();
        let key = config.encryption_key.clone();
        
        // Открываем соединение в blocking режиме
        let conn = tokio::task::spawn_blocking(move || {
            Self::open_connection(&path, &key, create_if_missing)
        })
        .await
        .context("Failed to spawn blocking task")??;
        
        let mut manager = Self {
            conn: Arc::new(Mutex::new(conn)),
            config,
            is_initialized: false,
        };
        
        // Инициализируем схему
        manager.init_schema().await?;
        manager.is_initialized = true;
        
        info!("Database initialized at {:?}", manager.config.path);
        
        Ok(manager)
    }
    
    /// Открыть соединение с SQLCipher
    fn open_connection(
        path: &Path,
        key: &[u8],
        create_if_missing: bool,
    ) -> Result<Connection> {
        let flags = if create_if_missing {
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
        } else {
            OpenFlags::SQLITE_OPEN_READ_WRITE
        };
        
        let conn = Connection::open_with_flags(path, flags)
            .context("Failed to open database connection")?;
        
        // Устанавливаем ключ шифрования
        conn.execute_batch(&format!(
            "PRAGMA key = \"x'{}'\";",
            hex::encode(key)
        )).context("Failed to set encryption key")?;
        
        // Включаем SQLCipher настройки
        conn.execute_batch(
            "
            PRAGMA cipher_page_size = 4096;
            PRAGMA kdf_iter = 256000;
            PRAGMA cipher_hmac_algorithm = HMAC_SHA512;
            PRAGMA cipher_kdf_algorithm = PBKDF2_HMAC_SHA512;
            "
        ).context("Failed to configure SQLCipher")?;
        
        // Включаем WAL режим для лучшей производительности
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA busy_timeout = 5000;
            PRAGMA foreign_keys = ON;
            "
        ).context("Failed to configure database mode")?;
        
        Ok(conn)
    }
    
    /// Инициализировать схему базы данных
    async fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute_batch(
            "
            -- Таблица пользователей
            CREATE TABLE IF NOT EXISTS users (
                user_id TEXT PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                display_name TEXT,
                public_key BLOB,
                pq_public_key BLOB,
                created_at INTEGER NOT NULL,
                last_seen INTEGER,
                is_contact INTEGER DEFAULT 0,
                nickname TEXT,
                bio TEXT,
                family_status TEXT,
                partner_id TEXT,
                wallpaper TEXT,
                synced_wallpaper_with TEXT,
                theme TEXT,
                is_verified INTEGER DEFAULT 0,
                is_bot INTEGER DEFAULT 0
            );
            
            -- Таблица чатов
            CREATE TABLE IF NOT EXISTS chats (
                chat_id TEXT PRIMARY KEY,
                chat_type TEXT NOT NULL,
                name TEXT,
                created_at INTEGER NOT NULL,
                last_message_at INTEGER,
                unread_count INTEGER DEFAULT 0
            );
            
            -- Таблица участников чата
            CREATE TABLE IF NOT EXISTS chat_members (
                chat_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                role TEXT DEFAULT 'member',
                joined_at INTEGER NOT NULL,
                PRIMARY KEY (chat_id, user_id),
                FOREIGN KEY (chat_id) REFERENCES chats(chat_id) ON DELETE CASCADE,
                FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
            );
            
            -- Таблица сообщений
            CREATE TABLE IF NOT EXISTS messages (
                message_id TEXT PRIMARY KEY,
                chat_id TEXT NOT NULL,
                sender_id TEXT NOT NULL,
                content_type TEXT NOT NULL,
                content BLOB,
                encrypted INTEGER DEFAULT 1,
                signature BLOB,
                nonce BLOB,
                timestamp INTEGER NOT NULL,
                status TEXT DEFAULT 'sent',
                is_read INTEGER DEFAULT 0,
                read_by TEXT,
                reply_to_message_id TEXT,
                FOREIGN KEY (chat_id) REFERENCES chats(chat_id) ON DELETE CASCADE,
                FOREIGN KEY (sender_id) REFERENCES users(user_id) ON DELETE CASCADE
            );
            
            -- Таблица звонков
            CREATE TABLE IF NOT EXISTS calls (
                call_id TEXT PRIMARY KEY,
                caller_id TEXT NOT NULL,
                callee_id TEXT NOT NULL,
                call_type TEXT NOT NULL,
                status TEXT NOT NULL,
                started_at INTEGER,
                ended_at INTEGER,
                duration_secs INTEGER,
                end_reason TEXT,
                FOREIGN KEY (caller_id) REFERENCES users(user_id) ON DELETE CASCADE,
                FOREIGN KEY (callee_id) REFERENCES users(user_id) ON DELETE CASCADE
            );
            
            -- Таблица сессий
            CREATE TABLE IF NOT EXISTS sessions (
                session_id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                device_info TEXT,
                created_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                is_active INTEGER DEFAULT 1,
                FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
            );
            
            -- Таблица настроек
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );
            
            -- Таблица ICE кандидатов для WebRTC
            CREATE TABLE IF NOT EXISTS ice_candidates (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                call_id TEXT NOT NULL,
                candidate TEXT NOT NULL,
                sdp_m_line_index INTEGER NOT NULL,
                sdp_mid TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (call_id) REFERENCES calls(call_id) ON DELETE CASCADE
            );
            
            -- Таблица реакций
            CREATE TABLE IF NOT EXISTS reactions (
                reaction_id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                emoji TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (message_id) REFERENCES messages(message_id) ON DELETE CASCADE
            );
            
            -- Таблица закреплённых сообщений
            CREATE TABLE IF NOT EXISTS pinned_messages (
                pin_id TEXT PRIMARY KEY,
                chat_id TEXT NOT NULL,
                message_id TEXT NOT NULL,
                pinned_by TEXT NOT NULL,
                pinned_at INTEGER NOT NULL,
                FOREIGN KEY (chat_id) REFERENCES chats(chat_id) ON DELETE CASCADE,
                FOREIGN KEY (message_id) REFERENCES messages(message_id) ON DELETE CASCADE
            );
            
            -- Таблица избранных сообщений
            CREATE TABLE IF NOT EXISTS favorite_messages (
                fav_id TEXT PRIMARY KEY,
                message_id TEXT NOT NULL,
                user_id TEXT NOT NULL,
                tags TEXT,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (message_id) REFERENCES messages(message_id) ON DELETE CASCADE
            );
            
            -- Таблица отложенных сообщений
            CREATE TABLE IF NOT EXISTS scheduled_messages (
                schedule_id TEXT PRIMARY KEY,
                chat_id TEXT NOT NULL,
                content BLOB NOT NULL,
                send_at INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                status TEXT DEFAULT 'pending',
                FOREIGN KEY (chat_id) REFERENCES chats(chat_id) ON DELETE CASCADE
            );
            
            -- Таблица пакетов стикеров
            CREATE TABLE IF NOT EXISTS sticker_packs (
                pack_id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                creator_id TEXT NOT NULL,
                stickers_count INTEGER DEFAULT 0,
                created_at INTEGER NOT NULL
            );
            
            -- Таблица стикеров
            CREATE TABLE IF NOT EXISTS stickers (
                sticker_id TEXT PRIMARY KEY,
                pack_id TEXT NOT NULL,
                data BLOB NOT NULL,
                emoji TEXT,
                FOREIGN KEY (pack_id) REFERENCES sticker_packs(pack_id) ON DELETE CASCADE
            );
            
            -- Индексы для производительности
            CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages(chat_id);
            CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages(timestamp);
            CREATE INDEX IF NOT EXISTS idx_messages_sender ON messages(sender_id);
            CREATE INDEX IF NOT EXISTS idx_chat_members_user ON chat_members(user_id);
            CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id);
            CREATE INDEX IF NOT EXISTS idx_calls_caller ON calls(caller_id);
            CREATE INDEX IF NOT EXISTS idx_calls_callee ON calls(callee_id);
            "
        ).context("Failed to create database schema")?;
        
        drop(conn);
        Ok(())
    }
    
    // ========================================================================
    // МЕТОДЫ ДЛЯ ПОЛЬЗОВАТЕЛЕЙ
    // ========================================================================
    
    /// Добавить пользователя
    pub async fn add_user(&self, user: &User) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "INSERT OR REPLACE INTO users (
                user_id, username, display_name, public_key, pq_public_key,
                created_at, last_seen, is_contact, nickname
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                user.user_id,
                user.username,
                user.display_name,
                user.public_key,
                user.pq_public_key,
                user.created_at,
                user.last_seen,
                if user.is_contact { 1 } else { 0 },
                user.nickname,
            ],
        ).context("Failed to insert user")?;
        
        Ok(())
    }
    
    /// Получить пользователя по ID
    pub async fn get_user(&self, user_id: &str) -> Result<Option<User>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT user_id, username, display_name, public_key, pq_public_key,
                    created_at, last_seen, is_contact, nickname
             FROM users WHERE user_id = ?1"
        )?;
        
        let user = stmt.query_row(params![user_id], |row| {
            Ok(User {
                user_id: row.get(0)?,
                username: row.get(1)?,
                display_name: row.get(2)?,
                public_key: row.get(3)?,
                pq_public_key: row.get(4)?,
                created_at: row.get(5)?,
                last_seen: row.get(6)?,
                is_contact: row.get::<_, i32>(7)? == 1,
                nickname: row.get(8)?,
                bio: row.get(9)?,
                family_status: row.get(10)?,
                partner_id: row.get(11)?,
                wallpaper: row.get(12)?,
                synced_wallpaper_with: row.get(13)?,
                theme: row.get(14)?,
                is_verified: row.get::<_, i32>(15)? == 1,
                is_bot: row.get::<_, i32>(16)? == 1,
            })
        }).ok();

        Ok(user)
    }
    
    /// Получить всех контактов
    pub async fn get_contacts(&self) -> Result<Vec<User>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn.prepare(
            "SELECT user_id, username, display_name, public_key, pq_public_key,
                    created_at, last_seen, is_contact, nickname, bio, family_status,
                    partner_id, wallpaper, synced_wallpaper_with, theme, is_verified, is_bot
             FROM users WHERE is_contact = 1 ORDER BY username"
        )?;

        let users = stmt.query_map([], |row| {
            Ok(User {
                user_id: row.get(0)?,
                username: row.get(1)?,
                display_name: row.get(2)?,
                public_key: row.get(3)?,
                pq_public_key: row.get(4)?,
                created_at: row.get(5)?,
                last_seen: row.get(6)?,
                is_contact: row.get::<_, i32>(7)? == 1,
                nickname: row.get(8)?,
                bio: row.get(9)?,
                family_status: row.get(10)?,
                partner_id: row.get(11)?,
                wallpaper: row.get(12)?,
                synced_wallpaper_with: row.get(13)?,
                theme: row.get(14)?,
                is_verified: row.get::<_, i32>(15)? == 1,
                is_bot: row.get::<_, i32>(16)? == 1,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

        Ok(users)
    }
    
    /// Обновить last_seen пользователя
    pub async fn update_user_last_seen(&self, user_id: &str, timestamp: u64) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "UPDATE users SET last_seen = ?1 WHERE user_id = ?2",
            params![timestamp as i64, user_id],
        )?;
        
        Ok(())
    }
    
    // ========================================================================
    // МЕТОДЫ ДЛЯ ЧАТОВ
    // ========================================================================
    
    /// Создать чат
    pub async fn create_chat(&self, chat: &Chat) -> Result<()> {
        let mut conn = self.conn.lock().await;

        let tx = conn.transaction()?;

        tx.execute(
            "INSERT INTO chats (chat_id, chat_type, name, created_at, last_message_at, unread_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                chat.chat_id,
                chat.chat_type,
                chat.name,
                chat.created_at,
                chat.last_message_at,
                chat.unread_count,
            ],
        )?;

        for member in &chat.members {
            tx.execute(
                "INSERT INTO chat_members (chat_id, user_id, role, joined_at)
                 VALUES (?1, ?2, ?3, ?4)",
                params![chat.chat_id, member.user_id, member.role, member.joined_at],
            )?;
        }

        tx.commit()?;
        Ok(())
    }
    
    /// Получить чат по ID
    pub async fn get_chat(&self, chat_id: &str) -> Result<Option<Chat>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT chat_id, chat_type, name, created_at, last_message_at, unread_count
             FROM chats WHERE chat_id = ?1"
        )?;
        
        let chat = stmt.query_row(params![chat_id], |row| {
            Ok(Chat {
                chat_id: row.get(0)?,
                chat_type: row.get(1)?,
                name: row.get(2)?,
                created_at: row.get(3)?,
                last_message_at: row.get(4)?,
                unread_count: row.get(5)?,
                members: vec![],
            })
        }).ok();

        if let Some(mut chat) = chat {
            // Загружаем участников
            chat.members = self.get_chat_members(chat_id).await?;
            return Ok(Some(chat));
        }
        
        Ok(None)
    }
    
    /// Получить всех участников чата
    pub async fn get_chat_members(&self, chat_id: &str) -> Result<Vec<ChatMember>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT user_id, role, joined_at FROM chat_members WHERE chat_id = ?1"
        )?;
        
        let members = stmt.query_map(params![chat_id], |row| {
            Ok(ChatMember {
                user_id: row.get(0)?,
                role: row.get(1)?,
                joined_at: row.get(2)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
        
        Ok(members)
    }
    
    /// Получить все чаты
    pub async fn get_all_chats(&self) -> Result<Vec<Chat>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT chat_id, chat_type, name, created_at, last_message_at, unread_count
             FROM chats ORDER BY last_message_at DESC"
        )?;
        
        let mut chats = Vec::new();
        let rows = stmt.query_map([], |row| {
            Ok(Chat {
                chat_id: row.get(0)?,
                chat_type: row.get(1)?,
                name: row.get(2)?,
                created_at: row.get(3)?,
                last_message_at: row.get(4)?,
                unread_count: row.get(5)?,
                members: vec![],
            })
        })?;
        
        for row in rows {
            if let Ok(mut chat) = row {
                chat.members = self.get_chat_members(&chat.chat_id).await?;
                chats.push(chat);
            }
        }
        
        Ok(chats)
    }
    
    // ========================================================================
    // МЕТОДЫ ДЛЯ СООБЩЕНИЙ
    // ========================================================================
    
    /// Сохранить сообщение
    pub async fn save_message(&self, message: &Message) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "INSERT INTO messages (
                message_id, chat_id, sender_id, content_type, content,
                encrypted, signature, nonce, timestamp, status, is_read,
                read_by, reply_to_message_id
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                message.message_id,
                message.chat_id,
                message.sender_id,
                message.content_type,
                message.content,
                if message.encrypted { 1 } else { 0 },
                message.signature,
                message.nonce,
                message.timestamp as i64,
                message.status,
                if message.is_read { 1 } else { 0 },
                message.read_by,
                message.reply_to_message_id,
            ],
        )?;
        
        // Обновляем last_message_at чата
        conn.execute(
            "UPDATE chats SET last_message_at = ?1 WHERE chat_id = ?2",
            params![message.timestamp as i64, message.chat_id],
        )?;
        
        Ok(())
    }
    
    /// Получить историю сообщений чата
    pub async fn get_message_history(
        &self,
        chat_id: &str,
        limit: u32,
        before_message_id: Option<&str>,
    ) -> Result<Vec<Message>> {
        let conn = self.conn.lock().await;

        let query = if before_message_id.is_some() {
            "SELECT message_id, chat_id, sender_id, content_type, content,
                    encrypted, signature, nonce, timestamp, status, is_read,
                    read_by, reply_to_message_id
             FROM messages
             WHERE chat_id = ?1 AND message_id < ?2
             ORDER BY timestamp DESC
             LIMIT ?3"
        } else {
            "SELECT message_id, chat_id, sender_id, content_type, content,
                    encrypted, signature, nonce, timestamp, status, is_read,
                    read_by, reply_to_message_id
             FROM messages
             WHERE chat_id = ?1
             ORDER BY timestamp DESC
             LIMIT ?2"
        };

        let mut stmt = conn.prepare(query)?;
        
        let messages: Vec<Message> = if let Some(before_id) = before_message_id {
            stmt.query_map(params![chat_id, before_id, limit], |row| {
                Self::row_to_message(row)
            })?
            .filter_map(|r| r.ok())
            .collect()
        } else {
            stmt.query_map(params![chat_id, limit], |row| {
                Self::row_to_message(row)
            })?
            .filter_map(|r| r.ok())
            .collect()
        };

        Ok(messages)
    }
    
    fn row_to_message(row: &rusqlite::Row) -> rusqlite::Result<Message> {
        Ok(Message {
            message_id: row.get(0)?,
            chat_id: row.get(1)?,
            sender_id: row.get(2)?,
            content_type: row.get(3)?,
            content: row.get(4)?,
            encrypted: row.get::<_, i32>(5)? == 1,
            signature: row.get(6)?,
            nonce: row.get(7)?,
            timestamp: row.get::<_, i64>(8)? as u64,
            status: row.get(9)?,
            is_read: row.get::<_, i32>(10)? == 1,
            read_by: row.get(11)?,
            reply_to_message_id: row.get(12)?,
        })
    }
    
    /// Пометить сообщение как прочитанное
    pub async fn mark_message_as_read(&self, message_id: &str, read_by: &[&str]) -> Result<()> {
        let conn = self.conn.lock().await;
        
        let read_by_json = serde_json::to_string(read_by)?;
        
        conn.execute(
            "UPDATE messages SET is_read = 1, read_by = ?1 WHERE message_id = ?2",
            params![read_by_json, message_id],
        )?;
        
        Ok(())
    }
    
    /// Удалить сообщение
    pub async fn delete_message(&self, message_id: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute("DELETE FROM messages WHERE message_id = ?1", params![message_id])?;
        
        Ok(())
    }
    
    // ========================================================================
    // МЕТОДЫ ДЛЯ ЗВОНКОВ
    // ========================================================================
    
    /// Сохранить информацию о звонке
    pub async fn save_call(&self, call: &Call) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "INSERT OR REPLACE INTO calls (
                call_id, caller_id, callee_id, call_type, status,
                started_at, ended_at, duration_secs, end_reason
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                call.call_id,
                call.caller_id,
                call.callee_id,
                call.call_type,
                call.status,
                call.started_at.map(|t| t as i64),
                call.ended_at.map(|t| t as i64),
                call.duration_secs.map(|t| t as i32),
                call.end_reason,
            ],
        )?;
        
        Ok(())
    }
    
    /// Получить историю звонков
    pub async fn get_call_history(&self, limit: u32) -> Result<Vec<Call>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT call_id, caller_id, callee_id, call_type, status,
                    started_at, ended_at, duration_secs, end_reason
             FROM calls ORDER BY started_at DESC LIMIT ?1"
        )?;
        
        let calls = stmt.query_map(params![limit], |row| {
            Ok(Call {
                call_id: row.get(0)?,
                caller_id: row.get(1)?,
                callee_id: row.get(2)?,
                call_type: row.get(3)?,
                status: row.get(4)?,
                started_at: row.get::<_, Option<i64>>(5)?.map(|t| t as u64),
                ended_at: row.get::<_, Option<i64>>(6)?.map(|t| t as u64),
                duration_secs: row.get::<_, Option<i32>>(7)?.map(|t| t as u32),
                end_reason: row.get(8)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
        
        Ok(calls)
    }
    
    /// Сохранить ICE кандидат
    pub async fn save_ice_candidate(
        &self,
        call_id: &str,
        candidate: &str,
        sdp_m_line_index: u16,
        sdp_mid: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().await;
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        conn.execute(
            "INSERT INTO ice_candidates (call_id, candidate, sdp_m_line_index, sdp_mid, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![call_id, candidate, sdp_m_line_index, sdp_mid, now],
        )?;
        
        Ok(())
    }
    
    /// Получить ICE кандидаты для звонка
    pub async fn get_ice_candidates(
        &self,
        call_id: &str,
    ) -> Result<Vec<(String, u16, String)>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT candidate, sdp_m_line_index, sdp_mid
             FROM ice_candidates WHERE call_id = ?1
             ORDER BY created_at"
        )?;
        
        let candidates = stmt.query_map(params![call_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .filter_map(|r| r.ok())
        .collect();
        
        Ok(candidates)
    }
    
    // ========================================================================
    // МЕТОДЫ ДЛЯ НАСТРОЕК
    // ========================================================================
    
    /// Сохранить настройку
    pub async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at)
             VALUES (?1, ?2, ?3)",
            params![key, value, now],
        )?;
        
        Ok(())
    }
    
    /// Получить настройку
    pub async fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;

        let value = stmt.query_row(params![key], |row| {
            row.get(0)
        }).ok();

        Ok(value)
    }
    
    // ========================================================================
    // УТИЛИТЫ
    // ========================================================================
    
    /// Получить статистику базы данных
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        let conn = self.conn.lock().await;
        
        let user_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM users WHERE is_contact = 1",
            [],
            |row| row.get(0),
        )?;
        
        let chat_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM chats",
            [],
            |row| row.get(0),
        )?;
        
        let message_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM messages",
            [],
            |row| row.get(0),
        )?;
        
        let call_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM calls",
            [],
            |row| row.get(0),
        )?;
        
        Ok(DatabaseStats {
            user_count: user_count as u32,
            chat_count: chat_count as u32,
            message_count: message_count as u32,
            call_count: call_count as u32,
        })
    }
    
    /// Проверить целостность базы данных
    pub async fn integrity_check(&self) -> Result<bool> {
        let conn = self.conn.lock().await;
        
        let result: String = conn.query_row("PRAGMA integrity_check", [], |row| {
            row.get(0)
        })?;
        
        Ok(result == "ok")
    }
    
    /// Закрыть базу данных
    pub async fn close(self) -> Result<()> {
        // Arc будет автоматически закрыт когда все ссылки будут удалены
        info!("Database closed");
        Ok(())
    }
    
    /// Проверить инициализирована ли база
    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }
    
    // ========================================================================
    // РЕАКЦИИ
    // ========================================================================
    
    /// Добавить реакцию
    pub async fn add_reaction(&self, reaction: &Reaction) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "INSERT INTO reactions (reaction_id, message_id, user_id, emoji, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                reaction.reaction_id,
                reaction.message_id,
                reaction.user_id,
                reaction.emoji,
                reaction.created_at as i64,
            ],
        )?;
        
        Ok(())
    }
    
    /// Получить реакции сообщения
    pub async fn get_reactions(&self, message_id: &str) -> Result<Vec<Reaction>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare("SELECT reaction_id, message_id, user_id, emoji, created_at FROM reactions WHERE message_id = ?1")?;
        
        let reactions = stmt.query_map(params![message_id], |row| {
            Ok(Reaction {
                reaction_id: row.get(0)?,
                message_id: row.get(1)?,
                user_id: row.get(2)?,
                emoji: row.get(3)?,
                created_at: row.get::<_, i64>(4)? as u64,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
        
        Ok(reactions)
    }
    
    /// Удалить реакцию
    pub async fn remove_reaction(&self, reaction_id: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute("DELETE FROM reactions WHERE reaction_id = ?1", params![reaction_id])?;
        
        Ok(())
    }
    
    // ========================================================================
    // ЗАКРЕПЛЁННЫЕ СООБЩЕНИЯ
    // ========================================================================
    
    /// Закрепить сообщение
    pub async fn pin_message(&self, pinned: &PinnedMessage) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "INSERT INTO pinned_messages (pin_id, chat_id, message_id, pinned_by, pinned_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                pinned.pin_id,
                pinned.chat_id,
                pinned.message_id,
                pinned.pinned_by,
                pinned.pinned_at as i64,
            ],
        )?;
        
        Ok(())
    }
    
    /// Получить закреплённые сообщения чата
    pub async fn get_pinned_messages(&self, chat_id: &str) -> Result<Vec<PinnedMessage>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare("SELECT pin_id, chat_id, message_id, pinned_by, pinned_at FROM pinned_messages WHERE chat_id = ?1")?;
        
        let pinned = stmt.query_map(params![chat_id], |row| {
            Ok(PinnedMessage {
                pin_id: row.get(0)?,
                chat_id: row.get(1)?,
                message_id: row.get(2)?,
                pinned_by: row.get(3)?,
                pinned_at: row.get::<_, i64>(4)? as u64,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
        
        Ok(pinned)
    }
    
    /// Открепить сообщение
    pub async fn unpin_message(&self, pin_id: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute("DELETE FROM pinned_messages WHERE pin_id = ?1", params![pin_id])?;
        
        Ok(())
    }
    
    // ========================================================================
    // ИЗБРАННОЕ
    // ========================================================================
    
    /// Добавить в избранное
    pub async fn add_to_favorites(&self, fav: &FavoriteMessage) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "INSERT INTO favorite_messages (fav_id, message_id, user_id, tags, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                fav.fav_id,
                fav.message_id,
                fav.user_id,
                fav.tags,
                fav.created_at as i64,
            ],
        )?;
        
        Ok(())
    }
    
    /// Получить избранные сообщения
    pub async fn get_favorites(&self, user_id: &str) -> Result<Vec<FavoriteMessage>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare("SELECT fav_id, message_id, user_id, tags, created_at FROM favorite_messages WHERE user_id = ?1")?;
        
        let favorites = stmt.query_map(params![user_id], |row| {
            Ok(FavoriteMessage {
                fav_id: row.get(0)?,
                message_id: row.get(1)?,
                user_id: row.get(2)?,
                tags: row.get(3)?,
                created_at: row.get::<_, i64>(4)? as u64,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
        
        Ok(favorites)
    }
    
    /// Удалить из избранного
    pub async fn remove_from_favorites(&self, fav_id: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute("DELETE FROM favorite_messages WHERE fav_id = ?1", params![fav_id])?;
        
        Ok(())
    }
    
    // ========================================================================
    // ОТЛОЖЕННЫЕ СООБЩЕНИЯ
    // ========================================================================
    
    /// Запланировать сообщение
    pub async fn schedule_message(&self, scheduled: &ScheduledMessage) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "INSERT INTO scheduled_messages (schedule_id, chat_id, content, send_at, created_at, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                scheduled.schedule_id,
                scheduled.chat_id,
                scheduled.content,
                scheduled.send_at as i64,
                scheduled.created_at as i64,
                scheduled.status,
            ],
        )?;
        
        Ok(())
    }
    
    /// Получить готовые к отправке сообщения
    pub async fn get_due_scheduled_messages(&self, now: u64) -> Result<Vec<ScheduledMessage>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare("SELECT schedule_id, chat_id, content, send_at, created_at, status FROM scheduled_messages WHERE send_at <= ?1 AND status = 'pending'")?;
        
        let scheduled = stmt.query_map(params![now as i64], |row| {
            Ok(ScheduledMessage {
                schedule_id: row.get(0)?,
                chat_id: row.get(1)?,
                content: row.get(2)?,
                send_at: row.get::<_, i64>(3)? as u64,
                created_at: row.get::<_, i64>(4)? as u64,
                status: row.get(5)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
        
        Ok(scheduled)
    }
    
    /// Обновить статус отложенного сообщения
    pub async fn update_scheduled_message_status(&self, schedule_id: &str, status: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "UPDATE scheduled_messages SET status = ?1 WHERE schedule_id = ?2",
            params![status, schedule_id],
        )?;
        
        Ok(())
    }
    
    // ========================================================================
    // СТИКЕРЫ
    // ========================================================================
    
    /// Создать пак стикеров
    pub async fn create_sticker_pack(&self, pack: &StickerPack) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "INSERT INTO sticker_packs (pack_id, name, creator_id, stickers_count, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                pack.pack_id,
                pack.name,
                pack.creator_id,
                pack.stickers_count,
                pack.created_at as i64,
            ],
        )?;
        
        Ok(())
    }
    
    /// Добавить стикер в пак
    pub async fn add_sticker(&self, sticker: &Sticker) -> Result<()> {
        let conn = self.conn.lock().await;
        
        conn.execute(
            "INSERT INTO stickers (sticker_id, pack_id, data, emoji)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                sticker.sticker_id,
                sticker.pack_id,
                sticker.data,
                sticker.emoji,
            ],
        )?;
        
        // Обновляем количество стикеров в паке
        conn.execute(
            "UPDATE sticker_packs SET stickers_count = stickers_count + 1 WHERE pack_id = ?1",
            params![sticker.pack_id],
        )?;
        
        Ok(())
    }
    
    /// Получить пак стикеров
    pub async fn get_sticker_pack(&self, pack_id: &str) -> Result<Option<StickerPack>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare("SELECT pack_id, name, creator_id, stickers_count, created_at FROM sticker_packs WHERE pack_id = ?1")?;
        
        let pack = stmt.query_row(params![pack_id], |row| {
            Ok(StickerPack {
                pack_id: row.get(0)?,
                name: row.get(1)?,
                creator_id: row.get(2)?,
                stickers_count: row.get(3)?,
                created_at: row.get::<_, i64>(4)? as u64,
            })
        }).ok();
        
        Ok(pack)
    }
    
    /// Получить стикеры пака
    pub async fn get_stickers(&self, pack_id: &str) -> Result<Vec<Sticker>> {
        let conn = self.conn.lock().await;
        
        let mut stmt = conn.prepare("SELECT sticker_id, pack_id, data, emoji FROM stickers WHERE pack_id = ?1")?;
        
        let stickers = stmt.query_map(params![pack_id], |row| {
            Ok(Sticker {
                sticker_id: row.get(0)?,
                pack_id: row.get(1)?,
                data: row.get(2)?,
                emoji: row.get(3)?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();
        
        Ok(stickers)
    }
}

// ============================================================================
// МОДЕЛИ ДАННЫХ
// ============================================================================

/// Семейный статус пользователя
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FamilyStatus {
    Single,
    InRelationship,
    Engaged,
    Married,
    InCivilPartnership,
    Divorced,
    Widowed,
}

impl Default for FamilyStatus {
    fn default() -> Self {
        Self::Single
    }
}

impl std::fmt::Display for FamilyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FamilyStatus::Single => write!(f, "Свободен"),
            FamilyStatus::InRelationship => write!(f, "Встречаюсь"),
            FamilyStatus::Engaged => write!(f, "Помолвлен"),
            FamilyStatus::Married => write!(f, "Женат/Замужем"),
            FamilyStatus::InCivilPartnership => write!(f, "В гражданском браке"),
            FamilyStatus::Divorced => write!(f, "Разведён"),
            FamilyStatus::Widowed => write!(f, "Вдовец/Вдова"),
        }
    }
}

impl rusqlite::types::FromSql for FamilyStatus {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let str = value.as_str()?;
        match str {
            "single" => Ok(FamilyStatus::Single),
            "in_relationship" => Ok(FamilyStatus::InRelationship),
            "engaged" => Ok(FamilyStatus::Engaged),
            "married" => Ok(FamilyStatus::Married),
            "in_civil_partnership" => Ok(FamilyStatus::InCivilPartnership),
            "divorced" => Ok(FamilyStatus::Divorced),
            "widowed" => Ok(FamilyStatus::Widowed),
            _ => Ok(FamilyStatus::Single),
        }
    }
}

/// Пользователь
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    pub username: String,
    pub display_name: Option<String>,
    pub public_key: Option<Vec<u8>>,
    pub pq_public_key: Option<Vec<u8>>,
    pub created_at: u64,
    pub last_seen: Option<u64>,
    pub is_contact: bool,
    pub nickname: Option<String>,
    // Новые поля
    pub bio: Option<String>,
    pub family_status: Option<FamilyStatus>,
    pub partner_id: Option<String>,
    pub wallpaper: Option<String>,
    pub synced_wallpaper_with: Option<String>,
    pub theme: Option<String>,
    pub is_verified: bool,
    pub is_bot: bool,
}

/// Чат
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub chat_id: String,
    pub chat_type: String,
    pub name: Option<String>,
    pub created_at: u64,
    pub last_message_at: Option<u64>,
    pub unread_count: u32,
    pub members: Vec<ChatMember>,
}

/// Участник чата
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMember {
    pub user_id: String,
    pub role: String,
    pub joined_at: u64,
}

/// Сообщение
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub message_id: String,
    pub chat_id: String,
    pub sender_id: String,
    pub content_type: String,
    pub content: Option<Vec<u8>>,
    pub encrypted: bool,
    pub signature: Option<Vec<u8>>,
    pub nonce: Option<Vec<u8>>,
    pub timestamp: u64,
    pub status: String,
    pub is_read: bool,
    pub read_by: Option<String>,
    pub reply_to_message_id: Option<String>,
}

/// Звонок
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Call {
    pub call_id: String,
    pub caller_id: String,
    pub callee_id: String,
    pub call_type: String,
    pub status: String,
    pub started_at: Option<u64>,
    pub ended_at: Option<u64>,
    pub duration_secs: Option<u32>,
    pub end_reason: Option<String>,
}

/// Реакция на сообщение
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub reaction_id: String,
    pub message_id: String,
    pub user_id: String,
    pub emoji: String,
    pub created_at: u64,
}

/// Закреплённое сообщение
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedMessage {
    pub pin_id: String,
    pub chat_id: String,
    pub message_id: String,
    pub pinned_by: String,
    pub pinned_at: u64,
}

/// Избранное сообщение
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteMessage {
    pub fav_id: String,
    pub message_id: String,
    pub user_id: String,
    pub tags: String,
    pub created_at: u64,
}

/// Отложенное сообщение
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledMessage {
    pub schedule_id: String,
    pub chat_id: String,
    pub content: Vec<u8>,
    pub send_at: u64,
    pub created_at: u64,
    pub status: String,
}

/// Стикеры
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sticker {
    pub sticker_id: String,
    pub pack_id: String,
    pub data: Vec<u8>,
    pub emoji: Option<String>,
}

/// Пак стикеров
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickerPack {
    pub pack_id: String,
    pub name: String,
    pub creator_id: String,
    pub stickers_count: u32,
    pub created_at: u64,
}

/// Статистика базы данных
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub user_count: u32,
    pub chat_count: u32,
    pub message_count: u32,
    pub call_count: u32,
}

// ============================================================================
// ГЕНЕРАЦИЯ КЛЮЧА ШИФРОВАНИЯ
// ============================================================================

/// Сгенерировать ключ шифрования из пароля
pub fn generate_encryption_key(password: &str, salt: &[u8]) -> Result<Vec<u8>> {
    use crate::crypto::Kdf;
    
    // Используем PBKDF2 с большим количеством итераций
    let key = Kdf::pbkdf2_derive(
        password.as_bytes(),
        salt,
        256000, // SQLCipher рекомендует
        32,     // AES-256
    )?;
    
    Ok(key)
}

/// Сгенерировать случайный ключ шифрования
pub fn generate_random_key() -> Vec<u8> {
    use rand::{RngCore, thread_rng};
    
    let mut key = [0u8; 32];
    thread_rng().fill_bytes(&mut key);
    key.to_vec()
}

/// Получить соль из файла базы данных (первые 16 байт)
pub fn get_salt_from_file(path: &Path) -> Result<Vec<u8>> {
    if path.exists() {
        let metadata = std::fs::metadata(path)?;
        if metadata.len() >= 16 {
            let mut file = std::fs::File::open(path)?;
            use std::io::Read;
            let mut buffer = [0u8; 16];
            file.read_exact(&mut buffer)?;
            return Ok(buffer.to_vec());
        }
    }
    
    // Генерируем новую соль
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    Ok(salt.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_database_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let key = generate_random_key();
        let config = DatabaseConfig {
            path: db_path.clone(),
            encryption_key: key,
            create_if_missing: true,
        };
        
        let db = DatabaseManager::new(config).await.unwrap();
        assert!(db.is_initialized());
        
        let stats = db.get_stats().await.unwrap();
        assert_eq!(stats.user_count, 0);
        assert_eq!(stats.chat_count, 0);
    }
    
    #[tokio::test]
    async fn test_user_crud() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let key = generate_random_key();
        let config = DatabaseConfig {
            path: db_path,
            encryption_key: key,
            create_if_missing: true,
        };
        
        let db = DatabaseManager::new(config).await.unwrap();
        
        let user = User {
            user_id: "user123".to_string(),
            username: "testuser".to_string(),
            display_name: Some("Test User".to_string()),
            public_key: Some(vec![1, 2, 3]),
            pq_public_key: Some(vec![4, 5, 6]),
            created_at: 1234567890,
            last_seen: None,
            is_contact: true,
            nickname: None,
        };
        
        db.add_user(&user).await.unwrap();
        
        let retrieved = db.get_user("user123").await.unwrap().unwrap();
        assert_eq!(retrieved.user_id, "user123");
        assert_eq!(retrieved.username, "testuser");
    }
    
    #[tokio::test]
    async fn test_message_crud() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        
        let key = generate_random_key();
        let config = DatabaseConfig {
            path: db_path,
            encryption_key: key,
            create_if_missing: true,
        };
        
        let db = DatabaseManager::new(config).await.unwrap();
        
        // Создаем чат
        let chat = Chat {
            chat_id: "chat123".to_string(),
            chat_type: "direct".to_string(),
            name: None,
            created_at: 1234567890,
            last_message_at: None,
            unread_count: 0,
            members: vec![],
        };
        db.create_chat(&chat).await.unwrap();
        
        // Добавляем сообщение
        let message = Message {
            message_id: "msg456".to_string(),
            chat_id: "chat123".to_string(),
            sender_id: "user123".to_string(),
            content_type: "text".to_string(),
            content: Some(b"Hello".to_vec()),
            encrypted: true,
            signature: None,
            nonce: None,
            timestamp: 1234567890,
            status: "sent".to_string(),
            is_read: false,
            read_by: None,
            reply_to_message_id: None,
        };
        db.save_message(&message).await.unwrap();
        
        // Получаем историю
        let history = db.get_message_history("chat123", 10, None).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].message_id, "msg456");
    }
}
