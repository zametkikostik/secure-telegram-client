//! P2P модуль
//!
//! Обеспечивает fallback коммуникацию когда Telegram недоступен

pub mod client;

pub use client::{P2PClient, P2PConfig, P2PMessage};

use anyhow::Result;

/// Инициализация P2P модуля
pub fn init() -> Result<()> {
    log::info!("Инициализация P2P модуля...");
    Ok(())
}
