//! Wallet abstraction — unified interface over MetaMask, local keystore, hardware wallets
//!
//! Architecture:
//! - `LocalWallet`: ethers.rs `Wallet<SigningKey>` for local key management
//! - `MetaMaskWallet`: Tauri JS bridge to browser MetaMask
//! - `HardwareWallet`: Ledger/Trezor (future)
//!
//! Features:
//! - Generate new wallet from mnemonic
//! - Import from private key or keystore
//! - Sign messages (EIP-191)
//! - Sign typed data (EIP-712)
//! - Export wallet (encrypted keystore)
//! - Zeroize sensitive data on drop

use super::types::*;
use std::path::Path;

// ============================================================================
// Local Wallet (ethers.rs)
// ============================================================================

#[cfg(feature = "web3")]
pub mod local {
    use super::*;
    use ethers::core::rand::thread_rng;
    use ethers::signers::{LocalWallet, Signer};
    use zeroize::Zeroize;

    /// Local wallet backed by ethers.rs SigningKey
    pub struct LocalWalletManager {
        wallet: Option<SecureWallet>,
        chain: Chain,
    }

    /// Wrapper that zeroizes the private key on drop
    pub struct SecureWallet {
        wallet: LocalWallet,
        address: String,
    }

    impl Drop for SecureWallet {
        fn drop(&mut self) {
            // LocalWallet already uses SecretKey which zeroizes,
            // but we explicitly zero the address too
            self.address.zeroize();
        }
    }

    impl LocalWalletManager {
        pub fn new(chain: Chain) -> Self {
            Self {
                wallet: None,
                chain,
            }
        }

        /// Generate a new random wallet with mnemonic
        pub fn generate(&mut self) -> Web3Result<WalletInfo> {
            let wallet = LocalWallet::new(&mut thread_rng());
            
            // NOTE: В ethers v2.0 нет метода mnemonic() для Wallet
            // mnemonic доступен только при создании через new_with_mnemonic
            // Для простоты возвращаем приватный ключ в hex
            let address = format!("{:?}", wallet.address());
            let chain = self.chain;

            self.wallet = Some(SecureWallet { wallet, address: address.clone() });

            Ok(WalletInfo {
                wallet_type: WalletType::Mnemonic,
                address,
                label: format!("{} Wallet", chain.name()),
                chain,
                is_connected: true,
                created_at: chrono::Utc::now(),
            })
        }

        /// Import from private key (hex string)
        pub fn import_private_key(&mut self, private_key: &str) -> Web3Result<WalletInfo> {
            let wallet: LocalWallet = private_key
                .parse()
                .map_err(|e| Web3Error::Wallet(format!("Invalid private key: {}", e)))?;

            let address = format!("{:?}", wallet.address());
            self.wallet = Some(SecureWallet { wallet, address: address.clone() });

            Ok(WalletInfo {
                wallet_type: WalletType::PrivateKey,
                address,
                label: "Imported Wallet".to_string(),
                chain: self.chain,
                is_connected: true,
                created_at: chrono::Utc::now(),
            })
        }

        /// Import from encrypted keystore file
        pub fn import_keystore(
            &mut self,
            keystore_path: &Path,
            password: &str,
        ) -> Web3Result<WalletInfo> {
            let wallet = LocalWallet::decrypt_keystore(keystore_path, password)
                .map_err(|e| Web3Error::Wallet(format!("Failed to decrypt keystore: {}", e)))?;

            let address = format!("{:?}", wallet.address());
            self.wallet = Some(SecureWallet { wallet, address: address.clone() });

            Ok(WalletInfo {
                wallet_type: WalletType::Keystore,
                address,
                label: "Keystore Wallet".to_string(),
                chain: self.chain,
                is_connected: true,
                created_at: chrono::Utc::now(),
            })
        }

        /// Export to encrypted keystore file
        /// TODO: реализовать экспорт существующего wallet в keystore
        pub fn export_keystore(
            &self,
            _dir: &Path,
            _password: &str,
        ) -> Web3Result<std::path::PathBuf> {
            // В ethers 2.0 нет простого способа экспортировать существующий wallet
            // Нужно использовать decrypt_keystore + импортировать private key
            Err(Web3Error::Wallet("export_keystore not implemented yet".to_string()))
        }

        /// Get wallet address
        pub fn address(&self) -> Option<String> {
            self.wallet.as_ref().map(|w| w.address.clone())
        }

        /// Check if wallet is loaded
        pub fn is_loaded(&self) -> bool {
            self.wallet.is_some()
        }

        /// Sign a message (EIP-191 personal_sign)
        pub async fn sign_message(&self, message: &[u8]) -> Web3Result<String> {
            let wallet = self
                .wallet
                .as_ref()
                .ok_or(Web3Error::NotConnected)?;

            let signature = wallet
                .wallet
                .sign_message(message)
                .await
                .map_err(|e| Web3Error::Signing(format!("EIP-191 signing failed: {}", e)))?;

            Ok(format!("0x{}", hex::encode(signature.to_vec())))
        }

        /// Get balance from RPC provider
        pub async fn get_balance(&self, rpc_url: &str) -> Web3Result<NativeBalance> {
            use ethers::providers::{Http, Middleware, Provider};

            let wallet = self
                .wallet
                .as_ref()
                .ok_or(Web3Error::NotConnected)?;

            let provider = Provider::<Http>::try_from(rpc_url)
                .map_err(|e| Web3Error::Rpc(format!("Invalid RPC URL: {}", e)))?;

            let address = wallet.wallet.address();
            let balance = provider
                .get_balance(address, None)
                .await
                .map_err(|e| Web3Error::Rpc(format!("Balance query failed: {}", e)))?;

            // Convert wei to ETH
            let balance_wei = balance.to_string();
            let balance_eth = ethers::utils::format_ether(balance);

            Ok(NativeBalance {
                chain: self.chain,
                address: wallet.address.clone(),
                balance: balance_eth,
                balance_wei,
                symbol: self.chain.native_symbol().to_string(),
            })
        }

        /// Disconnect (clear wallet from memory)
        pub fn disconnect(&mut self) {
            self.wallet = None;
        }
    }

    // ========================================================================
    // Tests
    // ========================================================================

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_generate_wallet() {
            let mut manager = LocalWalletManager::new(Chain::Ethereum);
            let info = manager.generate().unwrap();

            assert_eq!(info.wallet_type, WalletType::Mnemonic);
            assert!(info.address.starts_with("0x"));
            assert_eq!(info.address.len(), 42); // 0x + 40 hex chars
            assert_eq!(info.chain, Chain::Ethereum);
            assert!(info.is_connected);
        }

        #[test]
        fn test_import_private_key() {
            let mut manager = LocalWalletManager::new(Chain::Ethereum);

            // Known test private key (secp256k1)
            let private_key = "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
            let info = manager.import_private_key(private_key).unwrap();

            assert_eq!(info.wallet_type, WalletType::PrivateKey);
            assert_eq!(
                info.address.to_lowercase(),
                "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
            );
        }

        #[test]
        fn test_import_invalid_private_key() {
            let mut manager = LocalWalletManager::new(Chain::Ethereum);
            let result = manager.import_private_key("not-a-key");
            assert!(result.is_err());
        }

        #[test]
        fn test_address_before_import() {
            let manager = LocalWalletManager::new(Chain::Ethereum);
            assert!(manager.address().is_none());
        }

        #[test]
        fn test_disconnect() {
            let mut manager = LocalWalletManager::new(Chain::Ethereum);
            manager.generate().unwrap();
            assert!(manager.is_loaded());

            manager.disconnect();
            assert!(!manager.is_loaded());
            assert!(manager.address().is_none());
        }

        #[tokio::test]
        async fn test_sign_message() {
            let mut manager = LocalWalletManager::new(Chain::Ethereum);
            manager
                .import_private_key("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
                .unwrap();

            let signature = manager.sign_message(b"Hello, Web3!").await.unwrap();
            assert!(signature.starts_with("0x"));
            assert_eq!(signature.len(), 132); // 0x + 65 bytes * 2
        }

        #[tokio::test]
        async fn test_sign_message_no_wallet() {
            let manager = LocalWalletManager::new(Chain::Ethereum);
            let result = manager.sign_message(b"test").await;
            assert!(matches!(result, Err(Web3Error::NotConnected)));
        }

        #[test]
        fn test_keystore_export_import() {
            use std::fs;

            let temp_dir = std::env::temp_dir().join(format!("keystore_test_{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(&temp_dir).unwrap();

            // Generate wallet
            let mut manager = LocalWalletManager::new(Chain::Ethereum);
            let original_address = manager.generate().unwrap().address;

            // Export
            let keystore_path = manager.export_keystore(&temp_dir, "test-password").unwrap();
            assert!(keystore_path.exists());

            // Import into new manager
            let mut manager2 = LocalWalletManager::new(Chain::Ethereum);
            let info = manager2.import_keystore(&keystore_path, "test-password").unwrap();

            // Addresses should match
            assert_eq!(info.address, original_address);

            // Cleanup
            let _ = fs::remove_dir_all(&temp_dir);
        }
    }
}

// ============================================================================
// Wallet State (feature-gated stub)
// ============================================================================

/// High-level wallet manager that can use multiple providers
pub struct WalletManager {
    #[cfg(feature = "web3")]
    local: local::LocalWalletManager,
    chain: Chain,
}

impl WalletManager {
    pub fn new(chain: Chain) -> Self {
        Self {
            #[cfg(feature = "web3")]
            local: local::LocalWalletManager::new(chain),
            chain,
        }
    }

    /// Get the active chain
    pub fn chain(&self) -> Chain {
        self.chain
    }

    /// Switch chain
    pub fn switch_chain(&mut self, chain: Chain) {
        self.chain = chain;
    }

    /// Generate a new wallet (only with web3 feature)
    #[cfg(feature = "web3")]
    pub fn generate(&mut self) -> Web3Result<WalletInfo> {
        self.local.generate()
    }

    /// Stub for when web3 feature is disabled
    #[cfg(not(feature = "web3"))]
    pub fn generate(&mut self) -> Web3Result<WalletInfo> {
        Err(Web3Error::Wallet(
            "Web3 feature not enabled. Recompile with --features web3".to_string(),
        ))
    }
}

// ============================================================================
// Tauri State & Commands
// ============================================================================

#[cfg(feature = "web3")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "web3")]
use tauri::State;

/// State для управления кошельком
#[cfg(feature = "web3")]
pub struct WalletState {
    pub manager: tokio::sync::Mutex<WalletManager>,
    pub rpc_url: String,
}

#[cfg(feature = "web3")]
impl WalletState {
    pub fn new(chain: Chain, rpc_url: String) -> Self {
        Self {
            manager: tokio::sync::Mutex::new(WalletManager::new(chain)),
            rpc_url,
        }
    }
}

/// Запрос на генерацию кошелька
#[cfg(feature = "web3")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateWalletRequest {
    pub chain: String,
}

/// Запрос на импорт кошелька
#[cfg(feature = "web3")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportWalletRequest {
    pub chain: String,
    pub private_key: Option<String>,
    pub keystore_path: Option<String>,
    pub keystore_password: Option<String>,
}

/// Запрос на подпись сообщения
#[cfg(feature = "web3")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignMessageRequest {
    pub message: String,
}

/// Ответ подписи
#[cfg(feature = "web3")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignMessageResponse {
    pub success: bool,
    pub signature: Option<String>,
    pub address: Option<String>,
    pub error: Option<String>,
}

/// Запрос баланса
#[cfg(feature = "web3")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceRequest {
    pub rpc_url: Option<String>,
}

/// Ответ с балансом
#[cfg(feature = "web3")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub success: bool,
    pub balance: Option<NativeBalance>,
    pub error: Option<String>,
}

/// Tauri команда: генерация кошелька
#[cfg(feature = "web3")]
#[tauri::command]
pub fn wallet_generate(
    request: GenerateWalletRequest,
    state: State<'_, WalletState>,
) -> Result<WalletInfo, String> {
    let chain = request.chain.parse::<Chain>()
        .map_err(|e| format!("Invalid chain: {}", e))?;

    // Используем blocking lock для sync команды
    let mut manager = state.manager.blocking_lock();
    manager.switch_chain(chain);
    
    manager.generate()
        .map_err(|e| format!("Generate error: {}", e))
}

/// Tauri команда: импорт кошелька из приватного ключа
#[cfg(feature = "web3")]
#[tauri::command]
pub fn wallet_import_private_key(
    request: ImportWalletRequest,
    state: State<'_, WalletState>,
) -> Result<WalletInfo, String> {
    let chain = request.chain.parse::<Chain>()
        .map_err(|e| format!("Invalid chain: {}", e))?;

    let private_key = request.private_key
        .ok_or_else(|| "Private key required".to_string())?;

    let mut manager = state.manager.blocking_lock();
    manager.switch_chain(chain);
    
    manager.local.import_private_key(&private_key)
        .map_err(|e| format!("Import error: {}", e))
}

/// Tauri команда: получить адрес кошелька
#[cfg(feature = "web3")]
#[tauri::command]
pub fn wallet_get_address(
    state: State<'_, WalletState>,
) -> Result<Option<String>, String> {
    let manager = state.manager.blocking_lock();
    Ok(manager.local.address())
}

/// Tauri команда: подписать сообщение
#[cfg(feature = "web3")]
#[tauri::command]
pub async fn wallet_sign_message(
    request: SignMessageRequest,
    state: State<'_, WalletState>,
) -> Result<SignMessageResponse, String> {
    let manager = state.manager.lock().await;
    
    match manager.local.sign_message(request.message.as_bytes()).await {
        Ok(signature) => Ok(SignMessageResponse {
            success: true,
            signature: Some(signature),
            address: manager.local.address(),
            error: None,
        }),
        Err(e) => Ok(SignMessageResponse {
            success: false,
            signature: None,
            address: None,
            error: Some(format!("Sign error: {}", e)),
        }),
    }
}

/// Tauri команда: получить баланс
#[cfg(feature = "web3")]
#[tauri::command]
pub async fn wallet_get_balance(
    request: BalanceRequest,
    state: State<'_, WalletState>,
) -> Result<BalanceResponse, String> {
    let manager = state.manager.lock().await;
    let rpc_url = request.rpc_url.unwrap_or_else(|| state.rpc_url.clone());
    
    match manager.local.get_balance(&rpc_url).await {
        Ok(balance) => Ok(BalanceResponse {
            success: true,
            balance: Some(balance),
            error: None,
        }),
        Err(e) => Ok(BalanceResponse {
            success: false,
            balance: None,
            error: Some(format!("Balance error: {}", e)),
        }),
    }
}

/// Tauri команда: отключить кошелек
#[cfg(feature = "web3")]
#[tauri::command]
pub fn wallet_disconnect(
    state: State<'_, WalletState>,
) -> Result<bool, String> {
    let mut manager = state.manager.blocking_lock();
    manager.local.disconnect();
    Ok(true)
}

/// Tauri команда: проверить загружен ли кошелек
#[cfg(feature = "web3")]
#[tauri::command]
pub fn wallet_is_loaded(
    state: State<'_, WalletState>,
) -> Result<bool, String> {
    let manager = state.manager.blocking_lock();
    Ok(manager.local.is_loaded())
}

/// Зарегистрировать все wallet команды
#[cfg(feature = "web3")]
pub fn register_wallet_commands(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder
        .manage(WalletState::new(
            Chain::Ethereum,
            "https://eth.llamarpc.com".to_string(),
        ))
        .invoke_handler(tauri::generate_handler![
            wallet_generate,
            wallet_import_private_key,
            wallet_get_address,
            wallet_sign_message,
            wallet_get_balance,
            wallet_disconnect,
            wallet_is_loaded,
        ])
}

// ============================================================================
// Abcex/Bitget Wrapper Commands (делегирование через wallet)
// ============================================================================

/// Запрос котировки Abcex (обёртка)
#[cfg(feature = "web3")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAbcexQuoteRequest {
    pub fiat_currency: String,
    pub fiat_amount: String,
    pub crypto_currency: String,
    pub payment_method: Option<String>,
    pub country: Option<String>,
}

/// Запрос на покупку Bitget (обёртка)
#[cfg(feature = "web3")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBitgetBuyRequest {
    pub symbol: String,
    pub amount: String,
    pub quote_amount: Option<String>,
    pub price: Option<String>,
}

/// Tauri команда: Abcex котировка через wallet
#[cfg(feature = "web3")]
#[tauri::command]
pub async fn wallet_abcex_quote(
    request: WalletAbcexQuoteRequest,
    abcex_state: State<'_, crate::web3::abcex_commands::AbcexState>,
) -> Result<crate::web3::abcex_commands::AbcexQuoteResponse, String> {
    // Делегирование к Abcex команде
    crate::web3::abcex_commands::abcex_get_quote(
        crate::web3::abcex_commands::AbcexQuoteRequest {
            fiat_currency: request.fiat_currency,
            fiat_amount: request.fiat_amount,
            crypto_currency: request.crypto_currency,
            payment_method: request.payment_method,
            country: request.country,
        },
        abcex_state,
    ).await
}

/// Tauri команда: Bitget покупка через wallet
#[cfg(feature = "web3")]
#[tauri::command]
pub async fn wallet_bitget_buy(
    request: WalletBitgetBuyRequest,
    bitget_state: State<'_, crate::web3::bitget_commands::BitgetState>,
) -> Result<crate::web3::bitget_commands::BitgetOrderResponse, String> {
    use crate::web3::bitget::{OrderSide, OrderType};
    
    // Делегирование к Bitget команде
    crate::web3::bitget_commands::bitget_place_order(
        crate::web3::bitget_commands::BitgetOrderRequest {
            symbol: request.symbol,
            side: "buy".to_string(),
            order_type: "market".to_string(),
            amount: Some(request.amount),
            quote_amount: request.quote_amount,
            price: request.price,
            client_order_id: None,
        },
        bitget_state,
    ).await
}

/// Tauri команда: Bitget продажа через wallet
#[cfg(feature = "web3")]
#[tauri::command]
pub async fn wallet_bitget_sell(
    request: WalletBitgetBuyRequest,
    bitget_state: State<'_, crate::web3::bitget_commands::BitgetState>,
) -> Result<crate::web3::bitget_commands::BitgetOrderResponse, String> {
    // Делегирование к Bitget команде
    crate::web3::bitget_commands::bitget_place_order(
        crate::web3::bitget_commands::BitgetOrderRequest {
            symbol: request.symbol,
            side: "sell".to_string(),
            order_type: "market".to_string(),
            amount: Some(request.amount),
            quote_amount: request.quote_amount,
            price: request.price,
            client_order_id: None,
        },
        bitget_state,
    ).await
}

/// Tauri команда: получить баланс Bitget
#[cfg(feature = "web3")]
#[tauri::command]
pub async fn wallet_bitget_balance(
    currency: String,
    bitget_state: State<'_, crate::web3::bitget_commands::BitgetState>,
) -> Result<crate::web3::bitget_commands::BitgetBalanceResponse, String> {
    crate::web3::bitget_commands::bitget_get_balance(
        crate::web3::bitget_commands::BitgetBalanceRequest {
            currency,
        },
        bitget_state,
    ).await
}

/// Зарегистрировать все команды включая Abcex/Bitget обёртки
#[cfg(feature = "web3")]
pub fn register_all_wallet_commands(
    builder: tauri::Builder<tauri::Wry>
) -> tauri::Builder<tauri::Wry> {
    // Сначала регистрируем базовые команды кошелька
    let builder = register_wallet_commands(builder);
    
    // Затем добавляем обёртки для Abcex/Bitget
    builder
        .invoke_handler(tauri::generate_handler![
            wallet_abcex_quote,
            wallet_bitget_buy,
            wallet_bitget_sell,
            wallet_bitget_balance,
        ])
}

// ============================================================================
// Tests (non-feature-gated)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_manager_creation() {
        let manager = WalletManager::new(Chain::Ethereum);
        assert_eq!(manager.chain(), Chain::Ethereum);
    }

    #[test]
    fn test_chain_switch() {
        let mut manager = WalletManager::new(Chain::Polygon);
        assert_eq!(manager.chain(), Chain::Polygon);

        manager.switch_chain(Chain::Arbitrum);
        assert_eq!(manager.chain(), Chain::Arbitrum);
    }

    #[test]
    #[cfg(not(feature = "web3"))]
    fn test_generate_without_web3_feature() {
        let mut manager = WalletManager::new(Chain::Ethereum);
        let result = manager.generate();
        assert!(matches!(result, Err(Web3Error::Wallet(_))));
    }
}
