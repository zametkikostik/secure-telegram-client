-- Bot Platform Schema
-- Run these migrations to enable bot functionality

-- Registered bots
CREATE TABLE IF NOT EXISTS bots (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name TEXT NOT NULL,
    username TEXT UNIQUE NOT NULL,
    description TEXT,
    avatar_url TEXT,
    token_hash TEXT UNIQUE NOT NULL,  -- SHA3-256 hash of the actual token
    owner_id TEXT NOT NULL REFERENCES users(id),
    handler_type TEXT NOT NULL DEFAULT 'internal' CHECK (handler_type IN ('internal', 'webhook', 'ai')),
    webhook_url TEXT,
    ai_prompt TEXT,
    is_active INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Bot commands (for internal bots)
CREATE TABLE IF NOT EXISTS bot_commands (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    bot_id TEXT NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    command TEXT NOT NULL,  -- e.g. "/start"
    description TEXT,
    handler_type TEXT NOT NULL DEFAULT 'internal' CHECK (handler_type IN ('internal', 'webhook', 'ai')),
    handler_url TEXT,
    response_template TEXT,  -- Simple text response for internal bots
    created_at TEXT DEFAULT (datetime('now')),
    UNIQUE(bot_id, command)
);

-- Bot webhooks (for external bot servers)
CREATE TABLE IF NOT EXISTS bot_webhooks (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    bot_id TEXT NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    events TEXT NOT NULL DEFAULT '["message"]',  -- JSON array of event types
    secret TEXT NOT NULL DEFAULT (lower(hex(randomblob(16)))),  -- HMAC secret
    active INTEGER DEFAULT 1,
    last_triggered_at TEXT,
    last_status INTEGER,
    created_at TEXT DEFAULT (datetime('now'))
);

-- Bot sessions (FSM state for dialogues)
CREATE TABLE IF NOT EXISTS bot_sessions (
    bot_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    chat_id TEXT NOT NULL,
    state TEXT NOT NULL DEFAULT 'default',
    context TEXT DEFAULT '{}',  -- JSON context data
    updated_at TEXT DEFAULT (datetime('now')),
    PRIMARY KEY (bot_id, user_id, chat_id),
    FOREIGN KEY (bot_id) REFERENCES bots(id) ON DELETE CASCADE
);

-- Bot installations (from Bot Store)
CREATE TABLE IF NOT EXISTS bot_installations (
    bot_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    installed_at TEXT DEFAULT (datetime('now')),
    PRIMARY KEY (bot_id, user_id),
    FOREIGN KEY (bot_id) REFERENCES bots(id) ON DELETE CASCADE
);

-- Bot Store listings (pre-installed/system bots)
CREATE TABLE IF NOT EXISTS bot_store_listings (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    username TEXT UNIQUE NOT NULL,
    description TEXT NOT NULL,
    avatar_url TEXT,
    category TEXT NOT NULL DEFAULT 'utility',
    rating REAL DEFAULT 0.0,
    install_count INTEGER DEFAULT 0,
    is_verified INTEGER DEFAULT 0,
    is_premium INTEGER DEFAULT 0,
    author TEXT NOT NULL,
    commands TEXT DEFAULT '[]',  -- JSON array of sample commands
    bot_id TEXT REFERENCES bots(id),  -- Links to actual bot
    created_at TEXT DEFAULT (datetime('now'))
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_bots_owner ON bots(owner_id);
CREATE INDEX IF NOT EXISTS idx_bots_username ON bots(username);
CREATE INDEX IF NOT EXISTS idx_bot_commands_bot ON bot_commands(bot_id);
CREATE INDEX IF NOT EXISTS idx_bot_webhooks_bot ON bot_webhooks(bot_id);
CREATE INDEX IF NOT EXISTS idx_bot_sessions_user ON bot_sessions(user_id);
