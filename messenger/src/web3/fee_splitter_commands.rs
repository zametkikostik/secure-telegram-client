//! Tauri Commands for Fee Splitter
//!
//! Provides commands for managing fee distribution through the messenger UI.

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateSharesRequest {
    pub team_percent: u8,
    pub treasury_percent: u8,
    pub marketing_percent: u8,
    pub arbiters_percent: u8,
    pub reserve_percent: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateWalletRequest {
    pub role: String,
    pub new_wallet_address: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AddArbiterRequest {
    pub arbiter_address: String,
    pub share: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DistributionConfigResponse {
    pub success: bool,
    pub team_percent: u8,
    pub treasury_percent: u8,
    pub marketing_percent: u8,
    pub arbiters_percent: u8,
    pub reserve_percent: u8,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShareholderInfo {
    pub wallet: String,
    pub share_percent: u8,
    pub role: String,
    pub is_active: bool,
    pub total_received: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ShareholdersResponse {
    pub success: bool,
    pub team: Option<ShareholderInfo>,
    pub treasury: Option<ShareholderInfo>,
    pub marketing: Option<ShareholderInfo>,
    pub reserve: Option<ShareholderInfo>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DistributionStatsResponse {
    pub success: bool,
    pub total_distributed: String,
    pub total_fees_received: String,
    pub distribution_count: String,
    pub pending_balance: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TxResponse {
    pub success: bool,
    pub tx_hash: Option<String>,
    pub block_number: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArbiterPoolResponse {
    pub success: bool,
    pub total_shares: String,
    pub total_balance: String,
    pub my_share: String,
    pub error: Option<String>,
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Get current distribution configuration
#[tauri::command]
pub async fn get_distribution_config() -> Result<DistributionConfigResponse, String> {
    debug!("Fetching distribution config");
    
    // TODO: Initialize actual client from app state
    Ok(DistributionConfigResponse {
        success: true,
        team_percent: 40,
        treasury_percent: 25,
        marketing_percent: 15,
        arbiters_percent: 10,
        reserve_percent: 10,
        error: None,
    })
}

/// Get all shareholders information
#[tauri::command]
pub async fn get_shareholders() -> Result<ShareholdersResponse, String> {
    debug!("Fetching shareholders");
    
    Ok(ShareholdersResponse {
        success: true,
        team: None,
        treasury: None,
        marketing: None,
        reserve: None,
        error: None,
    })
}

/// Get distribution statistics
#[tauri::command]
pub async fn get_distribution_stats() -> Result<DistributionStatsResponse, String> {
    debug!("Fetching distribution stats");
    
    Ok(DistributionStatsResponse {
        success: true,
        total_distributed: "0".to_string(),
        total_fees_received: "0".to_string(),
        distribution_count: "0".to_string(),
        pending_balance: "0".to_string(),
        error: None,
    })
}

/// Execute distribution (ETH)
#[tauri::command]
pub async fn distribute_fees() -> Result<TxResponse, String> {
    info!("Distributing fees");
    
    Ok(TxResponse {
        success: true,
        tx_hash: Some("0xplaceholder".to_string()),
        block_number: None,
        error: None,
    })
}

/// Update distribution shares
#[tauri::command]
pub async fn update_shares(
    request: UpdateSharesRequest,
) -> Result<TxResponse, String> {
    let total = request.team_percent as u16
        + request.treasury_percent as u16
        + request.marketing_percent as u16
        + request.arbiters_percent as u16
        + request.reserve_percent as u16;
    
    if total != 100 {
        return Err(format!("Shares must sum to 100, got {}", total));
    }
    
    info!(
        "Updating shares: team={}, treasury={}, marketing={}, arbiters={}, reserve={}",
        request.team_percent,
        request.treasury_percent,
        request.marketing_percent,
        request.arbiters_percent,
        request.reserve_percent
    );
    
    Ok(TxResponse {
        success: true,
        tx_hash: Some("0xplaceholder".to_string()),
        block_number: None,
        error: None,
    })
}

/// Update shareholder wallet
#[tauri::command]
pub async fn update_shareholder_wallet(
    request: UpdateWalletRequest,
) -> Result<TxResponse, String> {
    info!("Updating {} wallet to {}", request.role, request.new_wallet_address);
    
    Ok(TxResponse {
        success: true,
        tx_hash: Some("0xplaceholder".to_string()),
        block_number: None,
        error: None,
    })
}

/// Add arbiter to pool
#[tauri::command]
pub async fn add_arbiter(
    request: AddArbiterRequest,
) -> Result<TxResponse, String> {
    info!(
        "Adding arbiter: {}, share: {}",
        request.arbiter_address, request.share
    );
    
    Ok(TxResponse {
        success: true,
        tx_hash: Some("0xplaceholder".to_string()),
        block_number: None,
        error: None,
    })
}

/// Remove arbiter from pool
#[tauri::command]
pub async fn remove_arbiter(
    arbiter_address: String,
) -> Result<TxResponse, String> {
    info!("Removing arbiter: {}", arbiter_address);
    
    Ok(TxResponse {
        success: true,
        tx_hash: Some("0xplaceholder".to_string()),
        block_number: None,
        error: None,
    })
}

/// Arbiter withdraw their share
#[tauri::command]
pub async fn arbiter_withdraw() -> Result<TxResponse, String> {
    info!("Arbiter withdrawing");
    
    Ok(TxResponse {
        success: true,
        tx_hash: Some("0xplaceholder".to_string()),
        block_number: None,
        error: None,
    })
}

/// Get arbiter pool information
#[tauri::command]
pub async fn get_arbiter_pool_info() -> Result<ArbiterPoolResponse, String> {
    debug!("Fetching arbiter pool info");
    
    Ok(ArbiterPoolResponse {
        success: true,
        total_shares: "0".to_string(),
        total_balance: "0".to_string(),
        my_share: "0".to_string(),
        error: None,
    })
}

/// Emergency pause/unpause
#[tauri::command]
pub async fn toggle_pause() -> Result<TxResponse, String> {
    info!("Toggling pause");
    
    Ok(TxResponse {
        success: true,
        tx_hash: Some("0xplaceholder".to_string()),
        block_number: None,
        error: None,
    })
}

/// Transfer ownership
#[tauri::command]
pub async fn transfer_ownership(
    new_owner_address: String,
) -> Result<TxResponse, String> {
    info!("Transferring ownership to {}", new_owner_address);
    
    Ok(TxResponse {
        success: true,
        tx_hash: Some("0xplaceholder".to_string()),
        block_number: None,
        error: None,
    })
}

/// Calculate distribution amounts for a given fee
#[tauri::command]
pub async fn calculate_distribution(
    fee_amount_wei: String,
) -> Result<serde_json::Value, String> {
    let fee: u128 = fee_amount_wei
        .parse()
        .map_err(|e| format!("Invalid amount: {}", e))?;
    
    // Default: 40/25/15/10/10
    let team = fee * 40 / 100;
    let treasury = fee * 25 / 100;
    let marketing = fee * 15 / 100;
    let arbiters = fee * 10 / 100;
    let reserve = fee * 10 / 100;
    
    Ok(serde_json::json!({
        "success": true,
        "total": fee_amount_wei,
        "breakdown": {
            "team": team.to_string(),
            "treasury": treasury.to_string(),
            "marketing": marketing.to_string(),
            "arbiters": arbiters.to_string(),
            "reserve": reserve.to_string()
        }
    }))
}
