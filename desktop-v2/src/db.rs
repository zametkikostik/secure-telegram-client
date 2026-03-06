use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path();
        
        // Создаём директорию если не существует
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let conn = Connection::open(&db_path)?;
        
        // Создаём таблицы
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT,
                public_key BLOB,
                created_at INTEGER
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS chats (
                id TEXT PRIMARY KEY,
                name TEXT,
                chat_type TEXT,
                created_at INTEGER
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                chat_id TEXT,
                sender_id TEXT,
                content TEXT,
                encrypted INTEGER,
                timestamp INTEGER,
                FOREIGN KEY (chat_id) REFERENCES chats(id),
                FOREIGN KEY (sender_id) REFERENCES users(id)
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS p2p_peers (
                peer_id TEXT PRIMARY KEY,
                address TEXT,
                last_seen INTEGER
            )",
            [],
        )?;
        
        Ok(Database { conn })
    }
    
    fn get_db_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("secure-telegram");
        
        config_dir.join("messages.db")
    }
    
    pub fn save_message(&self, message: &Message) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO messages (id, chat_id, sender_id, content, encrypted, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [
                &message.id,
                &message.chat_id,
                &message.sender_id,
                &message.content,
                &message.encrypted.to_string(),
                &message.timestamp.to_string(),
            ],
        )?;
        Ok(())
    }
    
    pub fn get_messages(&self, chat_id: &str) -> Result<Vec<Message>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, chat_id, sender_id, content, encrypted, timestamp
             FROM messages WHERE chat_id = ?1 ORDER BY timestamp ASC"
        )?;
        
        let messages = stmt.query_map([chat_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                chat_id: row.get(1)?,
                sender_id: row.get(2)?,
                content: row.get(3)?,
                encrypted: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })?
        .filter_map(|m| m.ok())
        .collect();
        
        Ok(messages)
    }
    
    pub fn save_user(&self, user: &User) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO users (id, username, public_key, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            [
                &user.id,
                &user.username,
                &user.public_key,
                &user.created_at.to_string(),
            ],
        )?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Message {
    pub id: String,
    pub chat_id: String,
    pub sender_id: String,
    pub content: String,
    pub encrypted: bool,
    pub timestamp: u64,
}

#[derive(Debug)]
pub struct User {
    pub id: String,
    pub username: String,
    pub public_key: Vec<u8>,
    pub created_at: u64,
}
