//! Configuration Module
//! 
//! Конфигурация API ключей и настроек приложения.

use serde::{Deserialize, Serialize};
use std::path::Path;
use anyhow::{Context, Result};

/// Основная конфигурация
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub app_name: String,
    pub version: String,
    pub debug_mode: bool,
    pub database: DatabaseConfig,
    pub api_keys: ApiKeysConfig,
    pub web3: Web3Config,
    pub ai: AIConfig,
}

/// Конфигурация базы данных
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
    pub encryption_key: String,
    pub max_connections: u32,
}

/// API ключи всех сервисов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeysConfig {
    /// Bitget API (покупка криптовалюты)
    pub bitget: Option<BitgetConfig>,
    
    /// 0x Protocol (DEX обмен)
    pub zeroex: Option<ZeroExConfig>,
    
    /// ABCEX (платёжный шлюз)
    pub abcex: Option<ABCEXConfig>,
    
    /// Qwen API (AI функции)
    pub qwen: Option<QwenConfig>,
    
    /// Infura (Web3 provider)
    pub infura: Option<InfuraConfig>,
}

/// Bitget конфигурация
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitgetConfig {
    pub api_key: String,
    pub secret_key: String,
    pub passphrase: String,
    pub testnet: bool,
}

impl BitgetConfig {
    pub fn base_url(&self) -> String {
        if self.testnet {
            "https://testnet.bitget.com".to_string()
        } else {
            "https://api.bitget.com".to_string()
        }
    }
}

/// 0x Protocol конфигурация
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroExConfig {
    pub api_key: Option<String>,
    pub chain_id: u64,
}

impl ZeroExConfig {
    pub fn base_url(&self) -> String {
        format!("https://api.0x.org/swap/v1")
    }
    
    pub fn chain_name(&self) -> String {
        match self.chain_id {
            1 => "ethereum".to_string(),
            137 => "polygon".to_string(),
            56 => "bsc".to_string(),
            42161 => "arbitrum".to_string(),
            _ => "unknown".to_string(),
        }
    }
}

/// ABCEX конфигурация
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABCEXConfig {
    pub api_key: String,
    pub merchant_id: String,
    pub testnet: bool,
}

impl ABCEXConfig {
    pub fn base_url(&self) -> String {
        if self.testnet {
            "https://sandbox.abcex.com".to_string()
        } else {
            "https://api.abcex.com".to_string()
        }
    }
}

/// Qwen API конфигурация
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QwenConfig {
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
}

impl QwenConfig {
    pub fn base_url(&self) -> String {
        "https://dashscope.aliyuncs.com/api/v1".to_string()
    }
}

/// Infura конфигурация
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfuraConfig {
    pub project_id: String,
    pub project_secret: Option<String>,
}

impl InfuraConfig {
    pub fn ethereum_url(&self) -> String {
        if let Some(secret) = &self.project_secret {
            format!("https://{}:{}@mainnet.infura.io/v3/{}", 
                self.project_id, secret, self.project_id)
        } else {
            format!("https://mainnet.infura.io/v3/{}", self.project_id)
        }
    }
    
    pub fn polygon_url(&self) -> String {
        format!("https://polygon-mainnet.infura.io/v3/{}", self.project_id)
    }
}

/// AI конфигурация
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub enabled: bool,
    pub provider: String,
    pub default_language: String,
    pub auto_translate: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app_name: "Liberty Reach Messenger".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            debug_mode: true,
            database: DatabaseConfig::default(),
            api_keys: ApiKeysConfig::default(),
            web3: Web3Config::default(),
            ai: AIConfig::default(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: "liberty_reach.db".to_string(),
            encryption_key: String::new(),
            max_connections: 10,
        }
    }
}

impl Default for ApiKeysConfig {
    fn default() -> Self {
        Self {
            bitget: None,
            zeroex: None,
            abcex: None,
            qwen: None,
            infura: None,
        }
    }
}

impl Default for Web3Config {
    fn default() -> Self {
        Self {
            default_chain_id: 1,
            gas_price_multiplier: 1.2,
        }
    }
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            provider: "qwen".to_string(),
            default_language: "en".to_string(),
            auto_translate: false,
        }
    }
}

/// Web3 конфигурация
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3Config {
    pub default_chain_id: u64,
    pub gas_price_multiplier: f64,
}

/// Загрузить конфигурацию из файла
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<AppConfig> {
    let content = std::fs::read_to_string(path.as_ref())
        .context("Failed to read config file")?;
    
    let config: AppConfig = toml::from_str(&content)
        .context("Failed to parse config file")?;
    
    Ok(config)
}

/// Сохранить конфигурацию в файл
pub fn save_config<P: AsRef<Path>>(config: &AppConfig, path: P) -> Result<()> {
    let content = toml::to_string_pretty(config)
        .context("Failed to serialize config")?;
    
    std::fs::write(path.as_ref(), content)
        .context("Failed to write config file")?;
    
    Ok(())
}

/// Создать шаблон конфигурации
pub fn create_config_template() -> String {
    r#"# Liberty Reach Messenger Configuration
app_name = "Liberty Reach Messenger"
version = "2.0.0"
debug_mode = true

[database]
path = "liberty_reach.db"
encryption_key = "YOUR_ENCRYPTION_KEY_HERE"
max_connections = 10

[api_keys.bitget]
api_key = "YOUR_BITGET_API_KEY"
secret_key = "YOUR_BITGET_SECRET_KEY"
passphrase = "YOUR_BITGET_PASSPHRASE"
testnet = true

[api_keys.zeroex]
api_key = "YOUR_0X_API_KEY"
chain_id = 1

[api_keys.abcex]
api_key = "YOUR_ABCEX_API_KEY"
merchant_id = "YOUR_MERCHANT_ID"
testnet = true

[api_keys.qwen]
api_key = "YOUR_QWEN_API_KEY"
model = "qwen-max"
max_tokens = 2000

[api_keys.infura]
project_id = "YOUR_INFURA_PROJECT_ID"
project_secret = "YOUR_INFURA_PROJECT_SECRET"

[web3]
default_chain_id = 1
gas_price_multiplier = 1.2

[ai]
enabled = true
provider = "qwen"
default_language = "en"
auto_translate = false
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_template() {
        let template = create_config_template();
        assert!(template.contains("YOUR_BITGET_API_KEY"));
        assert!(template.contains("YOUR_QWEN_API_KEY"));
    }
}
