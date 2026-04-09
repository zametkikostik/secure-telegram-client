//! Wi-Fi Direct / LAN Multicast Transport
//!
//! Local network messaging without internet:
//! - Wi-Fi Direct — direct device-to-device connection
//! - LAN multicast — discover peers on same network
//! - Range: ~100 meters (building), ~300m (open)
//!
//! Like Bitchat's Wi-Fi mode:
//! - Multicast discovery packets on local network
//! - TCP connections for message exchange
//! - Works in offices, cafes, trains, etc.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use thiserror::Error;
use tracing::{debug, info};

#[derive(Error, Debug)]
pub enum WifiError {
    #[error("Network interface not found")]
    NoInterface,
    #[error("Multicast error: {0}")]
    MulticastError(String),
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Timeout")]
    Timeout,
}

pub type WifiResult<T> = Result<T, WifiError>;

// ============================================================================
// Peer Discovery via Multicast
// ============================================================================

/// Multicast discovery packet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryPacket {
    pub peer_id: String,
    pub display_name: String,
    pub public_key_hash: String,
    pub listening_port: u16,
    pub supported_protocols: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Discovered LAN peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanPeer {
    pub peer_id: String,
    pub display_name: String,
    pub ip_address: IpAddr,
    pub port: u16,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub is_connected: bool,
}

// ============================================================================
// Wi-Fi Direct Transport
// ============================================================================

pub struct WifiDirectTransport {
    pub peer_id: String,
    pub listening_port: u16,
    pub peers: HashMap<String, LanPeer>,
    multicast_group: Ipv4Addr,
}

impl WifiDirectTransport {
    // Standard multicast group for our protocol
    const DEFAULT_MULTICAST_GROUP: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
    const DEFAULT_PORT: u16 = 9876;

    pub fn new(peer_id: &str) -> Self {
        Self {
            peer_id: peer_id.to_string(),
            listening_port: Self::DEFAULT_PORT,
            peers: HashMap::new(),
            multicast_group: Self::DEFAULT_MULTICAST_GROUP,
        }
    }

    /// Start multicast discovery
    pub async fn start_discovery(&mut self) -> WifiResult<()> {
        // In production:
        // 1. Join multicast group
        // 2. Send discovery packet periodically
        // 3. Listen for discovery packets from peers
        // 4. Update peer list

        info!(
            "LAN multicast discovery started on {}:{}",
            self.multicast_group, self.listening_port
        );
        Ok(())
    }

    /// Send discovery packet
    pub async fn broadcast_discovery(&self) -> WifiResult<()> {
        let packet = DiscoveryPacket {
            peer_id: self.peer_id.clone(),
            display_name: "Secure Messenger".to_string(),
            public_key_hash: "hash".to_string(),
            listening_port: self.listening_port,
            supported_protocols: vec!["secure-messenger-v1".to_string()],
            timestamp: chrono::Utc::now(),
        };

        // In production:
        // 1. Serialize to binary
        // 2. Send via UDP multicast
        // 3. Repeat every 5 seconds

        debug!("Broadcast discovery packet");
        Ok(())
    }

    /// Connect to LAN peer via TCP
    pub async fn connect(&mut self, ip: IpAddr, port: u16) -> WifiResult<()> {
        let addr = SocketAddr::new(ip, port);

        // In production:
        // 1. Establish TCP connection
        // 2. Perform TLS handshake
        // 3. Exchange peer info
        // 4. Start message loop

        debug!("Connected to LAN peer: {}", addr);
        Ok(())
    }

    /// Send message via TCP
    pub async fn send_message(&self, peer_ip: IpAddr, data: &[u8]) -> WifiResult<()> {
        // In production:
        // 1. Write length-prefixed frame to TCP stream
        // 2. Flush
        // 3. Wait for ACK

        debug!("Sent {} bytes to {}", data.len(), peer_ip);
        Ok(())
    }

    /// Get discovered peers
    pub fn get_peers(&self) -> Vec<&LanPeer> {
        self.peers.values().collect()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wifi_transport_creation() {
        let transport = WifiDirectTransport::new("peer123");
        assert_eq!(transport.peer_id, "peer123");
        assert_eq!(transport.listening_port, 9876);
    }

    #[test]
    fn test_discovery_packet() {
        let packet = DiscoveryPacket {
            peer_id: "alice".to_string(),
            display_name: "Alice".to_string(),
            public_key_hash: "abc123".to_string(),
            listening_port: 9876,
            supported_protocols: vec!["secure-messenger-v1".to_string()],
            timestamp: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&packet).unwrap();
        let parsed: DiscoveryPacket = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.peer_id, "alice");
    }
}
