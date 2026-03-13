//! Модуль биржевого агрегатора
//!
//! Интеграция с:
//! - Bitget API
//! - Bybit API
//!
//! Fee Layer: 0.5% комиссия на кошелек администратора

use anyhow::{Result, Context};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use chrono::{Utc, Duration as ChronoDuration};

type HmacSha256 = Hmac<Sha256>;

/// Конфигурация бирж
pub struct ExchangeConfig {
    pub bitget_api_key: String,
    pub bitget_secret: String,
    pub bitget_passphrase: String,
    pub bybit_api_key: String,
    pub bybit_secret: String,
    pub admin_wallet: String, // Кошелек для комиссии (Polygon)
    pub fee_percent: f64,     // Комиссия (0.005 = 0.5%)
}

impl ExchangeConfig {
    pub fn new() -> Self {
        Self {
            bitget_api_key: std::env::var("BITGET_API_KEY").unwrap_or_default(),
            bitget_secret: std::env::var("BITGET_SECRET").unwrap_or_default(),
            bitget_passphrase: std::env::var("BITGET_PASSPHRASE").unwrap_or_default(),
            bybit_api_key: std::env::var("BYBIT_API_KEY").unwrap_or_default(),
            bybit_secret: std::env::var("BYBIT_SECRET").unwrap_or_default(),
            admin_wallet: std::env::var("ADMIN_WALLET").unwrap_or_default(),
            fee_percent: 0.005, // 0.5%
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.bitget_api_key.is_empty() || !self.bybit_api_key.is_empty()
    }
}

impl Default for ExchangeConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Торговая пара
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingPair {
    pub symbol: String,
    pub base: String,
    pub quote: String,
    pub price: f64,
    pub volume_24h: f64,
    pub change_24h: f64,
}

/// Ордер на покупку/продажу
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeOrder {
    pub pair: String,
    pub side: TradeSide,
    pub amount: f64,
    pub price: f64,
    pub fee: f64,
    pub total: f64,
    pub timestamp: u64,
    pub status: OrderStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    Pending,
    Filled,
    Cancelled,
    Failed,
}

/// Менеджер бирж
pub struct ExchangeManager {
    client: Client,
    config: ExchangeConfig,
    /// Кэш цен (symbol -> price, timestamp)
    price_cache: HashMap<String, (f64, u64)>,
}

impl ExchangeManager {
    pub fn new(config: ExchangeConfig) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            config,
            price_cache: HashMap::new(),
        }
    }

    /// Получение цены с Bitget
    pub async fn get_bitget_price(&self, symbol: &str) -> Result<f64> {
        let url = format!(
            "https://api.bitget.com/api/spot/v1/market/ticker?symbol={}",
            symbol
        );

        let response = self.client.get(&url).send().await
            .context("Ошибка запроса к Bitget")?;

        let result: serde_json::Value = response.json().await
            .context("Ошибка парсинга ответа Bitget")?;

        let price_str = result["data"]["close"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Нет цены в ответе Bitget"))?;

        price_str.parse::<f64>()
            .context("Ошибка парсинга цены")
    }

    /// Получение цены с Bybit
    pub async fn get_bybit_price(&self, symbol: &str) -> Result<f64> {
        let url = format!(
            "https://api.bybit.com/v5/market/tickers?category=spot&symbol={}",
            symbol
        );

        let response = self.client.get(&url).send().await
            .context("Ошибка запроса к Bybit")?;

        let result: serde_json::Value = response.json().await
            .context("Ошибка парсинга ответа Bybit")?;

        if let Some(list) = result["result"]["list"].as_array() {
            if let Some(first) = list.first() {
                let price_str = first["lastPrice"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Нет цены в ответе Bybit"))?;
                return price_str.parse::<f64>()
                    .context("Ошибка парсинга цены");
            }
        }

        anyhow::bail!("Нет данных о цене в ответе Bybit")
    }

    /// Получение лучшей цены (агрегация)
    pub async fn get_best_price(&mut self, symbol: &str) -> Result<f64> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Проверка кэша (валидность 10 секунд)
        if let Some((price, timestamp)) = self.price_cache.get(symbol) {
            if now - timestamp < 10 {
                return Ok(*price);
            }
        }

        // Запрос с обеих бирж
        let bitget_price = self.get_bitget_price(symbol).await.ok();
        let bybit_price = self.get_bybit_price(symbol).await.ok();

        let best_price = match (bitget_price, bybit_price) {
            (Some(bg), Some(by)) => bg.min(by),
            (Some(bg), None) => bg,
            (None, Some(by)) => by,
            (None, None) => anyhow::bail!("Не удалось получить цену ни с одной биржи"),
        };

        // Обновление кэша
        self.price_cache.insert(symbol.to_string(), (best_price, now));

        Ok(best_price)
    }

    /// Расчет комиссии
    pub fn calculate_fee(&self, amount: f64) -> f64 {
        amount * self.config.fee_percent
    }

    /// Создание ордера с учетом комиссии
    pub async fn create_order(
        &mut self,
        pair: &str,
        side: TradeSide,
        amount: f64,
    ) -> Result<TradeOrder> {
        let price = self.get_best_price(pair).await?;
        let total = amount * price;
        let fee = self.calculate_fee(total);

        Ok(TradeOrder {
            pair: pair.to_string(),
            side,
            amount,
            price,
            fee,
            total: total + fee, // Включаем комиссию
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            status: OrderStatus::Pending,
        })
    }

    /// Подпись запроса для Bitget
    fn sign_bitget(&self, timestamp: &str, method: &str, path: &str, body: &str) -> String {
        let message = format!("{}{}{}{}", timestamp, method, path, body);
        
        let mut mac = HmacSha256::new_from_slice(self.config.bitget_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(message.as_bytes());
        
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    /// Подпись запроса для Bybit
    fn sign_bybit(&self, timestamp: u64, params: &str) -> String {
        let message = format!("{}{}{}", self.config.bybit_api_key, timestamp, params);
        
        let mut mac = HmacSha256::new_from_slice(self.config.bybit_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(message.as_bytes());
        
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    /// Получение баланса (заглушка - требует полной реализации)
    pub async fn get_balance(&self, _currency: &str) -> Result<f64> {
        // Требует авторизации и полной реализации API
        Ok(0.0)
    }

    /// Информация о кошельке администратора для комиссии
    pub fn get_admin_wallet(&self) -> &str {
        &self.config.admin_wallet
    }

    /// Размер комиссии
    pub fn get_fee_percent(&self) -> f64 {
        self.config.fee_percent * 100.0
    }
}

/// История торговли пользователя
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeHistory {
    pub peer_id: String,
    pub orders: Vec<TradeOrder>,
    pub total_volume: f64,
    pub total_fees: f64,
}

impl TradeHistory {
    pub fn new(peer_id: &str) -> Self {
        Self {
            peer_id: peer_id.to_string(),
            orders: Vec::new(),
            total_volume: 0.0,
            total_fees: 0.0,
        }
    }

    pub fn add_order(&mut self, order: TradeOrder) {
        self.total_volume += order.total;
        self.total_fees += order.fee;
        self.orders.push(order);
    }
}

/// Команды для торговли
pub const TRADE_COMMANDS: &[(&str, &str)] = &[
    ("trade [pair] [amount]", "Купить/продать пару (например: BTCUSDT)"),
    ("price [pair]", "Показать текущую цену"),
    ("balance", "Показать баланс"),
    ("history", "История торговли"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fee_calculation() {
        let config = ExchangeConfig::new();
        let manager = ExchangeManager::new(config);
        
        let amount = 1000.0;
        let fee = manager.calculate_fee(amount);
        
        assert!((fee - 5.0).abs() < 0.01); // 0.5% от 1000 = 5
    }

    #[test]
    fn test_trade_side_serialization() {
        let buy = TradeSide::Buy;
        let json = serde_json::to_string(&buy).unwrap();
        assert_eq!(json, "\"Buy\"");
    }
}
