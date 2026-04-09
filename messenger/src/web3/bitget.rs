//! Bitget Exchange Integration — покупка криптовалюты через Bitget API
//!
//! Документация: https://bitgetlimited.github.io/apidoc/en/spot/
//!
//! Features:
//! - Покупка криптовалюты за фиат (USD, EUR)
//! - Spot trading (limit/market orders)
//! - Комиссия 2-3% (настраиваемая)
//! - Account balance tracking
//! - Order management
//!
//! Комиссии Bitget:
//! - Maker: 0.1%
//! - Taker: 0.2%
//! - Наша комиссия: 2-3% (поверх комиссий Bitget)

use super::types::*;
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

// ============================================================================
// Bitget API Configuration
// ============================================================================

const BITGET_API_BASE: &str = "https://api.bitget.com";
const BITGET_FEE_BPS: u64 = 250; // 2.5% комиссия по умолчанию

/// Order types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
}

impl OrderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderType::Market => "market",
            OrderType::Limit => "limit",
        }
    }
}

/// Side (buy/sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    pub fn as_str(&self) -> &'static str {
        match self {
            OrderSide::Buy => "buy",
            OrderSide::Sell => "sell",
        }
    }
}

/// Order status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BitgetOrderStatus {
    Init,
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Failed,
}

// ============================================================================
// Bitget API Request/Response Types
// ============================================================================

/// Запрос на покупку
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyRequest {
    /// Trading pair (BTCUSDT, ETHUSDT, etc)
    pub symbol: String,
    /// Order side (buy/sell)
    pub side: OrderSide,
    /// Order type (market/limit)
    pub order_type: OrderType,
    /// Amount in crypto (for limit orders)
    pub amount: Option<String>,
    /// Amount in fiat/quote currency
    pub quote_amount: Option<String>,
    /// Price (for limit orders)
    pub price: Option<String>,
    /// Client order ID
    pub client_oid: Option<String>,
}

/// Ответ на покупку
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyResponse {
    /// Order ID
    pub order_id: String,
    /// Client order ID
    pub client_oid: Option<String>,
    /// Symbol
    pub symbol: String,
    /// Side
    pub side: String,
    /// Order type
    pub order_type: String,
    /// Amount
    pub amount: String,
    /// Price
    pub price: String,
    /// Filled amount
    pub filled_amount: String,
    /// Average price
    pub average_price: String,
    /// Status
    pub status: String,
    /// Fee amount
    pub fee: String,
    /// Fee currency
    pub fee_currency: String,
    /// Created at (timestamp ms)
    pub created_at: u64,
    /// Updated at (timestamp ms)
    pub updated_at: u64,
}

/// Информация об аккаунте
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    /// Available balance
    pub available_balance: String,
    /// Frozen balance
    pub frozen_balance: String,
    /// Total balance
    pub total_balance: String,
    /// Currency
    pub currency: String,
}

/// Market price
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPrice {
    pub symbol: String,
    pub price: String,
    pub high_24h: String,
    pub low_24h: String,
    pub volume_24h: String,
    pub change_24h: String,
    pub timestamp: u64,
}

// ============================================================================
// Bitget Client
// ============================================================================

/// Клиент для работы с Bitget Exchange
pub struct BitgetClient {
    http_client: Client,
    api_key: Option<String>,
    secret_key: Option<String>,
    passphrase: Option<String>,
    fee_bps: u64,
}

impl BitgetClient {
    /// Создать новый Bitget клиент
    pub fn new(
        api_key: Option<String>,
        secret_key: Option<String>,
        passphrase: Option<String>,
    ) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            api_key,
            secret_key,
            passphrase,
            fee_bps: BITGET_FEE_BPS, // 2.5% по умолчанию
        }
    }

    /// Создать с кастомной комиссией
    pub fn with_fee(
        api_key: Option<String>,
        secret_key: Option<String>,
        passphrase: Option<String>,
        fee_bps: u64,
    ) -> Self {
        // Валидация комиссии (2-3%)
        assert!(
            fee_bps >= 200 && fee_bps <= 300,
            "Fee must be between 2% (200bps) and 3% (300bps)"
        );

        Self::new(api_key, secret_key, passphrase).with_fee_config(fee_bps)
    }

    /// Установить конфигурацию комиссии
    pub fn with_fee_config(mut self, fee_bps: u64) -> Self {
        self.fee_bps = fee_bps;
        self
    }

    /// Получить заголовки авторизации
    fn auth_headers(
        &self,
        method: Method,
        request_path: &str,
        body: &str,
    ) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        if let (Some(api_key), Some(secret_key), Some(passphrase)) =
            (&self.api_key, &self.secret_key, &self.passphrase)
        {
            // Bitget требует подписи запросов
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
                .to_string();

            // Signature: timestamp + method + requestPath + body
            let sign_str = format!("{}{}{}{}", timestamp, method.as_str(), request_path, body);
            let signature = hmac_sha256(secret_key, &sign_str);

            headers.insert("ACCESS-KEY".to_string(), api_key.clone());
            headers.insert("ACCESS-SIGN".to_string(), signature);
            headers.insert("ACCESS-TIMESTAMP".to_string(), timestamp);
            headers.insert("ACCESS-PASSPHRASE".to_string(), passphrase.clone());
            headers.insert("Content-Type".to_string(), "application/json".to_string());
            headers.insert("locale".to_string(), "en-US".to_string());
        }

        headers
    }

    // ========================================================================
    // Spot Trading API
    // ========================================================================

    /// Разместить ордер на покупку
    pub async fn place_buy_order(&self, request: BuyRequest) -> Web3Result<BuyResponse> {
        let url = format!("{}/api/spot/v1/trade", BITGET_API_BASE);
        let path = "/api/spot/v1/trade";

        let order_body = serde_json::json!({
            "symbol": request.symbol,
            "side": request.side.as_str(),
            "orderType": request.order_type.as_str(),
            "amount": request.amount.unwrap_or_default(),
            "quantity": request.quote_amount.unwrap_or_default(),
            "price": request.price.unwrap_or_default(),
            "clientOid": request.client_oid.unwrap_or_else(|| {
                format!("order_{}", SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis())
            }),
        });

        let body_str = order_body.to_string();
        let auth_headers = self.auth_headers(Method::POST, path, &body_str);

        let mut req = self.http_client.post(&url).json(&order_body);

        for (key, value) in auth_headers {
            req = req.header(&key, &value);
        }

        let response = req
            .send()
            .await
            .map_err(|e| Web3Error::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("Bitget order error: {}", error_text);
            return Err(Web3Error::Network(format!(
                "Bitget order error: {}",
                error_text
            )));
        }

        let order_response: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Web3Error::Network(format!("Failed to parse response: {}", e)))?;

        // Проверить на ошибки в ответе
        if let Some(code) = order_response["code"].as_str() {
            if code != "00000" {
                let msg = order_response["msg"].as_str().unwrap_or("Unknown error");
                return Err(Web3Error::Network(format!("Bitget API error: {}", msg)));
            }
        }

        let data = &order_response["data"];
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            * 1000;

        let buy_response = BuyResponse {
            order_id: data["orderId"].as_str().unwrap_or("").to_string(),
            client_oid: data["clientOid"].as_str().map(String::from),
            symbol: data["symbol"].as_str().unwrap_or("").to_string(),
            side: data["side"].as_str().unwrap_or("").to_string(),
            order_type: data["orderType"].as_str().unwrap_or("").to_string(),
            amount: data["amount"].as_str().unwrap_or("").to_string(),
            price: data["price"].as_str().unwrap_or("").to_string(),
            filled_amount: data["filledAmount"].as_str().unwrap_or("").to_string(),
            average_price: data["averagePrice"].as_str().unwrap_or("").to_string(),
            status: data["status"].as_str().unwrap_or("").to_string(),
            fee: data["fee"].as_str().unwrap_or("").to_string(),
            fee_currency: data["feeCcy"].as_str().unwrap_or("").to_string(),
            created_at: now,
            updated_at: now,
        };

        info!(
            "Order placed: {} {} (id: {}, status: {})",
            buy_response.amount, buy_response.symbol, buy_response.order_id, buy_response.status
        );

        Ok(buy_response)
    }

    /// Получить статус ордера
    pub async fn get_order_status(&self, symbol: &str, order_id: &str) -> Web3Result<BuyResponse> {
        let url = format!("{}/api/spot/v1/trade/orderInfo", BITGET_API_BASE);

        let query = serde_json::json!({
            "symbol": symbol,
            "orderId": order_id,
        });

        let mut req = self.http_client.post(&url).json(&query);

        // Для GET запросов Bitget использует POST для получения информации
        let body_str = query.to_string();
        let auth_headers =
            self.auth_headers(Method::POST, "/api/spot/v1/trade/orderInfo", &body_str);

        for (key, value) in auth_headers {
            req = req.header(&key, &value);
        }

        let response = req
            .send()
            .await
            .map_err(|e| Web3Error::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Web3Error::Network(format!(
                "Bitget API error: {}",
                error_text
            )));
        }

        let order_data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Web3Error::Network(format!("Failed to parse response: {}", e)))?;

        // Parse order info...
        let data = &order_data["data"];
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            * 1000;

        Ok(BuyResponse {
            order_id: data["orderId"].as_str().unwrap_or("").to_string(),
            client_oid: data["clientOid"].as_str().map(String::from),
            symbol: data["symbol"].as_str().unwrap_or("").to_string(),
            side: data["side"].as_str().unwrap_or("").to_string(),
            order_type: data["orderType"].as_str().unwrap_or("").to_string(),
            amount: data["amount"].as_str().unwrap_or("").to_string(),
            price: data["price"].as_str().unwrap_or("").to_string(),
            filled_amount: data["filledAmount"].as_str().unwrap_or("").to_string(),
            average_price: data["averagePrice"].as_str().unwrap_or("").to_string(),
            status: data["status"].as_str().unwrap_or("").to_string(),
            fee: data["fee"].as_str().unwrap_or("").to_string(),
            fee_currency: data["feeCcy"].as_str().unwrap_or("").to_string(),
            created_at: now,
            updated_at: now,
        })
    }

    /// Отменить ордер
    pub async fn cancel_order(&self, symbol: &str, order_id: &str) -> Web3Result<()> {
        let url = format!("{}/api/spot/v1/trade/cancel-order", BITGET_API_BASE);
        let path = "/api/spot/v1/trade/cancel-order";

        let cancel_body = serde_json::json!({
            "symbol": symbol,
            "orderId": order_id,
        });

        let body_str = cancel_body.to_string();
        let auth_headers = self.auth_headers(Method::POST, path, &body_str);

        let mut req = self.http_client.post(&url).json(&cancel_body);

        for (key, value) in auth_headers {
            req = req.header(&key, &value);
        }

        let response = req
            .send()
            .await
            .map_err(|e| Web3Error::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Web3Error::Network(format!(
                "Bitget cancel error: {}",
                error_text
            )));
        }

        Ok(())
    }

    // ========================================================================
    // Account API
    // ========================================================================

    /// Получить баланс аккаунта
    pub async fn get_account_balance(&self, currency: &str) -> Web3Result<AccountInfo> {
        let url = format!("{}/api/spot/v1/account/getInfo", BITGET_API_BASE);

        let mut req = self.http_client.get(&url);

        let auth_headers = self.auth_headers(Method::GET, "/api/spot/v1/account/getInfo", "");
        for (key, value) in auth_headers {
            req = req.header(&key, &value);
        }

        let response = req
            .send()
            .await
            .map_err(|e| Web3Error::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Web3Error::Network(format!(
                "Bitget API error: {}",
                error_text
            )));
        }

        let account_data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Web3Error::Network(format!("Failed to parse response: {}", e)))?;

        let data = &account_data["data"];

        Ok(AccountInfo {
            available_balance: data["available"].as_str().unwrap_or("0").to_string(),
            frozen_balance: data["frozen"].as_str().unwrap_or("0").to_string(),
            total_balance: data["total"].as_str().unwrap_or("0").to_string(),
            currency: currency.to_string(),
        })
    }

    // ========================================================================
    // Market Data API
    // ========================================================================

    /// Получить текущую цену рынка
    pub async fn get_market_price(&self, symbol: &str) -> Web3Result<MarketPrice> {
        let url = format!("{}/api/market/v1/ticker", BITGET_API_BASE);

        let response = self
            .http_client
            .get(&url)
            .query(&[("symbol", symbol)])
            .send()
            .await
            .map_err(|e| Web3Error::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Web3Error::Network(format!(
                "Bitget API error: {}",
                error_text
            )));
        }

        let market_data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Web3Error::Network(format!("Failed to parse response: {}", e)))?;

        let data = &market_data["data"];
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(MarketPrice {
            symbol: symbol.to_string(),
            price: data["close"].as_str().unwrap_or("0").to_string(),
            high_24h: data["high24h"].as_str().unwrap_or("0").to_string(),
            low_24h: data["low24h"].as_str().unwrap_or("0").to_string(),
            volume_24h: data["volume24h"].as_str().unwrap_or("0").to_string(),
            change_24h: data["change24h"].as_str().unwrap_or("0").to_string(),
            timestamp,
        })
    }

    /// Получить список торговых пар
    pub async fn get_symbols(&self) -> Web3Result<Vec<String>> {
        let url = format!("{}/api/spot/v1/public/symbols", BITGET_API_BASE);

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| Web3Error::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Web3Error::Network(format!(
                "Bitget API error: {}",
                error_text
            )));
        }

        let symbols_data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Web3Error::Network(format!("Failed to parse response: {}", e)))?;

        let symbols: Vec<String> = symbols_data["data"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|s| s["symbolName"].as_str().map(String::from))
            .collect();

        Ok(symbols)
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    /// Полный процесс покупки (market order)
    pub async fn execute_buy(
        &self,
        symbol: String,
        quote_amount: String, // Amount in USDT
    ) -> Web3Result<BuyResponse> {
        info!("Starting market buy: {} {}", quote_amount, symbol);

        let request = BuyRequest {
            symbol: symbol.clone(),
            side: OrderSide::Buy,
            order_type: OrderType::Market,
            amount: None,
            quote_amount: Some(quote_amount.clone()),
            price: None,
            client_oid: None,
        };

        let order = self.place_buy_order(request).await?;

        info!(
            "Order placed: {} {} (id: {})",
            order.amount, symbol, order.order_id
        );

        Ok(order)
    }

    /// Полный процесс продажи
    pub async fn execute_sell(
        &self,
        symbol: String,
        amount: String, // Amount in crypto
    ) -> Web3Result<BuyResponse> {
        info!("Starting market sell: {} {}", amount, symbol);

        let request = BuyRequest {
            symbol: symbol.clone(),
            side: OrderSide::Sell,
            order_type: OrderType::Market,
            amount: Some(amount.clone()),
            quote_amount: None,
            price: None,
            client_oid: None,
        };

        let order = self.place_buy_order(request).await?;

        info!(
            "Order placed: {} {} (id: {})",
            order.amount, symbol, order.order_id
        );

        Ok(order)
    }
}

// ============================================================================
// Builder Pattern для Buy Request
// ============================================================================

/// Builder для создания запроса покупки
pub struct BuyRequestBuilder {
    request: BuyRequest,
}

impl BuyRequestBuilder {
    pub fn new(symbol: impl Into<String>) -> Self {
        Self {
            request: BuyRequest {
                symbol: symbol.into(),
                side: OrderSide::Buy,
                order_type: OrderType::Market,
                amount: None,
                quote_amount: None,
                price: None,
                client_oid: None,
            },
        }
    }

    pub fn market_buy(mut self, quote_amount: impl Into<String>) -> Self {
        self.request.order_type = OrderType::Market;
        self.request.quote_amount = Some(quote_amount.into());
        self
    }

    pub fn limit_buy(mut self, amount: impl Into<String>, price: impl Into<String>) -> Self {
        self.request.order_type = OrderType::Limit;
        self.request.amount = Some(amount.into());
        self.request.price = Some(price.into());
        self
    }

    pub fn market_sell(mut self, amount: impl Into<String>) -> Self {
        self.request.side = OrderSide::Sell;
        self.request.order_type = OrderType::Market;
        self.request.amount = Some(amount.into());
        self
    }

    pub fn client_order_id(mut self, id: impl Into<String>) -> Self {
        self.request.client_oid = Some(id.into());
        self
    }

    pub fn build(self) -> BuyRequest {
        self.request
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// HMAC-SHA256 signature
fn hmac_sha256(key: &str, message: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    let mut mac =
        HmacSha256::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");
    mac.update(message.as_bytes());

    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

/// Рассчитать комиссию для суммы
pub fn calculate_fee(amount: &str, fee_bps: u64) -> Web3Result<String> {
    let amount_f64: f64 = amount
        .parse()
        .map_err(|_| Web3Error::Wallet(format!("Invalid amount: {}", amount)))?;

    let fee = amount_f64 * (fee_bps as f64) / 10000.0;
    Ok(format!("{:.4}", fee))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buy_request_builder_market_buy() {
        let request = BuyRequestBuilder::new("BTCUSDT").market_buy("100").build();

        assert_eq!(request.symbol, "BTCUSDT");
        assert_eq!(request.side, OrderSide::Buy);
        assert_eq!(request.order_type, OrderType::Market);
        assert_eq!(request.quote_amount, Some("100".to_string()));
    }

    #[test]
    fn test_buy_request_builder_limit_buy() {
        let request = BuyRequestBuilder::new("ETHUSDT")
            .limit_buy("0.1", "2000")
            .build();

        assert_eq!(request.symbol, "ETHUSDT");
        assert_eq!(request.side, OrderSide::Buy);
        assert_eq!(request.order_type, OrderType::Limit);
        assert_eq!(request.amount, Some("0.1".to_string()));
        assert_eq!(request.price, Some("2000".to_string()));
    }

    #[test]
    fn test_buy_request_builder_market_sell() {
        let request = BuyRequestBuilder::new("BTCUSDT")
            .market_sell("0.01")
            .build();

        assert_eq!(request.symbol, "BTCUSDT");
        assert_eq!(request.side, OrderSide::Sell);
        assert_eq!(request.order_type, OrderType::Market);
        assert_eq!(request.amount, Some("0.01".to_string()));
    }

    #[test]
    fn test_order_type_as_str() {
        assert_eq!(OrderType::Market.as_str(), "market");
        assert_eq!(OrderType::Limit.as_str(), "limit");
    }

    #[test]
    fn test_order_side_as_str() {
        assert_eq!(OrderSide::Buy.as_str(), "buy");
        assert_eq!(OrderSide::Sell.as_str(), "sell");
    }

    #[test]
    fn test_calculate_fee() {
        // 2.5% комиссия от $100
        let fee = calculate_fee("100", 250).unwrap();
        assert_eq!(fee, "2.5000");

        // 2% комиссия от $500
        let fee = calculate_fee("500", 200).unwrap();
        assert_eq!(fee, "10.0000");

        // 3% комиссия от $1000
        let fee = calculate_fee("1000", 300).unwrap();
        assert_eq!(fee, "30.0000");
    }

    #[test]
    fn test_bitget_client_creation() {
        let client = BitgetClient::new(None, None, None);
        assert!(client.api_key.is_none());
        assert!(client.secret_key.is_none());
        assert_eq!(client.fee_bps, 250); // 2.5% по умолчанию
    }

    #[test]
    fn test_bitget_client_with_fee() {
        let client = BitgetClient::with_fee(None, None, None, 200);
        assert_eq!(client.fee_bps, 200); // 2%
    }

    #[test]
    #[should_panic(expected = "Fee must be between")]
    fn test_bitget_client_invalid_fee_too_low() {
        BitgetClient::with_fee(None, None, None, 100); // 1% - слишком мало
    }

    #[test]
    #[should_panic(expected = "Fee must be between")]
    fn test_bitget_client_invalid_fee_too_high() {
        BitgetClient::with_fee(None, None, None, 400); // 4% - слишком много
    }

    #[test]
    fn test_hmac_sha256() {
        let signature = hmac_sha256("secret_key", "test_message");
        assert!(!signature.is_empty());
        assert_eq!(signature.len(), 64); // 32 bytes = 64 hex chars
    }
}
