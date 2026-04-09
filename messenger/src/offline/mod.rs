//! Offline & Mesh Networking Module
//!
//! Inspired by Briar and Bitchat — enables communication WITHOUT internet:
//! - Bluetooth Low Energy (BLE) — proximity messaging (~10m)
//! - Wi-Fi Direct / LAN multicast — local network messaging (~100m)
//! - Tor bridges + hidden services — censorship circumvention
//! - Anti-DPI obfuscation — Deep Packet Inspection evasion
//! - Store-and-forward mesh — multi-hop message relay
//!
//! Architecture:
//! ┌─────────────────────────────────────────────────────────┐
//! │                    MESSAGE QUEUE                         │
//! │  (offline messages stored until delivery possible)       │
//! └────┬──────────┬───────────┬──────────┬──────────────────┘
//!      │          │           │          │
//! ┌────▼────┐┌────▼─────┐┌───▼────┐┌────▼────────┐
//! │  BLE    ││ Wi-Fi    ││  Tor   ││  Cloudflare │
//! │ (10m)   ││ LAN(100m)││(global)││  (fallback) │
//! └─────────┘└──────────┘└────────┘└─────────────┘
//!
//! Mesh Relay:
//!   Device A ←BLE→ Device B ←Wi-Fi→ Device C ←Tor→ Server
//!   Messages hop through available connections

pub mod bluetooth;
pub mod wifi_direct;
pub mod wifi_lan;
pub mod tor_transport;
pub mod obfuscation;
pub mod mesh;
pub mod transport_router;

// Re-exports
pub use bluetooth::{BleTransport, BlePeer, BleError};
pub use wifi_direct::{WifiDirectTransport, LanPeer, WifiError};
pub use wifi_lan::{WifiLanTransport, LanPeer as WifiLanPeer, WifiLanError, WifiLanResult};
pub use tor_transport::{TorTransport, TorConfig, TorError};
pub use obfuscation::{DpiObfuscator, ObfuscationProfile, ProtocolCamouflage};
pub use mesh::{MeshNetwork, MeshPeer, MeshMessage, MeshError, DeliveryStrategy};
pub use transport_router::{TransportRouter, TransportType, TransportStats, RoutingMessage};

// ============================================================================
// Common Types
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;

/// Peer discovery method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    /// Bluetooth Low Energy advertising
    Bluetooth,
    /// Wi-Fi LAN multicast/broadcast
    WifiMulticast,
    /// Tor rendezvous point
    Tor,
    /// Internet (Cloudflare/P2P)
    Internet,
}

/// Peer connection info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub display_name: String,
    pub public_key: Vec<u8>,
    pub discovery_method: DiscoveryMethod,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub signal_strength: Option<i8>, // RSSI for BLE/Wi-Fi
    pub is_online: bool,
}

/// Offline message (stored until delivery)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineMessage {
    pub id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub encrypted_payload: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub ttl: u32,              // Time-to-live (hops)
    pub hop_count: u32,        // Current hop count
    pub delivery_method: Option<DiscoveryMethod>,
    pub is_delivered: bool,
}
