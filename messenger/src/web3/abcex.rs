//! Abcex Exchange Integration — покупка криптовалюты через Abcex API
//!
//! Документация: https://abcex.io/docs/api
//!
//! Features:
//! - Покупка криптовалюты за фиат (USD, EUR, RUB)
//! - Комиссия 2-3% (настраиваемая)
//! - Поддержка multiple payment methods (card, SEPA, bank transfer)
//! - KYC verification status check
//! - Rate tracking и order management
//!
//! Комиссии:
//! - 2% для major cryptocurrencies (BTC, ETH, USDT)
//! - 2.5% для mid-cap
//! - 3% для остальных

use super::types::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info};

// ============================================================================
// Abcex API Configuration
// ============================================================================

const ABCEX_API_BASE: &str = "https://api.abcex.io/v1";
const ABCEX_FEE_BPS: u64 = 200; // 2% комиссия по умолчанию (200 basis points)

/// Payment methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentMethod {
    CreditCard,
    DebitCard,
    SEPA,
    BankTransfer,
    ApplePay,
    GooglePay,
}

impl PaymentMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            PaymentMethod::CreditCard => "credit_card",
            PaymentMethod::DebitCard => "debit_card",
            PaymentMethod::SEPA => "sepa",
            PaymentMethod::BankTransfer => "bank_transfer",
            PaymentMethod::ApplePay => "apple_pay",
            PaymentMethod::GooglePay => "google_pay",
        }
    }
}

/// Fiat currencies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FiatCurrency {
    USD,
    EUR,
    GBP,
    RUB,
}

impl FiatCurrency {
    pub fn as_str(&self) -> &'static str {
        match self {
            FiatCurrency::USD => "USD",
            FiatCurrency::EUR => "EUR",
            FiatCurrency::GBP => "GBP",
            FiatCurrency::RUB => "RUB",
        }
    }
}

// ============================================================================
// Abcex API Request/Response Types
// ============================================================================

/// Запрос на получение котировки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyQuoteRequest {
    /// fiat currency (USD, EUR, etc)
    pub fiat_currency: String,
    /// fiat amount
    pub fiat_amount: String,
    /// crypto currency symbol (BTC, ETH, etc)
    pub crypto_currency: String,
    /// payment method
    pub payment_method: Option<String>,
    /// country code (ISO 3166-1 alpha-2)
    pub country: Option<String>,
}

/// Ответ котировки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyQuoteResponse {
    /// ID котировки
    pub quote_id: String,
    /// Fiat currency
    pub fiat_currency: String,
    /// Fiat amount
    pub fiat_amount: String,
    /// Crypto currency
    pub crypto_currency: String,
    /// Crypto amount (expected to receive)
    pub crypto_amount: String,
    /// Exchange rate
    pub rate: String,
    /// Fee amount (in fiat)
    pub fee_amount: String,
    /// Fee percentage
    pub fee_percent: String,
    /// Payment methods available
    pub payment_methods: Vec<String>,
    /// Время действия котировки (секунды)
    pub expires_in: u64,
    /// Минимальная сумма покупки
    pub min_amount: String,
    /// Максимальная сумма покупки
    pub max_amount: String,
}

/// Информация о заказе
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuyOrder {
    /// ID заказа
    pub order_id: String,
    /// Quote ID
    pub quote_id: String,
    /// Статус
    pub status: BuyOrderStatus,
    /// Fiat currency
    pub fiat_currency: String,
    /// Fiat amount
    pub fiat_amount: String,
    /// Crypto currency
    pub crypto_currency: String,
    /// Crypto amount
    pub crypto_amount: String,
    /// Rate
    pub rate: String,
    /// Fee
    pub fee_amount: String,
    /// Payment method
    pub payment_method: String,
    /// Deposit address (куда отправить crypto)
    pub deposit_address: String,
    /// Payment URL (для редиректа на оплату)
    pub payment_url: Option<String>,
    /// Created at (timestamp)
    pub created_at: u64,
    /// Updated at (timestamp)
    pub updated_at: u64,
}

/// Статус заказа
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuyOrderStatus {
    /// Создан, ожидает оплаты
    Pending,
    /// Оплата получена, обработка
    Processing,
    /// Crypto отправлен
    CryptoSent { tx_hash: String },
    /// Завершен
    Completed,
    /// Отменен
    Cancelled,
    /// Ошибка
    Failed { error: String },
}

/// KYC статус
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycStatus {
    pub verified: bool,
    pub level: String,
    pub limits: KycLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycLimits {
    pub daily_limit: String,
    pub monthly_limit: String,
    pub remaining_daily: String,
    pub remaining_monthly: String,
}

// ============================================================================
// Abcex Client
// ============================================================================

/// Клиент для работы с Abcex Exchange
pub struct AbcexClient {
    http_client: Client,
    api_key: Option<String>,
    api_secret: Option<String>,
    fee_bps: u64,
    fee_recipient: String,
}

impl AbcexClient {
    /// Создать новый Abcex клиент
    pub fn new(api_key: Option<String>, api_secret: Option<String>) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            api_key,
            api_secret,
            fee_bps: ABCEX_FEE_BPS, // 2% по умолчанию
            fee_recipient: String::new(),
        }
    }

    /// Создать с кастомной комиссией
    pub fn with_fee(api_key: Option<String>, api_secret: Option<String>, fee_bps: u64) -> Self {
        // Валидация комиссии (2-3%)
        assert!(
            fee_bps >= 200 && fee_bps <= 300,
            "Fee must be between 2% (200bps) and 3% (300bps)"
        );

        Self::new(api_key, api_secret).with_fee_config(fee_bps, String::new())
    }

    /// Установить конфигурацию комиссии
    pub fn with_fee_config(mut self, fee_bps: u64, fee_recipient: String) -> Self {
        self.fee_bps = fee_bps;
        self.fee_recipient = fee_recipient;
        self
    }

    /// Получить заголовки авторизации
    fn auth_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        if let Some(ref key) = self.api_key {
            headers.insert("X-API-Key".to_string(), key.clone());
        }
        if let Some(ref secret) = self.api_secret {
            headers.insert("X-API-Secret".to_string(), secret.clone());
        }
        headers
    }

    // ========================================================================
    // Quote API
    // ========================================================================

    /// Получить котировку для покупки crypto
    pub async fn get_buy_quote(&self, request: BuyQuoteRequest) -> Web3Result<BuyQuoteResponse> {
        let url = format!("{}/buy/quote", ABCEX_API_BASE);
        debug!("Abcex Quote URL: {}", url);

        // Build query parameters
        let mut query_params = vec![
            ("fiat_currency", request.fiat_currency),
            ("fiat_amount", request.fiat_amount),
            ("crypto_currency", request.crypto_currency),
        ];

        if let Some(ref payment_method) = request.payment_method {
            query_params.push(("payment_method", payment_method.clone()));
        }

        if let Some(ref country) = request.country {
            query_params.push(("country", country.clone()));
        }

        // Выполнить запрос
        let mut req = self.http_client.get(&url).query(&query_params);

        for (key, value) in self.auth_headers() {
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
            error!("Abcex API error: {}", error_text);
            return Err(Web3Error::Network(format!(
                "Abcex API error: {}",
                error_text
            )));
        }

        let quote: BuyQuoteResponse = response
            .json()
            .await
            .map_err(|e| Web3Error::Network(format!("Failed to parse response: {}", e)))?;

        info!(
            "Quote: {} {} -> {} {} (rate: {}, fee: {}%)",
            quote.fiat_amount,
            quote.fiat_currency,
            quote.crypto_amount,
            quote.crypto_currency,
            quote.rate,
            quote.fee_percent
        );

        Ok(quote)
    }

    // ========================================================================
    // Order API
    // ========================================================================

    /// Создать заказ на покупку
    pub async fn create_buy_order(
        &self,
        quote_id: String,
        deposit_address: String,
        payment_method: PaymentMethod,
        user_email: String,
    ) -> Web3Result<BuyOrder> {
        let url = format!("{}/buy/order", ABCEX_API_BASE);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let order = serde_json::json!({
            "quote_id": quote_id,
            "deposit_address": deposit_address,
            "payment_method": payment_method.as_str(),
            "user_email": user_email,
            "timestamp": timestamp,
        });

        let mut req = self.http_client.post(&url).json(&order);

        for (key, value) in self.auth_headers() {
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
            error!("Abcex order error: {}", error_text);
            return Err(Web3Error::Network(format!(
                "Abcex order error: {}",
                error_text
            )));
        }

        let order_response: BuyOrder = response
            .json()
            .await
            .map_err(|e| Web3Error::Network(format!("Failed to parse response: {}", e)))?;

        info!(
            "Order created: {} {} (payment_url: {:?})",
            order_response.crypto_amount,
            order_response.crypto_currency,
            order_response.payment_url
        );

        Ok(order_response)
    }

    /// Получить статус заказа
    pub async fn get_order_status(&self, order_id: &str) -> Web3Result<BuyOrder> {
        let url = format!("{}/buy/order/{}", ABCEX_API_BASE, order_id);

        let mut req = self.http_client.get(&url);

        for (key, value) in self.auth_headers() {
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
                "Abcex API error: {}",
                error_text
            )));
        }

        let order: BuyOrder = response
            .json()
            .await
            .map_err(|e| Web3Error::Network(format!("Failed to parse response: {}", e)))?;

        Ok(order)
    }

    // ========================================================================
    // KYC API
    // ========================================================================

    /// Проверить KYC статус пользователя
    pub async fn check_kyc_status(&self, user_id: &str) -> Web3Result<KycStatus> {
        let url = format!("{}/kyc/status", ABCEX_API_BASE);

        let mut req = self.http_client.get(&url).query(&[("user_id", user_id)]);

        for (key, value) in self.auth_headers() {
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
                "Abcex API error: {}",
                error_text
            )));
        }

        let kyc: KycStatus = response
            .json()
            .await
            .map_err(|e| Web3Error::Network(format!("Failed to parse response: {}", e)))?;

        Ok(kyc)
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    /// Получить список доступных криптовалют
    pub async fn get_supported_cryptos(&self) -> Web3Result<Vec<String>> {
        let url = format!("{}/supported/cryptos", ABCEX_API_BASE);

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
                "Abcex API error: {}",
                error_text
            )));
        }

        let cryptos: Vec<String> = response
            .json()
            .await
            .map_err(|e| Web3Error::Network(format!("Failed to parse response: {}", e)))?;

        Ok(cryptos)
    }

    /// Получить лимиты для страны
    pub async fn get_limits(&self, country: &str) -> Web3Result<HashMap<String, String>> {
        let url = format!("{}/limits", ABCEX_API_BASE);

        let response = self
            .http_client
            .get(&url)
            .query(&[("country", country)])
            .send()
            .await
            .map_err(|e| Web3Error::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(Web3Error::Network(format!(
                "Abcex API error: {}",
                error_text
            )));
        }

        let limits: HashMap<String, String> = response
            .json()
            .await
            .map_err(|e| Web3Error::Network(format!("Failed to parse response: {}", e)))?;

        Ok(limits)
    }

    /// Полный процесс покупки (quote + create order)
    pub async fn execute_buy(
        &self,
        fiat_currency: String,
        fiat_amount: String,
        crypto_currency: String,
        deposit_address: String,
        payment_method: PaymentMethod,
        user_email: String,
        country: Option<String>,
    ) -> Web3Result<BuyOrder> {
        info!(
            "Starting buy: {} {} -> {} to {}",
            fiat_amount, fiat_currency, crypto_currency, deposit_address
        );

        // 1. Получить котировку
        let quote_request = BuyQuoteRequest {
            fiat_currency: fiat_currency.clone(),
            fiat_amount: fiat_amount.clone(),
            crypto_currency: crypto_currency.clone(),
            payment_method: Some(payment_method.as_str().to_string()),
            country,
        };

        let quote = self.get_buy_quote(quote_request).await?;

        info!(
            "Quote received: will get {} {} for {} {} (fee: {}%)",
            quote.crypto_amount,
            quote.crypto_currency,
            quote.fiat_amount,
            quote.fiat_currency,
            quote.fee_percent
        );

        // 2. Создать заказ
        let order = self
            .create_buy_order(quote.quote_id, deposit_address, payment_method, user_email)
            .await?;

        Ok(order)
    }
}

// ============================================================================
// Builder Pattern для Buy Quote
// ============================================================================

/// Builder для создания запроса покупки
pub struct BuyQuoteBuilder {
    request: BuyQuoteRequest,
}

impl BuyQuoteBuilder {
    pub fn new(
        fiat_currency: impl Into<String>,
        fiat_amount: impl Into<String>,
        crypto: impl Into<String>,
    ) -> Self {
        Self {
            request: BuyQuoteRequest {
                fiat_currency: fiat_currency.into(),
                fiat_amount: fiat_amount.into(),
                crypto_currency: crypto.into(),
                payment_method: None,
                country: None,
            },
        }
    }

    pub fn payment_method(mut self, method: PaymentMethod) -> Self {
        self.request.payment_method = Some(method.as_str().to_string());
        self
    }

    pub fn country(mut self, country: impl Into<String>) -> Self {
        self.request.country = Some(country.into());
        self
    }

    pub fn build(self) -> BuyQuoteRequest {
        self.request
    }
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Быстрая котировка для покупки
pub async fn quick_buy_quote(
    fiat_currency: &str,
    fiat_amount: &str,
    crypto: &str,
    payment_method: PaymentMethod,
    api_key: Option<String>,
    api_secret: Option<String>,
) -> Web3Result<BuyQuoteResponse> {
    let client = AbcexClient::new(api_key, api_secret);

    let request = BuyQuoteRequest {
        fiat_currency: fiat_currency.to_string(),
        fiat_amount: fiat_amount.to_string(),
        crypto_currency: crypto.to_string(),
        payment_method: Some(payment_method.as_str().to_string()),
        country: None,
    };

    client.get_buy_quote(request).await
}

/// Рассчитать комиссию для суммы
pub fn calculate_fee(amount: &str, fee_bps: u64) -> Web3Result<String> {
    let amount_f64: f64 = amount
        .parse()
        .map_err(|_| Web3Error::Wallet(format!("Invalid amount: {}", amount)))?;

    let fee = amount_f64 * (fee_bps as f64) / 10000.0;
    Ok(format!("{:.2}", fee))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buy_quote_builder_basic() {
        let request = BuyQuoteBuilder::new("USD", "100", "BTC")
            .payment_method(PaymentMethod::CreditCard)
            .country("US")
            .build();

        assert_eq!(request.fiat_currency, "USD");
        assert_eq!(request.fiat_amount, "100");
        assert_eq!(request.crypto_currency, "BTC");
        assert_eq!(request.payment_method, Some("credit_card".to_string()));
        assert_eq!(request.country, Some("US".to_string()));
    }

    #[test]
    fn test_calculate_fee() {
        // 2% комиссия от $100
        let fee = calculate_fee("100", 200).unwrap();
        assert_eq!(fee, "2.00");

        // 2.5% комиссия от $500
        let fee = calculate_fee("500", 250).unwrap();
        assert_eq!(fee, "12.50");

        // 3% комиссия от $1000
        let fee = calculate_fee("1000", 300).unwrap();
        assert_eq!(fee, "30.00");
    }

    #[test]
    fn test_payment_method_as_str() {
        assert_eq!(PaymentMethod::CreditCard.as_str(), "credit_card");
        assert_eq!(PaymentMethod::SEPA.as_str(), "sepa");
        assert_eq!(PaymentMethod::ApplePay.as_str(), "apple_pay");
    }

    #[test]
    fn test_fiat_currency_as_str() {
        assert_eq!(FiatCurrency::USD.as_str(), "USD");
        assert_eq!(FiatCurrency::EUR.as_str(), "EUR");
        assert_eq!(FiatCurrency::RUB.as_str(), "RUB");
    }

    #[test]
    fn test_abcex_client_creation() {
        let client = AbcexClient::new(None, None);
        assert!(client.api_key.is_none());
        assert!(client.api_secret.is_none());
        assert_eq!(client.fee_bps, 200); // 2% по умолчанию
    }

    #[test]
    fn test_abcex_client_with_fee() {
        let client = AbcexClient::with_fee(None, None, 250);
        assert_eq!(client.fee_bps, 250); // 2.5%
    }

    #[test]
    #[should_panic(expected = "Fee must be between")]
    fn test_abcex_client_invalid_fee_too_low() {
        AbcexClient::with_fee(None, None, 100); // 1% - слишком мало
    }

    #[test]
    #[should_panic(expected = "Fee must be between")]
    fn test_abcex_client_invalid_fee_too_high() {
        AbcexClient::with_fee(None, None, 400); // 4% - слишком много
    }

    #[test]
    fn test_buy_quote_builder_defaults() {
        let request = BuyQuoteBuilder::new("USD", "100", "BTC").build();

        assert_eq!(request.fiat_currency, "USD");
        assert_eq!(request.fiat_amount, "100");
        assert_eq!(request.crypto_currency, "BTC");
        assert!(request.payment_method.is_none());
        assert!(request.country.is_none());
    }
}
