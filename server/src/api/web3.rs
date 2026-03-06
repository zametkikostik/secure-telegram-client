// server/src/api/web3.rs
//! Web3 API (0x, ABCEX, Bitget)

use axum::{
    extract::State,
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use crate::api::AppState;

#[derive(Serialize)]
pub struct BalanceResponse {
    pub eth: String,
    pub usdt: String,
    pub usdc: String,
}

#[derive(Deserialize)]
pub struct SwapRequest {
    pub from_token: String,
    pub to_token: String,
    pub amount: String,
}

#[derive(Serialize)]
pub struct SwapResponse {
    pub quote_id: String,
    pub rate: String,
    pub fee: String,
    pub total: String,
}

/// Получить баланс
pub async fn get_balance(
    // State(state): State<AppState>,
    // claims: Claims,
) -> Result<Json<BalanceResponse>, StatusCode> {
    // В реальности здесь будет запрос к Web3 провайдеру
    Ok(Json(BalanceResponse {
        eth: "1.5".to_string(),
        usdt: "1000.0".to_string(),
        usdc: "500.0".to_string(),
    }))
}

/// Обменять токены через 0x Protocol
pub async fn swap_tokens(
    State(state): State<AppState>,
    Json(req): Json<SwapRequest>,
) -> Result<Json<SwapResponse>, StatusCode> {
    // Запрос к 0x API
    let client = reqwest::Client::new();
    let admin_wallet = std::env::var("ADMIN_WALLET").unwrap_or_default();
    let fee_percentage = 200; // 2%

    let response = client
        .get("https://api.0x.org/swap/v1/quote")
        .query(&[
            ("buyToken", &req.to_token),
            ("sellToken", &req.from_token),
            ("sellAmount", &req.amount),
            ("affiliateAddress", &admin_wallet),
            ("feeRecipient", &admin_wallet),
            ("buyTokenPercentageFee", &fee_percentage.to_string()),
        ])
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Ошибка 0x API: {}", e);
            StatusCode::BAD_GATEWAY
        })?
        .json::<serde_json::Value>()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let rate = response["price"].as_str().unwrap_or("1.0");
    let fee = response["estimatedAmountInEthFees"].as_str().unwrap_or("0");
    let total = response["buyAmount"].as_str().unwrap_or(&req.amount);

    Ok(Json(SwapResponse {
        quote_id: uuid::Uuid::new_v4().to_string(),
        rate: rate.to_string(),
        fee: fee.to_string(),
        total: total.to_string(),
    }))
}
