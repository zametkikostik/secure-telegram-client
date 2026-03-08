//! Liberty Reach Messenger v2.0 - Core Library
//! 
//! Thick Core Model: весь функционал (Network, SIP, Crypto, Database, Media)
//! реализован в изолированной Rust-библиотеке. UI (Flutter/Tauri) общается
//! с ядром только через асинхронный FFI-мост.
//!
//! # Architecture
//! 
//! - **Event-Driven**: Обмен данными через Commands (UI → Core) и Events (Core → UI)
//! - **Never-Die**: Экспоненциальный откат при переподключениях
//! - **Zero Panic**: Result и anyhow, никаких unwrap()
//! - **Resilience**: Работа в сетях 2G/EDGE с Opus 6-32 kbps

#![warn(rust_2018_idioms)]
#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

// Загрузка .env.local при старте
fn load_env() {
    let _ = dotenv::dotenv();
}

pub mod bridge;
pub mod config;
pub mod crypto;
pub mod engine;
pub mod ffi;
pub mod media;
pub mod obfuscation;
pub mod p2p;
pub mod sip;
pub mod storage;
pub mod ai;
pub mod webrtc;
pub mod bots;
pub mod web3;

pub use bridge::{LrCommand, LrEvent, LrEventTx};
pub use engine::{LrCore, LrCoreConfig};
pub use storage::{DatabaseManager, generate_random_key, FamilyStatus};
pub use obfuscation::ObfuscationManager;
pub use crypto::CryptoContainer;
pub use ai::{AIManager, MessengerAI, AIProvider, AITask};
pub use webrtc::{WebRtcCallManager, WebRtcConfig, CallType as WebRtcCallType};
pub use bots::{BotManager, Bot, BotFlow};
pub use web3::{Web3Manager, Web3Provider, Web3Command};

use anyhow::Result;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

/// Инициализация логирования
pub fn init_logging(level: Level) -> Result<()> {
    // Загружаем .env.local перед инициализацией
    load_env();
    
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .json()
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| anyhow::anyhow!("Failed to set subscriber: {}", e))?;
    
    tracing::info!("Liberty Reach Core v{} initialized", env!("CARGO_PKG_VERSION"));
    tracing::info!("Environment loaded from .env.local");
    Ok(())
}

/// Версия ядра
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Название проекта
pub const PROJECT_NAME: &str = "Liberty Reach Messenger";

/// Edition
pub const EDITION: &str = "Universal Resilient Edition";
