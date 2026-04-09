-- Migration: 001_initial_schema.sql
-- Created: 2026-04-04
-- Description: Initial schema for secure messenger worker

-- ============================================================================
-- User Registry
-- ============================================================================

CREATE TABLE IF NOT EXISTS users (
    user_id TEXT PRIMARY KEY,
    public_key_x25519 BLOB NOT NULL,
    public_key_kyber BLOB NOT NULL,
    public_key_ed25519 BLOB NOT NULL,
    push_token TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
    last_seen INTEGER
);

-- Index for fast user lookup by push token
CREATE INDEX IF NOT EXISTS idx_users_push_token ON users(push_token);

-- ============================================================================
-- Message Queue
-- ============================================================================

CREATE TABLE IF NOT EXISTS messages (
    message_id TEXT PRIMARY KEY,
    sender_id TEXT NOT NULL,
    recipient_id TEXT NOT NULL,
    ciphertext BLOB NOT NULL,
    signature BLOB NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    delivered_at INTEGER,
    ttl INTEGER NOT NULL DEFAULT 86400,
    retry_count INTEGER NOT NULL DEFAULT 0,
    error TEXT,
    FOREIGN KEY (sender_id) REFERENCES users(user_id),
    FOREIGN KEY (recipient_id) REFERENCES users(user_id)
);

-- Index for pending messages by recipient
CREATE INDEX IF NOT EXISTS idx_messages_recipient_status
    ON messages(recipient_id, status, created_at);

-- Index for cleanup of expired messages
CREATE INDEX IF NOT EXISTS idx_messages_ttl
    ON messages(created_at, ttl);

-- ============================================================================
-- Delivery Log (encrypted metadata only)
-- ============================================================================

CREATE TABLE IF NOT EXISTS delivery_log (
    log_id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL,
    route TEXT NOT NULL,  -- 'p2p' or 'cloudflare'
    sender_hash TEXT NOT NULL,
    recipient_hash TEXT NOT NULL,
    success INTEGER NOT NULL DEFAULT 0,
    timestamp INTEGER NOT NULL DEFAULT (unixepoch()),
    FOREIGN KEY (message_id) REFERENCES messages(message_id)
);

-- Index for analytics queries
CREATE INDEX IF NOT EXISTS idx_delivery_log_route
    ON delivery_log(route, success, timestamp);

-- ============================================================================
-- Rate Limiting
-- ============================================================================

CREATE TABLE IF NOT EXISTS rate_limits (
    identifier TEXT PRIMARY KEY,
    request_count INTEGER NOT NULL DEFAULT 0,
    window_start INTEGER NOT NULL DEFAULT (unixepoch()),
    window_end INTEGER NOT NULL
);

-- Index for cleanup of expired rate limits
CREATE INDEX IF NOT EXISTS idx_rate_limits_window
    ON rate_limits(window_end);

-- ============================================================================
-- Group Chats
-- ============================================================================

CREATE TABLE IF NOT EXISTS groups (
    group_id TEXT PRIMARY KEY,
    name_hash TEXT NOT NULL,
    creator_id TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    member_count INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY (creator_id) REFERENCES users(user_id)
);

CREATE TABLE IF NOT EXISTS group_members (
    group_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    joined_at INTEGER NOT NULL DEFAULT (unixepoch()),
    role TEXT NOT NULL DEFAULT 'member',  -- 'admin' or 'member'
    PRIMARY KEY (group_id, user_id),
    FOREIGN KEY (group_id) REFERENCES groups(group_id),
    FOREIGN KEY (user_id) REFERENCES users(user_id)
);

-- Index for group member lookup
CREATE INDEX IF NOT EXISTS idx_group_members_user
    ON group_members(user_id);

-- ============================================================================
-- Cleanup Views
-- ============================================================================

-- View for expired messages
CREATE VIEW IF NOT EXISTS expired_messages AS
SELECT message_id, recipient_id, status
FROM messages
WHERE (created_at + ttl) < unixepoch();

-- View for rate limit cleanup
CREATE VIEW IF NOT EXISTS expired_rate_limits AS
SELECT identifier, request_count
FROM rate_limits
WHERE window_end < unixepoch();
