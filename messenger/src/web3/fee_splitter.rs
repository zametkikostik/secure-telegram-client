//! Fee Splitter Smart Contract Integration
//!
//! Provides Rust bindings for the FeeSplitter Solidity contract.
//! Enables automatic distribution of platform fees among team, treasury,
//! marketing, arbiters, and reserve fund.
//!
//! # Features
//! - Receive fees from P2PEscrow contract
//! - Automatic distribution (40/25/15/10/10 by default)
//! - Arbiter pool management
//! - Emergency pause/resume
//! - ERC-20 and ETH support
//! - Transparent statistics
//!
//! # Default Distribution
//! - 40% → Development Team
//! - 25% → Platform Treasury
//! - 15% → Marketing & Growth
//! - 10% → Arbiters Pool
//! - 10% → Reserve Fund

use ethers::{
    core::types::{Address, TransactionReceipt, TxHash, U256},
    providers::{Middleware, Provider, ProviderError},
    contract::Contract,
    abi::Abi,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info, warn};

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum FeeSplitterError {
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),

    #[error("Contract error: {0}")]
    Contract(String),

    #[error("Transaction failed: {0}")]
    TxFailed(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Invalid percentage: {0}")]
    InvalidPercentage(String),

    #[error("Contract paused")]
    Paused,

    #[error("No pending balance")]
    NoPendingBalance,

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("ABI error: {0}")]
    Abi(String),
}

pub type FeeSplitterResult<T> = Result<T, FeeSplitterError>;

// ============================================================================
// Structs
// ============================================================================

/// Shareholder information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shareholder {
    pub wallet: Address,
    pub share_percent: u8,
    pub role: String,
    pub is_active: bool,
    pub total_received: U256,
    pub last_claimed_at: u64,
}

/// Distribution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionStats {
    pub total_distributed: U256,
    pub total_fees_received: U256,
    pub distribution_count: U256,
    pub last_distribution_at: u64,
    pub pending_balance: U256,
}

/// Arbiter pool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbiterPoolInfo {
    pub total_shares: U256,
    pub total_balance: U256,
    pub my_share: U256,
}

/// Distribution configuration
#[derive(Debug, Clone)]
pub struct DistributionConfig {
    pub team_percent: u8,
    pub treasury_percent: u8,
    pub marketing_percent: u8,
    pub arbiters_percent: u8,
    pub reserve_percent: u8,
}

impl DistributionConfig {
    pub fn validate(&self) -> FeeSplitterResult<()> {
        let total = self.team_percent as u16
            + self.treasury_percent as u16
            + self.marketing_percent as u16
            + self.arbiters_percent as u16
            + self.reserve_percent as u16;
        
        if total != 100 {
            return Err(FeeSplitterError::InvalidPercentage(
                format!("Shares must sum to 100, got {}", total)
            ));
        }
        
        Ok(())
    }

    pub fn default_config() -> Self {
        Self {
            team_percent: 40,
            treasury_percent: 25,
            marketing_percent: 15,
            arbiters_percent: 10,
            reserve_percent: 10,
        }
    }
}

/// Distribution log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionLog {
    pub timestamp: u64,
    pub amount: U256,
    pub token: Address,
    pub recipients: Vec<Address>,
    pub amounts: Vec<U256>,
}

/// Transaction receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeSplitterTxReceipt {
    pub tx_hash: TxHash,
    pub block_number: Option<u64>,
    pub gas_used: Option<U256>,
    pub status: bool,
    pub events: Vec<String>,
}

// ============================================================================
// Fee Splitter Client
// ============================================================================

/// Client for interacting with the FeeSplitter smart contract
pub struct FeeSplitterClient<M: Middleware> {
    contract: Contract<M>,
    provider: Arc<M>,
    contract_address: Address,
}

impl<M: Middleware> FeeSplitterClient<M>
where
    M::Error: 'static,
{
    /// Create a new fee splitter client
    pub fn new(
        provider: Arc<M>,
        contract_address: Address,
        abi: &str,
    ) -> FeeSplitterResult<Self> {
        let abi: Abi = serde_json::from_str(abi).map_err(|e| FeeSplitterError::Abi(e.to_string()))?;
        
        let contract = Contract::new(
            contract_address,
            abi,
            provider.clone(),
        );

        info!(
            "FeeSplitterClient initialized: address={}",
            contract_address
        );

        Ok(Self {
            contract,
            provider,
            contract_address,
        })
    }

    // ========================================================================
    // Read Functions
    // ========================================================================

    /// Get contract owner
    pub async fn owner(&self) -> FeeSplitterResult<Address> {
        let owner = self
            .contract
            .method::<_, Address>("owner", ())
            .map_err(|e| FeeSplitterError::Contract(e.to_string()))?
            .call()
            .await
            .map_err(|e| FeeSplitterError::Provider(e))?;

        Ok(owner)
    }

    /// Get distribution percentages
    pub async fn get_distribution_config(&self) -> FeeSplitterResult<DistributionConfig> {
        let team_percent = self
            .contract
            .method::<_, u8>("teamPercent", ())
            .map_err(|e| FeeSplitterError::Contract(e.to_string()))?
            .call()
            .await
            .map_err(|e| FeeSplitterError::Provider(e))?;

        let treasury_percent = self
            .contract
            .method::<_, u8>("treasuryPercent", ())
            .map_err(|e| FeeSplitterError::Contract(e.to_string()))?
            .call()
            .await
            .map_err(|e| FeeSplitterError::Provider(e))?;

        let marketing_percent = self
            .contract
            .method::<_, u8>("marketingPercent", ())
            .map_err(|e| FeeSplitterError::Contract(e.to_string()))?
            .call()
            .await
            .map_err(|e| FeeSplitterError::Provider(e))?;

        let arbiters_percent = self
            .contract
            .method::<_, u8>("arbitersPercent", ())
            .map_err(|e| FeeSplitterError::Contract(e.to_string()))?
            .call()
            .await
            .map_err(|e| FeeSplitterError::Provider(e))?;

        let reserve_percent = self
            .contract
            .method::<_, u8>("reservePercent", ())
            .map_err(|e| FeeSplitterError::Contract(e.to_string()))?
            .call()
            .await
            .map_err(|e| FeeSplitterError::Provider(e))?;

        Ok(DistributionConfig {
            team_percent,
            treasury_percent,
            marketing_percent,
            arbiters_percent,
            reserve_percent,
        })
    }

    /// Get all shareholders
    pub async fn get_shareholders(&self) -> FeeSplitterResult<(Shareholder, Shareholder, Shareholder, Shareholder)> {
        let result = self
            .contract
            .method::<_, (
                (Address, u8, String, bool, U256, u64),
                (Address, u8, String, bool, U256, u64),
                (Address, u8, String, bool, U256, u64),
                (Address, u8, String, bool, U256, u64),
            )>("getShareholders", ())
            .map_err(|e| FeeSplitterError::Contract(e.to_string()))?
            .call()
            .await
            .map_err(|e| FeeSplitterError::Provider(e))?;

        let team = Shareholder {
            wallet: result.0.0,
            share_percent: result.0.1,
            role: result.0.2,
            is_active: result.0.3,
            total_received: result.0.4,
            last_claimed_at: result.0.5,
        };

        let treasury = Shareholder {
            wallet: result.1.0,
            share_percent: result.1.1,
            role: result.1.2,
            is_active: result.1.3,
            total_received: result.1.4,
            last_claimed_at: result.1.5,
        };

        let marketing = Shareholder {
            wallet: result.2.0,
            share_percent: result.2.1,
            role: result.2.2,
            is_active: result.2.3,
            total_received: result.2.4,
            last_claimed_at: result.2.5,
        };

        let reserve = Shareholder {
            wallet: result.3.0,
            share_percent: result.3.1,
            role: result.3.2,
            is_active: result.3.3,
            total_received: result.3.4,
            last_claimed_at: result.3.5,
        };

        Ok((team, treasury, marketing, reserve))
    }

    /// Get distribution statistics
    pub async fn get_stats(&self) -> FeeSplitterResult<DistributionStats> {
        let result = self
            .contract
            .method::<_, (U256, U256, U256, u64, U256)>("stats", ())
            .map_err(|e| FeeSplitterError::Contract(e.to_string()))?
            .call()
            .await
            .map_err(|e| FeeSplitterError::Provider(e))?;

        Ok(DistributionStats {
            total_distributed: result.0,
            total_fees_received: result.1,
            distribution_count: result.2,
            last_distribution_at: result.3,
            pending_balance: result.4,
        })
    }

    /// Get arbiter pool information
    pub async fn get_arbiter_pool_info(&self) -> FeeSplitterResult<ArbiterPoolInfo> {
        let result = self
            .contract
            .method::<_, (U256, U256, U256)>("getArbiterPoolInfo", ())
            .map_err(|e| FeeSplitterError::Contract(e.to_string()))?
            .call()
            .await
            .map_err(|e| FeeSplitterError::Provider(e))?;

        Ok(ArbiterPoolInfo {
            total_shares: result.0,
            total_balance: result.1,
            my_share: result.2,
        })
    }

    /// Check if contract is paused
    pub async fn is_paused(&self) -> FeeSplitterResult<bool> {
        let paused = self
            .contract
            .method::<_, bool>("paused", ())
            .map_err(|e| FeeSplitterError::Contract(e.to_string()))?
            .call()
            .await
            .map_err(|e| FeeSplitterError::Provider(e))?;

        Ok(paused)
    }

    /// Calculate pending distribution amount
    pub async fn pending_distribution(&self) -> FeeSplitterResult<U256> {
        let stats = self.get_stats().await?;
        Ok(stats.pending_balance)
    }

    // ========================================================================
    // Write Functions
    // ========================================================================

    /// Distribute accumulated fees (ETH)
    pub async fn distribute(&self) -> FeeSplitterResult<FeeSplitterTxReceipt> {
        info!("Distributing fees");

        let tx = self
            .contract
            .method::<_, ()>("distribute", ())
            .map_err(|e| FeeSplitterError::Contract(e.to_string()))?;

        let pending_tx = tx.send().await.map_err(|e| FeeSplitterError::TxFailed(e.to_string()))?;
        
        let receipt = pending_tx
            .await
            .map_err(|e| FeeSplitterError::TxFailed(e.to_string()))?
            .ok_or_else(|| FeeSplitterError::TxFailed("No receipt".to_string()))?;

        let tx_receipt = self.parse_receipt(&receipt, "Distributed")?;

        info!("Fees distributed: tx={:?}", receipt.transaction_hash);

        Ok(tx_receipt)
    }

    /// Distribute ERC-20 tokens
    pub async fn distribute_token(&self, token_address: Address) -> FeeSplitterResult<FeeSplitterTxReceipt> {
        info!("Distributing tokens: {}", token_address);

        let tx = self
            .contract
            .method::<_, ()>("distributeToken", token_address)
            .map_err(|e| FeeSplitterError::Contract(e.to_string()))?;

        let pending_tx = tx.send().await.map_err(|e| FeeSplitterError::TxFailed(e.to_string()))?;
        
        let receipt = pending_tx
            .await
            .map_err(|e| FeeSplitterError::TxFailed(e.to_string()))?
            .ok_or_else(|| FeeSplitterError::TxFailed("No receipt".to_string()))?;

        let tx_receipt = self.parse_receipt(&receipt, "Distributed")?;

        info!("Tokens distributed: tx={:?}", receipt.transaction_hash);

        Ok(tx_receipt)
    }

    /// Update distribution shares (owner only)
    pub async fn update_shares(&self, config: DistributionConfig) -> FeeSplitterResult<FeeSplitterTxReceipt> {
        config.validate()?;
        
        info!("Updating shares: {:?}", config);

        let tx = self
            .contract
            .method::<_, ()>(
                "updateShares",
                (
                    config.team_percent,
                    config.treasury_percent,
                    config.marketing_percent,
                    config.arbiters_percent,
                    config.reserve_percent,
                ),
            )
            .map_err(|e| FeeSplitterError::Contract(e.to_string()))?;

        let pending_tx = tx.send().await.map_err(|e| FeeSplitterError::TxFailed(e.to_string()))?;
        
        let receipt = pending_tx
            .await
            .map_err(|e| FeeSplitterError::TxFailed(e.to_string()))?
            .ok_or_else(|| FeeSplitterError::TxFailed("No receipt".to_string()))?;

        let tx_receipt = self.parse_receipt(&receipt, "ShareholderUpdated")?;

        info!("Shares updated: tx={:?}", receipt.transaction_hash);

        Ok(tx_receipt)
    }

    /// Update shareholder wallet (owner only)
    pub async fn update_shareholder_wallet(&self, role: String, new_wallet: Address) -> FeeSplitterResult<FeeSplitterTxReceipt> {
        info!("Updating {} wallet to {}", role, new_wallet);

        let tx = self
            .contract
            .method::<_, ()>("updateShareholderWallet", (role, new_wallet))
            .map_err(|e| FeeSplitterError::Contract(e.to_string()))?;

        let pending_tx = tx.send().await.map_err(|e| FeeSplitterError::TxFailed(e.to_string()))?;
        
        let receipt = pending_tx
            .await
            .map_err(|e| FeeSplitterError::TxFailed(e.to_string()))?
            .ok_or_else(|| FeeSplitterError::TxFailed("No receipt".to_string()))?;

        let tx_receipt = self.parse_receipt(&receipt, "ShareholderUpdated")?;

        Ok(tx_receipt)
    }

    /// Add arbiter to pool (owner only)
    pub async fn add_arbiter(&self, arbiter: Address, share: U256) -> FeeSplitterResult<FeeSplitterTxReceipt> {
        info!("Adding arbiter: {}, share: {}", arbiter, share);

        let tx = self
            .contract
            .method::<_, ()>("addArbiter", (arbiter, share))
            .map_err(|e| FeeSplitterError::Contract(e.to_string()))?;

        let pending_tx = tx.send().await.map_err(|e| FeeSplitterError::TxFailed(e.to_string()))?;
        
        let receipt = pending_tx
            .await
            .map_err(|e| FeeSplitterError::TxFailed(e.to_string()))?
            .ok_or_else(|| FeeSplitterError::TxFailed("No receipt".to_string()))?;

        let tx_receipt = self.parse_receipt(&receipt, "ArbiterAdded")?;

        Ok(tx_receipt)
    }

    /// Remove arbiter from pool (owner only)
    pub async fn remove_arbiter(&self, arbiter: Address) -> FeeSplitterResult<FeeSplitterTxReceipt> {
        info!("Removing arbiter: {}", arbiter);

        let tx = self
            .contract
            .method::<_, ()>("removeArbiter", arbiter)
            .map_err(|e| FeeSplitterError::Contract(e.to_string()))?;

        let pending_tx = tx.send().await.map_err(|e| FeeSplitterError::TxFailed(e.to_string()))?;
        
        let receipt = pending_tx
            .await
            .map_err(|e| FeeSplitterError::TxFailed(e.to_string()))?
            .ok_or_else(|| FeeSplitterError::TxFailed("No receipt".to_string()))?;

        let tx_receipt = self.parse_receipt(&receipt, "ArbiterRemoved")?;

        Ok(tx_receipt)
    }

    /// Arbiter withdraw their share
    pub async fn arbiter_withdraw(&self) -> FeeSplitterResult<FeeSplitterTxReceipt> {
        info!("Arbiter withdrawing");

        let tx = self
            .contract
            .method::<_, ()>("arbiterWithdraw", ())
            .map_err(|e| FeeSplitterError::Contract(e.to_string()))?;

        let pending_tx = tx.send().await.map_err(|e| FeeSplitterError::TxFailed(e.to_string()))?;
        
        let receipt = pending_tx
            .await
            .map_err(|e| FeeSplitterError::TxFailed(e.to_string()))?
            .ok_or_else(|| FeeSplitterError::TxFailed("No receipt".to_string()))?;

        let tx_receipt = self.parse_receipt(&receipt, "ArbiterWithdrawal")?;

        Ok(tx_receipt)
    }

    /// Emergency pause/unpause (owner only)
    pub async fn toggle_pause(&self) -> FeeSplitterResult<FeeSplitterTxReceipt> {
        info!("Toggling pause");

        let tx = self
            .contract
            .method::<_, ()>("togglePause", ())
            .map_err(|e| FeeSplitterError::Contract(e.to_string()))?;

        let pending_tx = tx.send().await.map_err(|e| FeeSplitterError::TxFailed(e.to_string()))?;
        
        let receipt = pending_tx
            .await
            .map_err(|e| FeeSplitterError::TxFailed(e.to_string()))?
            .ok_or_else(|| FeeSplitterError::TxFailed("No receipt".to_string()))?;

        let tx_receipt = self.parse_receipt(&receipt, "EmergencyPause")?;

        Ok(tx_receipt)
    }

    /// Transfer ownership (owner only)
    pub async fn transfer_ownership(&self, new_owner: Address) -> FeeSplitterResult<FeeSplitterTxReceipt> {
        info!("Transferring ownership to {}", new_owner);

        let tx = self
            .contract
            .method::<_, ()>("transferOwnership", new_owner)
            .map_err(|e| FeeSplitterError::Contract(e.to_string()))?;

        let pending_tx = tx.send().await.map_err(|e| FeeSplitterError::TxFailed(e.to_string()))?;
        
        let receipt = pending_tx
            .await
            .map_err(|e| FeeSplitterError::TxFailed(e.to_string()))?
            .ok_or_else(|| FeeSplitterError::TxFailed("No receipt".to_string()))?;

        let tx_receipt = self.parse_receipt(&receipt, "OwnershipTransferred")?;

        Ok(tx_receipt)
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    fn parse_receipt(
        &self,
        receipt: &TransactionReceipt,
        event_name: &str,
    ) -> FeeSplitterResult<FeeSplitterTxReceipt> {
        let events = vec![event_name.to_string()];

        Ok(FeeSplitterTxReceipt {
            tx_hash: receipt.transaction_hash,
            block_number: receipt.block_number.map(|n| n.as_u64()),
            gas_used: receipt.gas_used,
            status: receipt.status.unwrap_or_default().as_u64() == 1,
            events,
        })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DistributionConfig::default_config();
        
        assert_eq!(config.team_percent, 40);
        assert_eq!(config.treasury_percent, 25);
        assert_eq!(config.marketing_percent, 15);
        assert_eq!(config.arbiters_percent, 10);
        assert_eq!(config.reserve_percent, 10);
    }

    #[test]
    fn test_config_validation_valid() {
        let config = DistributionConfig {
            team_percent: 40,
            treasury_percent: 25,
            marketing_percent: 15,
            arbiters_percent: 10,
            reserve_percent: 10,
        };
        
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_invalid_sum() {
        let config = DistributionConfig {
            team_percent: 50,
            treasury_percent: 30,
            marketing_percent: 20,
            arbiters_percent: 10,
            reserve_percent: 10, // Sum = 120
        };
        
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_shareholder_serialization() {
        let shareholder = Shareholder {
            wallet: Address::zero(),
            share_percent: 40,
            role: "Team".to_string(),
            is_active: true,
            total_received: U256::from(1_000_000_000_000_000_000u64), // 1 ETH
            last_claimed_at: 1234567890,
        };

        let json = serde_json::to_string(&shareholder).unwrap();
        let deserialized: Shareholder = serde_json::from_str(&json).unwrap();

        assert_eq!(shareholder.wallet, deserialized.wallet);
        assert_eq!(shareholder.share_percent, deserialized.share_percent);
        assert_eq!(shareholder.role, deserialized.role);
    }

    #[test]
    fn test_distribution_stats_serialization() {
        let stats = DistributionStats {
            total_distributed: U256::from(10_000_000_000_000_000_000u128),
            total_fees_received: U256::from(10_500_000_000_000_000_000u128),
            distribution_count: U256::from(100),
            last_distribution_at: 1234567890,
            pending_balance: U256::from(500_000_000_000_000_000u64),
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: DistributionStats = serde_json::from_str(&json).unwrap();

        assert_eq!(stats.total_distributed, deserialized.total_distributed);
        assert_eq!(stats.distribution_count, deserialized.distribution_count);
    }
}
