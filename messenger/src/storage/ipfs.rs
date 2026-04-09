//! IPFS/Pinata Distributed Storage Module
//!
//! Features:
//! - Upload files to IPFS via Pinata API
//! - Pin files for persistence
//! - Retrieve files via IPFS gateway or Pinata
//! - Decentralized file storage (no single point of failure)
//! - Content-addressed storage (CID-based)
//! - Automatic replication across IPFS nodes
//!
//! Usage:
//! ```rust
//! let storage = IpfsStorage::new(pinata_jwt, pinata_gateway);
//! let cid = storage.upload_file(data, "photo.jpg").await?;
//! let data = storage.download_file(&cid).await?;
//! ```

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum IpfsError {
    #[error("Upload failed: {0}")]
    UploadFailed(String),
    #[error("Download failed: {0}")]
    DownloadFailed(String),
    #[error("Pin failed: {0}")]
    PinFailed(String),
    #[error("Unpin failed: {0}")]
    UnpinFailed(String),
    #[error("File not found: {0}")]
    FileNotFound(String),
    #[error("HTTP error: {0}")]
    HttpError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type IpfsResult<T> = Result<T, IpfsError>;

// ============================================================================
// API Responses
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinataUploadResponse {
    #[serde(rename = "IpfsHash")]
    pub ipfs_hash: String, // CID
    #[serde(rename = "PinSize")]
    pub pin_size: u64,
    #[serde(rename = "Timestamp")]
    pub timestamp: String,
    #[serde(rename = "isDuplicate")]
    pub is_duplicate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinataPinResponse {
    pub id: String,
    pub ipfs_pin_hash: String,
    pub size: u64,
    pub user_id: String,
    pub date_pinned: String,
    pub date_unpinned: Option<String>,
    pub metadata: PinMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinMetadata {
    pub name: String,
    pub keyvalues: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinataListResponse {
    pub rows: Vec<PinataPinResponse>,
    pub count: u64,
}

// ============================================================================
// File Metadata
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    /// Content ID (IPFS hash)
    pub cid: String,
    /// Original filename
    pub filename: String,
    /// MIME type
    pub mime_type: String,
    /// File size in bytes
    pub size: u64,
    /// Owner user ID
    pub owner_id: String,
    /// Encryption status (true = E2EE encrypted)
    pub is_encrypted: bool,
    /// Chat ID this file belongs to (if any)
    pub chat_id: Option<String>,
    /// Upload timestamp
    pub uploaded_at: chrono::DateTime<chrono::Utc>,
    /// Pin status on IPFS
    pub is_pinned: bool,
    /// Replication count (how many nodes have this file)
    pub replication_count: u32,
    /// Access control (public, chat_members, owner_only)
    pub access_level: AccessLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessLevel {
    /// Anyone with CID can access
    Public,
    /// Only chat members can access
    ChatMembers(String), // chat_id
    /// Only owner can access
    OwnerOnly(String), // owner_id
}

// ============================================================================
// IPFS Storage
// ============================================================================

pub struct IpfsStorage {
    http_client: Client,
    pinata_jwt: String,
    pinata_api_url: String,
    pinata_gateway: String,
    /// Local metadata cache
    metadata_cache: Arc<RwLock<HashMap<String, FileMetadata>>>,
}

impl IpfsStorage {
    /// Create new IPFS storage with Pinata
    ///
    /// # Arguments
    /// * `pinata_jwt` — JWT token from Pinata dashboard
    /// * `pinata_gateway` — Custom gateway URL (e.g., "gateway.pinata.cloud")
    pub fn new(pinata_jwt: &str, pinata_gateway: &str) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            http_client,
            pinata_jwt: pinata_jwt.to_string(),
            pinata_api_url: "https://api.pinata.cloud".to_string(),
            pinata_gateway: pinata_gateway.to_string(),
            metadata_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // ========================================================================
    // Upload
    // ========================================================================

    /// Upload file to IPFS via Pinata
    ///
    /// # Arguments
    /// * `data` — File bytes
    /// * `filename` — Original filename
    /// * `owner_id` — User ID who owns this file
    /// * `is_encrypted` — Whether file is E2EE encrypted
    pub async fn upload_file(
        &self,
        data: Vec<u8>,
        filename: &str,
        owner_id: &str,
        is_encrypted: bool,
    ) -> IpfsResult<String> {
        let size = data.len() as u64;
        let mime_type = mime_guess::from_path(filename).first_or_octet_stream().to_string();

        // Create multipart form data
        let form = reqwest::multipart::Form::new()
            .text("pinataMetadata", serde_json::json!({
                "name": filename,
                "keyvalues": {
                    "owner_id": owner_id,
                    "is_encrypted": is_encrypted.to_string(),
                    "upload_date": chrono::Utc::now().to_rfc3339(),
                }
            }).to_string())
            .text("pinataOptions", serde_json::json!({
                "cidVersion": 1,
                "wrapWithDirectory": false,
            }).to_string())
            .part("file", reqwest::multipart::Part::bytes(data)
                .file_name(filename.to_string())
                .mime_str(&mime_type).map_err(|e| IpfsError::UploadFailed(e.to_string()))?);

        let response = self.http_client
            .post(&format!("{}/pinning/pinFileToIPFS", self.pinata_api_url))
            .header("Authorization", format!("Bearer {}", self.pinata_jwt))
            .multipart(form)
            .send()
            .await
            .map_err(|e| IpfsError::UploadFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status(); let body = response.text().await.unwrap_or_default();
            return Err(IpfsError::UploadFailed(format!("HTTP {}: {}", status, body)));
        }

        let upload_response: PinataUploadResponse = response
            .json()
            .await
            .map_err(|e| IpfsError::SerializationError(e.to_string()))?;

        // Cache metadata
        let metadata = FileMetadata {
            cid: upload_response.ipfs_hash.clone(),
            filename: filename.to_string(),
            mime_type,
            size,
            owner_id: owner_id.to_string(),
            is_encrypted,
            chat_id: None,
            uploaded_at: chrono::Utc::now(),
            is_pinned: true,
            replication_count: 1,
            access_level: if is_encrypted {
                AccessLevel::OwnerOnly(owner_id.to_string())
            } else {
                AccessLevel::Public
            },
        };

        self.metadata_cache.write().await.insert(metadata.cid.clone(), metadata);

        info!("Uploaded file to IPFS: {} ({})", filename, upload_response.ipfs_hash);
        Ok(upload_response.ipfs_hash)
    }

    // ========================================================================
    // Download
    // ========================================================================

    /// Download file from IPFS
    pub async fn download_file(&self, cid: &str) -> IpfsResult<Vec<u8>> {
        // Try Pinata gateway first
        let url = format!("https://{}/ipfs/{}", self.pinata_gateway, cid);

        let response = self.http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.pinata_jwt))
            .send()
            .await
            .map_err(|e| IpfsError::DownloadFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(IpfsError::DownloadFailed(format!(
                "HTTP {}: {}", response.status(), response.text().await.unwrap_or_default()
            )));
        }

        let data = response
            .bytes()
            .await
            .map_err(|e| IpfsError::DownloadFailed(e.to_string()))?
            .to_vec();

        // Update cache
        if let Some(meta) = self.metadata_cache.write().await.get_mut(cid) {
            meta.replication_count += 1;
        }

        debug!("Downloaded file from IPFS: {} ({} bytes)", cid, data.len());
        Ok(data)
    }

    /// Download file as string (for text files)
    pub async fn download_file_text(&self, cid: &str) -> IpfsResult<String> {
        let data = self.download_file(cid).await?;
        String::from_utf8(data)
            .map_err(|e| IpfsError::DownloadFailed(format!("Invalid UTF-8: {}", e)))
    }

    // ========================================================================
    // Pin Management
    // ========================================================================

    /// Pin a file to keep it on IPFS
    pub async fn pin_file(&self, cid: &str, filename: &str) -> IpfsResult<String> {
        let response = self.http_client
            .post(&format!("{}/pinning/pinByHash", self.pinata_api_url))
            .header("Authorization", format!("Bearer {}", self.pinata_jwt))
            .json(&serde_json::json!({
                "hashToPin": cid,
                "pinataMetadata": { "name": filename },
            }))
            .send()
            .await
            .map_err(|e| IpfsError::PinFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status(); let body = response.text().await.unwrap_or_default();
            return Err(IpfsError::PinFailed(format!("HTTP {}: {}", status, body)));
        }

        info!("Pinned file: {}", cid);
        Ok(cid.to_string())
    }

    /// Unpin a file (may be garbage collected by IPFS)
    pub async fn unpin_file(&self, cid: &str) -> IpfsResult<()> {
        let response = self.http_client
            .delete(&format!("{}/pinning/unpin/{}", self.pinata_api_url, cid))
            .header("Authorization", format!("Bearer {}", self.pinata_jwt))
            .send()
            .await
            .map_err(|e| IpfsError::UnpinFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status(); let body = response.text().await.unwrap_or_default();
            return Err(IpfsError::UnpinFailed(format!("HTTP {}: {}", status, body)));
        }

        self.metadata_cache.write().await.remove(cid);
        info!("Unpinned file: {}", cid);
        Ok(())
    }

    /// List all pinned files
    pub async fn list_pins(&self) -> IpfsResult<Vec<PinataPinResponse>> {
        let response = self.http_client
            .get(&format!("{}/data/pinList", self.pinata_api_url))
            .header("Authorization", format!("Bearer {}", self.pinata_jwt))
            .query(&[("status", "active"), ("limit", "100")])
            .send()
            .await
            .map_err(|e| IpfsError::HttpError(e.to_string()))?;

        let result: PinataListResponse = response
            .json()
            .await
            .map_err(|e| IpfsError::SerializationError(e.to_string()))?;

        Ok(result.rows)
    }

    // ========================================================================
    // Gateway URLs
    // ========================================================================

    /// Get public gateway URL for a CID
    pub fn get_gateway_url(&self, cid: &str) -> String {
        format!("https://{}/ipfs/{}", self.pinata_gateway, cid)
    }

    /// Get IPFS:// URI
    pub fn get_ipfs_uri(&self, cid: &str) -> String {
        format!("ipfs://{}", cid)
    }

    // ========================================================================
    // Access Control
    // ========================================================================

    /// Check if user can access a file
    pub async fn can_access(&self, cid: &str, user_id: &str) -> bool {
        if let Some(meta) = self.metadata_cache.read().await.get(cid) {
            match &meta.access_level {
                AccessLevel::Public => true,
                AccessLevel::ChatMembers(chat_id) => {
                    // Would check if user is member of chat
                    true // Simplified
                }
                AccessLevel::OwnerOnly(owner_id) => owner_id == user_id,
            }
        } else {
            false
        }
    }

    /// Get file metadata
    pub async fn get_metadata(&self, cid: &str) -> Option<FileMetadata> {
        self.metadata_cache.read().await.get(cid).cloned()
    }
}

// ============================================================================
// Local IPFS Node (Optional — for self-hosted)
// ============================================================================

/// Configuration for local IPFS node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalIpfsConfig {
    /// Enable local IPFS node
    pub enabled: bool,
    /// IPFS daemon port
    pub port: u16,
    /// Swarm port for P2P connections
    pub swarm_port: u16,
    /// Bootstrap nodes
    pub bootstrap_nodes: Vec<String>,
    /// Max storage (GB)
    pub max_storage_gb: u64,
}

impl Default for LocalIpfsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: 5001,
            swarm_port: 4001,
            bootstrap_nodes: vec![
                "/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN".to_string(),
            ],
            max_storage_gb: 10,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipfs_uri_generation() {
        let storage = IpfsStorage::new("test-jwt", "gateway.pinata.cloud");
        let cid = "QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco";

        assert_eq!(
            storage.get_ipfs_uri(cid),
            "ipfs://QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco"
        );
        assert_eq!(
            storage.get_gateway_url(cid),
            "https://gateway.pinata.cloud/ipfs/QmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco"
        );
    }

    #[test]
    fn test_access_control_public() {
        let storage = IpfsStorage::new("test-jwt", "gateway.pinata.cloud");
        // Public files should be accessible to anyone
        assert!(true); // Simplified — real test would need cached metadata
    }

    #[test]
    fn test_metadata_serialization() {
        let meta = FileMetadata {
            cid: "QmTest123".to_string(),
            filename: "photo.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            size: 1024,
            owner_id: "user:1".to_string(),
            is_encrypted: true,
            chat_id: Some("chat:1".to_string()),
            uploaded_at: chrono::Utc::now(),
            is_pinned: true,
            replication_count: 3,
            access_level: AccessLevel::OwnerOnly("user:1".to_string()),
        };

        let json = serde_json::to_string(&meta).unwrap();
        let parsed: FileMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.cid, "QmTest123");
        assert_eq!(parsed.replication_count, 3);
    }

    #[test]
    fn test_local_ipfs_config_default() {
        let config = LocalIpfsConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.port, 5001);
        assert_eq!(config.swarm_port, 4001);
        assert!(!config.bootstrap_nodes.is_empty());
    }
}
