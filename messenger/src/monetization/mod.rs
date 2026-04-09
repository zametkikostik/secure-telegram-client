//! Monetization Module — subscriptions, tips, credits, premium features
//!
//! Revenue streams:
//! 1. **Subscriptions** — monthly premium tier (ETH/stablecoins via Web3)
//! 2. **Tips** — peer-to-peer tipping with crypto or credits
//! 3. **Ad Credits** — users earn credits by viewing ads, spend on premium features
//! 4. **One-time purchases** — custom themes, stickers, bot access
//!
//! Premium features:
//! - Priority message delivery
//! - Extended chat history
//! - Custom themes
//! - Bot marketplace access
//! - No banner ads (reward ads still available)
//! - ENS name display
//! - Profile customization

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Subscription Plans
// ============================================================================

/// Subscription tier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionTier {
    /// Free — basic features + banner ads
    Free,
    /// Premium — ad-free + extra features
    Premium,
    /// Pro — power user features
    Pro,
}

impl SubscriptionTier {
    pub fn name(&self) -> &'static str {
        match self {
            SubscriptionTier::Free => "Free",
            SubscriptionTier::Premium => "Premium",
            SubscriptionTier::Pro => "Pro",
        }
    }

    pub fn monthly_price_usd(&self) -> u32 {
        match self {
            SubscriptionTier::Free => 0,
            SubscriptionTier::Premium => 5,
            SubscriptionTier::Pro => 15,
        }
    }

    pub fn monthly_price_credits(&self) -> u32 {
        // 1 credit = $0.01, so $5 = 500 credits
        match self {
            SubscriptionTier::Free => 0,
            SubscriptionTier::Premium => 500,
            SubscriptionTier::Pro => 1500,
        }
    }
}

/// Subscription record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub user_address: String,
    pub tier: SubscriptionTier,
    /// Start date
    pub start_date: chrono::DateTime<chrono::Utc>,
    /// Expiration date
    pub end_date: chrono::DateTime<chrono::Utc>,
    /// Auto-renew enabled
    pub auto_renew: bool,
    /// Payment method
    pub payment_method: PaymentMethod,
    /// Transaction hash of payment
    pub tx_hash: Option<String>,
    /// Whether subscription is active
    pub is_active: bool,
    /// Cancellation date (if cancelled)
    pub cancelled_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Subscription {
    /// Check if subscription is currently active
    pub fn is_valid(&self) -> bool {
        self.is_active && chrono::Utc::now() < self.end_date && self.cancelled_at.is_none()
    }

    /// Days remaining
    pub fn days_remaining(&self) -> i64 {
        let now = chrono::Utc::now();
        (self.end_date - now).num_days().max(0)
    }
}

// ============================================================================
// Payment Methods
// ============================================================================

/// How a payment was made
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentMethod {
    /// MetaMask / wallet (ETH or stablecoin)
    Crypto,
    /// Earned credits (watch-to-earn)
    Credits,
    /// External payment (Stripe, etc. — future)
    External,
}

// ============================================================================
// Tips
// ============================================================================

/// Tip record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tip {
    pub id: String,
    pub from_address: String,
    pub to_address: String,
    /// Amount in wei (native token)
    pub amount_wei: String,
    /// Amount in human-readable format
    pub amount_display: String,
    /// Token (ETH, USDC, etc.)
    pub token_symbol: String,
    /// Optional message
    pub message: Option<String>,
    /// Transaction hash
    pub tx_hash: Option<String>,
    /// Status
    pub status: TipStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TipStatus {
    Pending,
    Sent,
    Received,
    Failed(String),
}

impl Tip {
    /// Create a new tip
    pub fn new(
        from: String,
        to: String,
        amount_wei: String,
        amount_display: String,
        token_symbol: String,
        message: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from_address: from,
            to_address: to,
            amount_wei,
            amount_display,
            token_symbol,
            message,
            tx_hash: None,
            status: TipStatus::Pending,
            created_at: chrono::Utc::now(),
        }
    }
}

// ============================================================================
// Credit System
// ============================================================================

/// Credit balance and history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditAccount {
    pub address: String,
    pub balance: u32,
    pub lifetime_earned: u32,
    pub lifetime_spent: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl CreditAccount {
    pub fn new(address: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            address,
            balance: 0,
            lifetime_earned: 0,
            lifetime_spent: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// Earn credits
    pub fn earn(&mut self, amount: u32) {
        self.balance += amount;
        self.lifetime_earned += amount;
        self.updated_at = chrono::Utc::now();
    }

    /// Spend credits
    pub fn spend(&mut self, amount: u32) -> bool {
        if self.balance >= amount {
            self.balance -= amount;
            self.lifetime_spent += amount;
            self.updated_at = chrono::Utc::now();
            true
        } else {
            false
        }
    }
}

/// Credit transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditTransaction {
    pub id: String,
    pub address: String,
    pub amount: i32, // positive = earned, negative = spent
    pub source: CreditSource,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// How credits were earned/spent
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreditSource {
    /// Watched a reward ad
    AdView,
    /// Received a tip
    TipReceived,
    /// Purchased credits
    Purchased,
    /// Spent on subscription
    Subscription,
    /// Spent on premium feature
    PremiumFeature,
    /// Sent a tip
    TipSent,
    /// Referral bonus
    Referral,
    /// Welcome bonus for new users
    WelcomeBonus,
}

// ============================================================================
// Premium Features
// ============================================================================

/// Premium feature that can be purchased
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PremiumFeature {
    /// Remove banner ads for 30 days
    AdFreeMonth,
    /// Custom chat theme
    CustomTheme,
    /// Premium sticker pack
    PremiumStickers,
    /// Extended chat history (1 year)
    ExtendedHistory,
    /// Priority message delivery
    PriorityDelivery,
    /// Profile customization
    ProfileCustomization,
    /// Bot marketplace access
    BotMarketplace,
    /// ENS name display in profile
    EnsDisplay,
}

impl PremiumFeature {
    pub fn name(&self) -> &'static str {
        match self {
            PremiumFeature::AdFreeMonth => "Ad-Free Month",
            PremiumFeature::CustomTheme => "Custom Theme",
            PremiumFeature::PremiumStickers => "Premium Stickers",
            PremiumFeature::ExtendedHistory => "Extended History",
            PremiumFeature::PriorityDelivery => "Priority Delivery",
            PremiumFeature::ProfileCustomization => "Profile Customization",
            PremiumFeature::BotMarketplace => "Bot Marketplace",
            PremiumFeature::EnsDisplay => "ENS Display",
        }
    }

    pub fn cost_credits(&self) -> u32 {
        match self {
            PremiumFeature::AdFreeMonth => 300,
            PremiumFeature::CustomTheme => 100,
            PremiumFeature::PremiumStickers => 50,
            PremiumFeature::ExtendedHistory => 200,
            PremiumFeature::PriorityDelivery => 150,
            PremiumFeature::ProfileCustomization => 75,
            PremiumFeature::BotMarketplace => 100,
            PremiumFeature::EnsDisplay => 50,
        }
    }
}

// ============================================================================
// Monetization Manager
// ============================================================================

/// Main monetization manager
pub struct MonetizationManager {
    subscriptions: std::sync::Mutex<Vec<Subscription>>,
    tips: std::sync::Mutex<Vec<Tip>>,
    credit_accounts: std::sync::Mutex<std::collections::HashMap<String, CreditAccount>>,
    credit_transactions: std::sync::Mutex<Vec<CreditTransaction>>,
    purchased_features: std::sync::Mutex<
        std::collections::HashMap<String, Vec<(PremiumFeature, chrono::DateTime<chrono::Utc>)>>,
    >,
}

impl MonetizationManager {
    pub fn new() -> Self {
        Self {
            subscriptions: std::sync::Mutex::new(Vec::new()),
            tips: std::sync::Mutex::new(Vec::new()),
            credit_accounts: std::sync::Mutex::new(std::collections::HashMap::new()),
            credit_transactions: std::sync::Mutex::new(Vec::new()),
            purchased_features: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    // ========================================================================
    // Subscriptions
    // ========================================================================

    /// Create a new subscription
    pub fn create_subscription(
        &self,
        user_address: String,
        tier: SubscriptionTier,
        payment_method: PaymentMethod,
        tx_hash: Option<String>,
    ) -> Subscription {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now();
        let duration = chrono::Duration::days(30);

        let subscription = Subscription {
            id: id.clone(),
            user_address,
            tier,
            start_date: now,
            end_date: now + duration,
            auto_renew: false,
            payment_method,
            tx_hash,
            is_active: true,
            cancelled_at: None,
        };

        self.subscriptions
            .lock()
            .unwrap()
            .push(subscription.clone());
        subscription
    }

    /// Get active subscription for a user
    pub fn get_subscription(&self, address: &str) -> Option<Subscription> {
        self.subscriptions
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.user_address == address && s.is_valid())
            .max_by_key(|s| s.end_date)
            .cloned()
    }

    /// Cancel a subscription
    pub fn cancel_subscription(&self, subscription_id: &str) -> bool {
        let mut subs = self.subscriptions.lock().unwrap();
        if let Some(sub) = subs.iter_mut().find(|s| s.id == subscription_id) {
            sub.cancelled_at = Some(chrono::Utc::now());
            sub.auto_renew = false;
            true
        } else {
            false
        }
    }

    /// Check if user has premium features
    pub fn has_premium(&self, address: &str) -> bool {
        self.get_subscription(address)
            .map(|s| s.tier != SubscriptionTier::Free)
            .unwrap_or(false)
    }

    // ========================================================================
    // Tips
    // ========================================================================

    /// Create a tip
    pub fn create_tip(&self, tip: Tip) {
        self.tips.lock().unwrap().push(tip);
    }

    /// Get tips sent by a user
    pub fn get_tips_sent(&self, address: &str) -> Vec<Tip> {
        self.tips
            .lock()
            .unwrap()
            .iter()
            .filter(|t| t.from_address == address)
            .cloned()
            .collect()
    }

    /// Get tips received by a user
    pub fn get_tips_received(&self, address: &str) -> Vec<Tip> {
        self.tips
            .lock()
            .unwrap()
            .iter()
            .filter(|t| t.to_address == address)
            .cloned()
            .collect()
    }

    // ========================================================================
    // Credits
    // ========================================================================

    /// Get or create credit account
    pub fn get_credit_account(&self, address: &str) -> CreditAccount {
        let mut accounts = self.credit_accounts.lock().unwrap();
        accounts
            .entry(address.to_string())
            .or_insert_with(|| CreditAccount::new(address.to_string()))
            .clone()
    }

    /// Earn credits
    pub fn earn_credits(
        &self,
        address: &str,
        amount: u32,
        source: CreditSource,
        description: &str,
    ) {
        let mut accounts = self.credit_accounts.lock().unwrap();
        let account = accounts
            .entry(address.to_string())
            .or_insert_with(|| CreditAccount::new(address.to_string()));
        account.earn(amount);

        // Record transaction
        let tx = CreditTransaction {
            id: Uuid::new_v4().to_string(),
            address: address.to_string(),
            amount: amount as i32,
            source,
            description: description.to_string(),
            created_at: chrono::Utc::now(),
        };
        self.credit_transactions.lock().unwrap().push(tx);
    }

    /// Spend credits
    pub fn spend_credits(
        &self,
        address: &str,
        amount: u32,
        source: CreditSource,
        description: &str,
    ) -> bool {
        let mut accounts = self.credit_accounts.lock().unwrap();
        if let Some(account) = accounts.get_mut(address) {
            if account.spend(amount) {
                let tx = CreditTransaction {
                    id: Uuid::new_v4().to_string(),
                    address: address.to_string(),
                    amount: -(amount as i32),
                    source,
                    description: description.to_string(),
                    created_at: chrono::Utc::now(),
                };
                self.credit_transactions.lock().unwrap().push(tx);
                return true;
            }
        }
        false
    }

    /// Get credit balance
    pub fn get_credits(&self, address: &str) -> u32 {
        self.get_credit_account(address).balance
    }

    /// Get credit transaction history
    pub fn get_credit_history(&self, address: &str) -> Vec<CreditTransaction> {
        self.credit_transactions
            .lock()
            .unwrap()
            .iter()
            .filter(|tx| tx.address == address)
            .cloned()
            .collect()
    }

    // ========================================================================
    // Premium Features
    // ========================================================================

    /// Purchase a premium feature with credits
    pub fn purchase_feature(&self, address: &str, feature: PremiumFeature) -> bool {
        let cost = feature.cost_credits();

        if self.spend_credits(address, cost, CreditSource::PremiumFeature, feature.name()) {
            let mut features = self.purchased_features.lock().unwrap();
            features
                .entry(address.to_string())
                .or_default()
                .push((feature, chrono::Utc::now()));
            true
        } else {
            false
        }
    }

    /// Check if user has purchased a feature
    pub fn has_feature(&self, address: &str, feature: &PremiumFeature) -> bool {
        let features = self.purchased_features.lock().unwrap();
        features
            .get(address)
            .map(|f| f.iter().any(|(feat, _)| feat == feature))
            .unwrap_or(false)
    }

    /// Get all purchased features for a user
    pub fn get_features(&self, address: &str) -> Vec<PremiumFeature> {
        let features = self.purchased_features.lock().unwrap();
        features
            .get(address)
            .map(|f| f.iter().map(|(feat, _)| feat.clone()).collect())
            .unwrap_or_default()
    }

    // ========================================================================
    // Stats
    // ========================================================================

    /// Get total revenue stats
    pub fn get_revenue_stats(&self) -> RevenueStats {
        let subs = self.subscriptions.lock().unwrap();
        let tips = self.tips.lock().unwrap();

        let premium_count = subs.iter().filter(|s| s.is_valid()).count();
        let total_tip_volume = tips.iter().count();

        RevenueStats {
            active_subscriptions: premium_count,
            total_tips: total_tip_volume,
        }
    }
}

/// Revenue statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueStats {
    pub active_subscriptions: usize,
    pub total_tips: usize,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_manager() -> MonetizationManager {
        MonetizationManager::new()
    }

    #[test]
    fn test_subscription_tiers() {
        assert_eq!(SubscriptionTier::Free.monthly_price_usd(), 0);
        assert_eq!(SubscriptionTier::Premium.monthly_price_usd(), 5);
        assert_eq!(SubscriptionTier::Pro.monthly_price_usd(), 15);

        assert_eq!(SubscriptionTier::Premium.monthly_price_credits(), 500);
    }

    #[test]
    fn test_create_subscription() {
        let manager = make_manager();
        let sub = manager.create_subscription(
            "0x1234".to_string(),
            SubscriptionTier::Premium,
            PaymentMethod::Crypto,
            Some("0xtxhash".to_string()),
        );

        assert_eq!(sub.tier, SubscriptionTier::Premium);
        assert!(sub.is_active);
        assert!(sub.is_valid());
    }

    #[test]
    fn test_subscription_expiration() {
        let manager = make_manager();
        let mut sub = manager.create_subscription(
            "0x1234".to_string(),
            SubscriptionTier::Premium,
            PaymentMethod::Crypto,
            None,
        );

        // Set end date to the past
        sub.end_date = chrono::Utc::now() - chrono::Duration::days(1);
        manager.subscriptions.lock().unwrap().push(sub.clone());

        assert!(!sub.is_valid());
    }

    #[test]
    fn test_cancel_subscription() {
        let manager = make_manager();
        let sub = manager.create_subscription(
            "0x1234".to_string(),
            SubscriptionTier::Premium,
            PaymentMethod::Credits,
            None,
        );

        let cancelled = manager.cancel_subscription(&sub.id);
        assert!(cancelled);

        let retrieved = manager.get_subscription("0x1234");
        // Should return None because cancelled subs are not valid
        assert!(retrieved.is_none() || !retrieved.unwrap().is_valid());
    }

    #[test]
    fn test_has_premium() {
        let manager = make_manager();

        assert!(!manager.has_premium("0x1234"));

        manager.create_subscription(
            "0x1234".to_string(),
            SubscriptionTier::Premium,
            PaymentMethod::Crypto,
            None,
        );

        assert!(manager.has_premium("0x1234"));
    }

    #[test]
    fn test_credit_account() {
        let manager = make_manager();

        let account = manager.get_credit_account("0x1234");
        assert_eq!(account.balance, 0);

        manager.earn_credits("0x1234", 100, CreditSource::AdView, "Watched ad");
        assert_eq!(manager.get_credits("0x1234"), 100);

        let success = manager.spend_credits("0x1234", 50, CreditSource::PremiumFeature, "Theme");
        assert!(success);
        assert_eq!(manager.get_credits("0x1234"), 50);

        let fail = manager.spend_credits("0x1234", 100, CreditSource::PremiumFeature, "Theme");
        assert!(!fail);
    }

    #[test]
    fn test_credit_history() {
        let manager = make_manager();

        manager.earn_credits("0x1234", 100, CreditSource::AdView, "Watched ad");
        manager.spend_credits("0x1234", 50, CreditSource::PremiumFeature, "Theme");

        let history = manager.get_credit_history("0x1234");
        assert_eq!(history.len(), 2);

        // First should be earn (positive)
        assert_eq!(history[0].amount, 100);
        // Second should be spend (negative)
        assert_eq!(history[1].amount, -50);
    }

    #[test]
    fn test_tips() {
        let manager = make_manager();

        let tip = Tip::new(
            "0xAlice".to_string(),
            "0xBob".to_string(),
            "1000000000000000000".to_string(),
            "1.0".to_string(),
            "ETH".to_string(),
            Some("Great content!".to_string()),
        );
        manager.create_tip(tip);

        let sent = manager.get_tips_sent("0xAlice");
        assert_eq!(sent.len(), 1);

        let received = manager.get_tips_received("0xBob");
        assert_eq!(received.len(), 1);
    }

    #[test]
    fn test_purchase_feature() {
        let manager = make_manager();

        // Earn credits first
        manager.earn_credits("0x1234", 200, CreditSource::AdView, "Watched ads");

        // Purchase custom theme (100 credits)
        let success = manager.purchase_feature("0x1234", PremiumFeature::CustomTheme);
        assert!(success);

        // Should have the feature
        assert!(manager.has_feature("0x1234", &PremiumFeature::CustomTheme));

        // Balance should be reduced
        assert_eq!(manager.get_credits("0x1234"), 100);
    }

    #[test]
    fn test_premium_feature_costs() {
        assert_eq!(PremiumFeature::CustomTheme.cost_credits(), 100);
        assert_eq!(PremiumFeature::AdFreeMonth.cost_credits(), 300);
        assert_eq!(PremiumFeature::PremiumStickers.cost_credits(), 50);
        assert_eq!(PremiumFeature::EnsDisplay.cost_credits(), 50);
    }

    #[test]
    fn test_revenue_stats() {
        let manager = make_manager();

        manager.create_subscription(
            "0x1234".to_string(),
            SubscriptionTier::Premium,
            PaymentMethod::Crypto,
            None,
        );

        let stats = manager.get_revenue_stats();
        assert_eq!(stats.active_subscriptions, 1);
        assert_eq!(stats.total_tips, 0);
    }

    #[test]
    fn test_tip_creation() {
        let tip = Tip::new(
            "0xAlice".to_string(),
            "0xBob".to_string(),
            "500000000000000000".to_string(),
            "0.5".to_string(),
            "ETH".to_string(),
            None,
        );

        assert_eq!(tip.from_address, "0xAlice");
        assert_eq!(tip.to_address, "0xBob");
        assert_eq!(tip.amount_display, "0.5");
        assert!(tip.message.is_none());
        assert_eq!(tip.status, TipStatus::Pending);
    }
}
