//! FFI Module
//! 
//! Flutter Rust Bridge и другие FFI интерфейсы для связи с UI.

use flutter_rust_bridge::frb;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::Level;

use crate::{
    LrCore, LrCoreConfig,
    bridge::{LrCommand, LrEvent, LrEventTx, create_default_channels},
    init_logging,
};

/// Инициализировать ядро для Flutter
#[frb(sync)]
pub fn frb_init_logging() {
    let _ = init_logging(Level::INFO);
}

/// Создать новое ядро
#[frb(async)]
pub async fn frb_create_core(
    db_path: String,
    encryption_key: Vec<u8>,
    user_id: String,
    username: String,
) -> Result<FrbCoreHandle, String> {
    let (_cmd_rx, event_tx) = create_default_channels();
    
    let config = LrCoreConfig {
        db_path,
        encryption_key,
        user_id,
        username,
        bootstrap_nodes: vec![],
        enable_quic: true,
        enable_obfuscation: false,
        sip_config: None,
    };
    
    let core = LrCore::new(config, event_tx.clone())
        .await
        .map_err(|e| e.to_string())?;
    
    let handle = FrbCoreHandle {
        core: Arc::new(RwLock::new(Some(core))),
        event_tx,
    };
    
    Ok(handle)
}

/// Handle для доступа к ядру из Flutter
pub struct FrbCoreHandle {
    core: Arc<RwLock<Option<LrCore>>>,
    event_tx: LrEventTx,
}

impl FrbCoreHandle {
    /// Отправить команду
    pub async fn send_command(&self, command: LrCommand) -> Result<(), String> {
        let core_guard = self.core.read().await;
        let core = core_guard.as_ref().ok_or("Core not initialized")?;
        
        core.handle_command(command).await.map_err(|e| e.to_string())
    }
    
    /// Получить Peer ID
    pub async fn get_peer_id(&self) -> Result<String, String> {
        let core_guard = self.core.read().await;
        let core = core_guard.as_ref().ok_or("Core not initialized")?;
        
        Ok(core.peer_id().to_string())
    }
    
    /// Остановить ядро
    pub async fn shutdown(&self) -> Result<(), String> {
        let mut core_guard = self.core.write().await;
        let core = core_guard.take().ok_or("Core not initialized")?;
        
        core.shutdown().await.map_err(|e| e.to_string())
    }
}

/// Отправить команду в ядро
#[frb(async)]
pub async fn frb_send_command(
    handle: &FrbCoreHandle,
    command: LrCommand,
) -> Result<(), String> {
    handle.send_command(command).await
}

/// Получить Peer ID
#[frb(async)]
pub async fn frb_get_peer_id(handle: &FrbCoreHandle) -> Result<String, String> {
    handle.get_peer_id().await
}

/// Остановить ядро
#[frb(async)]
pub async fn frb_shutdown(handle: &FrbCoreHandle) -> Result<(), String> {
    handle.shutdown().await
}

/// Версия ядра
#[frb(sync)]
pub fn frb_get_version() -> String {
    crate::VERSION.to_string()
}

/// Название проекта
#[frb(sync)]
pub fn frb_get_project_name() -> String {
    crate::PROJECT_NAME.to_string()
}

/// Edition
#[frb(sync)]
pub fn frb_get_edition() -> String {
    crate::EDITION.to_string()
}

// ============================================================================
// МОСТИКИ ДЛЯ БАЗОВЫХ ТИПОВ
// ============================================================================

/// Простой результат для FFI
#[derive(Debug)]
pub struct FrbResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> FrbResult<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    pub fn err(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

/// Конвертировать Rust Result в FrbResult
pub fn to_frb_result<T, E: ToString>(result: Result<T, E>) -> FrbResult<T> {
    match result {
        Ok(data) => FrbResult::ok(data),
        Err(e) => FrbResult::err(e.to_string()),
    }
}

// ============================================================================
// BUILD SCRIPT HOOKS
// ============================================================================

/// Сгенерировать FFI bindings для Flutter
#[cfg(feature = "flutter")]
pub fn generate_flutter_bindings() {
    use flutter_rust_bridge::frb_generated;
    
    // Эта функция вызывается из build.rs
    frb_generated::generate_bindings();
}

/// Сгенерировать C bindings для Tauri
#[cfg(feature = "tauri")]
pub fn generate_c_bindings() {
    use cbindgen::Config;
    
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    
    let config = Config::default();
    
    cbindgen::generate(&crate_dir)
        .expect("Unable to generate C bindings")
        .write_to_file("bindings.h");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version() {
        let version = frb_get_version();
        assert!(!version.is_empty());
    }
    
    #[test]
    fn test_project_name() {
        let name = frb_get_project_name();
        assert_eq!(name, "Liberty Reach Messenger");
    }
}
