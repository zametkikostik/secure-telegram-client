//! Модуль конфигурации
//! 
//! Загрузка и управление настройками клиента.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;

/// Конфигурация клиента
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Telegram API ID
    pub api_id: u64,
    /// Telegram API Hash
    pub api_hash: String,
    /// Настройки шифрования
    pub encryption: EncryptionConfig,
    /// Настройки прокси
    pub proxy: ProxyConfig,
    /// Настройки автообновления
    pub auto_update: bool,
    /// Путь к базе данных TDLib
    pub database_path: Option<String>,
}

/// Настройки шифрования
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Включить постквантовое шифрование (Kyber)
    pub kyber_enabled: bool,
    /// Включить стенографию
    pub steganography_enabled: bool,
    /// Включить обфускацию
    pub obfuscation_enabled: bool,
    /// Автоматически применять стенографию к изображениям
    pub auto_steganography: bool,
}

/// Настройки прокси
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Включить прокси
    pub enabled: bool,
    /// Хост прокси
    pub host: String,
    /// Порт прокси
    pub port: u16,
    /// Тип прокси (socks5, http)
    pub proxy_type: String,
    /// Логин (опционально)
    pub username: Option<String>,
    /// Пароль (опционально)
    pub password: Option<String>,
}

impl Config {
    /// Конфигурация по умолчанию
    pub fn default() -> Self {
        Self {
            api_id: 0, // Должен быть установлен пользователем
            api_hash: String::new(),
            encryption: EncryptionConfig {
                kyber_enabled: true,
                steganography_enabled: true,
                obfuscation_enabled: true,
                auto_steganography: true,
            },
            proxy: ProxyConfig {
                enabled: false,
                host: "127.0.0.1".to_string(),
                port: 1080,
                proxy_type: "socks5".to_string(),
                username: None,
                password: None,
            },
            auto_update: true,
            database_path: None,
        }
    }
    
    /// Загрузка конфигурации из файла
    pub fn load() -> Result<Self> {
        let config_path = get_config_path();
        
        if !config_path.exists() {
            log::warn!("Файл конфигурации не найден, используется конфигурация по умолчанию");
            return Ok(Self::default());
        }
        
        let content = fs::read_to_string(&config_path)
            .map_err(|e| anyhow!("Ошибка чтения конфига: {}", e))?;
        
        let config: Config = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Ошибка парсинга конфига: {}", e))?;
        
        log::info!("Конфигурация загружена из {:?}", config_path);
        
        Ok(config)
    }
    
    /// Сохранение конфигурации в файл
    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path();
        
        // Создание директории если не существует
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Ошибка создания директории: {}", e))?;
        }
        
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| anyhow!("Ошибка сериализации конфига: {}", e))?;
        
        fs::write(&config_path, content)
            .map_err(|e| anyhow!("Ошибка записи конфига: {}", e))?;
        
        log::info!("Конфигурация сохранена в {:?}", config_path);
        
        Ok(())
    }
    
    /// Проверка валидности конфигурации
    pub fn validate(&self) -> Result<()> {
        if self.api_id == 0 {
            return Err(anyhow!("api_id должен быть установлен"));
        }
        
        if self.api_hash.is_empty() {
            return Err(anyhow!("api_hash не может быть пустым"));
        }
        
        if self.api_hash.len() < 32 {
            return Err(anyhow!("api_hash должен быть не менее 32 символов"));
        }
        
        Ok(())
    }
}

/// Получение пути к файлу конфигурации
fn get_config_path() -> PathBuf {
    // Проверка переменной окружения
    if let Ok(path) = std::env::var("SECURE_TG_CONFIG") {
        return PathBuf::from(path);
    }
    
    // Путь по умолчанию в домашней директории
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".config");
    path.push("secure-telegram-client");
    path.push("config.json");
    
    path
}

/// Создание конфигурации по умолчанию с шаблоном
pub fn create_default_config() -> Config {
    Config {
        api_id: 0, // Пользователь должен установить своё
        api_hash: "YOUR_API_HASH_HERE".to_string(),
        ..Config::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        
        assert_eq!(config.api_id, 0);
        assert!(config.encryption.kyber_enabled);
        assert!(config.encryption.obfuscation_enabled);
        assert!(!config.proxy.enabled);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        
        // Без api_id
        assert!(config.validate().is_err());
        
        // С api_id но без api_hash
        config.api_id = 123456;
        assert!(config.validate().is_err());
        
        // С полным конфигом
        config.api_hash = "0123456789abcdef0123456789abcdef";
        assert!(config.validate().is_ok());
    }
}
