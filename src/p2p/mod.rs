//! P2P модуль
//!
//! Обеспечивает fallback коммуникацию когда Telegram недоступен

pub mod client;

pub use client::{P2PClient, P2PMessage, P2PConfig};

use anyhow::Result;

/// Инициализация P2P модуля
pub fn init() -> Result<()> {
    log::info!("Инициализация P2P модуля...");
    Ok(())
}
