//! Privacy-First Advertising Module
//!
//! Principles:
//! - NO tracking pixels, beacons, or analytics
//! - NO user data sent to ad servers
//! - All ad selection done ON-DEVICE based on user-chosen categories
//! - Users EARN credits for viewing ads (watch-to-earn)
//! - Advertiser payments go through Web3 (no fiat tracking)
//!
//! Architecture:
//! - Ad bundles are fetched encrypted from Cloudflare Worker
//! - Decrypted locally and stored in SQLite
//! - Ad engine selects ads based on user preferences (local)
//! - Impressions are tracked locally, reported anonymously via zk-proof
//!
//! Ad types:
//! - Banner (small, non-intrusive)
//! - Native (matches chat UI style)
//! - Interstitial (between chats, user-accepted)
//! - Reward (watch to earn credits)

pub mod engine;
pub mod fetch;

#[cfg(feature = "tauri-commands")]
pub mod commands;

// Re-export fetch module types
pub use fetch::{
    EncryptedAdBundle,
    BundleFetchRequest,
    BundleFetchResponse,
    AdStorage,
    AdBundleFetcher,
    fetch_and_store_ads,
};

// Re-export commands when feature is enabled
#[cfg(feature = "tauri-commands")]
pub use commands::{
    AdState,
    register_ad_commands,
    FetchAdsRequest,
    FetchAdsResponse,
    SelectAdRequest,
    SelectAdResponse,
    RecordImpressionRequest,
    RecordImpressionResponse,
    RecordClickRequest,
    RecordClickResponse,
    AdSettings,
};

use serde::{Deserialize, Serialize};

// ============================================================================
// Ad Types
// ============================================================================

/// Ad display type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdType {
    /// Small banner at the bottom of the screen
    Banner,
    /// Native ad that matches the chat UI style
    Native,
    /// Full-screen interstitial (shown between chats, opt-in)
    Interstitial,
    /// Reward ad — user watches to earn credits
    Reward,
    /// Sponsored message in a channel
    Sponsored,
}

impl AdType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AdType::Banner => "banner",
            AdType::Native => "native",
            AdType::Interstitial => "interstitial",
            AdType::Reward => "reward",
            AdType::Sponsored => "sponsored",
        }
    }
}

/// Ad category (user selects which categories they want to see)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AdCategory {
    Crypto,
    Privacy,
    Security,
    OpenSource,
    Tech,
    Gaming,
    Finance,
    Education,
    Health,
    Entertainment,
    Shopping,
    Social,
    None, // General, non-categorized
}

impl AdCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            AdCategory::Crypto => "crypto",
            AdCategory::Privacy => "privacy",
            AdCategory::Security => "security",
            AdCategory::OpenSource => "open_source",
            AdCategory::Tech => "tech",
            AdCategory::Gaming => "gaming",
            AdCategory::Finance => "finance",
            AdCategory::Education => "education",
            AdCategory::Health => "health",
            AdCategory::Entertainment => "entertainment",
            AdCategory::Shopping => "shopping",
            AdCategory::Social => "social",
            AdCategory::None => "general",
        }
    }

    pub fn all() -> &'static [AdCategory] {
        &[
            AdCategory::Crypto,
            AdCategory::Privacy,
            AdCategory::Security,
            AdCategory::OpenSource,
            AdCategory::Tech,
            AdCategory::Gaming,
            AdCategory::Finance,
            AdCategory::Education,
            AdCategory::Health,
            AdCategory::Entertainment,
            AdCategory::Shopping,
            AdCategory::Social,
            AdCategory::None,
        ]
    }
}

/// Ad content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ad {
    /// Unique ad ID (from advertiser)
    pub id: String,
    /// Advertiser name
    pub advertiser: String,
    /// Ad type
    pub ad_type: AdType,
    /// Category
    pub category: AdCategory,
    /// Title / headline
    pub title: String,
    /// Body text
    pub body: String,
    /// Image URL (local path after download)
    pub image_url: Option<String>,
    /// Click-through URL
    pub url: Option<String>,
    /// Call-to-action button text
    pub cta: Option<String>,
    /// Credit reward for viewing (reward ads only)
    pub credit_reward: u32,
    /// Impressions cap (how many times this ad should be shown)
    pub impression_cap: u32,
    /// Start date
    pub start_date: chrono::DateTime<chrono::Utc>,
    /// End date
    pub end_date: chrono::DateTime<chrono::Utc>,
    /// Priority (higher = more likely to be shown)
    pub priority: u8,
    /// Whether the ad has been viewed
    pub viewed: bool,
    /// Click count
    pub click_count: u32,
    /// Local impressions count
    pub impression_count: u32,
}

impl Ad {
    /// Check if ad is currently valid (within date range)
    pub fn is_active(&self) -> bool {
        let now = chrono::Utc::now();
        now >= self.start_date && now <= self.end_date
    }

    /// Check if ad has reached its impression cap
    pub fn is_capped(&self) -> bool {
        self.impression_count >= self.impression_cap
    }

    /// Check if ad can be shown
    pub fn can_show(&self) -> bool {
        self.is_active() && !self.is_capped()
    }

    /// Get display priority (reward ads first, then by priority)
    pub fn display_priority(&self) -> u32 {
        let base = self.priority as u32;
        match self.ad_type {
            AdType::Reward => base + 1000, // Always show reward ads first
            AdType::Banner => base,
            AdType::Native => base + 100,
            AdType::Interstitial => base + 200,
            AdType::Sponsored => base + 50,
        }
    }
}

/// User's ad preferences (what categories they want to see)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdPreferences {
    /// Categories the user is interested in
    pub preferred_categories: Vec<AdCategory>,
    /// Categories the user wants to BLOCK
    pub blocked_categories: Vec<AdCategory>,
    /// Maximum ads per hour
    pub max_ads_per_hour: u32,
    /// Enable reward ads (watch-to-earn)
    pub enable_reward_ads: bool,
    /// Enable banner ads
    pub enable_banner_ads: bool,
    /// Enable native ads
    pub enable_native_ads: bool,
    /// Enable interstitial ads (opt-in)
    pub enable_interstitial_ads: bool,
    /// Dark mode ad style
    pub dark_mode: bool,
}

impl Default for AdPreferences {
    fn default() -> Self {
        Self {
            preferred_categories: vec![
                AdCategory::Crypto,
                AdCategory::Privacy,
                AdCategory::Security,
                AdCategory::Tech,
                AdCategory::OpenSource,
            ],
            blocked_categories: vec![
                AdCategory::Shopping,
                AdCategory::Social,
            ],
            max_ads_per_hour: 10,
            enable_reward_ads: true,
            enable_banner_ads: true,
            enable_native_ads: true,
            enable_interstitial_ads: false,
            dark_mode: false,
        }
    }
}

/// Ad impression record (stored locally, reported anonymously)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdImpression {
    pub ad_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub duration_secs: u32,
    pub was_clicked: bool,
    /// Hashed impression ID for anonymous reporting (zk-proof input)
    pub impression_hash: String,
}

/// Ad click event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdClick {
    pub ad_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub url: String,
}

/// Ad stats summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdStats {
    /// Total ads viewed
    pub total_views: u32,
    /// Total credits earned from ads
    pub credits_earned: u32,
    /// Total ad clicks
    pub total_clicks: u32,
    /// Ads viewed today
    pub views_today: u32,
    /// Average view duration (seconds)
    pub avg_view_duration_secs: f64,
    /// Top category viewed
    pub top_category: Option<AdCategory>,
}

// ============================================================================
// Errors
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum AdError {
    #[error("No ads available")]
    NoAdsAvailable,

    #[error("Ad not found: {0}")]
    AdNotFound(String),

    #[error("Ad is no longer active")]
    AdExpired,

    #[error("Ad impression cap reached")]
    ImpressionCapReached,

    #[error("Rate limited: max {max} ads per hour")]
    RateLimited { max: u32 },

    #[error("Category blocked by user")]
    CategoryBlocked,

    #[error("Failed to fetch ad bundle: {0}")]
    FetchFailed(String),

    #[error("Failed to decrypt ad bundle: {0}")]
    DecryptionFailed(String),

    #[error("Storage error: {0}")]
    Storage(String),
}

pub type AdResult<T> = Result<T, AdError>;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_ad() -> Ad {
        Ad {
            id: "test-ad-1".to_string(),
            advertiser: "Test Company".to_string(),
            ad_type: AdType::Banner,
            category: AdCategory::Tech,
            title: "Test Ad".to_string(),
            body: "This is a test advertisement".to_string(),
            image_url: None,
            url: Some("https://example.com".to_string()),
            cta: Some("Learn More".to_string()),
            credit_reward: 0,
            impression_cap: 100,
            start_date: chrono::Utc::now() - chrono::Duration::days(1),
            end_date: chrono::Utc::now() + chrono::Duration::days(30),
            priority: 5,
            viewed: false,
            click_count: 0,
            impression_count: 0,
        }
    }

    #[test]
    fn test_ad_is_active() {
        let ad = make_test_ad();
        assert!(ad.is_active());
    }

    #[test]
    fn test_ad_expired() {
        let mut ad = make_test_ad();
        ad.end_date = chrono::Utc::now() - chrono::Duration::days(1);
        assert!(!ad.is_active());
    }

    #[test]
    fn test_ad_not_started() {
        let mut ad = make_test_ad();
        ad.start_date = chrono::Utc::now() + chrono::Duration::days(1);
        assert!(!ad.is_active());
    }

    #[test]
    fn test_ad_impression_cap() {
        let mut ad = make_test_ad();
        assert!(!ad.is_capped());

        ad.impression_count = 100;
        assert!(ad.is_capped());
    }

    #[test]
    fn test_ad_can_show() {
        let ad = make_test_ad();
        assert!(ad.can_show());

        let mut capped_ad = make_test_ad();
        capped_ad.impression_count = 100;
        assert!(!capped_ad.can_show());
    }

    #[test]
    fn test_ad_display_priority() {
        let reward_ad = Ad {
            ad_type: AdType::Reward,
            priority: 1,
            ..make_test_ad()
        };
        let banner_ad = Ad {
            ad_type: AdType::Banner,
            priority: 10,
            ..make_test_ad()
        };

        // Reward ads always have higher display priority
        assert!(reward_ad.display_priority() > banner_ad.display_priority());
    }

    #[test]
    fn test_ad_preferences_default() {
        let prefs = AdPreferences::default();
        assert!(prefs.enable_reward_ads);
        assert!(prefs.enable_banner_ads);
        assert!(!prefs.enable_interstitial_ads);
        assert_eq!(prefs.max_ads_per_hour, 10);
    }

    #[test]
    fn test_ad_category_as_str() {
        assert_eq!(AdCategory::Crypto.as_str(), "crypto");
        assert_eq!(AdCategory::Privacy.as_str(), "privacy");
        assert_eq!(AdCategory::None.as_str(), "general");
    }

    #[test]
    fn test_ad_impression_record() {
        let impression = AdImpression {
            ad_id: "test-ad-1".to_string(),
            timestamp: chrono::Utc::now(),
            duration_secs: 5,
            was_clicked: false,
            impression_hash: "abc123".to_string(),
        };

        assert_eq!(impression.ad_id, "test-ad-1");
        assert_eq!(impression.duration_secs, 5);
        assert!(!impression.was_clicked);
    }

    #[test]
    fn test_ad_stats_default() {
        let stats = AdStats {
            total_views: 0,
            credits_earned: 0,
            total_clicks: 0,
            views_today: 0,
            avg_view_duration_secs: 0.0,
            top_category: None,
        };

        assert_eq!(stats.total_views, 0);
        assert!(stats.top_category.is_none());
    }
}
