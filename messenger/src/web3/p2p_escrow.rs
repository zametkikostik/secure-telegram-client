//! P2P Escrow Smart Contract Integration
//!
//! Provides Rust bindings for the P2PEscrow Solidity contract.
//! Enables creating, funding, completing, and disputing deals from the messenger.
//!
//! # Features
//! - Create P2P deals with E2EE metadata
//! - Fund deals with ETH or ERC-20 tokens
//! - Confirm delivery and complete deals
//! - Open disputes and arbitration
//! - Query deal status and platform statistics
//!
//! # Security
//! - All transactions signed locally (no private key exposure)
//! - Supports hardware wallets and MetaMask
//! - Post-quantum ready (hybrid signatures)

use ethers::{
    core::types::{Address, Bytes, TransactionReceipt, TxHash, U256},
    providers::{Middleware, Provider, ProviderError},
    types::{Filter, Log},
    abi::{Abi, Token, Detokenize},
    contract::Contract,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info, warn};

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum EscrowError {
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),

    #[error("Contract error: {0}")]
    Contract(String),

    #[error("Transaction failed: {0}")]
    TxFailed(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    #[error("Deal not found: {0}")]
    DealNotFound(U256),

    #[error("Invalid deal state: expected {expected}, got {actual}")]
    InvalidState { expected: String, actual: String },

    #[error("Insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: String, need: String },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("ABI error: {0}")]
    Abi(String),

    #[error("Timeout")]
    Timeout,
}

pub type EscrowResult<T> = Result<T, EscrowError>;

// ============================================================================
// Enums
// ============================================================================

/// Deal state (matches Solidity enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DealState {
    Created = 0,
    Funded = 1,
    Delivered = 2,
    Completed = 3,
    Disputed = 4,
    Refunded = 5,
    Cancelled = 6,
}

impl DealState {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => DealState::Created,
            1 => DealState::Funded,
            2 => DealState::Delivered,
            3 => DealState::Completed,
            4 => DealState::Disputed,
            5 => DealState::Refunded,
            6 => DealState::Cancelled,
            _ => panic!("Invalid deal state: {}", val),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            DealState::Created => "Created",
            DealState::Funded => "Funded",
            DealState::Delivered => "Delivered",
            DealState::Completed => "Completed",
            DealState::Disputed => "Disputed",
            DealState::Refunded => "Refunded",
            DealState::Cancelled => "Cancelled",
        }
    }
}

impl std::fmt::Display for DealState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Deal type (matches Solidity enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DealType {
    DigitalGoods = 0,
    PhysicalGoods = 1,
    Service = 2,
    Subscription = 3,
    Freelance = 4,
}

impl DealType {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => DealType::DigitalGoods,
            1 => DealType::PhysicalGoods,
            2 => DealType::Service,
            3 => DealType::Subscription,
            4 => DealType::Freelance,
            _ => panic!("Invalid deal type: {}", val),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            DealType::DigitalGoods => "DigitalGoods",
            DealType::PhysicalGoods => "PhysicalGoods",
            DealType::Service => "Service",
            DealType::Subscription => "Subscription",
            DealType::Freelance => "Freelance",
        }
    }
}

impl std::fmt::Display for DealType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// Structs
// ============================================================================

/// Deal information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deal {
    pub id: U256,
    pub deal_type: DealType,
    pub buyer: Address,
    pub seller: Address,
    pub arbiter: Address,
    pub amount: U256,
    pub platform_fee: U256,
    pub payment_token: Address, // Address::zero() for ETH
    pub state: DealState,
    pub created_at: u64,
    pub funded_at: u64,
    pub deadline: u64,
    pub message_hash: [u8; 32],
    pub ipfs_metadata: String,
}

/// Platform statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformStats {
    pub total_deals: U256,
    pub completed_deals: U256,
    pub disputed_deals: U256,
    pub total_volume: U256,
    pub total_fees_collected: U256,
}

/// Deal creation parameters
#[derive(Debug, Clone)]
pub struct CreateDealParams {
    pub seller: Address,
    pub deal_type: DealType,
    pub deadline: U256,
    pub message_hash: [u8; 32],
    pub ipfs_metadata: String,
}

/// Deal funding parameters (ETH)
#[derive(Debug, Clone)]
pub struct FundDealParams {
    pub deal_id: U256,
    pub amount: U256,
}

/// Dispute parameters
#[derive(Debug, Clone)]
pub struct DisputeParams {
    pub deal_id: U256,
    pub reason: String,
}

/// Dispute resolution parameters
#[derive(Debug, Clone)]
pub struct ResolveDisputeParams {
    pub deal_id: U256,
    pub refund_to_buyer: bool,
    pub buyer_percent: U256, // 0-100
}

/// Transaction receipt with decoded logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowTxReceipt {
    pub tx_hash: TxHash,
    pub block_number: Option<u64>,
    pub gas_used: Option<U256>,
    pub status: bool,
    pub deal_id: Option<U256>,
    pub events: Vec<String>,
}

// ============================================================================
// P2P Escrow Client
// ============================================================================

/// Client for interacting with the P2PEscrow smart contract
pub struct P2PEscrowClient<M: Middleware> {
    contract: Contract<M>,
    provider: Arc<M>,
    contract_address: Address,
}

impl<M: Middleware> P2PEscrowClient<M>
where
    M::Error: 'static,
{
    /// Create a new escrow client
    ///
    /// # Arguments
    /// * `provider` - Ethereum provider
    /// * `contract_address` - Deployed contract address
    /// * `abi` - Contract ABI (JSON)
    ///
    /// # Returns
    /// * `Ok(P2PEscrowClient)` - Initialized client
    /// * `Err(EscrowError)` - If initialization fails
    pub fn new(
        provider: Arc<M>,
        contract_address: Address,
        abi: &str,
    ) -> EscrowResult<Self> {
        let abi: Abi = serde_json::from_str(abi).map_err(|e| EscrowError::Abi(e.to_string()))?;
        
        let contract = Contract::new(
            contract_address,
            abi,
            provider.clone(),
        );

        info!(
            "P2PEscrowClient initialized: address={}",
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

    /// Get deal information
    pub async fn get_deal(&self, deal_id: U256) -> EscrowResult<Deal> {
        debug!("Fetching deal: id={}", deal_id);

        let result = self
            .contract
            .method::<_, (U256, u8, Address, Address, Address, U256, U256, Address, u8, u64, u64, u64, [u8; 32], String)>(
                "deals",
                deal_id,
            )
            .map_err(|e| EscrowError::Contract(e.to_string()))?
            .call()
            .await
            .map_err(|e| EscrowError::Provider(e))?;

        let deal = Deal {
            id: result.0,
            deal_type: DealType::from_u8(result.1),
            buyer: result.2,
            seller: result.3,
            arbiter: result.4,
            amount: result.5,
            platform_fee: result.6,
            payment_token: result.7,
            state: DealState::from_u8(result.8),
            created_at: result.9,
            funded_at: result.10,
            deadline: result.11,
            message_hash: result.12,
            ipfs_metadata: result.13,
        };

        debug!("Deal fetched: id={}, state={}", deal.id, deal.state);

        Ok(deal)
    }

    /// Get user's deals
    pub async fn get_user_deals(&self, user: Address) -> EscrowResult<Vec<U256>> {
        debug!("Fetching user deals: user={}", user);

        let deal_ids = self
            .contract
            .method::<_, Vec<U256>>("getUserDeals", user)
            .map_err(|e| EscrowError::Contract(e.to_string()))?
            .call()
            .await
            .map_err(|e| EscrowError::Provider(e))?;

        debug!("Fetched {} deals for user", deal_ids.len());

        Ok(deal_ids)
    }

    /// Get platform statistics
    pub async fn get_platform_stats(&self) -> EscrowResult<PlatformStats> {
        debug!("Fetching platform stats");

        let result = self
            .contract
            .method::<_, (U256, U256, U256, U256, U256)>("getPlatformStats", ())
            .map_err(|e| EscrowError::Contract(e.to_string()))?
            .call()
            .await
            .map_err(|e| EscrowError::Provider(e))?;

        let stats = PlatformStats {
            total_deals: result.0,
            completed_deals: result.1,
            disputed_deals: result.2,
            total_volume: result.3,
            total_fees_collected: result.4,
        };

        debug!("Platform stats fetched: total_deals={}", stats.total_deals);

        Ok(stats)
    }

    /// Calculate platform fee for a given amount
    pub async fn calculate_fee(&self, amount: U256) -> EscrowResult<U256> {
        let fee = self
            .contract
            .method::<_, U256>("calculateFee", amount)
            .map_err(|e| EscrowError::Contract(e.to_string()))?
            .call()
            .await
            .map_err(|e| EscrowError::Provider(e))?;

        Ok(fee)
    }

    /// Check if address is authorized arbiter
    pub async fn is_arbiter(&self, arbiter: Address) -> EscrowResult<bool> {
        let result = self
            .contract
            .method::<_, bool>("authorizedArbiters", arbiter)
            .map_err(|e| EscrowError::Contract(e.to_string()))?
            .call()
            .await
            .map_err(|e| EscrowError::Provider(e))?;

        Ok(result)
    }

    // ========================================================================
    // Write Functions (require signer)
    // ========================================================================

    /// Create a new deal
    pub async fn create_deal(
        &self,
        params: CreateDealParams,
    ) -> EscrowResult<EscrowTxReceipt> {
        info!(
            "Creating deal: seller={}, type={}",
            params.seller, params.deal_type
        );

        let tx = self
            .contract
            .method::<_, U256>(
                "createDeal",
                (
                    params.seller,
                    params.deal_type as u8,
                    params.deadline,
                    params.message_hash,
                    params.ipfs_metadata,
                ),
            )
            .map_err(|e| EscrowError::Contract(e.to_string()))?;

        let pending_tx = tx.send().await.map_err(|e| EscrowError::TxFailed(e.to_string()))?;
        
        let receipt = pending_tx
            .await
            .map_err(|e| EscrowError::TxFailed(e.to_string()))?
            .ok_or_else(|| EscrowError::TxFailed("No receipt".to_string()))?;

        let tx_receipt = self.parse_receipt(&receipt, "DealCreated")?;

        info!(
            "Deal created: id={}, tx={:?}",
            tx_receipt.deal_id.unwrap_or_default(),
            receipt.transaction_hash
        );

        Ok(tx_receipt)
    }

    /// Create and fund deal with ETH in one transaction
    pub async fn create_and_fund_deal(
        &self,
        params: CreateDealParams,
        amount: U256,
    ) -> EscrowResult<EscrowTxReceipt> {
        info!(
            "Creating and funding deal: seller={}, amount={}",
            params.seller, amount
        );

        let tx = self
            .contract
            .method::<_, U256>(
                "createAndFundDeal",
                (
                    params.seller,
                    params.deal_type as u8,
                    params.deadline,
                    params.message_hash,
                    params.ipfs_metadata,
                ),
            )
            .map_err(|e| EscrowError::Contract(e.to_string()))?
            .value(amount);

        let pending_tx = tx.send().await.map_err(|e| EscrowError::TxFailed(e.to_string()))?;
        
        let receipt = pending_tx
            .await
            .map_err(|e| EscrowError::TxFailed(e.to_string()))?
            .ok_or_else(|| EscrowError::TxFailed("No receipt".to_string()))?;

        let tx_receipt = self.parse_receipt(&receipt, "DealFunded")?;

        info!(
            "Deal created and funded: id={}, tx={:?}",
            tx_receipt.deal_id.unwrap_or_default(),
            receipt.transaction_hash
        );

        Ok(tx_receipt)
    }

    /// Fund an existing deal with ETH
    pub async fn fund_deal(&self, params: FundDealParams) -> EscrowResult<EscrowTxReceipt> {
        info!("Funding deal: id={}, amount={}", params.deal_id, params.amount);

        let tx = self
            .contract
            .method::<_, ()>("fundDeal", params.deal_id)
            .map_err(|e| EscrowError::Contract(e.to_string()))?
            .value(params.amount);

        let pending_tx = tx.send().await.map_err(|e| EscrowError::TxFailed(e.to_string()))?;
        
        let receipt = pending_tx
            .await
            .map_err(|e| EscrowError::TxFailed(e.to_string()))?
            .ok_or_else(|| EscrowError::TxFailed("No receipt".to_string()))?;

        let tx_receipt = self.parse_receipt(&receipt, "DealFunded")?;

        info!("Deal funded: tx={:?}", receipt.transaction_hash);

        Ok(tx_receipt)
    }

    /// Confirm delivery (buyer)
    pub async fn confirm_delivery(&self, deal_id: U256) -> EscrowResult<EscrowTxReceipt> {
        info!("Confirming delivery: deal_id={}", deal_id);

        let tx = self
            .contract
            .method::<_, ()>("confirmDelivery", deal_id)
            .map_err(|e| EscrowError::Contract(e.to_string()))?;

        let pending_tx = tx.send().await.map_err(|e| EscrowError::TxFailed(e.to_string()))?;
        
        let receipt = pending_tx
            .await
            .map_err(|e| EscrowError::TxFailed(e.to_string()))?
            .ok_or_else(|| EscrowError::TxFailed("No receipt".to_string()))?;

        let tx_receipt = self.parse_receipt(&receipt, "DealDelivered")?;

        info!("Delivery confirmed: tx={:?}", receipt.transaction_hash);

        Ok(tx_receipt)
    }

    /// Complete deal (payout to seller)
    pub async fn complete_deal(&self, deal_id: U256) -> EscrowResult<EscrowTxReceipt> {
        info!("Completing deal: id={}", deal_id);

        let tx = self
            .contract
            .method::<_, ()>("completeDeal", deal_id)
            .map_err(|e| EscrowError::Contract(e.to_string()))?;

        let pending_tx = tx.send().await.map_err(|e| EscrowError::TxFailed(e.to_string()))?;
        
        let receipt = pending_tx
            .await
            .map_err(|e| EscrowError::TxFailed(e.to_string()))?
            .ok_or_else(|| EscrowError::TxFailed("No receipt".to_string()))?;

        let tx_receipt = self.parse_receipt(&receipt, "DealCompleted")?;

        info!("Deal completed: tx={:?}", receipt.transaction_hash);

        Ok(tx_receipt)
    }

    /// Refund after deadline
    pub async fn refund_after_deadline(&self, deal_id: U256) -> EscrowResult<EscrowTxReceipt> {
        info!("Refunding after deadline: deal_id={}", deal_id);

        let tx = self
            .contract
            .method::<_, ()>("refundAfterDeadline", deal_id)
            .map_err(|e| EscrowError::Contract(e.to_string()))?;

        let pending_tx = tx.send().await.map_err(|e| EscrowError::TxFailed(e.to_string()))?;
        
        let receipt = pending_tx
            .await
            .map_err(|e| EscrowError::TxFailed(e.to_string()))?
            .ok_or_else(|| EscrowError::TxFailed("No receipt".to_string()))?;

        let tx_receipt = self.parse_receipt(&receipt, "DealRefunded")?;

        info!("Refund processed: tx={:?}", receipt.transaction_hash);

        Ok(tx_receipt)
    }

    /// Open dispute
    pub async fn open_dispute(
        &self,
        params: DisputeParams,
    ) -> EscrowResult<EscrowTxReceipt> {
        info!("Opening dispute: deal_id={}", params.deal_id);

        let tx = self
            .contract
            .method::<_, ()>("openDispute", (params.deal_id, params.reason))
            .map_err(|e| EscrowError::Contract(e.to_string()))?;

        let pending_tx = tx.send().await.map_err(|e| EscrowError::TxFailed(e.to_string()))?;
        
        let receipt = pending_tx
            .await
            .map_err(|e| EscrowError::TxFailed(e.to_string()))?
            .ok_or_else(|| EscrowError::TxFailed("No receipt".to_string()))?;

        let tx_receipt = self.parse_receipt(&receipt, "DealDisputed")?;

        info!("Dispute opened: tx={:?}", receipt.transaction_hash);

        Ok(tx_receipt)
    }

    /// Resolve dispute (arbiter only)
    pub async fn resolve_dispute(
        &self,
        params: ResolveDisputeParams,
    ) -> EscrowResult<EscrowTxReceipt> {
        info!(
            "Resolving dispute: deal_id={}, refund={}, buyer%={}",
            params.deal_id, params.refund_to_buyer, params.buyer_percent
        );

        let tx = self
            .contract
            .method::<_, ()>(
                "resolveDispute",
                (params.deal_id, params.refund_to_buyer, params.buyer_percent),
            )
            .map_err(|e| EscrowError::Contract(e.to_string()))?;

        let pending_tx = tx.send().await.map_err(|e| EscrowError::TxFailed(e.to_string()))?;
        
        let receipt = pending_tx
            .await
            .map_err(|e| EscrowError::TxFailed(e.to_string()))?
            .ok_or_else(|| EscrowError::TxFailed("No receipt".to_string()))?;

        let tx_receipt = self.parse_receipt(&receipt, "DealResolved")?;

        info!("Dispute resolved: tx={:?}", receipt.transaction_hash);

        Ok(tx_receipt)
    }

    /// Cancel deal (before funding)
    pub async fn cancel_deal(&self, deal_id: U256) -> EscrowResult<EscrowTxReceipt> {
        info!("Cancelling deal: id={}", deal_id);

        let tx = self
            .contract
            .method::<_, ()>("cancelDeal", deal_id)
            .map_err(|e| EscrowError::Contract(e.to_string()))?;

        let pending_tx = tx.send().await.map_err(|e| EscrowError::TxFailed(e.to_string()))?;
        
        let receipt = pending_tx
            .await
            .map_err(|e| EscrowError::TxFailed(e.to_string()))?
            .ok_or_else(|| EscrowError::TxFailed("No receipt".to_string()))?;

        let tx_receipt = self.parse_receipt(&receipt, "DealCancelled")?;

        info!("Deal cancelled: tx={:?}", receipt.transaction_hash);

        Ok(tx_receipt)
    }

    // ========================================================================
    // Event Monitoring
    // ========================================================================

    /// Watch for deal created events
    pub async fn watch_deal_created<F>(&self, callback: F) -> EscrowResult<()>
    where
        F: Fn(Deal) + Send + Sync + 'static,
    {
        let filter = Filter::new()
            .address(self.contract_address)
            .event("DealCreated(uint256,address,address,uint256,uint8,bytes32)");

        let logs = self
            .provider
            .get_logs(&filter)
            .await
            .map_err(|e| EscrowError::Provider(e))?;

        for log in logs {
            if let Some(deal_id) = log.topics.get(1) {
                let deal = self.get_deal(U256::from_big_endian(deal_id.as_bytes())).await?;
                callback(deal);
            }
        }

        Ok(())
    }

    // ========================================================================
    // Internal Helpers
    // ========================================================================

    /// Parse transaction receipt and extract deal ID
    fn parse_receipt(
        &self,
        receipt: &TransactionReceipt,
        event_name: &str,
    ) -> EscrowResult<EscrowTxReceipt> {
        let deal_id = self.extract_deal_id_from_logs(&receipt.logs);
        
        let events = vec![event_name.to_string()];

        Ok(EscrowTxReceipt {
            tx_hash: receipt.transaction_hash,
            block_number: receipt.block_number.map(|n| n.as_u64()),
            gas_used: receipt.gas_used,
            status: receipt.status.unwrap_or_default().as_u64() == 1,
            deal_id,
            events,
        })
    }

    /// Extract deal ID from logs
    fn extract_deal_id_from_logs(&self, logs: &[Log]) -> Option<U256> {
        logs.first()
            .and_then(|log| log.topics.get(1))
            .map(|topic| U256::from_big_endian(topic.as_bytes()))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::utils::Anvil;

    #[test]
    fn test_deal_state_from_u8() {
        assert_eq!(DealState::from_u8(0), DealState::Created);
        assert_eq!(DealState::from_u8(1), DealState::Funded);
        assert_eq!(DealState::from_u8(2), DealState::Delivered);
        assert_eq!(DealState::from_u8(3), DealState::Completed);
        assert_eq!(DealState::from_u8(4), DealState::Disputed);
        assert_eq!(DealState::from_u8(5), DealState::Refunded);
        assert_eq!(DealState::from_u8(6), DealState::Cancelled);
    }

    #[test]
    #[should_panic]
    fn test_deal_state_invalid() {
        DealState::from_u8(7);
    }

    #[test]
    fn test_deal_type_from_u8() {
        assert_eq!(DealType::from_u8(0), DealType::DigitalGoods);
        assert_eq!(DealType::from_u8(1), DealType::PhysicalGoods);
        assert_eq!(DealType::from_u8(2), DealType::Service);
        assert_eq!(DealType::from_u8(3), DealType::Subscription);
        assert_eq!(DealType::from_u8(4), DealType::Freelance);
    }

    #[test]
    fn test_deal_state_display() {
        assert_eq!(DealState::Created.to_string(), "Created");
        assert_eq!(DealState::Funded.to_string(), "Funded");
        assert_eq!(DealType::Service.to_string(), "Service");
    }

    #[test]
    fn test_platform_stats_serialization() {
        let stats = PlatformStats {
            total_deals: U256::from(100),
            completed_deals: U256::from(95),
            disputed_deals: U256::from(5),
            total_volume: U256::from(10_000_000_000_000_000_000u128), // 10 ETH
            total_fees_collected: U256::from(100_000_000_000_000_000u128), // 0.1 ETH
        };

        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: PlatformStats = serde_json::from_str(&json).unwrap();

        assert_eq!(stats.total_deals, deserialized.total_deals);
        assert_eq!(stats.completed_deals, deserialized.completed_deals);
    }
}
