// Application state management
// SECURITY: требует аудита перед production
// TODO: pentest перед release

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

/// Глобальное состояние приложения
#[derive(Clone)]
pub struct AppState {
    /// ID пользователя
    pub user_id: Arc<RwLock<Option<String>>>,
    /// Флаг инициализации
    pub initialized: Arc<RwLock<bool>>,
    /// Конфигурация
    pub config: Arc<RwLock<AppConfig>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: String,
    pub language: String,
    pub notifications: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            user_id: Arc::new(RwLock::new(None)),
            initialized: Arc::new(RwLock::new(false)),
            config: Arc::new(RwLock::new(AppConfig::default())),
        }
    }

    /// Инициализация (первый запуск)
    pub async fn initialize(&self) -> Result<(), AppError> {
        *self.initialized.write().await = true;
        tracing::info!("AppState initialized");
        Ok(())
    }

    /// Проверка инициализации
    pub async fn is_initialized(&self) -> bool {
        *self.initialized.read().await
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
    #[error("Not initialized")]
    NotInitialized,
}
