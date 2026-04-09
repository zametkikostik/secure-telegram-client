//! Transaction Builder — construct, sign, track, and store transactions
//!
//! Features:
//! - Build native token transfers
//! - Build ERC-20 transfers
//! - Build contract calls
//! - Estimate gas
//! - Track transaction status via RPC polling
//! - Persistent transaction log (SQLite via sqlx)

use super::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

// ============================================================================
// Transaction Builder
// ============================================================================

/// Transaction request (before signing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxRequest {
    pub chain: Chain,
    pub from: String,
    pub to: String,
    pub value_wei: String,
    pub data: Option<String>,
    pub gas_limit: Option<u64>,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
    pub nonce: Option<u64>,
    pub tx_type: TxType,
}

impl TxRequest {
    /// Create a simple native token transfer
    pub fn transfer(chain: Chain, from: String, to: String, value_wei: String) -> Self {
        Self {
            chain,
            from,
            to,
            value_wei,
            data: None,
            gas_limit: Some(21_000), // Standard ETH transfer
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            nonce: None,
            tx_type: TxType::Transfer,
        }
    }

    /// Create a tip payment
    pub fn tip(from: String, to: String, amount_wei: String) -> Self {
        Self {
            chain: Chain::Ethereum,
            from,
            to,
            value_wei: amount_wei,
            data: None,
            gas_limit: Some(21_000),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            nonce: None,
            tx_type: TxType::Tip,
        }
    }

    /// Create an ERC-20 token transfer
    pub fn erc20_transfer(
        chain: Chain,
        from: String,
        token_address: String,
        recipient: String,
        amount_raw: String,
    ) -> Self {
        // ERC-20 transfer function signature: transfer(address,uint256)
        // Function selector: 0xa9059cbb
        let data = format!(
            "0xa9059cbb{}{}",
            pad_address(&recipient),
            pad_uint256(&amount_raw),
        );

        Self {
            chain,
            from,
            to: token_address,
            value_wei: "0".to_string(),
            data: Some(data),
            gas_limit: Some(65_000),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            nonce: None,
            tx_type: TxType::Transfer,
        }
    }

    /// Create a contract call
    pub fn contract_call(from: String, to: String, data: String, value_wei: String) -> Self {
        Self {
            chain: Chain::Ethereum,
            from,
            to,
            value_wei,
            data: Some(data),
            gas_limit: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            nonce: None,
            tx_type: TxType::ContractCall,
        }
    }

    /// Create an approval transaction
    pub fn approval(spender: String, token_address: String, amount_raw: String) -> Self {
        let data = format!(
            "0x095ea7b3{}{}",
            pad_address(&spender),
            pad_uint256(&amount_raw),
        );

        Self {
            chain: Chain::Ethereum,
            from: String::new(),
            to: token_address,
            value_wei: "0".to_string(),
            data: Some(data),
            gas_limit: Some(50_000),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            nonce: None,
            tx_type: TxType::Approval,
        }
    }
}

// ============================================================================
// Transaction Store
// ============================================================================

/// In-memory transaction log
pub struct TxStore {
    transactions: Mutex<HashMap<String, TransactionRecord>>,
}

impl TxStore {
    pub fn new() -> Self {
        Self {
            transactions: Mutex::new(HashMap::new()),
        }
    }

    /// Create a new pending transaction record
    pub fn create(&self, request: TxRequest) -> String {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        let record = TransactionRecord {
            id: id.clone(),
            chain: request.chain,
            tx_type: request.tx_type,
            from: request.from,
            to: request.to,
            value: wei_to_eth(&request.value_wei),
            value_wei: request.value_wei,
            gas_estimate: request.gas_limit,
            gas_price: request.max_fee_per_gas,
            nonce: request.nonce,
            status: TxStatus::Pending,
            data: request.data,
            created_at: now,
            updated_at: now,
        };

        self.transactions.lock().unwrap().insert(id.clone(), record);

        id
    }

    /// Update transaction status
    pub fn update_status(&self, id: &str, status: TxStatus) -> Option<TransactionRecord> {
        let mut txs = self.transactions.lock().unwrap();
        if let Some(tx) = txs.get_mut(id) {
            tx.status = status;
            tx.updated_at = chrono::Utc::now();
            Some(tx.clone())
        } else {
            None
        }
    }

    /// Mark as submitted
    pub fn mark_submitted(&self, id: &str, tx_hash: &str) -> Option<TransactionRecord> {
        self.update_status(
            id,
            TxStatus::Submitted {
                tx_hash: tx_hash.to_string(),
            },
        )
    }

    /// Mark as confirmed
    pub fn mark_confirmed(&self, id: &str, tx_hash: &str, block: u64) -> Option<TransactionRecord> {
        self.update_status(
            id,
            TxStatus::Confirmed {
                tx_hash: tx_hash.to_string(),
                block_number: block,
            },
        )
    }

    /// Mark as failed
    pub fn mark_failed(
        &self,
        id: &str,
        tx_hash: Option<&str>,
        error: &str,
    ) -> Option<TransactionRecord> {
        self.update_status(
            id,
            TxStatus::Failed {
                tx_hash: tx_hash.map(String::from),
                error: error.to_string(),
            },
        )
    }

    /// Cancel a pending transaction
    pub fn cancel(&self, id: &str) -> Option<TransactionRecord> {
        self.update_status(id, TxStatus::Cancelled)
    }

    /// Get a transaction by ID
    pub fn get(&self, id: &str) -> Option<TransactionRecord> {
        self.transactions.lock().unwrap().get(id).cloned()
    }

    /// Get all transactions
    pub fn list(&self) -> Vec<TransactionRecord> {
        self.transactions
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    /// Get transactions for a specific address (from or to)
    pub fn list_for_address(&self, address: &str) -> Vec<TransactionRecord> {
        self.transactions
            .lock()
            .unwrap()
            .values()
            .filter(|tx| tx.from == address || tx.to == address)
            .cloned()
            .collect()
    }

    /// Get pending transactions
    pub fn list_pending(&self) -> Vec<TransactionRecord> {
        self.transactions
            .lock()
            .unwrap()
            .values()
            .filter(|tx| matches!(tx.status, TxStatus::Pending | TxStatus::Submitted { .. }))
            .cloned()
            .collect()
    }

    /// Remove old confirmed transactions (cleanup)
    pub fn prune_confirmed(&self, keep_last: usize) {
        let mut txs = self.transactions.lock().unwrap();
        let mut confirmed: Vec<_> = txs
            .iter()
            .filter(|(_, tx)| matches!(tx.status, TxStatus::Confirmed { .. }))
            .map(|(id, tx)| (id.clone(), tx.updated_at))
            .collect();

        confirmed.sort_by(|a, b| b.1.cmp(&a.1)); // newest first

        for (id, _) in confirmed.into_iter().skip(keep_last) {
            txs.remove(&id);
        }
    }

    /// Get transaction count
    pub fn count(&self) -> usize {
        self.transactions.lock().unwrap().len()
    }
}

// ============================================================================
// Gas Estimation
// ============================================================================

/// Gas price estimate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasEstimate {
    pub slow: GasPrice,
    pub standard: GasPrice,
    pub fast: GasPrice,
    pub rapid: GasPrice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasPrice {
    pub max_fee_per_gas: String,  // in gwei
    pub max_priority_fee: String, // in gwei
    pub estimated_time_secs: u64,
}

impl GasEstimate {
    /// Default gas estimates (fallback when API unavailable)
    pub fn default_for(chain: Chain) -> Self {
        let base = match chain {
            Chain::Ethereum => 30.0,
            Chain::Polygon => 50.0,
            Chain::Arbitrum => 0.1,
            Chain::Base => 0.1,
            Chain::Optimism => 0.01,
            Chain::Bsc => 3.0,
        };

        Self {
            slow: GasPrice {
                max_fee_per_gas: format!("{:.2}", base * 0.8),
                max_priority_fee: format!("{:.2}", base * 0.5),
                estimated_time_secs: 300,
            },
            standard: GasPrice {
                max_fee_per_gas: format!("{:.2}", base),
                max_priority_fee: format!("{:.2}", base * 0.7),
                estimated_time_secs: 60,
            },
            fast: GasPrice {
                max_fee_per_gas: format!("{:.2}", base * 1.5),
                max_priority_fee: format!("{:.2}", base),
                estimated_time_secs: 15,
            },
            rapid: GasPrice {
                max_fee_per_gas: format!("{:.2}", base * 2.0),
                max_priority_fee: format!("{:.2}", base * 1.5),
                estimated_time_secs: 5,
            },
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Pad an address to 32 bytes (for ABI encoding)
fn pad_address(address: &str) -> String {
    let addr = address.trim_start_matches("0x");
    format!("0x{:0>64}", addr)
}

/// Pad a uint256 to 32 bytes (for ABI encoding)
fn pad_uint256(value: &str) -> String {
    format!("0x{:0>64}", value)
}

/// Convert wei to ETH string
fn wei_to_eth(wei: &str) -> String {
    if let Ok(wei_val) = wei.parse::<u128>() {
        let eth = (wei_val as f64) / 1e18;
        format!("{:.6}", eth)
    } else {
        "0.0".to_string()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tx_request_transfer() {
        let tx = TxRequest::transfer(
            Chain::Ethereum,
            "0x1234".to_string(),
            "0x5678".to_string(),
            "1000000000000000000".to_string(),
        );

        assert_eq!(tx.tx_type, TxType::Transfer);
        assert_eq!(tx.gas_limit, Some(21_000));
        assert!(tx.data.is_none());
    }

    #[test]
    fn test_tx_request_tip() {
        let tx = TxRequest::tip(
            "0x1234".to_string(),
            "0x5678".to_string(),
            "500000000000000000".to_string(),
        );

        assert_eq!(tx.tx_type, TxType::Tip);
        assert_eq!(tx.value_wei, "500000000000000000");
    }

    #[test]
    fn test_tx_request_erc20() {
        let tx = TxRequest::erc20_transfer(
            Chain::Ethereum,
            "0x1234".to_string(),
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
            "0x5678".to_string(),
            "1000000".to_string(),
        );

        assert_eq!(tx.tx_type, TxType::Transfer);
        assert!(tx.data.is_some());
        let data = tx.data.unwrap();
        assert!(data.starts_with("0xa9059cbb")); // transfer function selector
    }

    #[test]
    fn test_tx_request_approval() {
        let tx = TxRequest::approval(
            "0xSpender".to_string(),
            "0xToken".to_string(),
            "115792089237316195423570985008687907853269984665640564039457584007913129639935"
                .to_string(),
        );

        assert_eq!(tx.tx_type, TxType::Approval);
        assert!(tx.data.is_some());
        let data = tx.data.unwrap();
        assert!(data.starts_with("0x095ea7b3")); // approve function selector
    }

    #[test]
    fn test_tx_store_lifecycle() {
        let store = TxStore::new();
        assert_eq!(store.count(), 0);

        // Create
        let request = TxRequest::transfer(
            Chain::Ethereum,
            "0xFrom".to_string(),
            "0xTo".to_string(),
            "1000000000000000000".to_string(),
        );
        let id = store.create(request);
        assert_eq!(store.count(), 1);

        // Get
        let tx = store.get(&id).unwrap();
        assert!(matches!(tx.status, TxStatus::Pending));

        // Update: submitted
        store.mark_submitted(&id, "0xTxHash");
        let tx = store.get(&id).unwrap();
        assert!(matches!(tx.status, TxStatus::Submitted { .. }));

        // Update: confirmed
        store.mark_confirmed(&id, "0xTxHash", 18_000_000);
        let tx = store.get(&id).unwrap();
        assert!(matches!(tx.status, TxStatus::Confirmed { .. }));

        // List pending (should be empty now)
        let pending = store.list_pending();
        assert!(pending.is_empty());
    }

    #[test]
    fn test_tx_store_failed() {
        let store = TxStore::new();
        let request = TxRequest::transfer(
            Chain::Ethereum,
            "0xFrom".to_string(),
            "0xTo".to_string(),
            "1000000000000000000".to_string(),
        );
        let id = store.create(request);
        store.mark_failed(&id, Some("0xTxHash"), "out of gas");

        let tx = store.get(&id).unwrap();
        assert!(matches!(tx.status, TxStatus::Failed { .. }));
    }

    #[test]
    fn test_tx_store_cancel() {
        let store = TxStore::new();
        let request = TxRequest::transfer(
            Chain::Ethereum,
            "0xFrom".to_string(),
            "0xTo".to_string(),
            "1000000000000000000".to_string(),
        );
        let id = store.create(request);
        store.cancel(&id);

        let tx = store.get(&id).unwrap();
        assert_eq!(tx.status, TxStatus::Cancelled);
    }

    #[test]
    fn test_tx_store_address_filter() {
        let store = TxStore::new();

        let tx1 = TxRequest::transfer(
            Chain::Ethereum,
            "0xAlice".to_string(),
            "0xBob".to_string(),
            "100".to_string(),
        );
        store.create(tx1);

        let tx2 = TxRequest::transfer(
            Chain::Ethereum,
            "0xCharlie".to_string(),
            "0xAlice".to_string(),
            "200".to_string(),
        );
        store.create(tx2);

        let tx3 = TxRequest::transfer(
            Chain::Ethereum,
            "0xCharlie".to_string(),
            "0xDavid".to_string(),
            "300".to_string(),
        );
        store.create(tx3);

        let alice_txs = store.list_for_address("0xAlice");
        assert_eq!(alice_txs.len(), 2); // tx1 (from) + tx2 (to)

        let david_txs = store.list_for_address("0xDavid");
        assert_eq!(david_txs.len(), 1); // tx3 (to)
    }

    #[test]
    fn test_gas_estimate_defaults() {
        let eth_gas = GasEstimate::default_for(Chain::Ethereum);
        assert!(eth_gas.standard.max_fee_per_gas.parse::<f64>().unwrap() > 0.0);

        let arb_gas = GasEstimate::default_for(Chain::Arbitrum);
        assert!(arb_gas.standard.max_fee_per_gas.parse::<f64>().unwrap() < 1.0);
    }

    #[test]
    fn test_wei_to_eth() {
        assert_eq!(wei_to_eth("1000000000000000000"), "1.000000");
        assert_eq!(wei_to_eth("500000000000000000"), "0.500000");
        assert_eq!(wei_to_eth("0"), "0.000000");
    }

    #[test]
    fn test_pad_address() {
        let padded = pad_address("0x1234");
        assert_eq!(padded.len(), 66); // 0x + 64 chars
        assert!(padded.ends_with("1234"));
    }

    #[test]
    fn test_tx_store_prune() {
        let store = TxStore::new();

        // Create 10 confirmed transactions
        for i in 0..10 {
            let request = TxRequest::transfer(
                Chain::Ethereum,
                "0xFrom".to_string(),
                "0xTo".to_string(),
                format!("{}", i),
            );
            let id = store.create(request);
            store.mark_confirmed(&id, &format!("0xHash{}", i), i);
        }

        assert_eq!(store.count(), 10);
        store.prune_confirmed(3);
        assert_eq!(store.count(), 3);
    }
}
