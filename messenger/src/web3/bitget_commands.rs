//! Tauri Commands для Bitget Integration
//!
//! Команды для вызова из JavaScript/TypeScript UI

use serde::{Deserialize, Serialize};
use tauri::State;

use super::bitget::{BitgetClient, BuyRequest, OrderSide, OrderType};
use super::types::Web3Error;

// ============================================================================
// Command Request/Response Types
// ============================================================================

/// Запрос на покупку/продажу
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitgetOrderRequest {
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub amount: Option<String>,
    pub quote_amount: Option<String>,
    pub price: Option<String>,
    pub client_order_id: Option<String>,
}

/// Результат создания ордера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitgetOrderResponse {
    pub success: bool,
    pub order: Option<BitgetOrderData>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitgetOrderData {
    pub order_id: String,
    pub client_oid: Option<String>,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub amount: String,
    pub price: String,
    pub filled_amount: String,
    pub average_price: String,
    pub status: String,
    pub fee: String,
    pub fee_currency: String,
    pub created_at: u64,
}

/// Запрос на проверку статуса ордера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitgetOrderStatusRequest {
    pub symbol: String,
    pub order_id: String,
}

/// Запрос на отмену ордера
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitgetCancelRequest {
    pub symbol: String,
    pub order_id: String,
}

/// Запрос на получение баланса
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitgetBalanceRequest {
    pub currency: String,
}

/// Ответ с балансом
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitgetBalanceResponse {
    pub success: bool,
    pub balance: Option<BalanceData>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceData {
    pub available_balance: String,
    pub frozen_balance: String,
    pub total_balance: String,
    pub currency: String,
}

/// Запрос на получение рыночной цены
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitgetMarketPriceRequest {
    pub symbol: String,
}

/// Ответ с рыночной ценой
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitgetMarketPriceResponse {
    pub success: bool,
    pub price: Option<PriceData>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    pub symbol: String,
    pub price: String,
    pub high_24h: String,
    pub low_24h: String,
    pub volume_24h: String,
    pub change_24h: String,
    pub timestamp: u64,
}

// ============================================================================
// Tauri State
// ============================================================================

/// State для хранения Bitget клиента
pub struct BitgetState {
    pub client: BitgetClient,
    pub api_key: Option<String>,
    pub secret_key: Option<String>,
    pub passphrase: Option<String>,
    pub fee_bps: u64,
}

impl BitgetState {
    pub fn new(
        api_key: Option<String>,
        secret_key: Option<String>,
        passphrase: Option<String>,
        fee_bps: u64,
    ) -> Self {
        let client = BitgetClient::with_fee(
            api_key.clone(),
            secret_key.clone(),
            passphrase.clone(),
            fee_bps,
        );

        Self {
            client,
            api_key,
            secret_key,
            passphrase,
            fee_bps,
        }
    }
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Разместить ордер на покупку/продажу
#[tauri::command]
pub async fn bitget_place_order(
    request: BitgetOrderRequest,
    state: State<'_, BitgetState>,
) -> Result<BitgetOrderResponse, String> {
    let order_side = match request.side.as_str() {
        "buy" => OrderSide::Buy,
        "sell" => OrderSide::Sell,
        _ => return Err(format!("Invalid order side: {}", request.side)),
    };

    let order_type = match request.order_type.as_str() {
        "market" => OrderType::Market,
        "limit" => OrderType::Limit,
        _ => return Err(format!("Invalid order type: {}", request.order_type)),
    };

    let order_request = BuyRequest {
        symbol: request.symbol,
        side: order_side,
        order_type,
        amount: request.amount,
        quote_amount: request.quote_amount,
        price: request.price,
        client_oid: request.client_order_id,
    };

    match state.client.place_buy_order(order_request).await {
        Ok(order) => Ok(BitgetOrderResponse {
            success: true,
            order: Some(BitgetOrderData {
                order_id: order.order_id,
                client_oid: order.client_oid,
                symbol: order.symbol,
                side: order.side,
                order_type: order.order_type,
                amount: order.amount,
                price: order.price,
                filled_amount: order.filled_amount,
                average_price: order.average_price,
                status: order.status,
                fee: order.fee,
                fee_currency: order.fee_currency,
                created_at: order.created_at,
            }),
            error: None,
        }),
        Err(e) => Ok(BitgetOrderResponse {
            success: false,
            order: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Проверить статус ордера
#[tauri::command]
pub async fn bitget_get_order_status(
    request: BitgetOrderStatusRequest,
    state: State<'_, BitgetState>,
) -> Result<BitgetOrderResponse, String> {
    match state
        .client
        .get_order_status(&request.symbol, &request.order_id)
        .await
    {
        Ok(order) => Ok(BitgetOrderResponse {
            success: true,
            order: Some(BitgetOrderData {
                order_id: order.order_id,
                client_oid: order.client_oid,
                symbol: order.symbol,
                side: order.side,
                order_type: order.order_type,
                amount: order.amount,
                price: order.price,
                filled_amount: order.filled_amount,
                average_price: order.average_price,
                status: order.status,
                fee: order.fee,
                fee_currency: order.fee_currency,
                created_at: order.created_at,
            }),
            error: None,
        }),
        Err(e) => Ok(BitgetOrderResponse {
            success: false,
            order: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Отменить ордер
#[tauri::command]
pub async fn bitget_cancel_order(
    request: BitgetCancelRequest,
    state: State<'_, BitgetState>,
) -> Result<BitgetOrderResponse, String> {
    match state
        .client
        .cancel_order(&request.symbol, &request.order_id)
        .await
    {
        Ok(()) => Ok(BitgetOrderResponse {
            success: true,
            order: None,
            error: None,
        }),
        Err(e) => Ok(BitgetOrderResponse {
            success: false,
            order: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Получить баланс аккаунта
#[tauri::command]
pub async fn bitget_get_balance(
    request: BitgetBalanceRequest,
    state: State<'_, BitgetState>,
) -> Result<BitgetBalanceResponse, String> {
    match state.client.get_account_balance(&request.currency).await {
        Ok(balance) => Ok(BitgetBalanceResponse {
            success: true,
            balance: Some(BalanceData {
                available_balance: balance.available_balance,
                frozen_balance: balance.frozen_balance,
                total_balance: balance.total_balance,
                currency: balance.currency,
            }),
            error: None,
        }),
        Err(e) => Ok(BitgetBalanceResponse {
            success: false,
            balance: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Получить текущую рыночную цену
#[tauri::command]
pub async fn bitget_get_market_price(
    request: BitgetMarketPriceRequest,
    state: State<'_, BitgetState>,
) -> Result<BitgetMarketPriceResponse, String> {
    match state.client.get_market_price(&request.symbol).await {
        Ok(price) => Ok(BitgetMarketPriceResponse {
            success: true,
            price: Some(PriceData {
                symbol: price.symbol,
                price: price.price,
                high_24h: price.high_24h,
                low_24h: price.low_24h,
                volume_24h: price.volume_24h,
                change_24h: price.change_24h,
                timestamp: price.timestamp,
            }),
            error: None,
        }),
        Err(e) => Ok(BitgetMarketPriceResponse {
            success: false,
            price: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Получить список торговых пар
#[tauri::command]
pub async fn bitget_get_symbols(state: State<'_, BitgetState>) -> Result<Vec<String>, String> {
    state.client.get_symbols().await.map_err(|e| e.to_string())
}

/// Быстрая покупка (market order)
#[tauri::command]
pub async fn bitget_quick_buy(
    symbol: String,
    quote_amount: String,
    state: State<'_, BitgetState>,
) -> Result<BitgetOrderResponse, String> {
    match state.client.execute_buy(symbol, quote_amount).await {
        Ok(order) => Ok(BitgetOrderResponse {
            success: true,
            order: Some(BitgetOrderData {
                order_id: order.order_id,
                client_oid: order.client_oid,
                symbol: order.symbol,
                side: order.side,
                order_type: order.order_type,
                amount: order.amount,
                price: order.price,
                filled_amount: order.filled_amount,
                average_price: order.average_price,
                status: order.status,
                fee: order.fee,
                fee_currency: order.fee_currency,
                created_at: order.created_at,
            }),
            error: None,
        }),
        Err(e) => Ok(BitgetOrderResponse {
            success: false,
            order: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Быстрая продажа (market order)
#[tauri::command]
pub async fn bitget_quick_sell(
    symbol: String,
    amount: String,
    state: State<'_, BitgetState>,
) -> Result<BitgetOrderResponse, String> {
    match state.client.execute_sell(symbol, amount).await {
        Ok(order) => Ok(BitgetOrderResponse {
            success: true,
            order: Some(BitgetOrderData {
                order_id: order.order_id,
                client_oid: order.client_oid,
                symbol: order.symbol,
                side: order.side,
                order_type: order.order_type,
                amount: order.amount,
                price: order.price,
                filled_amount: order.filled_amount,
                average_price: order.average_price,
                status: order.status,
                fee: order.fee,
                fee_currency: order.fee_currency,
                created_at: order.created_at,
            }),
            error: None,
        }),
        Err(e) => Ok(BitgetOrderResponse {
            success: false,
            order: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Рассчитать комиссию для суммы
#[tauri::command]
pub fn bitget_calculate_fee(amount: String, fee_bps: u64) -> Result<String, String> {
    super::bitget::calculate_fee(&amount, fee_bps).map_err(|e: Web3Error| e.to_string())
}

// ============================================================================
// Command Registration
// ============================================================================

/// Зарегистрировать все Bitget команды
pub fn register_bitget_commands(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder
        .manage(BitgetState::new(
            None, // API key из конфига
            None, // Secret key из конфига
            None, // Passphrase из конфига
            250,  // 2.5% fee по умолчанию
        ))
        .invoke_handler(tauri::generate_handler![
            bitget_place_order,
            bitget_get_order_status,
            bitget_cancel_order,
            bitget_get_balance,
            bitget_get_market_price,
            bitget_get_symbols,
            bitget_quick_buy,
            bitget_quick_sell,
            bitget_calculate_fee,
        ])
}
