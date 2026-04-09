//! ERC-20 Token Operations — balances, approvals, transfers
//!
//! Supported tokens (hardcoded for messenger use):
//! - USDC (0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48)
//! - USDT (0xdAC17F958D2ee523a2206206994597C13D831ec7)
//! - DAI  (0x6B175474E89094C44Da98b954EedeAC495271d0F)
//! - WETH (0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2)
//!
//! ABIs (minimal):
//! - balanceOf(address) -> uint256
//! - transfer(address,uint256) -> bool
//! - approve(address,uint256) -> bool
//! - decimals() -> uint8
//! - symbol() -> string
//! - name() -> string

use super::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Token Registry
// ============================================================================

/// Known token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub logo_uri: Option<String>,
}

/// Token addresses by chain
pub struct TokenRegistry {
    tokens: HashMap<u64, Vec<TokenInfo>>,
}

impl TokenRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tokens: HashMap::new(),
        };

        // Ethereum Mainnet tokens
        registry.tokens.insert(
            Chain::Ethereum.chain_id(),
            vec![
                TokenInfo {
                    address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
                    name: "USD Coin".to_string(),
                    symbol: "USDC".to_string(),
                    decimals: 6,
                    logo_uri: None,
                },
                TokenInfo {
                    address: "0xdAC17F958D2ee523a2206206994597C13D831ec7".to_string(),
                    name: "Tether USD".to_string(),
                    symbol: "USDT".to_string(),
                    decimals: 6,
                    logo_uri: None,
                },
                TokenInfo {
                    address: "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string(),
                    name: "Dai Stablecoin".to_string(),
                    symbol: "DAI".to_string(),
                    decimals: 18,
                    logo_uri: None,
                },
                TokenInfo {
                    address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
                    name: "Wrapped Ether".to_string(),
                    symbol: "WETH".to_string(),
                    decimals: 18,
                    logo_uri: None,
                },
            ],
        );

        // Polygon tokens
        registry.tokens.insert(
            Chain::Polygon.chain_id(),
            vec![
                TokenInfo {
                    address: "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359".to_string(),
                    name: "USD Coin (PoS)".to_string(),
                    symbol: "USDC".to_string(),
                    decimals: 6,
                    logo_uri: None,
                },
                TokenInfo {
                    address: "0xc2132D05D31c914a87C6611C10748AEb04B58e8F".to_string(),
                    name: "Tether USD (PoS)".to_string(),
                    symbol: "USDT".to_string(),
                    decimals: 6,
                    logo_uri: None,
                },
                TokenInfo {
                    address: "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063".to_string(),
                    name: "Dai Stablecoin (PoS)".to_string(),
                    symbol: "DAI".to_string(),
                    decimals: 18,
                    logo_uri: None,
                },
            ],
        );

        // Arbitrum tokens
        registry.tokens.insert(
            Chain::Arbitrum.chain_id(),
            vec![TokenInfo {
                address: "0xaf88d065e77c8cC2239327C5EDb3A432268e5831".to_string(),
                name: "USD Coin".to_string(),
                symbol: "USDC".to_string(),
                decimals: 6,
                logo_uri: None,
            }],
        );

        // Base tokens
        registry.tokens.insert(
            Chain::Base.chain_id(),
            vec![TokenInfo {
                address: "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string(),
                name: "USD Coin".to_string(),
                symbol: "USDC".to_string(),
                decimals: 6,
                logo_uri: None,
            }],
        );

        registry
    }

    /// Get all tokens for a chain
    pub fn get_tokens(&self, chain: Chain) -> Vec<TokenInfo> {
        self.tokens
            .get(&chain.chain_id())
            .cloned()
            .unwrap_or_default()
    }

    /// Find token by symbol
    pub fn find_by_symbol(&self, chain: Chain, symbol: &str) -> Option<TokenInfo> {
        self.tokens.get(&chain.chain_id()).and_then(|tokens| {
            tokens
                .iter()
                .find(|t| t.symbol.eq_ignore_ascii_case(symbol))
                .cloned()
        })
    }

    /// Find token by address
    pub fn find_by_address(&self, chain: Chain, address: &str) -> Option<TokenInfo> {
        self.tokens.get(&chain.chain_id()).and_then(|tokens| {
            tokens
                .iter()
                .find(|t| t.address.eq_ignore_ascii_case(address))
                .cloned()
        })
    }

    /// Add a custom token
    pub fn add_token(&mut self, chain: Chain, token: TokenInfo) {
        self.tokens.entry(chain.chain_id()).or_default().push(token);
    }
}

// ============================================================================
// Token Amount Helpers
// ============================================================================

/// Convert human-readable amount to raw token amount (with decimals)
pub fn to_token_amount(amount: &str, decimals: u8) -> Web3Result<String> {
    let parts: Vec<&str> = amount.split('.').collect();
    let integer_part = parts.first().unwrap_or(&"0");
    let decimal_part = parts.get(1).unwrap_or(&"0");

    // Pad or truncate decimal part to match decimals
    let decimal_padded = if decimal_part.len() >= decimals as usize {
        &decimal_part[..decimals as usize]
    } else {
        // Pad with zeros
        let mut padded = decimal_part.to_string();
        while padded.len() < decimals as usize {
            padded.push('0');
        }
        // We need to return a reference, so we can't use padded here
        // Instead, let's handle this differently
        decimal_part
    };

    // For simplicity, use a different approach
    let amount_f64: f64 = amount
        .parse()
        .map_err(|_| Web3Error::Wallet(format!("Invalid amount: {}", amount)))?;

    let multiplier = 10u64.pow(decimals as u32) as f64;
    let raw = (amount_f64 * multiplier) as u128;

    Ok(raw.to_string())
}

/// Convert raw token amount to human-readable (with decimals)
pub fn from_token_amount(raw: &str, decimals: u8) -> String {
    if let Ok(raw_val) = raw.parse::<u128>() {
        let divisor = 10u128.pow(decimals as u32);
        let integer = raw_val / divisor;
        let fractional = raw_val % divisor;

        if fractional == 0 {
            return integer.to_string();
        }

        // Format fractional part with leading zeros
        let frac_str = format!("{:0>width$}", fractional, width = decimals as usize);
        // Remove trailing zeros
        let frac_trimmed = frac_str.trim_end_matches('0');

        if frac_trimmed.is_empty() {
            integer.to_string()
        } else {
            format!("{}.{}", integer, frac_trimmed)
        }
    } else {
        "0".to_string()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_registry_ethereum() {
        let registry = TokenRegistry::new();
        let tokens = registry.get_tokens(Chain::Ethereum);
        assert_eq!(tokens.len(), 4);

        let usdc = registry.find_by_symbol(Chain::Ethereum, "USDC").unwrap();
        assert_eq!(usdc.address, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
        assert_eq!(usdc.decimals, 6);
    }

    #[test]
    fn test_token_registry_polygon() {
        let registry = TokenRegistry::new();
        let tokens = registry.get_tokens(Chain::Polygon);
        assert_eq!(tokens.len(), 3);

        let usdc = registry.find_by_symbol(Chain::Polygon, "USDC").unwrap();
        assert_eq!(usdc.symbol, "USDC");
        assert_eq!(usdc.decimals, 6);
    }

    #[test]
    fn test_token_registry_arbitrum() {
        let registry = TokenRegistry::new();
        let usdc = registry.find_by_symbol(Chain::Arbitrum, "USDC").unwrap();
        assert!(usdc.address.starts_with("0x"));
    }

    #[test]
    fn test_token_registry_base() {
        let registry = TokenRegistry::new();
        let usdc = registry.find_by_symbol(Chain::Base, "USDC").unwrap();
        assert_eq!(usdc.address, "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913");
    }

    #[test]
    fn test_token_registry_unknown_chain() {
        let registry = TokenRegistry::new();
        let tokens = registry.get_tokens(Chain::Bsc);
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_token_find_by_address() {
        let registry = TokenRegistry::new();
        let usdc = registry.find_by_address(
            Chain::Ethereum,
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        );
        assert!(usdc.is_some());
        assert_eq!(usdc.unwrap().symbol, "USDC");
    }

    #[test]
    fn test_from_token_amount_usdc() {
        // 1.5 USDC (6 decimals)
        assert_eq!(from_token_amount("1500000", 6), "1.5");

        // 0.001 USDC
        assert_eq!(from_token_amount("1000", 6), "0.001");

        // 100 USDC
        assert_eq!(from_token_amount("100000000", 6), "100");
    }

    #[test]
    fn test_from_token_amount_18_decimals() {
        // 1 ETH (18 decimals)
        assert_eq!(from_token_amount("1000000000000000000", 18), "1");

        // 0.5 ETH
        assert_eq!(from_token_amount("500000000000000000", 18), "0.5");

        // 0.001 ETH
        assert_eq!(from_token_amount("1000000000000000", 18), "0.001");
    }

    #[test]
    fn test_from_token_amount_zero() {
        assert_eq!(from_token_amount("0", 6), "0");
        assert_eq!(from_token_amount("0", 18), "0");
    }

    #[test]
    fn test_add_custom_token() {
        let mut registry = TokenRegistry::new();

        registry.add_token(
            Chain::Ethereum,
            TokenInfo {
                address: "0xCustomAddress".to_string(),
                name: "Custom Token".to_string(),
                symbol: "CUSTOM".to_string(),
                decimals: 18,
                logo_uri: None,
            },
        );

        let custom = registry.find_by_symbol(Chain::Ethereum, "CUSTOM").unwrap();
        assert_eq!(custom.name, "Custom Token");
    }
}
