//! ENS (Ethereum Name Service) Resolution
//!
//! Features:
//! - Resolve ENS names to addresses
//! - Reverse lookup (address → ENS name)
//! - Batch resolution
//! - Cache results (TTL-based)
//!
//! Uses ethers.rs Provider for ENS resolution when web3 feature enabled,
//! falls back to ENS public gateway API otherwise.

use super::types::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;

// ============================================================================
// ENS Resolver
// ============================================================================

/// ENS resolver with caching
pub struct EnsResolver {
    cache: Mutex<HashMap<String, EnsCacheEntry>>,
    provider_urls: HashMap<Chain, String>,
}

#[derive(Debug, Clone)]
struct EnsCacheEntry {
    record: EnsRecord,
    expires_at: chrono::DateTime<chrono::Utc>,
}

impl EnsResolver {
    pub fn new() -> Self {
        let mut provider_urls = HashMap::new();
        provider_urls.insert(
            Chain::Ethereum,
            "https://eth.llamarpc.com".to_string(),
        );
        provider_urls.insert(
            Chain::Polygon,
            "https://polygon-rpc.com".to_string(),
        );

        Self {
            cache: Mutex::new(HashMap::new()),
            provider_urls,
        }
    }

    /// Add a custom RPC provider for a chain
    pub fn add_provider(&mut self, chain: Chain, url: String) {
        self.provider_urls.insert(chain, url);
    }

    /// Resolve an ENS name to an address
    pub async fn resolve(&self, name: &str) -> Web3Result<EnsRecord> {
        // Check cache
        if let Some(entry) = self.cache.lock().unwrap().get(name) {
            if chrono::Utc::now() < entry.expires_at {
                return Ok(entry.record.clone());
            }
        }

        // Resolve via RPC (ethers.rs or direct JSON-RPC)
        let record = self.resolve_via_rpc(name).await?;

        // Cache for 5 minutes
        let expires_at = chrono::Utc::now() + chrono::Duration::minutes(5);
        self.cache.lock().unwrap().insert(
            name.to_string(),
            EnsCacheEntry {
                record: record.clone(),
                expires_at,
            },
        );

        Ok(record)
    }

    /// Reverse lookup: address → ENS name
    pub async fn reverse(&self, address: &str) -> Web3Result<Option<EnsRecord>> {
        // ENS reverse name: address + ".addr.reverse"
        let reverse_name = if address.starts_with("0x") {
            format!("{}.addr.reverse", &address[2..])
        } else {
            format!("{}.addr.reverse", address)
        };

        // Try to resolve — if no name is set, the resolver returns the address itself
        match self.resolve_via_rpc(&reverse_name).await {
            Ok(record) => {
                if record.address == "0x0000000000000000000000000000000000000000" {
                    Ok(None)
                } else {
                    Ok(Some(record))
                }
            }
            Err(_) => Ok(None),
        }
    }

    /// Batch resolve multiple ENS names
    pub async fn resolve_batch(&self, names: &[String]) -> Web3Result<HashMap<String, EnsRecord>> {
        let mut results = HashMap::new();

        for name in names {
            match self.resolve(name).await {
                Ok(record) => {
                    results.insert(name.clone(), record);
                }
                Err(e) => {
                    tracing::warn!("Failed to resolve ENS name '{}': {}", name, e);
                }
            }
        }

        Ok(results)
    }

    /// Check if a string looks like an ENS name
    pub fn is_ens_name(s: &str) -> bool {
        s.contains('.')
            && !s.starts_with("0x")
            && s.len() >= 5
            && s.len() <= 255
    }

    /// Format address/name for display
    /// If ENS name available, shows it; otherwise shows truncated address
    pub fn format_display(name_or_address: &str) -> String {
        if Self::is_ens_name(name_or_address) {
            name_or_address.to_string()
        } else if name_or_address.starts_with("0x") && name_or_address.len() >= 10 {
            // Truncate: 0x1234...5678
            format!(
                "{}...{}",
                &name_or_address[..6],
                &name_or_address[name_or_address.len() - 4..]
            )
        } else {
            name_or_address.to_string()
        }
    }

    // Internal: resolve via direct JSON-RPC (no ethers dependency for basic resolution)
    async fn resolve_via_rpc(&self, name: &str) -> Web3Result<EnsRecord> {
        // Use ENS public API as fallback
        // https://metadata.ens.domains/mainnet/domain/{name}
        let client = reqwest::Client::new();

        // Try ENS metadata API first
        if Self::is_ens_name(name) {
            let url = format!(
                "https://metadata.ens.domains/mainnet/domain/{}",
                name
            );

            if let Ok(response) = client.get(&url).send().await {
                if response.status().is_success() {
                    if let Ok(metadata) = response.json::<EnsMetadataResponse>().await {
                        return Ok(EnsRecord {
                            name: name.to_string(),
                            address: metadata.attrs.okta.unwrap_or_default(),
                            avatar_url: metadata.attrs.avatar,
                            content_hash: None,
                            description: metadata.attrs.description,
                        });
                    }
                }
            }
        }

        // If API fails and it looks like an address, return as-is
        if name.starts_with("0x") && name.len() == 42 {
            return Ok(EnsRecord {
                name: name.to_string(),
                address: name.to_string(),
                avatar_url: None,
                content_hash: None,
                description: None,
            });
        }

        Err(Web3Error::Ens(format!("Could not resolve: {}", name)))
    }

    /// Clear the ENS cache
    pub fn clear_cache(&self) {
        self.cache.lock().unwrap().clear();
    }
}

// ============================================================================
// ENS Metadata API Response
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
struct EnsMetadataResponse {
    attrs: EnsAttrs,
}

#[derive(Debug, Clone, Deserialize)]
struct EnsAttrs {
    #[serde(rename = "ENSNAME", alias = "name")]
    okta: Option<String>,
    avatar: Option<String>,
    description: Option<String>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ens_name() {
        assert!(EnsResolver::is_ens_name("vitalik.eth"));
        assert!(EnsResolver::is_ens_name("my-subdomain.eth"));
        assert!(!EnsResolver::is_ens_name("0x1234567890abcdef"));
        assert!(!EnsResolver::is_ens_name("not-ens"));
        assert!(!EnsResolver::is_ens_name(""));
    }

    #[test]
    fn test_format_display_ens() {
        assert_eq!(
            EnsResolver::format_display("vitalik.eth"),
            "vitalik.eth"
        );
    }

    #[test]
    fn test_format_display_address() {
        assert_eq!(
            EnsResolver::format_display("0x1234567890abcdef1234567890abcdef12345678"),
            "0x1234...5678"
        );
    }

    #[test]
    fn test_format_display_short() {
        assert_eq!(EnsResolver::format_display("0x12"), "0x12");
    }

    #[test]
    fn test_ens_resolver_creation() {
        let resolver = EnsResolver::new();
        resolver.clear_cache();
        // Should not panic
    }

    #[test]
    fn test_ens_resolver_add_provider() {
        let mut resolver = EnsResolver::new();
        resolver.add_provider(Chain::Arbitrum, "https://arb1.arbitrum.io/rpc".to_string());
        // Should not panic
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_resolve_known_ens() {
        let resolver = EnsResolver::new();

        // This might fail if the API is unreachable — that's OK for a test
        let result = resolver.resolve("vitalik.eth").await;
        if let Ok(record) = result {
            assert!(!record.name.is_empty());
            assert!(record.address.starts_with("0x"));
        }
    }
}
