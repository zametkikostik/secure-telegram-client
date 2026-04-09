//! Tauri Commands для Abcex Integration
//!
//! Команды для вызова из JavaScript/TypeScript UI

use serde::{Deserialize, Serialize};
use tauri::State;

use super::abcex::{AbcexClient, BuyQuoteRequest, PaymentMethod};
use super::types::Web3Error;

// ============================================================================
// Command Request/Response Types
// ============================================================================

/// Запрос котировки для покупки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbcexQuoteRequest {
    pub fiat_currency: String,
    pub fiat_amount: String,
    pub crypto_currency: String,
    pub payment_method: Option<String>,
    pub country: Option<String>,
}

/// Ответ котировки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbcexQuoteResponse {
    pub success: bool,
    pub quote: Option<AbcexQuoteData>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbcexQuoteData {
    pub quote_id: String,
    pub fiat_currency: String,
    pub fiat_amount: String,
    pub crypto_currency: String,
    pub crypto_amount: String,
    pub rate: String,
    pub fee_amount: String,
    pub fee_percent: String,
    pub payment_methods: Vec<String>,
    pub expires_in: u64,
    pub min_amount: String,
    pub max_amount: String,
}

/// Запрос на создание заказа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbcexOrderRequest {
    pub quote_id: String,
    pub deposit_address: String,
    pub payment_method: String,
    pub user_email: String,
}

/// Результат создания заказа
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbcexOrderResponse {
    pub success: bool,
    pub order: Option<AbcexOrderData>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbcexOrderData {
    pub order_id: String,
    pub quote_id: String,
    pub status: String,
    pub fiat_currency: String,
    pub fiat_amount: String,
    pub crypto_currency: String,
    pub crypto_amount: String,
    pub rate: String,
    pub fee_amount: String,
    pub payment_method: String,
    pub deposit_address: String,
    pub payment_url: Option<String>,
    pub created_at: u64,
}

/// Запрос на проверку KYC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbcexKycRequest {
    pub user_id: String,
}

/// Ответ KYC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbcexKycResponse {
    pub success: bool,
    pub kyc_status: Option<KycData>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycData {
    pub verified: bool,
    pub level: String,
    pub daily_limit: String,
    pub monthly_limit: String,
    pub remaining_daily: String,
    pub remaining_monthly: String,
}

/// Запрос на получение лимитов
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbcexLimitsRequest {
    pub country: String,
}

/// Ответ с лимитами
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbcexLimitsResponse {
    pub success: bool,
    pub limits: Option<std::collections::HashMap<String, String>>,
    pub error: Option<String>,
}

// ============================================================================
// Tauri State
// ============================================================================

/// State для хранения Abcex клиента
pub struct AbcexState {
    pub client: AbcexClient,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub fee_bps: u64,
}

impl AbcexState {
    pub fn new(api_key: Option<String>, api_secret: Option<String>, fee_bps: u64) -> Self {
        let client = AbcexClient::with_fee(api_key.clone(), api_secret.clone(), fee_bps);

        Self {
            client,
            api_key,
            api_secret,
            fee_bps,
        }
    }
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Получить котировку для покупки криптовалюты
#[tauri::command]
pub async fn abcex_get_quote(
    request: AbcexQuoteRequest,
    state: State<'_, AbcexState>,
) -> Result<AbcexQuoteResponse, String> {
    let quote_request = BuyQuoteRequest {
        fiat_currency: request.fiat_currency,
        fiat_amount: request.fiat_amount,
        crypto_currency: request.crypto_currency,
        payment_method: request.payment_method,
        country: request.country,
    };

    match state.client.get_buy_quote(quote_request).await {
        Ok(quote) => Ok(AbcexQuoteResponse {
            success: true,
            quote: Some(AbcexQuoteData {
                quote_id: quote.quote_id,
                fiat_currency: quote.fiat_currency,
                fiat_amount: quote.fiat_amount,
                crypto_currency: quote.crypto_currency,
                crypto_amount: quote.crypto_amount,
                rate: quote.rate,
                fee_amount: quote.fee_amount,
                fee_percent: quote.fee_percent,
                payment_methods: quote.payment_methods,
                expires_in: quote.expires_in,
                min_amount: quote.min_amount,
                max_amount: quote.max_amount,
            }),
            error: None,
        }),
        Err(e) => Ok(AbcexQuoteResponse {
            success: false,
            quote: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Создать заказ на покупку
#[tauri::command]
pub async fn abcex_create_order(
    request: AbcexOrderRequest,
    state: State<'_, AbcexState>,
) -> Result<AbcexOrderResponse, String> {
    let payment_method = match request.payment_method.as_str() {
        "credit_card" => PaymentMethod::CreditCard,
        "debit_card" => PaymentMethod::DebitCard,
        "sepa" => PaymentMethod::SEPA,
        "bank_transfer" => PaymentMethod::BankTransfer,
        "apple_pay" => PaymentMethod::ApplePay,
        "google_pay" => PaymentMethod::GooglePay,
        _ => {
            return Err(format!(
                "Invalid payment method: {}",
                request.payment_method
            ))
        }
    };

    match state
        .client
        .create_buy_order(
            request.quote_id,
            request.deposit_address,
            payment_method,
            request.user_email,
        )
        .await
    {
        Ok(order) => Ok(AbcexOrderResponse {
            success: true,
            order: Some(AbcexOrderData {
                order_id: order.order_id,
                quote_id: order.quote_id,
                status: format!("{:?}", order.status),
                fiat_currency: order.fiat_currency,
                fiat_amount: order.fiat_amount,
                crypto_currency: order.crypto_currency,
                crypto_amount: order.crypto_amount,
                rate: order.rate,
                fee_amount: order.fee_amount,
                payment_method: order.payment_method,
                deposit_address: order.deposit_address,
                payment_url: order.payment_url,
                created_at: order.created_at,
            }),
            error: None,
        }),
        Err(e) => Ok(AbcexOrderResponse {
            success: false,
            order: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Проверить статус заказа
#[tauri::command]
pub async fn abcex_get_order_status(
    order_id: String,
    state: State<'_, AbcexState>,
) -> Result<AbcexOrderResponse, String> {
    match state.client.get_order_status(&order_id).await {
        Ok(order) => Ok(AbcexOrderResponse {
            success: true,
            order: Some(AbcexOrderData {
                order_id: order.order_id,
                quote_id: order.quote_id,
                status: format!("{:?}", order.status),
                fiat_currency: order.fiat_currency,
                fiat_amount: order.fiat_amount,
                crypto_currency: order.crypto_currency,
                crypto_amount: order.crypto_amount,
                rate: order.rate,
                fee_amount: order.fee_amount,
                payment_method: order.payment_method,
                deposit_address: order.deposit_address,
                payment_url: order.payment_url,
                created_at: order.created_at,
            }),
            error: None,
        }),
        Err(e) => Ok(AbcexOrderResponse {
            success: false,
            order: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Проверить KYC статус пользователя
#[tauri::command]
pub async fn abcex_check_kyc(
    request: AbcexKycRequest,
    state: State<'_, AbcexState>,
) -> Result<AbcexKycResponse, String> {
    match state.client.check_kyc_status(&request.user_id).await {
        Ok(kyc) => Ok(AbcexKycResponse {
            success: true,
            kyc_status: Some(KycData {
                verified: kyc.verified,
                level: kyc.level,
                daily_limit: kyc.limits.daily_limit,
                monthly_limit: kyc.limits.monthly_limit,
                remaining_daily: kyc.limits.remaining_daily,
                remaining_monthly: kyc.limits.remaining_monthly,
            }),
            error: None,
        }),
        Err(e) => Ok(AbcexKycResponse {
            success: false,
            kyc_status: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Получить список доступных криптовалют
#[tauri::command]
pub async fn abcex_get_supported_cryptos(
    state: State<'_, AbcexState>,
) -> Result<Vec<String>, String> {
    state
        .client
        .get_supported_cryptos()
        .await
        .map_err(|e| e.to_string())
}

/// Получить лимиты для страны
#[tauri::command]
pub async fn abcex_get_limits(
    request: AbcexLimitsRequest,
    state: State<'_, AbcexState>,
) -> Result<AbcexLimitsResponse, String> {
    match state.client.get_limits(&request.country).await {
        Ok(limits) => Ok(AbcexLimitsResponse {
            success: true,
            limits: Some(limits),
            error: None,
        }),
        Err(e) => Ok(AbcexLimitsResponse {
            success: false,
            limits: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Быстрая котировка (human-readable)
#[tauri::command]
pub async fn abcex_quick_quote(
    fiat_currency: String,
    fiat_amount: String,
    crypto_currency: String,
    payment_method: String,
    country: Option<String>,
    state: State<'_, AbcexState>,
) -> Result<String, String> {
    let pm = match payment_method.as_str() {
        "credit_card" => PaymentMethod::CreditCard,
        "debit_card" => PaymentMethod::DebitCard,
        "sepa" => PaymentMethod::SEPA,
        "bank_transfer" => PaymentMethod::BankTransfer,
        "apple_pay" => PaymentMethod::ApplePay,
        "google_pay" => PaymentMethod::GooglePay,
        _ => return Err(format!("Invalid payment method: {}", payment_method)),
    };

    let request = BuyQuoteRequest {
        fiat_currency,
        fiat_amount,
        crypto_currency,
        payment_method: Some(pm.as_str().to_string()),
        country,
    };

    state
        .client
        .get_buy_quote(request)
        .await
        .map(|q| serde_json::to_string_pretty(&q).unwrap_or_default())
        .map_err(|e| e.to_string())
}

/// Рассчитать комиссию для суммы
#[tauri::command]
pub fn abcex_calculate_fee(
    amount: String,
    fee_bps: u64,
    state: State<'_, AbcexState>,
) -> Result<String, String> {
    super::abcex::calculate_fee(&amount, fee_bps).map_err(|e: Web3Error| e.to_string())
}

// ============================================================================
// Command Registration
// ============================================================================

/// Зарегистрировать все Abcex команды
pub fn register_abcex_commands(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder
        .manage(AbcexState::new(
            None, // API key из конфига
            None, // API secret из конфига
            200,  // 2% fee по умолчанию
        ))
        .invoke_handler(tauri::generate_handler![
            abcex_get_quote,
            abcex_create_order,
            abcex_get_order_status,
            abcex_check_kyc,
            abcex_get_supported_cryptos,
            abcex_get_limits,
            abcex_quick_quote,
            abcex_calculate_fee,
        ])
}
