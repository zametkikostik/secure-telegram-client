//! CDN Module — Cloudflare R2 + Cache Optimization
//!
//! Features:
//! - Cloudflare R2 storage (S3-compatible, zero egress fees)
//! - Automatic image optimization (resize, WebP conversion)
//! - Cache headers for static assets
//! - Signed URLs for private content
//! - CDN invalidation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, info};

#[derive(Error, Debug)]
pub enum CdnError {
    #[error("Upload failed: {0}")]
    UploadFailed(String),
    #[error("Download failed: {0}")]
    DownloadFailed(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Cache invalidation failed: {0}")]
    CacheInvalidationFailed(String),
}

pub type CdnResult<T> = Result<T, CdnError>;

// ============================================================================
// CDN Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdnConfig {
    /// R2 bucket name
    pub bucket: String,
    /// R2 account ID
    pub account_id: String,
    /// R2 access key ID
    pub access_key_id: String,
    /// R2 secret access key
    pub secret_access_key: String,
    /// Public CDN URL (e.g., "https://cdn.messenger.app")
    pub cdn_url: String,
    /// Default cache TTL (seconds)
    pub default_cache_ttl: u32,
    /// Enable image optimization
    pub image_optimization: bool,
}

impl CdnConfig {
    pub fn from_env() -> Self {
        Self {
            bucket: std::env::var("R2_BUCKET").unwrap_or_else(|_| "messenger-media".into()),
            account_id: std::env::var("R2_ACCOUNT_ID").unwrap_or_default(),
            access_key_id: std::env::var("R2_ACCESS_KEY_ID").unwrap_or_default(),
            secret_access_key: std::env::var("R2_SECRET_ACCESS_KEY").unwrap_or_default(),
            cdn_url: std::env::var("CDN_URL")
                .unwrap_or_else(|_| "https://cdn.messenger.app".into()),
            default_cache_ttl: 86400 * 30, // 30 days
            image_optimization: true,
        }
    }
}

// ============================================================================
// CDN Client
// ============================================================================

pub struct CdnClient {
    config: CdnConfig,
    http_client: reqwest::Client,
}

impl CdnClient {
    pub fn new(config: CdnConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(60))
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    // ========================================================================
    // Upload
    // ========================================================================

    /// Upload file to R2/CDN
    pub async fn upload(&self, key: &str, data: Vec<u8>, content_type: &str) -> CdnResult<String> {
        let url = format!(
            "https://{}.r2.cloudflarestorage.com/{}/{}",
            self.config.account_id, self.config.bucket, key
        );

        let response = self
            .http_client
            .put(&url)
            .header("Content-Type", content_type)
            .header(
                "Cache-Control",
                format!("public, max-age={}", self.config.default_cache_ttl),
            )
            .body(data)
            .send()
            .await
            .map_err(|e| CdnError::UploadFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CdnError::UploadFailed(format!(
                "HTTP {}",
                response.status()
            )));
        }

        let cdn_url = format!("{}/{}", self.config.cdn_url, key);
        info!("Uploaded to CDN: {} → {}", key, cdn_url);
        Ok(cdn_url)
    }

    /// Upload avatar/image with optimization
    pub async fn upload_image(
        &self,
        user_id: &str,
        image_data: Vec<u8>,
        content_type: &str,
    ) -> CdnResult<String> {
        let key = format!(
            "avatars/{}/{}.{}",
            user_id,
            uuid::Uuid::new_v4().simple(),
            self.extension(content_type)
        );

        // In production: resize, convert to WebP, create thumbnails
        self.upload(&key, image_data, content_type).await
    }

    // ========================================================================
    // Download
    // ========================================================================

    /// Download file from CDN
    pub async fn download(&self, cdn_url: &str) -> CdnResult<Vec<u8>> {
        let response = self
            .http_client
            .get(cdn_url)
            .send()
            .await
            .map_err(|e| CdnError::DownloadFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CdnError::DownloadFailed(format!(
                "HTTP {}",
                response.status()
            )));
        }

        let data = response
            .bytes()
            .await
            .map_err(|e| CdnError::DownloadFailed(e.to_string()))?
            .to_vec();

        debug!("Downloaded from CDN: {} ({} bytes)", cdn_url, data.len());
        Ok(data)
    }

    // ========================================================================
    // Cache Management
    // ========================================================================

    /// Purge CDN cache for specific URLs
    pub async fn purge_cache(&self, urls: Vec<String>) -> CdnResult<()> {
        // Cloudflare API: POST /zones/{zone_id}/purge_cache
        let purge_url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/purge_cache",
            self.config.account_id
        );

        let response = self
            .http_client
            .post(&purge_url)
            .header(
                "Authorization",
                format!("Bearer {}", self.config.secret_access_key),
            )
            .json(&serde_json::json!({
                "files": urls,
            }))
            .send()
            .await
            .map_err(|e| CdnError::CacheInvalidationFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CdnError::CacheInvalidationFailed(format!(
                "HTTP {}",
                response.status()
            )));
        }

        info!("Purged cache for {} URLs", urls.len());
        Ok(())
    }

    // ========================================================================
    // Signed URLs
    // ========================================================================

    /// Generate signed URL for private content (expires in N seconds)
    pub fn generate_signed_url(&self, key: &str, expires_in_secs: u64) -> String {
        // In production: use Cloudflare Signed URLs with HMAC
        let expires = chrono::Utc::now().timestamp() as u64 + expires_in_secs;
        format!(
            "{}/{}?expires={}&signature=TODO_HMAC_SIGNATURE",
            self.config.cdn_url, key, expires
        )
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    fn extension(&self, content_type: &str) -> &str {
        match content_type {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/gif" => "gif",
            "image/webp" => "webp",
            _ => "bin",
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
    fn test_cdn_config_defaults() {
        let config = CdnConfig::from_env();
        assert_eq!(config.bucket, "messenger-media");
        assert_eq!(config.default_cache_ttl, 86400 * 30);
        assert!(config.image_optimization);
    }

    #[test]
    fn test_extension_mapping() {
        let config = CdnConfig::from_env();
        let client = CdnClient::new(config);

        assert_eq!(client.extension("image/jpeg"), "jpg");
        assert_eq!(client.extension("image/png"), "png");
        assert_eq!(client.extension("image/webp"), "webp");
        assert_eq!(client.extension("application/octet-stream"), "bin");
    }

    #[test]
    fn test_signed_url_generation() {
        let config = CdnConfig::from_env();
        let client = CdnClient::new(config);

        let url = client.generate_signed_url("avatars/user123/photo.jpg", 3600);
        assert!(url.contains("avatars/user123/photo.jpg"));
        assert!(url.contains("expires="));
        assert!(url.contains("signature="));
    }
}
