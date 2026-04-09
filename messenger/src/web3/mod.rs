//! Web3 Module — MetaMask, wallet management, transactions
//!
//! Architecture:
//! - Native Rust wallet via `ethers` crate (local key management, signing)
//! - MetaMask integration via Tauri JS bridge (window.ethereum)
//! - Privacy-first: no data leaves device without user consent
//!
//! Supported networks:
//! - Ethereum Mainnet (chain_id: 1)
//! - Polygon (chain_id: 137)
//! - Arbitrum (chain_id: 42161)
//! - Base (chain_id: 8453)
//!
//! Features:
//! - Wallet connection (MetaMask or local keystore)
//! - Message signing (EIP-191, EIP-712)
//! - Transaction building and broadcasting
//! - ENS name resolution
//! - ERC-20 token operations
//! - Balance queries

#[cfg(feature = "web3")]
pub mod metamask;
#[cfg(feature = "web3")]
pub mod wallet;
#[cfg(feature = "web3")]
pub mod transactions;
#[cfg(feature = "web3")]
pub mod tokens;
#[cfg(feature = "web3")]
pub mod ens;
#[cfg(feature = "web3")]
pub mod swap_commands;
#[cfg(feature = "web3")]
pub mod abcex;
#[cfg(feature = "web3")]
pub mod bitget;
#[cfg(feature = "web3")]
pub mod zerox_swap;
#[cfg(feature = "web3")]
pub mod abcex_commands;
#[cfg(feature = "web3")]
pub mod bitget_commands;
#[cfg(feature = "web3")]
pub mod transaction_commands;
#[cfg(feature = "web3")]
pub mod p2p_escrow;
#[cfg(feature = "web3")]
pub mod p2p_escrow_commands;
#[cfg(feature = "web3")]
pub mod fee_splitter;
#[cfg(feature = "web3")]
pub mod fee_splitter_commands;

// Re-export types (available even without web3 feature for UI compatibility)
pub use types::*;
pub use types::Web3Error;
#[cfg(feature = "web3")]
pub use swap_commands::*;
// Re-export specific types from abcex and bitget to avoid name conflicts
#[cfg(feature = "web3")]
pub use abcex::{AbcexClient, BuyQuoteRequest, BuyQuoteResponse, BuyOrder, BuyOrderStatus, PaymentMethod, FiatCurrency, KycStatus, KycLimits, BuyQuoteBuilder};
#[cfg(feature = "web3")]
pub use bitget::{BitgetClient, BuyRequest, BuyResponse, AccountInfo, MarketPrice, OrderType, OrderSide, BitgetOrderStatus, BuyRequestBuilder};
#[cfg(feature = "web3")]
pub use zerox_swap::*;
#[cfg(feature = "web3")]
pub use abcex_commands::*;
#[cfg(feature = "web3")]
pub use bitget_commands::*;

/// Web3-specific types (always compiled for type compatibility)
mod types {
    use serde::{Deserialize, Serialize};

    // ========================================================================
    // Network / Chain
    // ========================================================================

    /// Supported EVM chains
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum Chain {
        Ethereum,
        Polygon,
        Arbitrum,
        Base,
        Optimism,
        Bsc,
    }

    impl Chain {
        pub fn chain_id(&self) -> u64 {
            match self {
                Chain::Ethereum => 1,
                Chain::Polygon => 137,
                Chain::Arbitrum => 42161,
                Chain::Base => 8453,
                Chain::Optimism => 10,
                Chain::Bsc => 56,
            }
        }

        pub fn name(&self) -> &'static str {
            match self {
                Chain::Ethereum => "Ethereum Mainnet",
                Chain::Polygon => "Polygon",
                Chain::Arbitrum => "Arbitrum One",
                Chain::Base => "Base",
                Chain::Optimism => "Optimism",
                Chain::Bsc => "BNB Smart Chain",
            }
        }

        pub fn native_symbol(&self) -> &'static str {
            match self {
                Chain::Ethereum => "ETH",
                Chain::Polygon => "MATIC",
                Chain::Arbitrum => "ETH",
                Chain::Base => "ETH",
                Chain::Optimism => "ETH",
                Chain::Bsc => "BNB",
            }
        }

        pub fn rpc_url(&self) -> &'static str {
            match self {
                Chain::Ethereum => "https://eth.llamarpc.com",
                Chain::Polygon => "https://polygon-rpc.com",
                Chain::Arbitrum => "https://arb1.arbitrum.io/rpc",
                Chain::Base => "https://mainnet.base.org",
                Chain::Optimism => "https://mainnet.optimism.io",
                Chain::Bsc => "https://bsc-dataseed.binance.org",
            }
        }

        pub fn explorer_url(&self) -> &'static str {
            match self {
                Chain::Ethereum => "https://etherscan.io",
                Chain::Polygon => "https://polygonscan.com",
                Chain::Arbitrum => "https://arbiscan.io",
                Chain::Base => "https://basescan.org",
                Chain::Optimism => "https://optimistic.etherscan.io",
                Chain::Bsc => "https://bscscan.com",
            }
        }

        pub fn from_chain_id(id: u64) -> Option<Self> {
            match id {
                1 => Some(Chain::Ethereum),
                137 => Some(Chain::Polygon),
                42161 => Some(Chain::Arbitrum),
                8453 => Some(Chain::Base),
                10 => Some(Chain::Optimism),
                56 => Some(Chain::Bsc),
                _ => None,
            }
        }
    }

    impl std::str::FromStr for Chain {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.to_lowercase().as_str() {
                "ethereum" | "eth" | "1" => Ok(Chain::Ethereum),
                "polygon" | "matic" | "137" => Ok(Chain::Polygon),
                "arbitrum" | "arb" | "42161" => Ok(Chain::Arbitrum),
                "base" | "8453" => Ok(Chain::Base),
                "optimism" | "op" | "10" => Ok(Chain::Optimism),
                "bsc" | "bnb" | "56" => Ok(Chain::Bsc),
                _ => Err(format!("Unknown chain: {}", s)),
            }
        }
    }

    impl std::fmt::Display for Chain {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.name())
        }
    }

    // ========================================================================
    // Wallet Types
    // ========================================================================

    /// Wallet connection type
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum WalletType {
        /// MetaMask via Tauri JS bridge
        MetaMask,
        /// Local keystore file (ethers.rs)
        Keystore,
        /// Hardware wallet (Ledger/Trezor)
        Hardware,
        /// Mnemonic seed phrase
        Mnemonic,
        /// Private key (imported, warn user)
        PrivateKey,
    }

    /// Wallet information
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WalletInfo {
        pub wallet_type: WalletType,
        pub address: String,
        pub label: String,
        pub chain: Chain,
        pub is_connected: bool,
        pub created_at: chrono::DateTime<chrono::Utc>,
    }

    // ========================================================================
    // Balance / Token Types
    // ========================================================================

    /// Token balance
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TokenBalance {
        pub token_address: String,
        pub token_name: String,
        pub token_symbol: String,
        pub decimals: u8,
        pub balance: String,       // formatted string (e.g., "1.5")
        pub balance_raw: String,   // raw wei value
        pub chain: Chain,
    }

    /// Native balance
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct NativeBalance {
        pub chain: Chain,
        pub address: String,
        pub balance: String,       // formatted
        pub balance_wei: String,   // raw wei
        pub symbol: String,
    }

    // ========================================================================
    // Transaction Types
    // ========================================================================

    /// Transaction status
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum TxStatus {
        Pending,
        Submitted { tx_hash: String },
        Confirmed { tx_hash: String, block_number: u64 },
        Failed { tx_hash: Option<String>, error: String },
        Cancelled,
    }

    /// Transaction type
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum TxType {
        Transfer,
        Swap,
        ContractCall,
        Approval,
        Tip,
        Subscription,
        AdPayment,
    }

    /// Transaction record
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TransactionRecord {
        pub id: String,             // UUID
        pub chain: Chain,
        pub tx_type: TxType,
        pub from: String,
        pub to: String,
        pub value: String,          // in native token units
        pub value_wei: String,      // raw wei
        pub gas_estimate: Option<u64>,
        pub gas_price: Option<String>,
        pub nonce: Option<u64>,
        pub status: TxStatus,
        pub data: Option<String>,   // calldata hex
        pub created_at: chrono::DateTime<chrono::Utc>,
        pub updated_at: chrono::DateTime<chrono::Utc>,
    }

    // ========================================================================
    // Signing Types
    // ========================================================================

    /// EIP-191 personal sign
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PersonalSignRequest {
        pub message: String,
        pub address: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PersonalSignResponse {
        pub signature: String,
        pub address: String,
    }

    /// EIP-712 typed data
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Eip712Request {
        pub domain: serde_json::Value,
        pub types: serde_json::Value,
        pub primary_type: String,
        pub message: serde_json::Value,
        pub address: String,
    }

    // ========================================================================
    // ENS
    // ========================================================================

    /// ENS name record
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EnsRecord {
        pub name: String,
        pub address: String,
        pub avatar_url: Option<String>,
        pub content_hash: Option<String>,
        pub description: Option<String>,
    }

    // ========================================================================
    // Errors
    // ========================================================================

    #[derive(Debug, thiserror::Error)]
    pub enum Web3Error {
        #[error("Wallet not connected")]
        NotConnected,

        #[error("Wallet error: {0}")]
        Wallet(String),

        #[error("Network error: {0}")]
        Network(String),

        #[error("Transaction failed: {0}")]
        TxFailed(String),

        #[error("Signing failed: {0}")]
        Signing(String),

        #[error("Insufficient balance: have {have}, need {need}")]
        InsufficientBalance { have: String, need: String },

        #[error("ENS resolution failed: {0}")]
        Ens(String),

        #[error("Unsupported chain: {0}")]
        UnsupportedChain(u64),

        #[error("RPC error: {0}")]
        Rpc(String),

        #[error("Invalid address: {0}")]
        InvalidAddress(String),

        #[error("User rejected request")]
        UserRejected,

        #[error("Timeout")]
        Timeout,

        #[cfg(feature = "web3")]
        #[error("Ethers error: {0}")]
        Ethers(#[from] ethers::providers::ProviderError),
    }

    pub type Web3Result<T> = Result<T, Web3Error>;
}
