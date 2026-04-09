//! Tauri Commands для 0x Swap Integration
//!
//! Команды для вызова из JavaScript/TypeScript UI

use serde::{Deserialize, Serialize};
use tauri::State;

use super::zerox_swap::{ZeroExClient, QuoteRequest};
use super::types::Web3Error;
use super::Chain;

// ============================================================================
// Command Request/Response Types
// ============================================================================

/// Запрос котировки из UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapQuoteRequest {
    pub chain_id: u64,
    pub sell_token: String,
    pub buy_token: String,
    pub sell_amount: String,
    pub buy_amount: Option<String>,
    pub slippage_bps: Option<u32>,
    pub taker_address: String,
}

/// Ответ котировки для UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapQuoteResponse {
    pub success: bool,
    pub quote: Option<QuoteData>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteData {
    pub sell_token: String,
    pub buy_token: String,
    pub sell_amount: String,
    pub buy_amount: String,
    pub price: String,
    pub gas_estimate: String,
    pub fee_bps: u64,
    pub to: String,
    pub data: String,
    pub value: String,
}

/// Запрос на обмен
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapExecuteRequest {
    pub chain_id: u64,
    pub sell_token: String,
    pub buy_token: String,
    pub sell_amount: String,
    pub taker_address: String,
    pub slippage_bps: Option<u32>,
}

/// Результат обмена
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapExecuteResponse {
    pub success: bool,
    pub swap_record: Option<SwapRecordData>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapRecordData {
    pub id: String,
    pub chain: String,
    pub from_token: String,
    pub to_token: String,
    pub from_amount: String,
    pub to_amount: String,
    pub price: String,
    pub gas_estimate: String,
    pub fee_bps: u64,
    pub status: String,
}

// ============================================================================
// Tauri State
// ============================================================================

/// State для хранения 0x клиента
pub struct SwapState {
    pub client: ZeroExClient,
    pub api_key: Option<String>,
    pub fee_recipient: String,
    pub fee_bps: u64,
}

impl SwapState {
    pub fn new(api_key: Option<String>, fee_recipient: String, fee_bps: u64) -> Self {
        let client = ZeroExClient::with_fee(
            api_key.clone(),
            fee_recipient.clone(),
            fee_bps,
        );

        Self {
            client,
            api_key,
            fee_recipient,
            fee_bps,
        }
    }
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Получить котировку для обмена
#[tauri::command]
pub async fn get_swap_quote(
    request: SwapQuoteRequest,
    state: State<'_, SwapState>,
) -> Result<SwapQuoteResponse, String> {
    // Конвертировать chain_id в Chain
    let chain = Chain::from_chain_id(request.chain_id)
        .ok_or_else(|| format!("Unsupported chain: {}", request.chain_id))?;

    // Создать запрос котировки
    let quote_request = QuoteRequest {
        sell_token: request.sell_token,
        buy_token: request.buy_token,
        sell_amount: Some(request.sell_amount),
        buy_amount: request.buy_amount,
        slippage_bps: request.slippage_bps,
        taker_address: Some(request.taker_address),
        fee_recipient: Some(state.fee_recipient.clone()),
        buy_token_percentage_fee: Some(state.fee_bps),
    };

    // Получить котировку
    match state.client.get_quote(quote_request, chain).await {
        Ok(quote) => Ok(SwapQuoteResponse {
            success: true,
            quote: Some(QuoteData {
                sell_token: quote.sell_token,
                buy_token: quote.buy_token,
                sell_amount: quote.sell_amount,
                buy_amount: quote.buy_amount,
                price: quote.price,
                gas_estimate: quote.gas,
                fee_bps: state.fee_bps,
                to: quote.to,
                data: quote.data,
                value: quote.value,
            }),
            error: None,
        }),
        Err(e) => Ok(SwapQuoteResponse {
            success: false,
            quote: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Выполнить обмен
#[tauri::command]
pub async fn execute_swap(
    request: SwapExecuteRequest,
    state: State<'_, SwapState>,
) -> Result<SwapExecuteResponse, String> {
    // Конвертировать chain_id в Chain
    let chain = Chain::from_chain_id(request.chain_id)
        .ok_or_else(|| format!("Unsupported chain: {}", request.chain_id))?;

    // Выполнить обмен
    match state.client.execute_swap(
        chain,
        &request.sell_token,
        &request.buy_token,
        &request.sell_amount,
        &request.taker_address,
        request.slippage_bps,
    ).await {
        Ok(record) => Ok(SwapExecuteResponse {
            success: true,
            swap_record: Some(SwapRecordData {
                id: record.id,
                chain: format!("{:?}", record.chain),
                from_token: record.from_token,
                to_token: record.to_token,
                from_amount: record.from_amount,
                to_amount: record.to_amount,
                price: record.price,
                gas_estimate: record.gas_estimate,
                fee_bps: record.fee_bps,
                status: format!("{:?}", record.status),
            }),
            error: None,
        }),
        Err(e) => Ok(SwapExecuteResponse {
            success: false,
            swap_record: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Рассчитать рекомендуемое slippage
#[tauri::command]
pub fn calculate_slippage(
    token_symbol: String,
    gas_price_gwei: u64,
    state: State<'_, SwapState>,
) -> Result<u32, String> {
    Ok(state.client.calculate_slippage(&token_symbol, gas_price_gwei))
}

/// Получить адрес для allowance
#[tauri::command]
pub async fn get_allowance_target(
    token_address: String,
    chain_id: u64,
    state: State<'_, SwapState>,
) -> Result<String, String> {
    let chain = Chain::from_chain_id(chain_id)
        .ok_or_else(|| format!("Unsupported chain: {}", chain_id))?;

    state.client
        .get_allowance_target(&token_address, chain)
        .await
        .map_err(|e| e.to_string())
}

/// Быстрая котировка (human-readable)
#[tauri::command]
pub async fn quick_swap_quote(
    sell_token: String,
    buy_token: String,
    sell_amount: String,
    decimals: u8,
    chain_id: u64,
) -> Result<String, String> {
    let chain = Chain::from_chain_id(chain_id)
        .ok_or_else(|| format!("Unsupported chain: {}", chain_id))?;

    super::zerox_swap::quick_quote(
        &sell_token,
        &buy_token,
        &sell_amount,
        decimals,
        chain,
        None,
    )
    .await
    .map_err(|e: Web3Error| e.to_string())
}

// ============================================================================
// Command Registration
// ============================================================================

/// Зарегистрировать все swap команды
pub fn register_swap_commands(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder
        .manage(SwapState::new(
            None, // API key из конфига
            "0x0000000000000000000000000000000000000000".to_string(), // fee recipient
            100, // 1% fee
        ))
        .invoke_handler(tauri::generate_handler![
            get_swap_quote,
            execute_swap,
            calculate_slippage,
            get_allowance_target,
            quick_swap_quote,
        ])
}
