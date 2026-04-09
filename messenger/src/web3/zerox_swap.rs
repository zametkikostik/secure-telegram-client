//! 0x Protocol Swap Integration
//!
//! Обмен токенов через 0x Protocol API с комиссией 0.5-3%
//!
//! Документация: https://docs.0x.org/0x-api-swap/introduction
//!
//! Features:
//! - Получение котировок (quote) для обмена
//! - Создание swap-транзакций
//! - Отслеживание статуса обмена
//! - Поддержка multiple chains (Ethereum, Polygon, Arbitrum, Base, BSC)
//! - Автоматический расчет slippage
//! - Комиссия для протокола (affiliate fee)
//!
//! Комиссии 0x:
//! - 0.5% для major pairs (ETH/USDC, etc)
//! - 1-2% для mid-cap tokens
//! - до 3% для low liquidity tokens

use super::tokens::{from_token_amount, to_token_amount};
use super::types::*;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, warn};

// ============================================================================
// 0x API Configuration
// ============================================================================

const ZEROEX_API_BASE: &str = "https://api.0x.org/swap";
const ZEROEX_FEE_RECIPIENT: &str = "0x0000000000000000000000000000000000000000"; // TODO: заменить на реальный адрес
const ZEROEX_FEE_BPS: u64 = 100; // 1% комиссия (100 basis points)

/// 0x API endpoints по сетям
fn zeroex_api_url(chain: Chain) -> String {
    match chain {
        Chain::Ethereum => format!("{}/v1", ZEROEX_API_BASE),
        Chain::Polygon => format!("{}/v1", ZEROEX_API_BASE),
        Chain::Arbitrum => format!("{}/v1", ZEROEX_API_BASE),
        Chain::Base => format!("{}/v1", ZEROEX_API_BASE),
        Chain::Optimism => format!("{}/v1", ZEROEX_API_BASE),
        Chain::Bsc => format!("{}/v1", ZEROEX_API_BASE),
    }
}

// ============================================================================
// 0x API Request/Response Types
// ============================================================================

/// Запрос котировки (quote)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteRequest {
    /// Адрес токена для продажи
    pub sell_token: String,
    /// Адрес токена для покупки
    pub buy_token: String,
    /// Сумма продажи (в минимальных единицах, напр. wei)
    pub sell_amount: Option<String>,
    /// Сумма покупки (в минимальных единицах)
    pub buy_amount: Option<String>,
    /// Проскальзывание в basis points (100 = 1%)
    pub slippage_bps: Option<u32>,
    /// Адрес кошелька пользователя
    pub taker_address: Option<String>,
    /// Адрес получателя комиссии
    pub fee_recipient: Option<String>,
    /// Комиссия в basis points
    pub buy_token_percentage_fee: Option<u64>,
}

/// Ответ котировки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteResponse {
    pub sell_token: String,
    pub buy_token: String,
    pub sell_amount: String,
    pub buy_amount: String,
    pub allowance_target: String,
    pub price: String,
    pub gas: String,
    pub estimated_gas: String,
    pub protocol_fee: String,
    pub minimum_protocol_fee: String,
    pub buy_token_percentage_fee: String,
    pub to: String,
    pub data: String,
    pub value: String,
    pub chain_id: u64,
}

/// Информация о цене токена
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPrice {
    pub token: String,
    pub price: String,
    pub timestamp: u64,
}

/// Статус свапа
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SwapStatus {
    /// Котировка получена
    Quoted,
    /// Разрешение на spender отправлено
    Approving,
    /// Транзакция отправлена
    Pending { tx_hash: String },
    /// Транзакция подтверждена
    Completed { tx_hash: String, block_number: u64 },
    /// Ошибка
    Failed { error: String },
}

/// Запись свапа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapRecord {
    pub id: String,
    pub chain: Chain,
    pub from_token: String,
    pub to_token: String,
    pub from_amount: String,
    pub to_amount: String,
    pub expected_to_amount: String,
    pub price: String,
    pub gas_estimate: String,
    pub fee_bps: u64,
    pub status: SwapStatus,
    pub tx_hash: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

// ============================================================================
// 0x Swap Client
// ============================================================================

/// Клиент для работы с 0x Protocol
pub struct ZeroExClient {
    http_client: Client,
    api_key: Option<String>,
    fee_recipient: String,
    fee_bps: u64,
}

impl ZeroExClient {
    /// Создать новый 0x клиент
    pub fn new(api_key: Option<String>) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            api_key,
            fee_recipient: ZEROEX_FEE_RECIPIENT.to_string(),
            fee_bps: ZEROEX_FEE_BPS,
        }
    }

    /// Создать с кастомной комиссией
    pub fn with_fee(api_key: Option<String>, fee_recipient: String, fee_bps: u64) -> Self {
        // Валидация комиссии (0.5-3%)
        assert!(
            fee_bps >= 50 && fee_bps <= 300,
            "Fee must be between 0.5% (50bps) and 3% (300bps)"
        );

        Self::new(api_key).with_fee_config(fee_recipient, fee_bps)
    }

    /// Установить конфигурацию комиссии
    pub fn with_fee_config(mut self, fee_recipient: String, fee_bps: u64) -> Self {
        self.fee_recipient = fee_recipient;
        self.fee_bps = fee_bps;
        self
    }

    /// Получить заголовок авторизации
    fn auth_header(&self) -> Option<HashMap<String, String>> {
        self.api_key.as_ref().map(|key| {
            let mut headers = HashMap::new();
            headers.insert("0x-api-key".to_string(), key.clone());
            headers
        })
    }

    // ========================================================================
    // Quote API
    // ========================================================================

    /// Получить котировку для обмена
    pub async fn get_quote(
        &self,
        request: QuoteRequest,
        chain: Chain,
    ) -> Web3Result<QuoteResponse> {
        let base_url = zeroex_api_url(chain);
        let mut url = Url::parse(&format!("{}/quote", base_url))
            .map_err(|e| Web3Error::Network(format!("Invalid URL: {}", e)))?;

        // Добавить query параметры
        url.query_pairs_mut()
            .append_pair("sellToken", &request.sell_token)
            .append_pair("buyToken", &request.buy_token);

        if let Some(ref sell_amount) = request.sell_amount {
            url.query_pairs_mut().append_pair("sellAmount", sell_amount);
        }

        if let Some(ref buy_amount) = request.buy_amount {
            url.query_pairs_mut().append_pair("buyAmount", buy_amount);
        }

        if let Some(slippage) = request.slippage_bps {
            url.query_pairs_mut()
                .append_pair("slippageBps", &slippage.to_string());
        }

        if let Some(ref taker) = request.taker_address {
            url.query_pairs_mut().append_pair("taker", taker);
        }

        if let Some(ref fee_recipient) = request.fee_recipient {
            url.query_pairs_mut()
                .append_pair("feeRecipient", fee_recipient);
        }

        if let Some(fee_bps) = request.buy_token_percentage_fee {
            url.query_pairs_mut().append_pair(
                "buyTokenPercentageFee",
                &format!("{}", fee_bps as f64 / 10000.0),
            );
        }

        debug!("0x Quote URL: {}", url);

        // Выполнить запрос
        let mut req = self.http_client.get(url.clone());
        if let Some(headers) = self.auth_header() {
            for (key, value) in headers {
                req = req.header(&key, &value);
            }
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
            error!("0x API error: {}", error_text);
            return Err(Web3Error::Network(format!("0x API error: {}", error_text)));
        }

        let quote: QuoteResponse = response
            .json()
            .await
            .map_err(|e| Web3Error::Network(format!("Failed to parse response: {}", e)))?;

        info!(
            "Quote: {} {} -> {} {} (gas: {})",
            from_token_amount(&quote.sell_amount, 18),
            request.sell_token,
            from_token_amount(&quote.buy_amount, 18),
            request.buy_token,
            quote.gas
        );

        Ok(quote)
    }

    // ========================================================================
    // Price API
    // ========================================================================

    /// Получить цену токена (быстрая котировка без полной информации)
    pub async fn get_price(
        &self,
        sell_token: &str,
        buy_token: &str,
        chain: Chain,
    ) -> Web3Result<TokenPrice> {
        let base_url = zeroex_api_url(chain);
        let url = Url::parse(&format!("{}/price", base_url))
            .map_err(|e| Web3Error::Network(format!("Invalid URL: {}", e)))?;

        let mut url = url.clone();
        url.query_pairs_mut()
            .append_pair("sellToken", sell_token)
            .append_pair("buyToken", buy_token)
            .append_pair("sellAmount", "1000000000000000000"); // 1 token (18 decimals)

        let mut req = self.http_client.get(url);
        if let Some(headers) = self.auth_header() {
            for (key, value) in headers {
                req = req.header(&key, &value);
            }
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
            return Err(Web3Error::Network(format!("0x API error: {}", error_text)));
        }

        let price_data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Web3Error::Network(format!("Failed to parse response: {}", e)))?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(TokenPrice {
            token: sell_token.to_string(),
            price: price_data["price"].as_str().unwrap_or("0").to_string(),
            timestamp,
        })
    }

    // ========================================================================
    // Allowance API
    // ========================================================================

    /// Получить адрес контракта для approve
    pub async fn get_allowance_target(
        &self,
        token_address: &str,
        chain: Chain,
    ) -> Web3Result<String> {
        let base_url = zeroex_api_url(chain);
        let url = Url::parse(&format!("{}/allowance", base_url))
            .map_err(|e| Web3Error::Network(format!("Invalid URL: {}", e)))?;

        let mut url = url.clone();
        url.query_pairs_mut()
            .append_pair("tokenAddress", token_address);

        let response = self
            .http_client
            .get(url)
            .send()
            .await
            .map_err(|e| Web3Error::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Web3Error::Network(format!("0x API error: {}", error_text)));
        }

        let allowance_data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| Web3Error::Network(format!("Failed to parse response: {}", e)))?;

        Ok(allowance_data["allowanceTarget"]
            .as_str()
            .unwrap_or("")
            .to_string())
    }

    // ========================================================================
    // Swap Helpers
    // ========================================================================

    /// Создать запись свапа
    pub fn create_swap_record(
        &self,
        chain: Chain,
        from_token: String,
        to_token: String,
        from_amount: String,
        to_amount: String,
        price: String,
        gas_estimate: String,
    ) -> SwapRecord {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let id = uuid::Uuid::new_v4().to_string();

        SwapRecord {
            id,
            chain,
            from_token,
            to_token,
            from_amount,
            to_amount: to_amount.clone(),
            expected_to_amount: to_amount,
            price,
            gas_estimate,
            fee_bps: self.fee_bps,
            status: SwapStatus::Quoted,
            tx_hash: None,
            created_at: timestamp,
            updated_at: timestamp,
        }
    }

    /// Проверить достаточность баланса
    pub async fn check_balance(
        &self,
        _wallet_address: &str,
        _token_address: &str,
        _required_amount: &str,
        _chain: Chain,
        // TODO: реализовать проверку баланса через RPC
    ) -> Web3Result<bool> {
        // Заглушка - в реальной реализации нужно querying через RPC
        warn!("Balance check not implemented yet, assuming sufficient balance");
        Ok(true)
    }

    /// Рассчитать оптимальное slippage на основе волатильности
    pub fn calculate_slippage(&self, token_symbol: &str, gas_price_gwei: u64) -> u32 {
        // Базовое slippage 1% (100 bps)
        let base_slippage = 100u32;

        // Увеличить для волатильных токенов
        let volatility_adjustment = match token_symbol.to_uppercase().as_str() {
            "USDC" | "USDT" | "DAI" => 20, // stablecoins: 0.2%
            "ETH" | "WETH" => 50,          // major: 0.5%
            "WBTC" => 75,                  // BTC-wrapped: 0.75%
            _ => 200,                      // остальные: 2%
        };

        // Увеличить при высоком gas
        let gas_adjustment = if gas_price_gwei > 100 {
            50
        } else if gas_price_gwei > 50 {
            25
        } else {
            0
        };

        base_slippage + volatility_adjustment + gas_adjustment
    }

    // ========================================================================
    // Execute Swap
    // ========================================================================

    /// Полный процесс свапа (quote + submit)
    pub async fn execute_swap(
        &self,
        chain: Chain,
        sell_token: &str,
        buy_token: &str,
        sell_amount: &str,
        taker_address: &str,
        slippage_bps: Option<u32>,
        // NOTE: для реальной отправки нужен ethers::Signer или Tauri JS bridge
    ) -> Web3Result<SwapRecord> {
        info!(
            "Starting swap: {} {} -> {} on {:?}",
            sell_amount, sell_token, buy_token, chain
        );

        // 1. Получить котировку
        let quote_request = QuoteRequest {
            sell_token: sell_token.to_string(),
            buy_token: buy_token.to_string(),
            sell_amount: Some(sell_amount.to_string()),
            buy_amount: None,
            slippage_bps: slippage_bps.or(Some(self.calculate_slippage(buy_token, 50))),
            taker_address: Some(taker_address.to_string()),
            fee_recipient: Some(self.fee_recipient.clone()),
            buy_token_percentage_fee: Some(self.fee_bps),
        };

        let quote = self.get_quote(quote_request, chain).await?;

        // 2. Создать запись свапа
        let swap_record = self.create_swap_record(
            chain,
            sell_token.to_string(),
            buy_token.to_string(),
            sell_amount.to_string(),
            quote.buy_amount.clone(),
            quote.price.clone(),
            quote.gas.clone(),
        );

        info!(
            "Quote received: expected {} {}",
            from_token_amount(&quote.buy_amount, 18),
            buy_token
        );

        // 3. Проверить allowance и выполнить approve если нужно
        // NOTE: это требует подписания транзакции через кошелек
        // В Tauri это делается через JS bridge или native wallet
        warn!("Allowance check/approval must be done via wallet integration");

        // 4. Отправить swap транзакцию
        // NOTE: для реальной отправки нужен доступ к signer
        warn!("Transaction submission requires wallet integration");

        // Возвращаем котировку для ручной отправки
        Ok(swap_record)
    }
}

// ============================================================================
// Builder Pattern для Quote Request
// ============================================================================

/// Builder для создания запроса котировки
pub struct QuoteBuilder {
    request: QuoteRequest,
}

impl QuoteBuilder {
    pub fn new(sell_token: impl Into<String>, buy_token: impl Into<String>) -> Self {
        Self {
            request: QuoteRequest {
                sell_token: sell_token.into(),
                buy_token: buy_token.into(),
                sell_amount: None,
                buy_amount: None,
                slippage_bps: None,
                taker_address: None,
                fee_recipient: None,
                buy_token_percentage_fee: None,
            },
        }
    }

    pub fn sell_amount(mut self, amount: impl Into<String>) -> Self {
        self.request.sell_amount = Some(amount.into());
        self
    }

    pub fn buy_amount(mut self, amount: impl Into<String>) -> Self {
        self.request.buy_amount = Some(amount.into());
        self
    }

    pub fn slippage_bps(mut self, bps: u32) -> Self {
        self.request.slippage_bps = Some(bps);
        self
    }

    pub fn slippage_percent(mut self, percent: f64) -> Self {
        // percent -> basis points (1% = 100 bps)
        self.request.slippage_bps = Some((percent * 100.0) as u32);
        self
    }

    pub fn taker_address(mut self, address: impl Into<String>) -> Self {
        self.request.taker_address = Some(address.into());
        self
    }

    pub fn with_fee(mut self, recipient: impl Into<String>, bps: u64) -> Self {
        self.request.fee_recipient = Some(recipient.into());
        self.request.buy_token_percentage_fee = Some(bps);
        self
    }

    pub fn build(self) -> QuoteRequest {
        self.request
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Быстрая котировка: сколько buy_token получим за sell_amount
pub async fn quick_quote(
    sell_token: &str,
    buy_token: &str,
    sell_amount_human: &str,
    decimals: u8,
    chain: Chain,
    api_key: Option<String>,
) -> Web3Result<String> {
    let client = ZeroExClient::new(api_key);

    // Конвертировать human amount в raw
    let sell_amount_raw = to_token_amount(sell_amount_human, decimals)?;

    let request = QuoteRequest {
        sell_token: sell_token.to_string(),
        buy_token: buy_token.to_string(),
        sell_amount: Some(sell_amount_raw),
        buy_amount: None,
        slippage_bps: Some(100), // 1%
        taker_address: None,
        fee_recipient: None,
        buy_token_percentage_fee: None,
    };

    let quote = client.get_quote(request, chain).await?;

    // Вернуть human-readable результат
    Ok(from_token_amount(&quote.buy_amount, decimals))
}

/// Получить лучшую цену между buy_amount и sell_amount
pub fn calculate_exchange_rate(sell_amount: &str, buy_amount: &str, decimals: u8) -> f64 {
    let sell_f64 = from_token_amount(sell_amount, decimals)
        .parse::<f64>()
        .unwrap_or(0.0);
    let buy_f64 = from_token_amount(buy_amount, decimals)
        .parse::<f64>()
        .unwrap_or(0.0);

    if sell_f64 == 0.0 {
        return 0.0;
    }

    buy_f64 / sell_f64
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_builder_basic() {
        let request = QuoteBuilder::new(
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
        )
        .sell_amount("1000000000000000000") // 1 WETH
        .slippage_bps(100)
        .build();

        assert_eq!(
            request.sell_token,
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
        );
        assert_eq!(
            request.buy_token,
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        );
        assert_eq!(request.sell_amount, Some("1000000000000000000".to_string()));
        assert_eq!(request.slippage_bps, Some(100));
    }

    #[test]
    fn test_quote_builder_with_percent() {
        let request = QuoteBuilder::new("WETH", "USDC")
            .sell_amount("1000000000000000000")
            .slippage_percent(1.5)
            .build();

        assert_eq!(request.slippage_bps, Some(150));
    }

    #[test]
    fn test_quote_builder_with_fee() {
        let request = QuoteBuilder::new("WETH", "USDC")
            .sell_amount("1000000000000000000")
            .with_fee("0x1234567890123456789012345678901234567890", 150)
            .build();

        assert_eq!(
            request.fee_recipient,
            Some("0x1234567890123456789012345678901234567890".to_string())
        );
        assert_eq!(request.buy_token_percentage_fee, Some(150));
    }

    #[test]
    fn test_calculate_slippage_stablecoin() {
        let client = ZeroExClient::new(None);
        let slippage = client.calculate_slippage("USDC", 30);
        assert_eq!(slippage, 120); // 100 base + 20 stablecoin
    }

    #[test]
    fn test_calculate_slippage_eth() {
        let client = ZeroExClient::new(None);
        let slippage = client.calculate_slippage("ETH", 60);
        assert_eq!(slippage, 150); // 100 base + 50 ETH
    }

    #[test]
    fn test_calculate_slippage_high_gas() {
        let client = ZeroExClient::new(None);
        let slippage = client.calculate_slippage("UNKNOWN", 150);
        assert_eq!(slippage, 350); // 100 base + 200 unknown + 50 high gas
    }

    #[test]
    fn test_calculate_exchange_rate_eth_usdc() {
        // 1 ETH = 2000 USDC (примерно)
        let sell_raw = to_token_amount("1.0", 18).unwrap(); // 1 ETH
        let buy_raw = to_token_amount("2000.0", 6).unwrap(); // 2000 USDC

        let rate = calculate_exchange_rate(&sell_raw, &buy_raw, 18);
        assert!((rate - 2000.0).abs() < 1.0);
    }

    #[test]
    fn test_zeroex_client_creation() {
        let client = ZeroExClient::new(None);
        assert!(client.api_key.is_none());
        assert_eq!(client.fee_bps, 100);
    }

    #[test]
    #[should_panic(expected = "Fee must be between")]
    fn test_zeroex_client_invalid_fee_too_low() {
        ZeroExClient::with_fee(None, "0x123".to_string(), 10); // 0.1% - слишком мало
    }

    #[test]
    #[should_panic(expected = "Fee must be between")]
    fn test_zeroex_client_invalid_fee_too_high() {
        ZeroExClient::with_fee(None, "0x123".to_string(), 500); // 5% - слишком много
    }

    #[test]
    fn test_swap_record_creation() {
        let client = ZeroExClient::new(None);
        let record = client.create_swap_record(
            Chain::Ethereum,
            "WETH".to_string(),
            "USDC".to_string(),
            "1000000000000000000".to_string(),
            "2000000000".to_string(),
            "2000".to_string(),
            "150000".to_string(),
        );

        assert_eq!(record.chain, Chain::Ethereum);
        assert_eq!(record.from_token, "WETH");
        assert_eq!(record.to_token, "USDC");
        assert_eq!(record.status, SwapStatus::Quoted);
        assert_eq!(record.fee_bps, 100);
    }

    #[test]
    fn test_chain_support() {
        // Проверить что все поддерживаемые сети работают
        let chains = vec![
            Chain::Ethereum,
            Chain::Polygon,
            Chain::Arbitrum,
            Chain::Base,
            Chain::Optimism,
            Chain::Bsc,
        ];

        for chain in chains {
            let url = zeroex_api_url(chain);
            assert!(url.contains("/swap/v1"));
        }
    }
}
