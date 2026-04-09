-- Payment & Monetization Schema

-- User subscriptions
CREATE TABLE IF NOT EXISTS subscriptions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    user_id TEXT NOT NULL REFERENCES users(id),
    tier TEXT NOT NULL DEFAULT 'free' CHECK (tier IN ('free', 'premium', 'pro')),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'cancelled', 'expired', 'past_due')),
    current_period_start TEXT NOT NULL DEFAULT (datetime('now')),
    current_period_end TEXT NOT NULL,
    cancel_at_period_end INTEGER DEFAULT 0,
    payment_method TEXT DEFAULT 'stripe' CHECK (payment_method IN ('stripe', 'crypto', 'credits')),
    stripe_subscription_id TEXT,
    stripe_customer_id TEXT,
    crypto_tx_hash TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    UNIQUE(user_id)
);

-- Credit transactions ledger
CREATE TABLE IF NOT EXISTS credit_transactions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    user_id TEXT NOT NULL REFERENCES users(id),
    amount INTEGER NOT NULL,  -- Positive = earned, Negative = spent
    balance_after INTEGER NOT NULL,
    source TEXT NOT NULL CHECK (source IN ('purchase', 'ad_reward', 'referral', 'tip', 'subscription', 'feature_purchase')),
    description TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

-- Tips between users
CREATE TABLE IF NOT EXISTS tips (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    sender_id TEXT NOT NULL REFERENCES users(id),
    recipient_id TEXT NOT NULL REFERENCES users(id),
    amount REAL NOT NULL,
    currency TEXT NOT NULL DEFAULT 'credits',
    message TEXT,
    status TEXT NOT NULL DEFAULT 'completed' CHECK (status IN ('pending', 'completed', 'failed')),
    tx_hash TEXT,  -- For crypto tips
    created_at TEXT DEFAULT (datetime('now'))
);

-- Premium feature purchases
CREATE TABLE IF NOT EXISTS premium_feature_purchases (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    user_id TEXT NOT NULL REFERENCES users(id),
    feature_id TEXT NOT NULL,
    price_paid INTEGER NOT NULL,  -- In credits
    created_at TEXT DEFAULT (datetime('now'))
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_subscriptions_user ON subscriptions(user_id);
CREATE INDEX IF NOT EXISTS idx_subscriptions_status ON subscriptions(status);
CREATE INDEX IF NOT EXISTS idx_credit_transactions_user ON credit_transactions(user_id);
CREATE INDEX IF NOT EXISTS idx_credit_transactions_created ON credit_transactions(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_tips_sender ON tips(sender_id);
CREATE INDEX IF NOT EXISTS idx_tips_recipient ON tips(recipient_id);
