//! Bluetooth Low Energy (BLE) Transport — Stub Architecture
//!
//! Architecture ready for btleplug integration.
//! To enable: add `btleplug = "0.11"` to Cargo.toml and implement the `#[cfg(feature = "ble")]` blocks.
//!
//! When enabled, provides:
//! - BLE advertising for peer discovery (~10m range)
//! - GATT characteristic read/write for message exchange
//! - Works WITHOUT internet

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info};

#[derive(Error, Debug)]
pub enum BleError {
    #[error("BLE adapter not available")]
    AdapterNotAvailable,
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Timeout")]
    Timeout,
    #[error("Protocol error: {0}")]
    ProtocolError(String),
}

pub type BleResult<T> = Result<T, BleError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlePeer {
    pub peer_id: String,
    pub display_name: String,
    pub ble_address: String,
    pub rssi: Option<i16>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub is_connected: bool,
}

pub type MessageCallback = Arc<dyn Fn(Vec<u8>, String) + Send + Sync>;
pub type PeerCallback = Arc<dyn Fn(BlePeer) + Send + Sync>;

pub struct BleTransport {
    peer_id: String,
    peers: Arc<RwLock<HashMap<String, BlePeer>>>,
    message_callback: Arc<Mutex<Option<MessageCallback>>>,
    peer_callback: Arc<Mutex<Option<PeerCallback>>>,
    available: bool,
}

impl BleTransport {
    pub fn new(peer_id: &str) -> Self {
        // In production with btleplug:
        // Check if BLE adapter is available
        Self {
            peer_id: peer_id.to_string(),
            peers: Arc::new(RwLock::new(HashMap::new())),
            message_callback: Arc::new(Mutex::new(None)),
            peer_callback: Arc::new(Mutex::new(None)),
            available: false, // Set to true when btleplug is enabled
        }
    }

    pub fn is_available(&self) -> bool {
        self.available
    }

    pub async fn on_message<F>(&self, cb: F)
    where
        F: Fn(Vec<u8>, String) + Send + Sync + 'static,
    {
        *self.message_callback.lock().await = Some(Arc::new(cb));
    }

    pub async fn on_peer_discovered<F>(&self, cb: F)
    where
        F: Fn(BlePeer) + Send + Sync + 'static,
    {
        *self.peer_callback.lock().await = Some(Arc::new(cb));
    }

    pub async fn start_scanning(&self) -> BleResult<()> {
        if !self.available {
            return Err(BleError::AdapterNotAvailable);
        }
        // With btleplug:
        // let manager = Manager::new().await?;
        // let adapters = manager.adapters().await?;
        // adapter.start_scan(ScanFilter { services: vec![SERVICE_UUID] }).await?;
        info!("BLE scanning started (stub)");
        Ok(())
    }

    pub async fn start_advertising(&self) -> BleResult<()> {
        if !self.available {
            return Err(BleError::AdapterNotAvailable);
        }
        info!("BLE advertising started (stub)");
        Ok(())
    }

    pub async fn connect(&self, _ble_address: &str) -> BleResult<()> {
        if !self.available {
            return Err(BleError::AdapterNotAvailable);
        }
        Ok(())
    }

    pub async fn send_message(&self, _ble_address: &str, _data: &[u8]) -> BleResult<()> {
        if !self.available {
            return Err(BleError::AdapterNotAvailable);
        }
        // With btleplug:
        // peripheral.write_characteristic(msg_char, data, WriteType::WithoutResponse).await?;
        Ok(())
    }

    pub async fn get_peers(&self) -> Vec<BlePeer> {
        self.peers.read().await.values().cloned().collect()
    }

    pub async fn stop(&self) {
        info!("BLE stopped");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ble_transport_creation() {
        let transport = BleTransport::new("peer123");
        assert_eq!(transport.peer_id, "peer123");
        assert!(!transport.is_available()); // No btleplug
    }

    #[tokio::test]
    async fn test_ble_not_available() {
        let transport = BleTransport::new("test");
        assert!(!transport.is_available());
        assert!(transport.start_scanning().await.is_err());
        assert!(transport.get_peers().await.is_empty());
    }
}
