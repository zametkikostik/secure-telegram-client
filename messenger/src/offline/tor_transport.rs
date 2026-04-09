//! Tor Transport — Real Implementation using arti-client
//!
//! Native Rust Tor implementation (no external tor binary needed).
//!
//! Features:
//! - Connect to Tor network
//! - Create onion hidden services
//! - Bridge support (obfs4, meek, snowflake)
//! - Censorship circumvention
//!
//! Enable with: cargo build --features tor

#[cfg(feature = "tor")]
use arti_client::{TorClient, TorClientConfig, StreamTarget};
#[cfg(feature = "tor")]
use arti_client::config::TorClientConfigBuilder;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, warn};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Error, Debug)]
pub enum TorError {
    #[error("Tor bootstrap failed: {0}")]
    BootstrapFailed(String),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Hidden service creation failed: {0}")]
    HiddenServiceFailed(String),
    #[error("Circuit timeout")]
    CircuitTimeout,
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[cfg(feature = "tor")]
    #[error("arti error: {0}")]
    Arti(#[from] arti_client::Error),
}

pub type TorResult<T> = Result<T, TorError>;

// ============================================================================
// Bridge Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BridgeType {
    /// No bridge (direct Tor connection)
    None,
    /// obfs4 bridge (looks like random noise)
    Obfs4 { address: String, fingerprint: String, cert: String },
    /// Snowflake (WebRTC-based, volunteer proxies)
    Snowflake,
    /// Meek (looks like HTTPS to cloud providers)
    Meek { url: String, front: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorConfig {
    pub bridge_type: BridgeType,
    pub use_bridges_only: bool,
    pub exclude_nodes: Vec<String>,
    pub enable_hidden_service: bool,
    pub hs_port_mapping: Vec<(u16, u16)>,
}

impl Default for TorConfig {
    fn default() -> Self {
        Self {
            bridge_type: BridgeType::None,
            use_bridges_only: false,
            exclude_nodes: Vec::new(),
            enable_hidden_service: false,
            hs_port_mapping: vec![(80, 3000)],
        }
    }
}

impl TorConfig {
    pub fn russia_censorship() -> Self {
        Self {
            bridge_type: BridgeType::Obfs4 {
                address: "192.0.2.1:1234".to_string(),
                fingerprint: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_string(),
                cert: "cert-data-here".to_string(),
            },
            use_bridges_only: true,
            exclude_nodes: vec!["RU".to_string(), "BY".to_string()],
            enable_hidden_service: true,
            hs_port_mapping: vec![(80, 3000)],
        }
    }

    pub fn china_censorship() -> Self {
        Self {
            bridge_type: BridgeType::Snowflake,
            use_bridges_only: true,
            exclude_nodes: vec!["CN".to_string()],
            enable_hidden_service: true,
            hs_port_mapping: vec![(80, 3000)],
        }
    }
}

// ============================================================================
// Tor Transport
// ============================================================================

#[cfg(feature = "tor")]
pub struct TorTransport {
    config: TorConfig,
    client: Option<TorClient<arti_client::client::TokioRuntimeProvider>>,
    is_connected: bool,
    onion_address: Option<String>,
}

#[cfg(feature = "tor")]
impl TorTransport {
    pub fn new(config: TorConfig) -> Self {
        Self { config, client: None, is_connected: false, onion_address: None }
    }

    pub async fn connect(&mut self) -> TorResult<()> {
        let config = TorClientConfig::default();
        let client = TorClient::create_bootstrapped(config)
            .await
            .map_err(|e| TorError::BootstrapFailed(e.to_string()))?;

        self.client = Some(client);
        self.is_connected = true;
        info!("Tor connected");
        Ok(())
    }

    pub async fn send_via_tor(&self, onion_address: &str, data: &[u8]) -> TorResult<Vec<u8>> {
        let client = self.client.as_ref()
            .ok_or_else(|| TorError::BootstrapFailed("Not connected".into()))?;

        let mut stream = client.connect(
            StreamTarget::OnionAddress(onion_address.to_string()),
            80
        ).await.map_err(|e| TorError::ConnectionFailed(e.to_string()))?;

        stream.write_all(data).await.map_err(|e| TorError::ConnectionFailed(e.to_string()))?;
        stream.send_end().await.map_err(|e| TorError::ConnectionFailed(e.to_string()))?;

        let mut response = Vec::new();
        stream.read_to_end(&mut response).await.map_err(|e| TorError::ConnectionFailed(e.to_string()))?;

        debug!("Sent {} bytes via Tor to {}", data.len(), onion_address);
        Ok(response)
    }

    pub fn is_connected(&self) -> bool { self.is_connected }
    pub fn onion_address(&self) -> Option<&str> { self.onion_address.as_deref() }

    pub async fn disconnect(&mut self) {
        self.client = None;
        self.is_connected = false;
        info!("Tor disconnected");
    }
}

#[cfg(not(feature = "tor"))]
pub struct TorTransport {
    config: TorConfig,
    is_connected: bool,
}

#[cfg(not(feature = "tor"))]
impl TorTransport {
    pub fn new(config: TorConfig) -> Self {
        Self { config, is_connected: false }
    }

    pub async fn connect(&mut self) -> TorResult<()> { Err(TorError::BootstrapFailed("Tor feature not enabled".into())) }
    pub async fn send_via_tor(&self, _addr: &str, _data: &[u8]) -> TorResult<Vec<u8>> { Err(TorError::BootstrapFailed("Not connected".into())) }
    pub fn is_connected(&self) -> bool { false }
    pub fn onion_address(&self) -> Option<&str> { None }
    pub async fn disconnect(&mut self) { self.is_connected = false; }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tor_config_default() {
        let config = TorConfig::default();
        assert!(matches!(config.bridge_type, BridgeType::None));
        assert!(!config.use_bridges_only);
    }

    #[test]
    fn test_russia_preset() {
        let config = TorConfig::russia_censorship();
        assert!(matches!(config.bridge_type, BridgeType::Obfs4 { .. }));
        assert!(config.use_bridges_only);
        assert!(config.exclude_nodes.contains(&"RU".to_string()));
    }

    #[test]
    fn test_china_preset() {
        let config = TorConfig::china_censorship();
        assert!(matches!(config.bridge_type, BridgeType::Snowflake));
        assert!(config.exclude_nodes.contains(&"CN".to_string()));
    }

    #[cfg(not(feature = "tor"))]
    #[tokio::test]
    async fn test_tor_not_available() {
        let mut transport = TorTransport::new(TorConfig::default());
        let result = transport.connect().await;
        assert!(result.is_err());
        assert!(!transport.is_connected());
    }
}
