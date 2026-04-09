//! Transport Layer
//!
//! Multi-transport messaging layer:
//! - **P2P**: Direct libp2p connections
//! - **Cloudflare**: HTTPS relay via Cloudflare Workers with offline queue
//! - **Telegram Bot**: Fallback delivery via Telegram Bot API (E2EE encrypted)
//! - **Router**: Intelligent route selection (P2P → Cloudflare → Telegram fallback)

pub mod cloudflare;
pub mod router;
pub mod telegram_bot;

pub use cloudflare::{
    CloudflareMessage, CloudflareTransport, DeliveryStatus, QueuedMessage, TransportError,
};

pub use router::{EncryptedRouteLog, Route, RouteStats, RouterError, TransportRouter};

pub use telegram_bot::{
    DeliveryRoute, EncryptedTelegramPayload, MessageRouter, TelegramBotError, TelegramBotResult,
    TelegramBotTransport,
};
