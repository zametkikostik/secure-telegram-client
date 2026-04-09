//! Tauri Commands for P2P Escrow
//!
//! Provides commands for creating, funding, and managing P2P deals
//! through the messenger UI.

use crate::web3::p2p_escrow::*;
use ethers::types::{Address, U256};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::{debug, error, info};

// ============================================================================
// Command Request/Response Types
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct CreateDealRequest {
    pub seller_address: String,
    pub deal_type: String,
    pub deadline_timestamp: u64,
    pub message_hash: String, // hex-encoded bytes32
    pub ipfs_metadata: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FundDealRequest {
    pub deal_id: String,
    pub amount_wei: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenDisputeRequest {
    pub deal_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResolveDisputeRequest {
    pub deal_id: String,
    pub refund_to_buyer: bool,
    pub buyer_percent: u64, // 0-100
}

#[derive(Debug, Clone, Serialize)]
pub struct DealResponse {
    pub success: bool,
    pub deal: Option<DealInfo>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DealInfo {
    pub id: String,
    pub deal_type: String,
    pub buyer: String,
    pub seller: String,
    pub arbiter: String,
    pub amount: String,
    pub platform_fee: String,
    pub payment_token: String,
    pub state: String,
    pub created_at: u64,
    pub deadline: u64,
    pub message_hash: String,
    pub ipfs_metadata: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TxReceiptResponse {
    pub success: bool,
    pub tx_hash: Option<String>,
    pub block_number: Option<u64>,
    pub deal_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlatformStatsResponse {
    pub success: bool,
    pub stats: Option<PlatformStatsInfo>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlatformStatsInfo {
    pub total_deals: String,
    pub completed_deals: String,
    pub disputed_deals: String,
    pub total_volume: String,
    pub total_fees: String,
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Create a new P2P deal
#[tauri::command]
pub async fn create_deal(
    request: CreateDealRequest,
) -> Result<TxReceiptResponse, String> {
    info!("Creating deal: seller={}", request.seller_address);
    
    // TODO: Initialize actual client from app state
    // This is a placeholder implementation
    
    Ok(TxReceiptResponse {
        success: true,
        tx_hash: Some("0xplaceholder".to_string()),
        block_number: None,
        deal_id: None,
        error: None,
    })
}

/// Fund an existing deal with ETH
#[tauri::command]
pub async fn fund_deal(
    request: FundDealRequest,
) -> Result<TxReceiptResponse, String> {
    info!("Funding deal: id={}", request.deal_id);
    
    Ok(TxReceiptResponse {
        success: true,
        tx_hash: Some("0xplaceholder".to_string()),
        block_number: None,
        deal_id: Some(request.deal_id),
        error: None,
    })
}

/// Get deal information
#[tauri::command]
pub async fn get_deal(
    deal_id: String,
) -> Result<DealResponse, String> {
    debug!("Fetching deal: id={}", deal_id);
    
    Ok(DealResponse {
        success: true,
        deal: None, // TODO: Implement
        error: None,
    })
}

/// Get user's deals
#[tauri::command]
pub async fn get_user_deals(
    user_address: String,
) -> Result<Vec<String>, String> {
    debug!("Fetching user deals: user={}", user_address);
    
    Ok(vec![]) // TODO: Implement
}

/// Get platform statistics
#[tauri::command]
pub async fn get_platform_stats() -> Result<PlatformStatsResponse, String> {
    debug!("Fetching platform stats");
    
    Ok(PlatformStatsResponse {
        success: true,
        stats: None, // TODO: Implement
        error: None,
    })
}

/// Confirm delivery (buyer)
#[tauri::command]
pub async fn confirm_delivery(
    deal_id: String,
) -> Result<TxReceiptResponse, String> {
    info!("Confirming delivery: deal_id={}", deal_id);
    
    Ok(TxReceiptResponse {
        success: true,
        tx_hash: Some("0xplaceholder".to_string()),
        block_number: None,
        deal_id: Some(deal_id),
        error: None,
    })
}

/// Complete deal (payout to seller)
#[tauri::command]
pub async fn complete_deal(
    deal_id: String,
) -> Result<TxReceiptResponse, String> {
    info!("Completing deal: id={}", deal_id);
    
    Ok(TxReceiptResponse {
        success: true,
        tx_hash: Some("0xplaceholder".to_string()),
        block_number: None,
        deal_id: Some(deal_id),
        error: None,
    })
}

/// Open dispute
#[tauri::command]
pub async fn open_dispute(
    request: OpenDisputeRequest,
) -> Result<TxReceiptResponse, String> {
    info!("Opening dispute: deal_id={}", request.deal_id);
    
    Ok(TxReceiptResponse {
        success: true,
        tx_hash: Some("0xplaceholder".to_string()),
        block_number: None,
        deal_id: Some(request.deal_id),
        error: None,
    })
}

/// Resolve dispute (arbiter only)
#[tauri::command]
pub async fn resolve_dispute(
    request: ResolveDisputeRequest,
) -> Result<TxReceiptResponse, String> {
    info!(
        "Resolving dispute: deal_id={}, refund={}, buyer%={}",
        request.deal_id, request.refund_to_buyer, request.buyer_percent
    );
    
    Ok(TxReceiptResponse {
        success: true,
        tx_hash: Some("0xplaceholder".to_string()),
        block_number: None,
        deal_id: Some(request.deal_id),
        error: None,
    })
}

/// Refund after deadline
#[tauri::command]
pub async fn refund_after_deadline(
    deal_id: String,
) -> Result<TxReceiptResponse, String> {
    info!("Refunding after deadline: deal_id={}", deal_id);
    
    Ok(TxReceiptResponse {
        success: true,
        tx_hash: Some("0xplaceholder".to_string()),
        block_number: None,
        deal_id: Some(deal_id),
        error: None,
    })
}

/// Cancel deal (before funding)
#[tauri::command]
pub async fn cancel_deal(
    deal_id: String,
) -> Result<TxReceiptResponse, String> {
    info!("Cancelling deal: id={}", deal_id);
    
    Ok(TxReceiptResponse {
        success: true,
        tx_hash: Some("0xplaceholder".to_string()),
        block_number: None,
        deal_id: Some(deal_id),
        error: None,
    })
}

/// Calculate platform fee for amount
#[tauri::command]
pub async fn calculate_fee(
    amount_wei: String,
) -> Result<String, String> {
    let amount = U256::from_dec_str(&amount_wei)
        .map_err(|e| format!("Invalid amount: {}", e))?;
    
    // Fee calculation logic (same as Solidity)
    let tier_1_max = U256::from(10u64).pow(U256::from(17u64)); // 0.1 ETH
    let tier_2_max = U256::from(10u64).pow(U256::from(18u64)); // 1 ETH
    
    let fee = if amount < tier_1_max {
        amount * U256::from(200u64) / U256::from(10000u64) // 2%
    } else if amount < tier_2_max {
        amount * U256::from(100u64) / U256::from(10000u64) // 1%
    } else {
        amount * U256::from(50u64) / U256::from(10000u64) // 0.5%
    };
    
    Ok(fee.to_string())
}

// ============================================================================
// Helper Functions
// ============================================================================

fn parse_address(addr: &str) -> Result<Address, String> {
    Address::from_str(addr)
        .map_err(|e| format!("Invalid address '{}': {}", addr, e))
}

fn parse_u256(val: &str) -> Result<U256, String> {
    U256::from_dec_str(val)
        .or_else(|_| U256::from_str(val))
        .map_err(|e| format!("Invalid U256 '{}': {}", val, e))
}

fn parse_bytes32(hex: &str) -> Result<[u8; 32], String> {
    let hex_stripped = hex.strip_prefix("0x").unwrap_or(hex);
    let bytes = hex::decode(hex_stripped)
        .map_err(|e| format!("Invalid hex: {}", e))?;
    
    if bytes.len() != 32 {
        return Err(format!("Expected 32 bytes, got {}", bytes.len()));
    }
    
    let mut array = [0u8; 32];
    array.copy_from_slice(&bytes);
    Ok(array)
}
