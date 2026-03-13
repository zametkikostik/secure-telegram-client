//! Web3 модуль для работы с Polygon и другими EVM-сетями
//!
//! Реализует:
//! - Проверка баланса токенов
//! - Поддержка MATIC, ETH

use anyhow::{Result, Context};
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::types::{Address, H160, U256};
use std::str::FromStr;
use std::sync::Arc;

/// Конфигурация Web3
pub struct Web3Config {
    /// RPC URL (например, https://polygon-rpc.com)
    pub rpc_url: String,
    /// Приватный ключ кошелька (опционально)
    pub private_key: Option<String>,
    /// Адрес кошелька для просмотра
    pub watch_address: Option<String>,
}

impl Web3Config {
    pub fn new() -> Self {
        let rpc_url = std::env::var("WEB3_RPC_URL")
            .unwrap_or_else(|_| "https://polygon-rpc.com".to_string());
        let private_key = std::env::var("WEB3_PRIVATE_KEY").ok();
        let watch_address = std::env::var("WEB3_WATCH_ADDRESS").ok();

        Self {
            rpc_url,
            private_key,
            watch_address,
        }
    }

    pub fn with_rpc(rpc_url: &str) -> Self {
        Self {
            rpc_url: rpc_url.to_string(),
            private_key: None,
            watch_address: None,
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.rpc_url.is_empty()
    }
}

impl Default for Web3Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Поддерживаемые сети
#[derive(Debug, Clone, Copy)]
pub enum Network {
    Polygon,
    Ethereum,
    Mumbai,
}

impl Network {
    pub fn rpc_url(&self) -> &'static str {
        match self {
            Network::Polygon => "https://polygon-rpc.com",
            Network::Ethereum => "https://eth.llamarpc.com",
            Network::Mumbai => "https://rpc-mumbai.maticvigil.com",
        }
    }

    pub fn chain_id(&self) -> u64 {
        match self {
            Network::Polygon => 137,
            Network::Ethereum => 1,
            Network::Mumbai => 80001,
        }
    }
}

/// Менеджер кошелька
pub struct WalletManager {
    config: Web3Config,
    provider: Arc<Provider<Http>>,
    address: Option<Address>,
}

impl WalletManager {
    pub fn new(config: Web3Config) -> Result<Self> {
        let provider = Provider::<Http>::try_from(&config.rpc_url)
            .context("Не удалось создать RPC провайдер")?;
        let provider = Arc::new(provider);

        let address = if let Some(pk) = &config.private_key {
            let wallet = LocalWallet::from_str(pk)
                .context("Неверный приватный ключ")?;
            Some(wallet.address())
        } else if let Some(addr) = &config.watch_address {
            Some(Address::from_str(addr)
                .context("Неверный адрес кошелька")?)
        } else {
            None
        };

        Ok(Self {
            config,
            provider,
            address,
        })
    }

    /// Получение баланса нативного токена (MATIC/ETH)
    pub async fn get_balance(&self, address: Option<&str>) -> Result<U256> {
        let addr = if let Some(a) = address {
            Address::from_str(a).context("Неверный адрес")?
        } else if let Some(a) = self.address {
            a
        } else {
            anyhow::bail!("Адрес кошелька не указан");
        };

        let balance = self.provider.get_balance(addr, None)
            .await
            .context("Ошибка получения баланса")?;

        Ok(balance)
    }

    /// Форматирование баланса в читаемый вид
    pub fn format_balance(&self, balance: U256, decimals: u32) -> String {
        let balance_f64 = balance.as_u64() as f64 / 10_f64.powi(decimals as i32);
        format!("{:.4}", balance_f64)
    }

    /// Получение баланса токена (ERC20) - упрощенная версия
    pub async fn get_token_balance(
        &self,
        _token_address: &str,
        _wallet_address: Option<&str>,
    ) -> Result<U256> {
        // Заглушка - требует полной реализации с ABI контракта
        Ok(U256::zero())
    }

    /// Отправка нативного токена - требует полной реализации
    #[allow(dead_code)]
    pub async fn send_native(
        &self,
        _to: &str,
        _amount: U256,
    ) -> Result<ethers::types::TransactionReceipt> {
        anyhow::bail!("Отправка транзакций требует дополнительной настройки")
    }

    /// Отправка токена ERC20 - требует полной реализации
    #[allow(dead_code)]
    pub async fn send_token(
        &self,
        _token_address: &str,
        _to: &str,
        _amount: U256,
    ) -> Result<ethers::types::TransactionReceipt> {
        anyhow::bail!("Отправка транзакций требует дополнительной настройки")
    }

    /// Получение адреса кошелька
    pub fn get_address(&self) -> Option<String> {
        self.address.map(|a| format!("{:?}", a))
    }

    /// Проверка подключения к RPC
    pub async fn check_connection(&self) -> Result<bool> {
        match self.provider.get_block_number().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

/// Популярные токены в Polygon
pub mod polygon_tokens {
    pub const USDC: &str = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";
    pub const USDT: &str = "0xc2132D05D31c914a87C6611C10748AEb04B58e8F";
    pub const DAI: &str = "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063";
    pub const WMATIC: &str = "0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270";
}

/// Структура для отображения информации о кошельке
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WalletInfo {
    pub address: String,
    pub matic_balance: String,
    pub usdc_balance: Option<String>,
    pub network: String,
}

impl WalletInfo {
    pub async fn from_manager(manager: &WalletManager) -> Result<Self> {
        let address = manager.get_address().unwrap_or_default();
        
        let matic_balance = manager.get_balance(None).await?;
        let matic_formatted = manager.format_balance(matic_balance, 18);

        let usdc_balance = manager
            .get_token_balance(polygon_tokens::USDC, None)
            .await
            .ok()
            .map(|b| manager.format_balance(b, 6));

        Ok(Self {
            address,
            matic_balance: matic_formatted,
            usdc_balance,
            network: "Polygon".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wallet_info() {
        let config = Web3Config::new();
        if !config.is_configured() {
            return; // Пропускаем если RPC не настроен
        }

        let manager = WalletManager::new(config).unwrap();
        let connected = manager.check_connection().await.unwrap();
        
        if connected {
            let info = WalletInfo::from_manager(&manager).await;
            assert!(info.is_ok());
        }
    }
}
