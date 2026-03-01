//! P2P модуль
//!
//! Обеспечивает fallback коммуникацию когда Telegram недоступен

pub mod client;


use anyhow::Result;

/// Инициализация P2P модуля
pub fn init() -> Result<()> {
    log::info!("Инициализация P2P модуля...");
    Ok(())
}
