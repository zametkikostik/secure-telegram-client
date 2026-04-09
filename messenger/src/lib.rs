//! Secure Messenger Library
//!
//! Post-quantum encrypted decentralized messenger core.
//!
//! # Features
//! - **Hybrid E2EE**: X25519 + Kyber1024 + ChaCha20-Poly1305 + HMAC-SHA3
//! - **Steganography**: LSB image steganography for plausible deniability
//! - **Secure Keychain**: OS-level keyring storage
//! - **Password Security**: Argon2id hashing with secure buffers
//!
//! SECURITY: требует аудита перед production
//! TODO: pentest перед release

pub mod ads;
pub mod ai;
pub mod antispam;
pub mod auth;
pub mod cdn;
pub mod chat;
pub mod crypto;
#[cfg(test)]
pub mod e2e;
pub mod monetization;
pub mod mtproto;
pub mod offline;
pub mod p2p;
pub mod queue;
pub mod storage;
pub mod transport;

#[cfg(feature = "tauri-commands")]
pub mod commands;

// Web3 module — optional, requires `web3` feature
#[cfg(feature = "web3")]
pub mod web3;

// Re-export top-level types for convenience
pub use crypto::{
    decrypt, encrypt, sign, verify_signature, CryptoError, HybridCiphertext, HybridKeypair,
    PublicBundle,
};

pub use auth::{Keychain, KeychainError, SecurePassword};

pub use p2p::{MessageType, P2PError, P2PEvent, P2PMessage, P2PNode};

pub use transport::{
    CloudflareMessage, CloudflareTransport, DeliveryStatus, EncryptedRouteLog, Route, RouteStats,
    RouterError, TransportError, TransportRouter,
};

pub use chat::{
    Channel, ChannelSettings, ChatError, ChatManager, ChatManagerStats, ChatMessage, ChatStatus,
    ChatTransport, DeliveryState, GroupChat, GroupMember, GroupSettings, IntegrationError,
    MemberRole, MemberStatus, MessageContentType, PrivateChat, UserKeysRegistry,
};

pub use storage::{
    AppSettings, CachedMessage, ChatHistoryEntry, Contact, LocalStorage, SettingEntry,
    StorageError, StorageStats,
};

pub use ai::{
    AiClient, AiConfig, AiError, AiProvider, AiProviderRegistry, AiResult, ModelHint, ModelInfo,
    ProviderConfig, ProviderStatus,
};

// Ads module re-exports
pub use ads::{
    engine::AdEngine, fetch::fetch_and_store_ads, fetch::AdBundleFetcher, fetch::AdStorage,
    fetch::BundleFetchRequest, fetch::BundleFetchResponse, fetch::EncryptedAdBundle, Ad,
    AdCategory, AdError, AdImpression, AdPreferences, AdResult, AdStats, AdType,
};

// Ads commands (optional, requires `tauri-commands` feature)
#[cfg(feature = "tauri-commands")]
pub use ads::{
    register_ad_commands, AdSettings, AdState, FetchAdsRequest, FetchAdsResponse,
    RecordClickRequest, RecordClickResponse, RecordImpressionRequest, RecordImpressionResponse,
    SelectAdRequest, SelectAdResponse,
};

// Monetization module re-exports
pub use monetization::{
    CreditAccount, CreditSource, CreditTransaction, MonetizationManager, PaymentMethod,
    PremiumFeature, RevenueStats, Subscription, SubscriptionTier, Tip, TipStatus,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
