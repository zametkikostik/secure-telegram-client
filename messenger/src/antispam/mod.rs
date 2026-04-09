//! Anti-Spam System
//!
//! Multi-layer spam protection:
//! 1. Rate limiting (per-user, per-IP)
//! 2. Content filtering (profanity, links, media)
//! 3. Reputation scoring
//! 4. Machine learning classification (future)
//! 5. User reporting system
//! 6. Automatic bans for severe violations

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

// ============================================================================
// Spam Detection Results
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpamVerdict {
    /// Message is clean
    Clean,
    /// Message is likely spam
    Spam(f32), // confidence 0.0-1.0
    /// Message should be blocked
    Blocked,
}

// ============================================================================
// Rate Limiter
// ============================================================================

#[derive(Debug, Clone)]
pub struct RateLimit {
    pub max_requests: u32,
    pub window_secs: u64,
}

pub struct RateLimiter {
    limits: HashMap<String, VecDeque<Instant>>,
    default_limit: RateLimit,
}

impl RateLimiter {
    pub fn new(default_limit: RateLimit) -> Self {
        Self {
            limits: HashMap::new(),
            default_limit,
        }
    }

    pub fn check(&mut self, key: &str) -> bool {
        let now = Instant::now();
        let window = Duration::from_secs(self.default_limit.window_secs);

        let requests = self.limits.entry(key.to_string()).or_default();

        // Remove old requests
        while requests
            .front()
            .map_or(false, |t| now.duration_since(*t) > window)
        {
            requests.pop_front();
        }

        if requests.len() >= self.default_limit.max_requests as usize {
            return false;
        }

        requests.push_back(now);
        true
    }
}

// ============================================================================
// Content Filter
// ============================================================================

pub struct ContentFilter {
    blocked_words: HashSet<String>,
    blocked_domains: HashSet<String>,
    max_links: u32,
    max_message_length: usize,
}

impl ContentFilter {
    pub fn new() -> Self {
        Self {
            blocked_words: HashSet::new(),
            blocked_domains: HashSet::new(),
            max_links: 5,
            max_message_length: 4096,
        }
    }

    /// Add blocked word
    pub fn block_word(&mut self, word: &str) {
        self.blocked_words.insert(word.to_lowercase());
    }

    /// Add blocked domain
    pub fn block_domain(&mut self, domain: &str) {
        self.blocked_domains.insert(domain.to_lowercase());
    }

    /// Check message content
    pub fn check(&self, content: &str) -> SpamVerdict {
        // Check length
        if content.len() > self.max_message_length {
            return SpamVerdict::Blocked;
        }

        // Check for blocked words
        let lower = content.to_lowercase();
        for word in &self.blocked_words {
            if lower.contains(word) {
                return SpamVerdict::Blocked;
            }
        }

        // Check links
        let link_count = content.matches("http://").count() + content.matches("https://").count();
        if link_count > self.max_links as usize {
            return SpamVerdict::Spam(0.8);
        }

        // Check for blocked domains in links
        for domain in &self.blocked_domains {
            if lower.contains(domain) {
                return SpamVerdict::Blocked;
            }
        }

        SpamVerdict::Clean
    }
}

// ============================================================================
// Reputation System
// ============================================================================

pub struct ReputationSystem {
    scores: RwLock<HashMap<String, f32>>, // user_id -> score (0.0-100.0)
    penalties: RwLock<HashMap<String, u32>>, // user_id -> penalty count
}

impl ReputationSystem {
    pub fn new() -> Self {
        Self {
            scores: RwLock::new(HashMap::new()),
            penalties: RwLock::new(HashMap::new()),
        }
    }

    /// Get user reputation score
    pub async fn get_score(&self, user_id: &str) -> f32 {
        self.scores
            .read()
            .await
            .get(user_id)
            .copied()
            .unwrap_or(50.0) // Default: 50
    }

    /// Increase reputation (good behavior)
    pub async fn increase(&self, user_id: &str, amount: f32) {
        let mut scores = self.scores.write().await;
        let score = scores.entry(user_id.to_string()).or_insert(50.0);
        *score = (*score + amount).min(100.0);
    }

    /// Decrease reputation (spam detected)
    pub async fn decrease(&self, user_id: &str, amount: f32) {
        let mut scores = self.scores.write().await;
        let score = scores.entry(user_id.to_string()).or_insert(50.0);
        *score = (*score - amount).max(0.0);

        let mut penalties = self.penalties.write().await;
        *penalties.entry(user_id.to_string()).or_insert(0) += 1;
    }

    /// Check if user should be banned (score < threshold)
    pub async fn should_ban(&self, user_id: &str, threshold: f32) -> bool {
        self.get_score(user_id).await < threshold
    }

    /// Reset user reputation
    pub async fn reset(&self, user_id: &str) {
        self.scores.write().await.insert(user_id.to_string(), 50.0);
        self.penalties.write().await.remove(user_id);
    }
}

// ============================================================================
// Anti-Spam Engine
// ============================================================================

pub struct AntiSpamEngine {
    rate_limiter: RwLock<RateLimiter>,
    content_filter: RwLock<ContentFilter>,
    reputation: ReputationSystem,
    banned_users: RwLock<HashSet<String>>,
}

impl AntiSpamEngine {
    pub fn new() -> Self {
        Self {
            rate_limiter: RwLock::new(RateLimiter::new(RateLimit {
                max_requests: 60,
                window_secs: 60,
            })),
            content_filter: RwLock::new(ContentFilter::new()),
            reputation: ReputationSystem::new(),
            banned_users: RwLock::new(HashSet::new()),
        }
    }

    /// Check message for spam
    pub async fn check_message(&self, user_id: &str, content: &str) -> SpamVerdict {
        // Check if user is banned
        if self.banned_users.read().await.contains(user_id) {
            return SpamVerdict::Blocked;
        }

        // Check rate limit
        {
            let mut limiter = self.rate_limiter.write().await;
            if !limiter.check(user_id) {
                warn!("Rate limit exceeded for user {}", user_id);
                self.reputation.decrease(user_id, 10.0).await;
                return SpamVerdict::Spam(0.9);
            }
        }

        // Check content
        let filter = self.content_filter.read().await;
        let content_verdict = filter.check(content);

        match content_verdict {
            SpamVerdict::Blocked => {
                self.reputation.decrease(user_id, 20.0).await;
                SpamVerdict::Blocked
            }
            SpamVerdict::Spam(confidence) => {
                self.reputation.decrease(user_id, 5.0).await;
                SpamVerdict::Spam(confidence)
            }
            SpamVerdict::Clean => {
                // Good behavior — increase reputation
                self.reputation.increase(user_id, 1.0).await;
                SpamVerdict::Clean
            }
        }
    }

    /// Ban user
    pub async fn ban_user(&self, user_id: &str, reason: &str) {
        self.banned_users.write().await.insert(user_id.to_string());
        info!("Banned user: {} (reason: {})", user_id, reason);
    }

    /// Unban user
    pub async fn unban_user(&self, user_id: &str) {
        self.banned_users.write().await.remove(user_id);
        info!("Unbanned user: {}", user_id);
    }

    /// Check if user is banned
    pub async fn is_banned(&self, user_id: &str) -> bool {
        self.banned_users.read().await.contains(user_id)
    }

    /// Get stats
    pub async fn get_stats(&self, user_id: &str) -> (f32, u32, bool) {
        (
            self.reputation.get_score(user_id).await,
            self.reputation
                .penalties
                .read()
                .await
                .get(user_id)
                .copied()
                .unwrap_or(0),
            self.is_banned(user_id).await,
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(RateLimit {
            max_requests: 3,
            window_secs: 60,
        });

        assert!(limiter.check("user1"));
        assert!(limiter.check("user1"));
        assert!(limiter.check("user1"));
        assert!(!limiter.check("user1")); // 4th request — blocked
        assert!(limiter.check("user2")); // Different user — allowed
    }

    #[test]
    fn test_content_filter_clean() {
        let filter = ContentFilter::new();
        assert!(matches!(
            filter.check("Hello, how are you?"),
            SpamVerdict::Clean
        ));
    }

    #[test]
    fn test_content_filter_too_long() {
        let filter = ContentFilter {
            max_message_length: 10,
            ..ContentFilter::new()
        };
        assert!(matches!(
            filter.check("This is a very long message"),
            SpamVerdict::Blocked
        ));
    }

    #[test]
    fn test_content_filter_too_many_links() {
        let filter = ContentFilter {
            max_links: 2,
            ..ContentFilter::new()
        };
        assert!(matches!(
            filter.check("http://a.com http://b.com http://c.com http://d.com"),
            SpamVerdict::Spam(_)
        ));
    }

    #[tokio::test]
    async fn test_anti_spam_engine() {
        let engine = AntiSpamEngine::new();

        // Clean message
        let verdict = engine.check_message("user1", "Hello!").await;
        assert!(matches!(verdict, SpamVerdict::Clean));

        // Check reputation increased
        let (score, _, _) = engine.get_stats("user1").await;
        assert!(score > 50.0);
    }

    #[tokio::test]
    async fn test_ban_user() {
        let engine = AntiSpamEngine::new();

        engine.ban_user("spammer", "spam").await;
        assert!(engine.is_banned("spammer").await);

        // Banned user messages are blocked
        let verdict = engine.check_message("spammer", "Hello").await;
        assert!(matches!(verdict, SpamVerdict::Blocked));

        engine.unban_user("spammer").await;
        assert!(!engine.is_banned("spammer").await);
    }

    #[tokio::test]
    async fn test_rate_limit_spam_detection() {
        let engine = AntiSpamEngine::new();

        // Send messages rapidly
        for _ in 0..60 {
            let _ = engine.check_message("user1", "msg").await;
        }

        // Should be flagged as spam
        let verdict = engine.check_message("user1", "msg").await;
        assert!(matches!(verdict, SpamVerdict::Spam(_)));
    }
}
