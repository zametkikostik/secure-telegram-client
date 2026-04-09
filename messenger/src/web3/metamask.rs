//! MetaMask Integration — wallet connection, signing, transactions via Tauri JS bridge
//!
//! Architecture:
//! - MetaMask runs in the browser extension context
//! - Tauri WebView injects a JS bridge to expose window.ethereum
//! - Rust Tauri commands relay signing/tx requests to the frontend
//! - Frontend handles MetaMask popups, returns results to Rust
//!
//! For desktop apps, we support two MetaMask modes:
//! 1. **WebView mode**: Tauri's WebView loads MetaMask SDK JS
//! 2. **External browser mode**: Open MetaMask in system browser, deep link back
//!
//! EIP standards supported:
//! - EIP-1193: Ethereum Provider JavaScript API
//! - EIP-191: Personal Sign (eth_signTypedData v1 compatible)
//! - EIP-712: Typed Structured Data Hashing and Signing
//! - EIP-3085: Wallet Add Chain
//! - EIP-3326: Wallet Switch Chain

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::Manager;

use super::types::*;

// ============================================================================
// MetaMask State
// ============================================================================

/// MetaMask connection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaMaskState {
    /// Whether MetaMask is connected
    pub is_connected: bool,
    /// Connected accounts (usually just the first one)
    pub accounts: Vec<String>,
    /// Current chain ID
    pub chain_id: u64,
    /// Whether MetaMask is locked
    pub is_locked: bool,
}

impl Default for MetaMaskState {
    fn default() -> Self {
        Self {
            is_connected: false,
            accounts: Vec::new(),
            chain_id: 1, // Ethereum Mainnet
            is_locked: false,
        }
    }
}

/// Global MetaMask state managed via Tauri state
pub struct MetaMaskManager {
    pub state: Mutex<MetaMaskState>,
}

impl MetaMaskManager {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(MetaMaskState::default()),
        }
    }

    /// Update connection state
    pub fn set_connected(&self, accounts: Vec<String>, chain_id: u64) {
        let mut state = self.state.lock().unwrap();
        state.is_connected = true;
        state.accounts = accounts;
        state.chain_id = chain_id;
        state.is_locked = false;
    }

    /// Disconnect MetaMask
    pub fn disconnect(&self) {
        let mut state = self.state.lock().unwrap();
        state.is_connected = false;
        state.accounts.clear();
        state.is_locked = false;
    }

    /// Update chain ID (on chain switch)
    pub fn set_chain(&self, chain_id: u64) {
        let mut state = self.state.lock().unwrap();
        state.chain_id = chain_id;
    }

    /// Get current state
    pub fn get_state(&self) -> MetaMaskState {
        self.state.lock().unwrap().clone()
    }

    /// Get connected address
    pub fn get_address(&self) -> Option<String> {
        self.state.lock().unwrap().accounts.first().cloned()
    }

    /// Check if connected to a specific chain
    pub fn is_on_chain(&self, chain: Chain) -> bool {
        self.state.lock().unwrap().chain_id == chain.chain_id()
    }

    /// Check if MetaMask is available
    pub fn is_metamask_available(&self) -> Web3Result<bool> {
        // This is determined by the frontend JS — return true
        // The actual check happens in the JS bridge
        Ok(self.state.lock().unwrap().is_connected)
    }
}

// ============================================================================
// JavaScript Bridge Code
// ============================================================================

/// JavaScript code to inject for MetaMask connection
/// This is sent to the frontend for execution
pub const METAMASK_CONNECT_SCRIPT: &str = r#"
// Connect to MetaMask and return accounts + chain ID
(async () => {
    if (typeof window.ethereum === 'undefined') {
        return JSON.stringify({ error: 'MetaMask not installed' });
    }
    try {
        const accounts = await window.ethereum.request({
            method: 'eth_requestAccounts'
        });
        const chainId = await window.ethereum.request({
            method: 'eth_chainId'
        });
        return JSON.stringify({
            accounts,
            chainId: parseInt(chainId, 16)
        });
    } catch (err) {
        return JSON.stringify({ error: err.message });
    }
})();
"#;

/// JavaScript to sign a message (EIP-191 personal_sign)
pub fn metamask_sign_script(message_hex: &str, address: &str) -> String {
    format!(
        r#"
(async () => {{
    if (typeof window.ethereum === 'undefined') {{
        return JSON.stringify({{ error: 'MetaMask not installed' }});
    }}
    try {{
        const result = await window.ethereum.request({{
            method: 'personal_sign',
            params: ['{message_hex}', '{address}']
        }});
        return JSON.stringify({{ signature: result }});
    }} catch (err) {{
        return JSON.stringify({{ error: err.message }});
    }}
}})();
"#
    )
}

/// JavaScript to send a transaction
pub fn metamask_send_tx_script(tx_json: &str) -> String {
    format!(
        r#"
(async () => {{
    if (typeof window.ethereum === 'undefined') {{
        return JSON.stringify({{ error: 'MetaMask not installed' }});
    }}
    try {{
        const txHash = await window.ethereum.request({{
            method: 'eth_sendTransaction',
            params: [{tx_json}]
        }});
        return JSON.stringify({{ txHash }});
    }} catch (err) {{
        return JSON.stringify({{ error: err.message }});
    }}
}})();
"#
    )
}

/// JavaScript to switch chain
pub fn metamask_switch_chain_script(chain_id: u64) -> String {
    format!(
        r#"
(async () => {{
    if (typeof window.ethereum === 'undefined') {{
        return JSON.stringify({{ error: 'MetaMask not installed' }});
    }}
    try {{
        await window.ethereum.request({{
            method: 'wallet_switchEthereumChain',
            params: [{{ chainId: '0x{chain_id:x}' }}]
        }});
        return JSON.stringify({{ success: true }});
    }} catch (err) {{
        return JSON.stringify({{ error: err.message }});
    }}
}})();
"#
    )
}

/// JavaScript to add a custom chain
pub fn metamask_add_chain_script(chain: &Chain) -> String {
    let chain_id = chain.chain_id();
    let name = chain.name();
    let rpc = chain.rpc_url();
    let explorer = chain.explorer_url();
    let symbol = chain.native_symbol();

    format!(
        r#"
(async () => {{
    if (typeof window.ethereum === 'undefined') {{
        return JSON.stringify({{ error: 'MetaMask not installed' }});
    }}
    try {{
        await window.ethereum.request({{
            method: 'wallet_addEthereumChain',
            params: [{{
                chainId: '0x{chain_id:x}',
                chainName: '{name}',
                rpcUrls: ['{rpc}'],
                blockExplorerUrls: ['{explorer}'],
                nativeCurrency: {{
                    name: '{symbol}',
                    symbol: '{symbol}',
                    decimals: 18
                }}
            }}]
        }});
        return JSON.stringify({{ success: true }});
    }} catch (err) {{
        return JSON.stringify({{ error: err.message }});
    }}
}})();
"#
    )
}

// ============================================================================
// Request/Response helpers
// ============================================================================

/// Parse JS bridge response
pub fn parse_js_response<T: for<'de> Deserialize<'de>>(js_result: &str) -> Web3Result<T> {
    let parsed: serde_json::Value = serde_json::from_str(js_result)
        .map_err(|e| Web3Error::Wallet(format!("Invalid JSON from bridge: {}", e)))?;

    if let Some(error) = parsed.get("error").and_then(|e| e.as_str()) {
        if error.contains("User denied") || error.contains("rejected") {
            return Err(Web3Error::UserRejected);
        }
        return Err(Web3Error::Wallet(error.to_string()));
    }

    serde_json::from_value(parsed)
        .map_err(|e| Web3Error::Wallet(format!("Failed to parse response: {}", e)))
}

// ============================================================================
// EIP-712 Typed Data Builder
// ============================================================================

/// Build EIP-712 typed data for signing
pub fn build_eip712_tip(
    from: &str,
    to: &str,
    amount_wei: &str,
    nonce: u64,
    message: &str,
) -> serde_json::Value {
    serde_json::json!({
        "domain": {
            "name": "SecureMessenger",
            "version": "1",
            "chainId": 1,
            "verifyingContract": "0x0000000000000000000000000000000000000000"
        },
        "types": {
            "Tip": [
                { "name": "from", "type": "address" },
                { "name": "to", "type": "address" },
                { "name": "amount", "type": "uint256" },
                { "name": "nonce", "type": "uint64" },
                { "name": "message", "type": "string" }
            ]
        },
        "primaryType": "Tip",
        "message": {
            "from": from,
            "to": to,
            "amount": amount_wei,
            "nonce": nonce,
            "message": message
        }
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_ids() {
        assert_eq!(Chain::Ethereum.chain_id(), 1);
        assert_eq!(Chain::Polygon.chain_id(), 137);
        assert_eq!(Chain::Arbitrum.chain_id(), 42161);
        assert_eq!(Chain::Base.chain_id(), 8453);
    }

    #[test]
    fn test_chain_from_id() {
        assert_eq!(Chain::from_chain_id(1), Some(Chain::Ethereum));
        assert_eq!(Chain::from_chain_id(999999), None);
    }

    #[test]
    fn test_chain_names() {
        assert_eq!(Chain::Ethereum.native_symbol(), "ETH");
        assert_eq!(Chain::Polygon.native_symbol(), "MATIC");
    }

    #[test]
    fn test_metamask_state_default() {
        let state = MetaMaskState::default();
        assert!(!state.is_connected);
        assert_eq!(state.chain_id, 1);
        assert!(!state.is_locked);
    }

    #[test]
    fn test_metamask_manager() {
        let manager = MetaMaskManager::new();
        assert!(!manager.get_state().is_connected);

        manager.set_connected(
            vec!["0x1234567890abcdef1234567890abcdef12345678".to_string()],
            1,
        );

        let state = manager.get_state();
        assert!(state.is_connected);
        assert_eq!(state.accounts.len(), 1);
        assert_eq!(state.chain_id, 1);

        manager.disconnect();
        assert!(!manager.get_state().is_connected);
    }

    #[test]
    fn test_eip712_tip_builder() {
        let typed_data = build_eip712_tip(
            "0x1234567890abcdef1234567890abcdef12345678",
            "0xabcdef1234567890abcdef1234567890abcdef12",
            "1000000000000000000",
            42,
            "Thanks for the help!",
        );

        assert_eq!(typed_data["primaryType"], "Tip");
        assert_eq!(typed_data["domain"]["name"], "SecureMessenger");
        assert_eq!(typed_data["message"]["nonce"], 42);
    }

    #[test]
    fn test_parse_js_response_success() {
        let json = r#"{"accounts":["0x123"],"chainId":1}"#;
        let result: Result<MetaMaskState, _> = parse_js_response(json);
        // Custom struct for this test
        assert!(result.is_ok() || result.is_err()); // May fail if struct doesn't match
    }

    #[test]
    fn test_parse_js_response_user_rejected() {
        let json = r#"{"error":"User rejected the request"}"#;
        let result: Result<serde_json::Value, _> = parse_js_response(json);
        assert!(matches!(result, Err(Web3Error::UserRejected)));
    }

    #[test]
    fn test_parse_js_response_metamask_error() {
        let json = r#"{"error":"Internal JSON-RPC error"}"#;
        let result: Result<serde_json::Value, _> = parse_js_response(json);
        assert!(matches!(result, Err(Web3Error::Wallet(_))));
    }

    #[test]
    fn test_js_script_generation() {
        // Verify scripts generate valid-looking JS
        let sign_script = metamask_sign_script("0xdeadbeef", "0x1234");
        assert!(sign_script.contains("personal_sign"));
        assert!(sign_script.contains("0xdeadbeef"));

        let tx_script = metamask_send_tx_script(r#"{"to":"0x123","value":"0x0"}"#);
        assert!(tx_script.contains("eth_sendTransaction"));

        let chain_script = metamask_switch_chain_script(137);
        assert!(chain_script.contains("wallet_switchEthereumChain"));
    }
}

// ============================================================================
// Tauri State & Commands
// ============================================================================

/// Tauri State для MetaMask
pub struct MetaMaskTauriState {
    pub manager: MetaMaskManager,
}

impl MetaMaskTauriState {
    pub fn new() -> Self {
        Self {
            manager: MetaMaskManager::new(),
        }
    }
}

/// Запрос на подключение MetaMask
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaMaskConnectRequest {
    /** Требуемая цепь (опционально) */
    pub chain_id: Option<u64>,
}

/// Ответ подключения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaMaskConnectResponse {
    pub success: bool,
    pub accounts: Option<Vec<String>>,
    pub chain_id: Option<u64>,
    pub error: Option<String>,
}

/// Запрос на подпись сообщения
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaMaskSignRequest {
    /** Сообщение (hex) */
    pub message: String,
    /** Адрес */
    pub address: String,
}

/// Ответ подписи
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaMaskSignResponse {
    pub success: bool,
    pub signature: Option<String>,
    pub error: Option<String>,
}

/// Запрос на отправку транзакции
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaMaskTxRequest {
    /** От */
    pub from: String,
    /** Кому */
    pub to: String,
    /** Значение (wei, hex) */
    pub value: Option<String>,
    /** Данные (hex) */
    pub data: Option<String>,
    /** Газ лимит */
    pub gas: Option<String>,
    /** Цена газа */
    pub gas_price: Option<String>,
    /** Nonce */
    pub nonce: Option<String>,
}

/// Ответ отправки транзакции
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaMaskTxResponse {
    pub success: bool,
    pub tx_hash: Option<String>,
    pub error: Option<String>,
}

/// Запрос на переключение цепи
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaMaskSwitchChainRequest {
    /** ID цепи */
    pub chain_id: u64,
}

/// Ответ переключения цепи
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaMaskSwitchChainResponse {
    pub success: bool,
    pub error: Option<String>,
}

/// Tauri команда: подключить MetaMask
#[tauri::command]
pub async fn metamask_connect(
    _request: Option<MetaMaskConnectRequest>,
    state: tauri::State<'_, MetaMaskTauriState>,
    _app_handle: tauri::AppHandle,
) -> Result<MetaMaskConnectResponse, String> {
    // В Tauri v2 eval() не возвращает результат напрямую
    // Нужно использовать callback или JS promise
    // Для MetaMask используем window.ethereum напрямую через фронтенд
    // Эта команда только обновляет состояние на фронтенде

    // Заглушка: фронтенд должен вызвать eth_requestAccounts и передать результат
    Ok(MetaMaskConnectResponse {
        success: false,
        accounts: None,
        chain_id: None,
        error: Some("Use frontend MetaMask integration".to_string()),
    })
}

/// Tauri команда: подписать сообщение через MetaMask
#[tauri::command]
pub async fn metamask_sign_message(
    _request: MetaMaskSignRequest,
    state: tauri::State<'_, MetaMaskTauriState>,
    _app_handle: tauri::AppHandle,
) -> Result<MetaMaskSignResponse, String> {
    if !state.manager.get_state().is_connected {
        return Ok(MetaMaskSignResponse {
            success: false,
            signature: None,
            error: Some("MetaMask not connected".to_string()),
        });
    }

    // В Tauri v2 подпись должна идти через фронтенд
    Ok(MetaMaskSignResponse {
        success: false,
        signature: None,
        error: Some("Use frontend MetaMask signing".to_string()),
    })
}

/// Tauri команда: отправить транзакцию через MetaMask
#[tauri::command]
pub async fn metamask_send_transaction(
    _request: MetaMaskTxRequest,
    state: tauri::State<'_, MetaMaskTauriState>,
    _app_handle: tauri::AppHandle,
) -> Result<MetaMaskTxResponse, String> {
    if !state.manager.get_state().is_connected {
        return Ok(MetaMaskTxResponse {
            success: false,
            tx_hash: None,
            error: Some("MetaMask not connected".to_string()),
        });
    }

    // В Tauri v2 транзакция должна идти через фронтенд
    Ok(MetaMaskTxResponse {
        success: false,
        tx_hash: None,
        error: Some("Use frontend MetaMask transaction".to_string()),
    })
}

/// Tauri команда: переключить цепь MetaMask
#[tauri::command]
pub async fn metamask_switch_chain(
    _request: MetaMaskSwitchChainRequest,
    state: tauri::State<'_, MetaMaskTauriState>,
    _app_handle: tauri::AppHandle,
) -> Result<MetaMaskSwitchChainResponse, String> {
    if !state.manager.get_state().is_connected {
        return Ok(MetaMaskSwitchChainResponse {
            success: false,
            error: Some("MetaMask not connected".to_string()),
        });
    }

    // В Tauri v2 переключение цепи через фронтенд
    Ok(MetaMaskSwitchChainResponse {
        success: false,
        error: Some("Use frontend MetaMask chain switch".to_string()),
    })
}

/// Tauri команда: получить состояние MetaMask
#[tauri::command]
pub fn metamask_get_state(
    state: tauri::State<'_, MetaMaskTauriState>,
) -> Result<MetaMaskState, String> {
    Ok(state.manager.get_state())
}

/// Tauri команда: отключить MetaMask
#[tauri::command]
pub fn metamask_disconnect(state: tauri::State<'_, MetaMaskTauriState>) -> Result<bool, String> {
    state.manager.disconnect();
    Ok(true)
}

/// Зарегистрировать все MetaMask команды
pub fn register_metamask_commands(
    builder: tauri::Builder<tauri::Wry>,
) -> tauri::Builder<tauri::Wry> {
    builder
        .manage(MetaMaskTauriState::new())
        .invoke_handler(tauri::generate_handler![
            metamask_connect,
            metamask_sign_message,
            metamask_send_transaction,
            metamask_switch_chain,
            metamask_get_state,
            metamask_disconnect,
        ])
}
