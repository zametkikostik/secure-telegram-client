//! Encrypted Ad Bundle Fetcher
//!
//! Fetches encrypted ad bundles from Cloudflare Worker,
//! decrypts them locally, and stores in SQLite.
//!
//! Privacy principles:
//! - NO user data sent to ad server
//! - Bundles are encrypted end-to-end ( advertiser → client )
//! - Decryption happens ON-DEVICE only
//! - Ad selection is purely local based on user-chosen categories

use super::*;
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Key, Nonce,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use sqlx::Row;
use std::time::Duration;
use tracing::{debug, info, warn};

// ============================================================================
// Encrypted Bundle Types
// ============================================================================

/// Encrypted ad bundle from Cloudflare Worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedAdBundle {
    /// Encrypted ad data (ChaCha20-Poly1305 ciphertext)
    pub ciphertext: Vec<u8>,
    /// Nonce for decryption
    pub nonce: Vec<u8>,
    /// Advertiser's public key (for verification)
    pub advertiser_key: String,
    /// Bundle timestamp (for freshness check)
    pub timestamp: i64,
    /// Bundle version
    pub version: u32,
}

/// Ad bundle fetch request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleFetchRequest {
    /// Categories the user is interested in (sent to server for filtering)
    pub categories: Vec<String>,
    /// Client's public key (for response encryption)
    pub client_public_key: String,
    /// Last bundle timestamp the client has (for delta updates)
    pub last_sync: Option<i64>,
    /// Maximum number of ads to fetch
    pub max_ads: u32,
}

/// Ad bundle fetch response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleFetchResponse {
    /// Encrypted ad bundle
    pub bundle: EncryptedAdBundle,
    /// Total ads available
    pub total_ads: u32,
    /// Server timestamp
    pub server_timestamp: i64,
}

// ============================================================================
// Ad Bundle Storage (SQLite)
// ============================================================================

/// SQLite storage for decrypted ads
pub struct AdStorage {
    pool: sqlx::SqlitePool,
}

impl AdStorage {
    /// Create new storage instance
    pub async fn new(db_path: &str) -> Result<Self, AdError> {
        let pool = sqlx::SqlitePool::connect(db_path)
            .await
            .map_err(|e| AdError::Storage(e.to_string()))?;

        let storage = Self { pool };
        storage.init_schema().await?;

        Ok(storage)
    }

    /// Initialize database schema
    async fn init_schema(&self) -> Result<(), AdError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ads (
                id TEXT PRIMARY KEY,
                advertiser TEXT NOT NULL,
                ad_type TEXT NOT NULL,
                category TEXT NOT NULL,
                title TEXT NOT NULL,
                body TEXT NOT NULL,
                image_url TEXT,
                url TEXT,
                cta TEXT,
                credit_reward INTEGER NOT NULL DEFAULT 0,
                impression_cap INTEGER NOT NULL DEFAULT 0,
                start_date TEXT NOT NULL,
                end_date TEXT NOT NULL,
                priority INTEGER NOT NULL DEFAULT 0,
                viewed INTEGER NOT NULL DEFAULT 0,
                click_count INTEGER NOT NULL DEFAULT 0,
                impression_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AdError::Storage(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ad_impressions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ad_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                duration_secs INTEGER NOT NULL,
                was_clicked INTEGER NOT NULL DEFAULT 0,
                impression_hash TEXT NOT NULL,
                reported INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (ad_id) REFERENCES ads(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AdError::Storage(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ad_clicks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ad_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                url TEXT NOT NULL,
                FOREIGN KEY (ad_id) REFERENCES ads(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AdError::Storage(e.to_string()))?;

        // Index for fast queries (without datetime in index)
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_ads_category
            ON ads (category, impression_count)
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AdError::Storage(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_impressions_ad_time
            ON ad_impressions (ad_id, timestamp)
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AdError::Storage(e.to_string()))?;

        Ok(())
    }

    /// Save decrypted ads to storage
    pub async fn save_ads(&self, ads: &[Ad]) -> Result<(), AdError> {
        for ad in ads {
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO ads
                    (id, advertiser, ad_type, category, title, body, image_url, url, cta,
                     credit_reward, impression_cap, start_date, end_date, priority,
                     viewed, click_count, impression_count)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&ad.id)
            .bind(&ad.advertiser)
            .bind(serde_json::to_string(&ad.ad_type).unwrap_or_default())
            .bind(ad.category.as_str())
            .bind(&ad.title)
            .bind(&ad.body)
            .bind(&ad.image_url)
            .bind(&ad.url)
            .bind(&ad.cta)
            .bind(ad.credit_reward as i64)
            .bind(ad.impression_cap as i64)
            .bind(&ad.start_date.to_rfc3339())
            .bind(&ad.end_date.to_rfc3339())
            .bind(ad.priority as i64)
            .bind(if ad.viewed { 1 } else { 0 })
            .bind(ad.click_count as i64)
            .bind(ad.impression_count as i64)
            .execute(&self.pool)
            .await
            .map_err(|e| AdError::Storage(e.to_string()))?;
        }

        debug!("Saved {} ads to storage", ads.len());
        Ok(())
    }

    /// Load all active ads from storage
    pub async fn load_ads(&self) -> Result<Vec<Ad>, AdError> {
        // Use a simpler approach - fetch as JSON
        let rows = sqlx::query(
            r#"
            SELECT id, advertiser, ad_type, category, title, body,
                   image_url, url, cta, credit_reward, impression_cap,
                   start_date, end_date, priority, viewed, click_count,
                   impression_count, created_at
            FROM ads
            WHERE end_date > datetime('now')
              AND impression_count < impression_cap
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AdError::Storage(e.to_string()))?;

        let mut ads = Vec::new();
        for row in rows {
            let id: String = row
                .try_get("id")
                .map_err(|e| AdError::Storage(e.to_string()))?;
            let advertiser: String = row
                .try_get("advertiser")
                .map_err(|e| AdError::Storage(e.to_string()))?;
            let ad_type_str: String = row
                .try_get("ad_type")
                .map_err(|e| AdError::Storage(e.to_string()))?;
            let category_str: String = row
                .try_get("category")
                .map_err(|e| AdError::Storage(e.to_string()))?;
            let title: String = row
                .try_get("title")
                .map_err(|e| AdError::Storage(e.to_string()))?;
            let body: String = row
                .try_get("body")
                .map_err(|e| AdError::Storage(e.to_string()))?;
            let image_url: Option<String> = row
                .try_get("image_url")
                .map_err(|e| AdError::Storage(e.to_string()))?;
            let url: Option<String> = row
                .try_get("url")
                .map_err(|e| AdError::Storage(e.to_string()))?;
            let cta: Option<String> = row
                .try_get("cta")
                .map_err(|e| AdError::Storage(e.to_string()))?;
            let credit_reward: i64 = row
                .try_get("credit_reward")
                .map_err(|e| AdError::Storage(e.to_string()))?;
            let impression_cap: i64 = row
                .try_get("impression_cap")
                .map_err(|e| AdError::Storage(e.to_string()))?;
            let start_date_str: String = row
                .try_get("start_date")
                .map_err(|e| AdError::Storage(e.to_string()))?;
            let end_date_str: String = row
                .try_get("end_date")
                .map_err(|e| AdError::Storage(e.to_string()))?;
            let priority: i64 = row
                .try_get("priority")
                .map_err(|e| AdError::Storage(e.to_string()))?;
            let viewed: i64 = row
                .try_get("viewed")
                .map_err(|e| AdError::Storage(e.to_string()))?;
            let click_count: i64 = row
                .try_get("click_count")
                .map_err(|e| AdError::Storage(e.to_string()))?;
            let impression_count: i64 = row
                .try_get("impression_count")
                .map_err(|e| AdError::Storage(e.to_string()))?;

            let ad_type = serde_json::from_str(&ad_type_str)
                .map_err(|e| AdError::Storage(format!("Invalid ad_type: {}", e)))?;

            let category = AdCategory::all()
                .iter()
                .find(|c| c.as_str() == category_str)
                .cloned()
                .unwrap_or(AdCategory::None);

            let start_date = chrono::DateTime::parse_from_rfc3339(&start_date_str)
                .map_err(|e| AdError::Storage(format!("Invalid start_date: {}", e)))?
                .with_timezone(&chrono::Utc);

            let end_date = chrono::DateTime::parse_from_rfc3339(&end_date_str)
                .map_err(|e| AdError::Storage(format!("Invalid end_date: {}", e)))?
                .with_timezone(&chrono::Utc);

            ads.push(Ad {
                id,
                advertiser,
                ad_type,
                category,
                title,
                body,
                image_url,
                url,
                cta,
                credit_reward: credit_reward as u32,
                impression_cap: impression_cap as u32,
                start_date,
                end_date,
                priority: priority as u8,
                viewed: viewed != 0,
                click_count: click_count as u32,
                impression_count: impression_count as u32,
            });
        }

        Ok(ads)
    }

    /// Save impression to storage
    pub async fn save_impression(
        &self,
        ad_id: &str,
        timestamp: &chrono::DateTime<chrono::Utc>,
        duration_secs: u32,
        was_clicked: bool,
        impression_hash: &str,
    ) -> Result<(), AdError> {
        sqlx::query(
            r#"
            INSERT INTO ad_impressions (ad_id, timestamp, duration_secs, was_clicked, impression_hash)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(ad_id)
        .bind(&timestamp.to_rfc3339())
        .bind(duration_secs as i64)
        .bind(if was_clicked { 1 } else { 0 })
        .bind(impression_hash)
        .execute(&self.pool)
        .await
        .map_err(|e| AdError::Storage(e.to_string()))?;

        Ok(())
    }

    /// Save click to storage
    pub async fn save_click(
        &self,
        ad_id: &str,
        timestamp: &chrono::DateTime<chrono::Utc>,
        url: &str,
    ) -> Result<(), AdError> {
        sqlx::query(
            r#"
            INSERT INTO ad_clicks (ad_id, timestamp, url)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(ad_id)
        .bind(&timestamp.to_rfc3339())
        .bind(url)
        .execute(&self.pool)
        .await
        .map_err(|e| AdError::Storage(e.to_string()))?;

        Ok(())
    }

    /// Get pending (unreported) impressions for anonymous reporting
    pub async fn get_pending_impressions(&self) -> Result<Vec<AdImpression>, AdError> {
        let rows = sqlx::query_as::<_, (String, String, i64, i64, String)>(
            r#"
            SELECT ad_id, timestamp, duration_secs, was_clicked, impression_hash
            FROM ad_impressions
            WHERE reported = 0
            ORDER BY timestamp ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AdError::Storage(e.to_string()))?;

        let impressions = rows
            .into_iter()
            .map(|(ad_id, timestamp_str, duration, clicked, hash)| {
                let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp_str)
                    .unwrap_or_else(|_| chrono::Utc::now().fixed_offset())
                    .with_timezone(&chrono::Utc);

                AdImpression {
                    ad_id,
                    timestamp,
                    duration_secs: duration as u32,
                    was_clicked: clicked != 0,
                    impression_hash: hash,
                }
            })
            .collect();

        Ok(impressions)
    }

    /// Mark impressions as reported
    pub async fn mark_impressions_reported(&self) -> Result<(), AdError> {
        sqlx::query(
            r#"
            UPDATE ad_impressions
            SET reported = 1
            WHERE reported = 0
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AdError::Storage(e.to_string()))?;

        Ok(())
    }

    /// Clean up expired ads
    pub async fn cleanup_expired(&self) -> Result<usize, AdError> {
        let result = sqlx::query(
            r#"
            DELETE FROM ads
            WHERE end_date <= datetime('now')
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AdError::Storage(e.to_string()))?;

        Ok(result.rows_affected() as usize)
    }

    /// Update ad impression count
    pub async fn update_ad_impression_count(&self, ad_id: &str) -> Result<(), AdError> {
        sqlx::query(
            r#"
            UPDATE ads
            SET impression_count = impression_count + 1
            WHERE id = ?
            "#,
        )
        .bind(ad_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AdError::Storage(e.to_string()))?;

        Ok(())
    }

    /// Update ad click count
    pub async fn update_ad_click_count(&self, ad_id: &str) -> Result<(), AdError> {
        sqlx::query(
            r#"
            UPDATE ads
            SET click_count = click_count + 1
            WHERE id = ?
            "#,
        )
        .bind(ad_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AdError::Storage(e.to_string()))?;

        Ok(())
    }
}

// ============================================================================
// Bundle Fetcher
// ============================================================================

/// Ad bundle fetcher from Cloudflare Worker
pub struct AdBundleFetcher {
    http_client: Client,
    worker_url: String,
    client_public_key: String,
}

impl AdBundleFetcher {
    /// Create new fetcher
    pub fn new(worker_url: &str, client_public_key: &str) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            http_client,
            worker_url: worker_url.to_string(),
            client_public_key: client_public_key.to_string(),
        }
    }

    /// Fetch encrypted ad bundle from Cloudflare Worker
    pub async fn fetch_bundle(
        &self,
        categories: &[AdCategory],
        last_sync: Option<i64>,
        max_ads: u32,
    ) -> Result<EncryptedAdBundle, AdError> {
        let request = BundleFetchRequest {
            categories: categories.iter().map(|c| c.as_str().to_string()).collect(),
            client_public_key: self.client_public_key.clone(),
            last_sync,
            max_ads,
        };

        let url = format!("{}/api/v1/ads/bundle", self.worker_url);

        debug!(
            "Fetching ad bundle from: {} (categories: {:?})",
            url, categories
        );

        let response = self
            .http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AdError::FetchFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AdError::FetchFailed(format!("HTTP {}: {}", status, body)));
        }

        let fetch_response: BundleFetchResponse = response
            .json()
            .await
            .map_err(|e| AdError::FetchFailed(format!("Failed to parse response: {}", e)))?;

        info!(
            "Fetched ad bundle: {} ads available, version {}",
            fetch_response.total_ads, fetch_response.bundle.version
        );

        Ok(fetch_response.bundle)
    }

    /// Report impressions anonymously (batch, no PII)
    pub async fn report_impressions(&self, impressions: &[AdImpression]) -> Result<(), AdError> {
        if impressions.is_empty() {
            return Ok(());
        }

        // Only send hashed impression IDs (no user data, no timestamps)
        let impression_hashes: Vec<&str> = impressions
            .iter()
            .map(|imp| imp.impression_hash.as_str())
            .collect();

        let url = format!("{}/api/v1/ads/report", self.worker_url);

        debug!(
            "Reporting {} impressions to: {}",
            impression_hashes.len(),
            url
        );

        let response = self
            .http_client
            .post(&url)
            .json(&impression_hashes)
            .send()
            .await
            .map_err(|e| {
                warn!("Failed to report impressions: {}", e);
                AdError::FetchFailed(e.to_string())
            })?;

        if response.status().is_success() {
            info!("Reported {} impressions", impression_hashes.len());
        } else {
            warn!("Failed to report impressions: HTTP {}", response.status());
        }

        Ok(())
    }
}

// ============================================================================
// Ad Decryption
// ============================================================================

/// Decrypt ad bundle using ChaCha20-Poly1305
///
/// The bundle is encrypted by the advertiser using a shared secret
/// derived from the client's public key and advertiser's private key.
///
/// # Arguments
/// * `encrypted` - Encrypted ad bundle
/// * `client_private_key` - Client's private key (32 bytes)
///
/// # Returns
/// * `Ok(Vec<Ad>)` - Decrypted ads
/// * `Err(AdError)` - Decryption failed
pub fn decrypt_ad_bundle(
    encrypted: &EncryptedAdBundle,
    client_private_key: &[u8],
) -> AdResult<Vec<Ad>> {
    if client_private_key.len() != 32 {
        return Err(AdError::DecryptionFailed(
            "Invalid client private key length (expected 32 bytes)".to_string(),
        ));
    }

    if encrypted.nonce.len() != 12 {
        return Err(AdError::DecryptionFailed(
            "Invalid nonce length (expected 12 bytes)".to_string(),
        ));
    }

    // Derive decryption key from client private key using SHA3-256
    // In production, this would be a proper KDF (e.g., HKDF)
    let mut hasher = Sha3_256::new();
    hasher.update(client_private_key);
    hasher.update(b"ad-bundle-key-derivation-v1");
    let key_bytes = hasher.finalize();

    // Create cipher and nonce
    let key = Key::from_slice(&key_bytes);
    let nonce = Nonce::from_slice(&encrypted.nonce);
    let cipher = ChaCha20Poly1305::new(key);

    // Decrypt payload
    let payload = Payload {
        msg: &encrypted.ciphertext,
        aad: b"ad-bundle", // Additional authenticated data
    };

    let plaintext = cipher.decrypt(nonce, payload).map_err(|e| {
        AdError::DecryptionFailed(format!("ChaCha20-Poly1305 decryption failed: {}", e))
    })?;

    // Parse JSON into ads
    let ads: Vec<Ad> = serde_json::from_slice(&plaintext)
        .map_err(|e| AdError::DecryptionFailed(format!("Failed to parse decrypted ads: {}", e)))?;

    info!("Decrypted ad bundle: {} ads", ads.len());
    Ok(ads)
}

/// Encrypt ad bundle for testing
#[cfg(test)]
pub fn encrypt_ad_bundle(ads: &[Ad], client_key: &[u8; 32]) -> Result<EncryptedAdBundle, String> {
    use chacha20poly1305::aead::OsRng;
    use rand::RngCore;

    // Generate random nonce
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);

    // Derive encryption key (same as decryption)
    let mut hasher = Sha3_256::new();
    hasher.update(client_key);
    hasher.update(b"ad-bundle-key-derivation-v1");
    let key_bytes = hasher.finalize();

    // Serialize ads to JSON
    let plaintext =
        serde_json::to_vec(ads).map_err(|e| format!("Failed to serialize ads: {}", e))?;

    // Create cipher
    let key = Key::from_slice(&key_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let cipher = ChaCha20Poly1305::new(key);

    // Encrypt
    let payload = Payload {
        msg: &plaintext,
        aad: b"ad-bundle",
    };

    let ciphertext = cipher
        .encrypt(nonce, payload)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    Ok(EncryptedAdBundle {
        ciphertext,
        nonce: nonce_bytes.to_vec(),
        advertiser_key: hex::encode(client_key),
        timestamp: chrono::Utc::now().timestamp(),
        version: 1,
    })
}

/// Fetch, decrypt, and store ad bundle
pub async fn fetch_and_store_ads(
    fetcher: &AdBundleFetcher,
    storage: &AdStorage,
    categories: &[AdCategory],
    client_private_key: &[u8],
    last_sync: Option<i64>,
) -> AdResult<usize> {
    // Fetch encrypted bundle
    let encrypted_bundle = fetcher.fetch_bundle(categories, last_sync, 50).await?;

    // Decrypt bundle
    let ads = decrypt_ad_bundle(&encrypted_bundle, client_private_key)?;

    // Store decrypted ads
    storage
        .save_ads(&ads)
        .await
        .map_err(|e| AdError::Storage(e.to_string()))?;

    Ok(ads.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundle_fetch_request_serialization() {
        let request = BundleFetchRequest {
            categories: vec!["tech".to_string(), "crypto".to_string()],
            client_public_key: "test-key".to_string(),
            last_sync: Some(1234567890),
            max_ads: 10,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: BundleFetchRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.categories.len(), 2);
        assert_eq!(deserialized.max_ads, 10);
    }

    #[test]
    fn test_encrypted_bundle_serialization() {
        let bundle = EncryptedAdBundle {
            ciphertext: vec![1, 2, 3, 4],
            nonce: vec![5, 6, 7, 8],
            advertiser_key: "advertiser-key".to_string(),
            timestamp: 1234567890,
            version: 1,
        };

        let json = serde_json::to_string(&bundle).unwrap();
        let deserialized: EncryptedAdBundle = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.ciphertext, vec![1, 2, 3, 4]);
        assert_eq!(deserialized.version, 1);
    }

    #[test]
    fn test_bundle_fetch_response_serialization() {
        let response = BundleFetchResponse {
            bundle: EncryptedAdBundle {
                ciphertext: vec![1, 2, 3],
                nonce: vec![4, 5, 6],
                advertiser_key: "key".to_string(),
                timestamp: 1234567890,
                version: 1,
            },
            total_ads: 5,
            server_timestamp: 1234567890,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: BundleFetchResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_ads, 5);
    }

    #[tokio::test]
    async fn test_ad_storage_creation() {
        // Use in-memory SQLite for testing
        let storage = AdStorage::new(":memory:").await.unwrap();
        assert!(storage.load_ads().await.is_ok());
    }

    #[tokio::test]
    async fn test_ad_storage_save_and_load() {
        let storage = AdStorage::new(":memory:").await.unwrap();

        let ad = Ad {
            id: "test-ad-1".to_string(),
            advertiser: "Test Company".to_string(),
            ad_type: AdType::Banner,
            category: AdCategory::Tech,
            title: "Test Ad".to_string(),
            body: "Test body".to_string(),
            image_url: None,
            url: Some("https://example.com".to_string()),
            cta: Some("Learn More".to_string()),
            credit_reward: 5,
            impression_cap: 100,
            start_date: chrono::Utc::now() - chrono::Duration::days(1),
            end_date: chrono::Utc::now() + chrono::Duration::days(30),
            priority: 5,
            viewed: false,
            click_count: 0,
            impression_count: 0,
        };

        storage.save_ads(&[ad]).await.unwrap();
        let loaded = storage.load_ads().await.unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "test-ad-1");
        assert_eq!(loaded[0].title, "Test Ad");
    }

    #[tokio::test]
    async fn test_ad_storage_cleanup_expired() {
        let storage = AdStorage::new(":memory:").await.unwrap();

        // Add expired ad
        let expired_ad = Ad {
            id: "expired-ad".to_string(),
            advertiser: "Expired".to_string(),
            ad_type: AdType::Banner,
            category: AdCategory::Tech,
            title: "Expired".to_string(),
            body: "This ad has expired".to_string(),
            image_url: None,
            url: None,
            cta: None,
            credit_reward: 0,
            impression_cap: 100,
            start_date: chrono::Utc::now() - chrono::Duration::days(60),
            end_date: chrono::Utc::now() - chrono::Duration::days(1),
            priority: 5,
            viewed: false,
            click_count: 0,
            impression_count: 0,
        };

        storage.save_ads(&[expired_ad]).await.unwrap();

        // Load should not return expired ads
        let loaded = storage.load_ads().await.unwrap();
        assert_eq!(loaded.len(), 0);

        // Cleanup should remove the expired ad
        let cleaned = storage.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 1);
    }

    #[tokio::test]
    async fn test_ad_storage_impression_tracking() {
        let storage = AdStorage::new(":memory:").await.unwrap();

        let ad = Ad {
            id: "test-ad-1".to_string(),
            advertiser: "Test".to_string(),
            ad_type: AdType::Banner,
            category: AdCategory::Tech,
            title: "Test".to_string(),
            body: "Test body".to_string(),
            image_url: None,
            url: None,
            cta: None,
            credit_reward: 0,
            impression_cap: 100,
            start_date: chrono::Utc::now() - chrono::Duration::days(1),
            end_date: chrono::Utc::now() + chrono::Duration::days(30),
            priority: 5,
            viewed: false,
            click_count: 0,
            impression_count: 0,
        };

        storage.save_ads(&[ad]).await.unwrap();

        // Record impression
        let now = chrono::Utc::now();
        storage
            .save_impression("test-ad-1", &now, 5, false, "hash123")
            .await
            .unwrap();

        // Get pending impressions
        let pending = storage.get_pending_impressions().await.unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].ad_id, "test-ad-1");
        assert_eq!(pending[0].duration_secs, 5);

        // Mark as reported
        storage.mark_impressions_reported().await.unwrap();

        // Should be empty now
        let pending = storage.get_pending_impressions().await.unwrap();
        assert_eq!(pending.len(), 0);
    }

    #[tokio::test]
    async fn test_ad_storage_click_tracking() {
        let storage = AdStorage::new(":memory:").await.unwrap();

        let ad = Ad {
            id: "test-ad-1".to_string(),
            advertiser: "Test".to_string(),
            ad_type: AdType::Banner,
            category: AdCategory::Tech,
            title: "Test".to_string(),
            body: "Test body".to_string(),
            image_url: None,
            url: Some("https://example.com".to_string()),
            cta: Some("Click".to_string()),
            credit_reward: 0,
            impression_cap: 100,
            start_date: chrono::Utc::now() - chrono::Duration::days(1),
            end_date: chrono::Utc::now() + chrono::Duration::days(30),
            priority: 5,
            viewed: false,
            click_count: 0,
            impression_count: 0,
        };

        storage.save_ads(&[ad]).await.unwrap();

        // Record click
        let now = chrono::Utc::now();
        storage
            .save_click("test-ad-1", &now, "https://example.com")
            .await
            .unwrap();

        // Update click count
        storage.update_ad_click_count("test-ad-1").await.unwrap();

        // Verify click count
        let ads = storage.load_ads().await.unwrap();
        assert_eq!(ads[0].click_count, 1);
    }

    #[test]
    fn test_encrypt_decrypt_bundle() {
        // Create test ads
        let ads = vec![
            Ad {
                id: "test-ad-1".to_string(),
                advertiser: "Test Company".to_string(),
                ad_type: AdType::Banner,
                category: AdCategory::Tech,
                title: "Test Ad 1".to_string(),
                body: "Test body 1".to_string(),
                image_url: None,
                url: Some("https://example.com/1".to_string()),
                cta: Some("Click".to_string()),
                credit_reward: 5,
                impression_cap: 100,
                start_date: chrono::Utc::now() - chrono::Duration::days(1),
                end_date: chrono::Utc::now() + chrono::Duration::days(30),
                priority: 5,
                viewed: false,
                click_count: 0,
                impression_count: 0,
            },
            Ad {
                id: "test-ad-2".to_string(),
                advertiser: "Test Company 2".to_string(),
                ad_type: AdType::Reward,
                category: AdCategory::Crypto,
                title: "Test Ad 2".to_string(),
                body: "Test body 2".to_string(),
                image_url: None,
                url: Some("https://example.com/2".to_string()),
                cta: Some("Earn".to_string()),
                credit_reward: 10,
                impression_cap: 50,
                start_date: chrono::Utc::now() - chrono::Duration::days(1),
                end_date: chrono::Utc::now() + chrono::Duration::days(30),
                priority: 10,
                viewed: false,
                click_count: 0,
                impression_count: 0,
            },
        ];

        // Generate test key (32 bytes)
        let client_key = [42u8; 32];

        // Encrypt bundle
        let encrypted = encrypt_ad_bundle(&ads, &client_key).unwrap();

        // Verify encryption
        assert!(!encrypted.ciphertext.is_empty());
        assert_eq!(encrypted.nonce.len(), 12);
        assert_eq!(encrypted.version, 1);

        // Decrypt bundle
        let decrypted = decrypt_ad_bundle(&encrypted, &client_key).unwrap();

        // Verify decryption
        assert_eq!(decrypted.len(), 2);
        assert_eq!(decrypted[0].id, "test-ad-1");
        assert_eq!(decrypted[1].id, "test-ad-2");
        assert_eq!(decrypted[0].title, "Test Ad 1");
        assert_eq!(decrypted[1].credit_reward, 10);
    }

    #[test]
    fn test_decrypt_wrong_key() {
        let ads = vec![Ad {
            id: "test-ad".to_string(),
            advertiser: "Test".to_string(),
            ad_type: AdType::Banner,
            category: AdCategory::Tech,
            title: "Test".to_string(),
            body: "Body".to_string(),
            image_url: None,
            url: None,
            cta: None,
            credit_reward: 0,
            impression_cap: 100,
            start_date: chrono::Utc::now() - chrono::Duration::days(1),
            end_date: chrono::Utc::now() + chrono::Duration::days(30),
            priority: 5,
            viewed: false,
            click_count: 0,
            impression_count: 0,
        }];

        let client_key = [42u8; 32];
        let encrypted = encrypt_ad_bundle(&ads, &client_key).unwrap();

        // Try to decrypt with wrong key
        let wrong_key = [99u8; 32];
        let result = decrypt_ad_bundle(&encrypted, &wrong_key);

        // Should fail (authenticated encryption)
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_invalid_key_length() {
        let encrypted = EncryptedAdBundle {
            ciphertext: vec![1, 2, 3],
            nonce: vec![4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
            advertiser_key: "key".to_string(),
            timestamp: 1234567890,
            version: 1,
        };

        let wrong_key = [0u8; 16]; // Wrong length
        let result = decrypt_ad_bundle(&encrypted, &wrong_key);

        assert!(result.is_err());
        match result {
            Err(AdError::DecryptionFailed(msg)) => {
                assert!(msg.contains("Invalid client private key length"));
            }
            _ => panic!("Expected DecryptionFailed error"),
        }
    }

    #[test]
    fn test_decrypt_invalid_nonce_length() {
        let encrypted = EncryptedAdBundle {
            ciphertext: vec![1, 2, 3],
            nonce: vec![1, 2, 3], // Wrong nonce length (should be 12)
            advertiser_key: "key".to_string(),
            timestamp: 1234567890,
            version: 1,
        };

        let key = [0u8; 32];
        let result = decrypt_ad_bundle(&encrypted, &key);

        assert!(result.is_err());
        match result {
            Err(AdError::DecryptionFailed(msg)) => {
                assert!(msg.contains("Invalid nonce length"));
            }
            _ => panic!("Expected DecryptionFailed error"),
        }
    }
}
