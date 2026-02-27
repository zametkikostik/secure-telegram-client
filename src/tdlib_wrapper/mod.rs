//! TDLib обёртка
//!
//! Интеграция с TDLib для работы с Telegram.

pub mod client;

pub use client::TdClient;

use anyhow::Result;

/// Инициализация TDLib модуля
pub fn init() -> Result<()> {
    log::info!("Инициализация TDLib модуля...");
    Ok(())
}
