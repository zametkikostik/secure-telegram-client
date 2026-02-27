//! Модуль хранения данных
//!
//! Предоставляет:
//! - Зашифрованную SQLite базу
//! - Очередь сообщений
//! - Кэш данных

pub mod message_queue;

pub use message_queue::{MessageQueue, MessageQueueConfig, QueuedMessage, MessageStatus};

use anyhow::Result;

/// Инициализация модуля хранения
pub fn init() -> Result<()> {
    log::info!("Инициализация модуля хранения...");
    Ok(())
}
