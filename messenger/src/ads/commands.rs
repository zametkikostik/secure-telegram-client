//! Tauri Commands for Ad Module
//!
//! Provides secure ad operations through Tauri commands:
//! - Fetch encrypted ad bundles
//! - Select ads based on local preferences
//! - Record impressions and clicks
//! - Manage ad preferences and credits
//!
//! Privacy: All ad selection happens ON-DEVICE.
//! NO user data is sent to ad servers.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::AdEngine;
use crate::ads::*;

// ============================================================================
// Ad State Management
// ============================================================================

/// Global ad state (shared across Tauri commands)
#[derive(Clone)]
pub struct AdState {
    /// Ad engine (in-memory)
    pub engine: Arc<AdEngine>,
    /// Ad storage (SQLite)
    pub storage: Arc<Mutex<Option<AdStorage>>>,
    /// Bundle fetcher
    pub fetcher: Arc<Mutex<Option<AdBundleFetcher>>>,
    /// Last sync timestamp
    pub last_sync: Arc<Mutex<Option<i64>>>,
    /// Client private key for decryption (32 bytes)
    pub client_private_key: Arc<Mutex<Option<[u8; 32]>>>,
}

impl AdState {
    /// Create new ad state
    pub fn new(preferences: AdPreferences) -> Self {
        Self {
            engine: Arc::new(AdEngine::new(preferences)),
            storage: Arc::new(Mutex::new(None)),
            fetcher: Arc::new(Mutex::new(None)),
            last_sync: Arc::new(Mutex::new(None)),
            client_private_key: Arc::new(Mutex::new(None)),
        }
    }

    /// Initialize storage and fetcher
    pub async fn init(
        &self,
        db_path: &str,
        worker_url: &str,
        client_public_key: &str,
    ) -> Result<(), AdError> {
        let storage = AdStorage::new(db_path)
            .await
            .map_err(|e| AdError::Storage(e.to_string()))?;

        // Load ads from storage into engine
        let ads = storage
            .load_ads()
            .await
            .map_err(|e| AdError::Storage(e.to_string()))?;

        self.engine.load_ads(ads);

        // Create fetcher
        let fetcher = AdBundleFetcher::new(worker_url, client_public_key);

        *self.storage.lock().await = Some(storage);
        *self.fetcher.lock().await = Some(fetcher);

        // Generate a test client key (in production, use actual key from keychain)
        let mut client_key = [0u8; 32];
        // Simple derivation from public key for testing
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        hasher.update(client_public_key.as_bytes());
        hasher.update(b"client-private-key-derivation-v1");
        let result = hasher.finalize();
        client_key.copy_from_slice(&result);

        *self.client_private_key.lock().await = Some(client_key);

        info!("Ad state initialized: db={}, worker_url={}", db_path, worker_url);
        Ok(())
    }
}

// ============================================================================
// Command Request/Response Types
// ============================================================================

/// Request to fetch ads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchAdsRequest {
    /// Maximum number of ads to fetch
    pub max_ads: Option<u32>,
}

/// Response from fetching ads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchAdsResponse {
    /// Number of ads fetched
    pub fetched_count: u32,
    /// Total ads available
    pub total_count: u32,
}

/// Request to select an ad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectAdRequest {
    /// Ad type to select
    pub ad_type: AdType,
}

/// Response from selecting an ad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectAdResponse {
    /// Selected ad (if available)
    pub ad: Option<Ad>,
}

/// Request to record impression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordImpressionRequest {
    /// Ad ID
    pub ad_id: String,
    /// View duration in seconds
    pub duration_secs: u32,
}

/// Response from recording impression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordImpressionResponse {
    /// Success status
    pub success: bool,
    /// Credits earned (if reward ad)
    pub credits_earned: u32,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Request to record click
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordClickRequest {
    /// Ad ID
    pub ad_id: String,
}

/// Response from recording click
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordClickResponse {
    /// Success status
    pub success: bool,
    /// Click URL (to open)
    pub url: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Ad settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdSettings {
    /// User preferences
    pub preferences: AdPreferences,
    /// Current credit balance
    pub credits: u32,
    /// Ad stats
    pub stats: AdStats,
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Fetch encrypted ad bundle from Cloudflare Worker
#[tauri::command]
pub async fn cmd_fetch_ads(
    state: tauri::State<'_, AdState>,
    request: FetchAdsRequest,
) -> Result<FetchAdsResponse, String> {
    debug!("Fetching ads: max_ads={:?}", request.max_ads);

    let storage_guard = state.storage.lock().await;
    let fetcher_guard = state.fetcher.lock().await;

    let storage = storage_guard.as_ref().ok_or("Ad storage not initialized")?;
    let fetcher = fetcher_guard.as_ref().ok_or("Ad fetcher not initialized")?;

    let preferences = state.engine.get_preferences();
    let categories = preferences.preferred_categories.clone();

    let last_sync = *state.last_sync.lock().await;
    let client_key = state.client_private_key.lock().await;
    let key_bytes = client_key.ok_or("Client private key not initialized")?;

    // Fetch and store ads
    match fetch_and_store_ads(fetcher, storage, &categories, &key_bytes, last_sync).await {
        Ok(count) => {
            // Update last sync time
            *state.last_sync.lock().await = Some(chrono::Utc::now().timestamp());

            // Reload ads into engine
            let ads = storage
                .load_ads()
                .await
                .map_err(|e| e.to_string())?;

            state.engine.load_ads(ads);

            let total_count = state.engine.list_ads().len() as u32;

            info!("Fetched {} ads, total: {}", count, total_count);

            Ok(FetchAdsResponse {
                fetched_count: count as u32,
                total_count,
            })
        }
        Err(e) => {
            warn!("Failed to fetch ads: {}", e);
            Err(e.to_string())
        }
    }
}

/// Select an ad based on type and preferences
#[tauri::command]
pub async fn cmd_select_ad(
    state: tauri::State<'_, AdState>,
    request: SelectAdRequest,
) -> Result<SelectAdResponse, String> {
    debug!("Selecting ad: type={:?}", request.ad_type);

    match state.engine.select_ad(request.ad_type) {
        Ok(ad) => Ok(SelectAdResponse {
            ad: Some(ad),
        }),
        Err(AdError::NoAdsAvailable) => Ok(SelectAdResponse { ad: None }),
        Err(e) => Err(e.to_string()),
    }
}

/// Record an ad impression
#[tauri::command]
pub async fn cmd_record_impression(
    state: tauri::State<'_, AdState>,
    request: RecordImpressionRequest,
) -> Result<RecordImpressionResponse, String> {
    debug!(
        "Recording impression: ad_id={}, duration={}s",
        request.ad_id, request.duration_secs
    );

    let credits_before = state.engine.get_credits();

    match state
        .engine
        .record_impression(&request.ad_id, request.duration_secs)
    {
        Ok(()) => {
            let credits_after = state.engine.get_credits();
            let credits_earned = credits_after.saturating_sub(credits_before);

            // Store impression in SQLite
            if let Some(storage) = state.storage.lock().await.as_ref() {
                let impression = AdImpression {
                    ad_id: request.ad_id.clone(),
                    timestamp: chrono::Utc::now(),
                    duration_secs: request.duration_secs,
                    was_clicked: false,
                    impression_hash: generate_impression_hash(&request.ad_id),
                };

                if let Err(e) = storage
                    .save_impression(
                        &impression.ad_id,
                        &impression.timestamp,
                        impression.duration_secs,
                        impression.was_clicked,
                        &impression.impression_hash,
                    )
                    .await
                {
                    warn!("Failed to save impression to storage: {}", e);
                }
            }

            Ok(RecordImpressionResponse {
                success: true,
                credits_earned,
                error: None,
            })
        }
        Err(e) => {
            // Don't treat rate limit or caps as errors
            let error_msg = e.to_string();
            let is_soft_error = matches!(
                e,
                AdError::RateLimited { .. } | AdError::ImpressionCapReached
            );

            if is_soft_error {
                Ok(RecordImpressionResponse {
                    success: false,
                    credits_earned: 0,
                    error: Some(error_msg),
                })
            } else {
                Err(error_msg)
            }
        }
    }
}

/// Record an ad click
#[tauri::command]
pub async fn cmd_record_click(
    state: tauri::State<'_, AdState>,
    request: RecordClickRequest,
) -> Result<RecordClickResponse, String> {
    debug!("Recording click: ad_id={}", request.ad_id);

    match state.engine.record_click(&request.ad_id) {
        Ok(url) => {
            // Store click in SQLite
            if let Some(storage) = state.storage.lock().await.as_ref() {
                if let Err(e) = storage
                    .save_click(&request.ad_id, &chrono::Utc::now(), &url)
                    .await
                {
                    warn!("Failed to save click to storage: {}", e);
                }
            }

            Ok(RecordClickResponse {
                success: true,
                url: Some(url),
                error: None,
            })
        }
        Err(e) => {
            let error_msg = e.to_string();
            Err(error_msg)
        }
    }
}

/// Get ad settings and stats
#[tauri::command]
pub async fn cmd_get_ad_settings(
    state: tauri::State<'_, AdState>,
) -> Result<AdSettings, String> {
    Ok(AdSettings {
        preferences: state.engine.get_preferences(),
        credits: state.engine.get_credits(),
        stats: state.engine.get_stats(),
    })
}

/// Update ad preferences
#[tauri::command]
pub async fn cmd_update_ad_preferences(
    state: tauri::State<'_, AdState>,
    preferences: AdPreferences,
) -> Result<(), String> {
    debug!("Updating ad preferences");

    state.engine.update_preferences(preferences);

    // Persist to storage (optional)
    if let Some(storage) = state.storage.lock().await.as_ref() {
        // TODO: Save preferences to storage if needed
    }

    Ok(())
}

/// Get current credit balance
#[tauri::command]
pub async fn cmd_get_ad_credits(state: tauri::State<'_, AdState>) -> Result<u32, String> {
    Ok(state.engine.get_credits())
}

/// Spend credits
#[tauri::command]
pub async fn cmd_spend_ad_credits(
    state: tauri::State<'_, AdState>,
    amount: u32,
) -> Result<bool, String> {
    debug!("Spending {} ad credits", amount);
    Ok(state.engine.spend_credits(amount))
}

/// Get all available ads (for settings page)
#[tauri::command]
pub async fn cmd_list_ads(state: tauri::State<'_, AdState>) -> Result<Vec<Ad>, String> {
    Ok(state.engine.list_ads())
}

/// Report impressions to server (anonymous batch)
#[tauri::command]
pub async fn cmd_report_impressions(state: tauri::State<'_, AdState>) -> Result<(), String> {
    debug!("Reporting impressions anonymously");

    let fetcher_guard = state.fetcher.lock().await;
    let fetcher = fetcher_guard
        .as_ref()
        .ok_or("Ad fetcher not initialized")?;

    let pending = state.engine.get_pending_impressions();

    if pending.is_empty() {
        debug!("No pending impressions to report");
        return Ok(());
    }

    match fetcher.report_impressions(&pending).await {
        Ok(()) => {
            state.engine.clear_reported_impressions();

            // Also mark in storage
            if let Some(storage) = state.storage.lock().await.as_ref() {
                if let Err(e) = storage.mark_impressions_reported().await {
                    warn!("Failed to mark impressions reported in storage: {}", e);
                }
            }

            info!("Reported {} impressions", pending.len());
            Ok(())
        }
        Err(e) => {
            warn!("Failed to report impressions: {}", e);
            Err(e.to_string())
        }
    }
}

/// Clean up expired ads
#[tauri::command]
pub async fn cmd_cleanup_ads(state: tauri::State<'_, AdState>) -> Result<u32, String> {
    debug!("Cleaning up expired ads");

    let removed = state.engine.cleanup();

    if let Some(storage) = state.storage.lock().await.as_ref() {
        match storage.cleanup_expired().await {
            Ok(count) => info!("Cleaned up {} expired ads from storage", count),
            Err(e) => warn!("Failed to cleanup expired ads: {}", e),
        }
    }

    Ok(removed as u32)
}

// ============================================================================
// Helpers
// ============================================================================

/// Generate anonymous impression hash
fn generate_impression_hash(ad_id: &str) -> String {
    use sha3::{Digest, Sha3_256};

    let mut hasher = Sha3_256::new();
    hasher.update(ad_id.as_bytes());
    let ts = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    hasher.update(ts.to_be_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

// ============================================================================
// Command Registration
// ============================================================================

/// Register all ad commands with Tauri builder
pub fn register_ad_commands(
    builder: tauri::Builder<tauri::Wry>,
    ad_state: AdState,
) -> tauri::Builder<tauri::Wry> {
    builder
        .invoke_handler(tauri::generate_handler![
            cmd_fetch_ads,
            cmd_select_ad,
            cmd_record_impression,
            cmd_record_click,
            cmd_get_ad_settings,
            cmd_update_ad_preferences,
            cmd_get_ad_credits,
            cmd_spend_ad_credits,
            cmd_list_ads,
            cmd_report_impressions,
            cmd_cleanup_ads,
        ])
}
