//! Модуль конфигурации
//!
//! Загрузка и управление настройками клиента.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Путь к конфигурации по умолчанию
const CONFIG_DIR_NAME: &str = "secure-telegram-client";
const CONFIG_FILE_NAME: &str = "config.json";

/// Конфигурация клиента
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Telegram API ID
    pub api_id: u64,
    /// Telegram API Hash
    pub api_hash: String,
    /// Настройки шифрования
    #[serde(default)]
    pub encryption: EncryptionConfig,
    /// Настройки прокси
    #[serde(default)]
    pub proxy: ProxyConfig,
    /// Настройки автообновления
    #[serde(default = "default_true")]
    pub auto_update: bool,
    /// Путь к базе данных TDLib
    pub database_path: Option<String>,
    /// Уровень логирования
    #[serde(default)]
    pub log_level: String,
    /// Настройки P2P
    #[serde(default)]
    pub p2p: P2PConfig,
    /// Настройки хранилища
    #[serde(default)]
    pub storage: StorageConfig,
    /// Stealth mode (anti-censorship)
    #[serde(default = "default_true")]
    pub stealth_mode: bool,
}

/// Настройки шифрования
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Включить постквантовое шифрование (Kyber)
    #[serde(default = "default_true")]
    pub kyber_enabled: bool,
    /// Включить стенографию
    #[serde(default = "default_true")]
    pub steganography_enabled: bool,
    /// Включить обфускацию
    #[serde(default = "default_true")]
    pub obfuscation_enabled: bool,
    /// Автоматически применять стенографию к изображениям
    #[serde(default = "default_true")]
    pub auto_steganography: bool,
}

/// Настройки прокси
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// Включить прокси
    #[serde(default)]
    pub enabled: bool,
    /// Хост прокси
    #[serde(default = "default_proxy_host")]
    pub host: String,
    /// Порт прокси
    #[serde(default = "default_proxy_port")]
    pub port: u16,
    /// Тип прокси (socks5, http)
    #[serde(default = "default_proxy_type")]
    pub proxy_type: String,
    /// Логин (опционально)
    pub username: Option<String>,
    /// Пароль (опционально)
    pub password: Option<String>,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            kyber_enabled: true,
            steganography_enabled: true,
            obfuscation_enabled: true,
            auto_steganography: true,
        }
    }
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: default_proxy_host(),
            port: default_proxy_port(),
            proxy_type: default_proxy_type(),
            username: None,
            password: None,
        }
    }
}

/// Настройки P2P
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PConfig {
    /// Включить P2P fallback
    #[serde(default)]
    pub enabled: bool,
    /// Порт для прослушивания
    #[serde(default = "default_p2p_port")]
    pub listen_port: u16,
    /// Начальные пиры для подключения
    #[serde(default)]
    pub bootstrap_peers: Vec<String>,
    /// Включить mDNS для локальной сети
    #[serde(default = "default_true")]
    pub mdns_enabled: bool,
}

fn default_p2p_port() -> u16 {
    4001
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            listen_port: 4001,
            bootstrap_peers: vec![],
            mdns_enabled: true,
        }
    }
}

/// Настройки хранилища
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Путь к базе данных
    #[serde(default = "default_db_path")]
    pub db_path: String,
    /// Ключ шифрования
    #[serde(default = "default_db_key")]
    pub encryption_key: String,
    /// Максимальный возраст сообщений (дни)
    #[serde(default = "default_max_age")]
    pub max_age_days: i64,
}

fn default_db_path() -> String {
    "./data/messages.db".to_string()
}

fn default_db_key() -> String {
    "change-me-in-production".to_string()
}

fn default_max_age() -> i64 {
    30
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            db_path: default_db_path(),
            encryption_key: default_db_key(),
            max_age_days: 30,
        }
    }
}

impl Config {
    /// Конфигурация по умолчанию
    pub fn default() -> Self {
        Self {
            api_id: 0, // Должен быть установлен пользователем
            api_hash: String::new(),
            encryption: EncryptionConfig::default(),
            proxy: ProxyConfig::default(),
            auto_update: true,
            database_path: None,
            log_level: String::from("info"),
            p2p: P2PConfig::default(),
            storage: StorageConfig::default(),
            stealth_mode: true,
        }
    }

    /// Загрузка конфигурации из файла
    pub fn load() -> Result<Self> {
        let config_path = get_config_path();

        if !config_path.exists() {
            log::warn!("Файл конфигурации не найден: {:?}", config_path);
            log::warn!("Используется конфигурация по умолчанию");
            log::warn!("Создайте файл конфигурации с помощью: secure-tg --init-config");
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Ошибка чтения конфига: {:?}", config_path))?;

        let config: Config = serde_json::from_str(&content)
            .with_context(|| format!("Ошибка парсинга конфига: {:?}", config_path))?;

        log::info!("Конфигурация загружена из {:?}", config_path);

        Ok(config)
    }

    /// Сохранение конфигурации в файл
    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path();

        // Создание директории если не существует
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Ошибка создания директории: {:?}", parent))?;
        }

        let content = serde_json::to_string_pretty(self).context("Ошибка сериализации конфига")?;

        fs::write(&config_path, content)
            .with_context(|| format!("Ошибка записи конфига: {:?}", config_path))?;

        log::info!("Конфигурация сохранена в {:?}", config_path);

        Ok(())
    }

    /// Проверка валидности конфигурации
    pub fn validate(&self) -> Result<()> {
        if self.api_id == 0 {
            return Err(anyhow!(
                "api_id должен быть установлен (получите на https://my.telegram.org)"
            ));
        }

        if self.api_hash.is_empty() {
            return Err(anyhow!("api_hash не может быть пустым"));
        }

        if self.api_hash.len() < 32 {
            return Err(anyhow!(
                "api_hash должен быть не менее 32 символов, текущая длина: {}",
                self.api_hash.len()
            ));
        }

        // Валидация прокси настроек
        if self.proxy.enabled {
            if self.proxy.host.is_empty() {
                return Err(anyhow!(
                    "proxy host не может быть пустым когда прокси включен"
                ));
            }
            if self.proxy.port == 0 {
                return Err(anyhow!("proxy port должен быть больше 0"));
            }
        }

        Ok(())
    }

    /// Проверка необходимости шифрования
    pub fn is_encryption_enabled(&self) -> bool {
        self.encryption.kyber_enabled || self.encryption.obfuscation_enabled
    }

    /// Проверка необходимости стенографии
    pub fn is_steganography_enabled(&self) -> bool {
        self.encryption.steganography_enabled && self.encryption.auto_steganography
    }
}

/// Получение пути к файлу конфигурации
fn get_config_path() -> PathBuf {
    // Проверка переменной окружения
    if let Ok(path) = std::env::var("SECURE_TG_CONFIG") {
        return PathBuf::from(path);
    }

    // Путь по умолчанию в домашней директории
    let mut path = dirs::config_dir()
        .or_else(|| dirs::home_dir())
        .unwrap_or_else(|| PathBuf::from("."));

    path.push(CONFIG_DIR_NAME);
    path.push(CONFIG_FILE_NAME);

    path
}

/// Получение пути к директории конфигурации
pub fn get_config_dir() -> PathBuf {
    let mut path = dirs::config_dir()
        .or_else(|| dirs::home_dir())
        .unwrap_or_else(|| PathBuf::from("."));

    path.push(CONFIG_DIR_NAME);
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

/// Сохранение шаблона конфигурации
pub fn save_config_template() -> Result<PathBuf> {
    let config = create_default_config();
    let config_path = get_config_path();

    if config_path.exists() {
        return Err(anyhow!(
            "Файл конфигурации уже существует: {:?}",
            config_path
        ));
    }

    config.save()?;
    Ok(config_path)
}

// Helper functions for serde defaults
fn default_true() -> bool {
    true
}

fn default_proxy_host() -> String {
    "127.0.0.1".to_string()
}

fn default_proxy_port() -> u16 {
    1080
}

fn default_proxy_type() -> String {
    "socks5".to_string()
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
        assert!(config.auto_update);
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

    #[test]
    fn test_proxy_validation() {
        let mut config = Config::default();
        config.api_id = 123456;
        config.api_hash = "0123456789abcdef0123456789abcdef";

        // Включенный прокси без хоста
        config.proxy.enabled = true;
        config.proxy.host = String::new();
        assert!(config.validate().is_err());

        // Включенный прокси с нулевым портом
        config.proxy.host = "127.0.0.1".to_string();
        config.proxy.port = 0;
        assert!(config.validate().is_err());

        // Валидный прокси
        config.proxy.port = 1080;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_encryption_flags() {
        let config = Config::default();
        assert!(config.is_encryption_enabled());
        assert!(config.is_steganography_enabled());
    }
}
