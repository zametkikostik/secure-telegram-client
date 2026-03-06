// server/src/db.rs
//! База данных SQLite/PostgreSQL

use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::sync::Arc;

pub type DbPool = Arc<SqlitePool>;

/// Инициализация базы данных
pub async fn init_database() -> anyhow::Result<DbPool> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./liberty_reach.db".to_string());

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .connect(&database_url)
        .await?;

    // Создание таблиц
    sqlx::query(
        r#"
        -- Пользователи
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            email TEXT UNIQUE,
            password_hash TEXT NOT NULL,
            public_key TEXT NOT NULL,
            avatar_url TEXT,
            status TEXT DEFAULT 'offline',
            family_status TEXT DEFAULT 'single',
            bio TEXT,
            theme TEXT DEFAULT 'light',
            night_mode BOOLEAN DEFAULT FALSE,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- P2P Ноды
        CREATE TABLE IF NOT EXISTS peer_nodes (
            id TEXT PRIMARY KEY,
            user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
            username TEXT NOT NULL,
            public_key TEXT NOT NULL,
            peer_id TEXT NOT NULL,
            multiaddr TEXT,
            status TEXT DEFAULT 'offline',
            joined_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            last_seen DATETIME DEFAULT CURRENT_TIMESTAMP,
            version TEXT DEFAULT '1.0.0',
            capabilities TEXT
        );

        -- Чаты
        CREATE TABLE IF NOT EXISTS chats (
            id TEXT PRIMARY KEY,
            type TEXT NOT NULL,
            name TEXT,
            description TEXT,
            owner_id TEXT REFERENCES users(id),
            wallpaper_url TEXT,
            wallpaper_sync BOOLEAN DEFAULT FALSE,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Участники чатов
        CREATE TABLE IF NOT EXISTS chat_members (
            chat_id TEXT REFERENCES chats(id) ON DELETE CASCADE,
            user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
            role TEXT DEFAULT 'member',
            joined_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (chat_id, user_id)
        );

        -- Сообщения
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            chat_id TEXT REFERENCES chats(id) ON DELETE CASCADE,
            sender_id TEXT REFERENCES users(id),
            content TEXT NOT NULL,
            translated_content TEXT,
            message_type TEXT DEFAULT 'text',
            file_url TEXT,
            reply_to_id TEXT REFERENCES messages(id),
            is_edited BOOLEAN DEFAULT FALSE,
            is_deleted BOOLEAN DEFAULT FALSE,
            is_pinned BOOLEAN DEFAULT FALSE,
            pinned_at DATETIME,
            pinned_by TEXT,
            scheduled_for DATETIME,
            auto_delete_hours INTEGER DEFAULT NULL,
            delete_at DATETIME DEFAULT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Закреплённые сообщения
        CREATE TABLE IF NOT EXISTS pinned_messages (
            chat_id TEXT REFERENCES chats(id) ON DELETE CASCADE,
            message_id TEXT REFERENCES messages(id) ON DELETE CASCADE,
            pinned_by TEXT REFERENCES users(id),
            pinned_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (chat_id, message_id)
        );

        -- Избранные сообщения (заметки пользователя)
        CREATE TABLE IF NOT EXISTS saved_messages (
            id TEXT PRIMARY KEY,
            user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
            chat_id TEXT,
            message_id TEXT,
            content TEXT NOT NULL,
            message_type TEXT DEFAULT 'text',
            file_url TEXT,
            tags TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Отложенные сообщения
        CREATE TABLE IF NOT EXISTS scheduled_messages (
            id TEXT PRIMARY KEY,
            chat_id TEXT REFERENCES chats(id) ON DELETE CASCADE,
            sender_id TEXT REFERENCES users(id),
            content TEXT NOT NULL,
            message_type TEXT DEFAULT 'text',
            file_url TEXT,
            send_at DATETIME NOT NULL,
            status TEXT DEFAULT 'pending',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Прочитанные сообщения
        CREATE TABLE IF NOT EXISTS message_reads (
            message_id TEXT REFERENCES messages(id) ON DELETE CASCADE,
            user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
            read_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (message_id, user_id)
        );

        -- Файлы
        CREATE TABLE IF NOT EXISTS files (
            id TEXT PRIMARY KEY,
            owner_id TEXT REFERENCES users(id),
            filename TEXT NOT NULL,
            original_name TEXT,
            mime_type TEXT,
            size INTEGER,
            url TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Контакты
        CREATE TABLE IF NOT EXISTS contacts (
            user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
            contact_id TEXT REFERENCES users(id) ON DELETE CASCADE,
            added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (user_id, contact_id)
        );

        -- Семейные связи
        CREATE TABLE IF NOT EXISTS family_relations (
            id TEXT PRIMARY KEY,
            user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
            relative_id TEXT REFERENCES users(id) ON DELETE CASCADE,
            relation_type TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(user_id, relative_id)
        );

        -- Обои чата (синхронизированные)
        CREATE TABLE IF NOT EXISTS chat_wallpapers (
            chat_id TEXT REFERENCES chats(id) ON DELETE CASCADE,
            user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
            wallpaper_url TEXT NOT NULL,
            wallpaper_type TEXT DEFAULT 'custom',
            synced BOOLEAN DEFAULT FALSE,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (chat_id, user_id)
        );

        -- Стикеры
        CREATE TABLE IF NOT EXISTS stickers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            emoji TEXT,
            pack_id TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Паки стикеров
        CREATE TABLE IF NOT EXISTS sticker_packs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            owner_id TEXT REFERENCES users(id),
            cover_url TEXT,
            is_animated BOOLEAN DEFAULT FALSE,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- GIF
        CREATE TABLE IF NOT EXISTS gifs (
            id TEXT PRIMARY KEY,
            url TEXT NOT NULL,
            title TEXT,
            width INTEGER,
            height INTEGER,
            size INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- Эмодзи реакции
        CREATE TABLE IF NOT EXISTS message_reactions (
            message_id TEXT REFERENCES messages(id) ON DELETE CASCADE,
            user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
            emoji TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            PRIMARY KEY (message_id, user_id, emoji)
        );

        -- Демонстрация экрана (сессии)
        CREATE TABLE IF NOT EXISTS screen_share_sessions (
            id TEXT PRIMARY KEY,
            chat_id TEXT REFERENCES chats(id) ON DELETE CASCADE,
            user_id TEXT REFERENCES users(id),
            stream_url TEXT,
            started_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            ended_at DATETIME
        );

        -- Индексы для производительности
        CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages(chat_id);
        CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at);
        CREATE INDEX IF NOT EXISTS idx_messages_delete_at ON messages(delete_at) WHERE delete_at IS NOT NULL;
        CREATE INDEX IF NOT EXISTS idx_messages_scheduled_for ON messages(scheduled_for) WHERE scheduled_for IS NOT NULL;
        CREATE INDEX IF NOT EXISTS idx_messages_pinned ON messages(is_pinned) WHERE is_pinned = 1;
        CREATE INDEX IF NOT EXISTS idx_chat_members_user ON chat_members(user_id);
        CREATE INDEX IF NOT EXISTS idx_message_reads_user ON message_reads(user_id);
        CREATE INDEX IF NOT EXISTS idx_family_relations_user ON family_relations(user_id);
        CREATE INDEX IF NOT EXISTS idx_saved_messages_user ON saved_messages(user_id);
        CREATE INDEX IF NOT EXISTS idx_scheduled_messages_send_at ON scheduled_messages(send_at);
        "#,
    )
    .execute(&pool)
    .await?;

    tracing::info!("Таблицы базы данных созданы");

    Ok(Arc::new(pool))
}

/// Получить пул соединений
/// Примечание: Используйте пул из AppState вместо этой функции
#[deprecated(note = "Используйте pool из AppState")]
pub fn get_pool() -> DbPool {
    panic!("get_pool() устарела. Используйте pool из AppState.")
}
