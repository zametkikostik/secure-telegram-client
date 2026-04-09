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

pub mod crypto;
pub mod auth;
pub mod p2p;
pub mod transport;
pub mod chat;
pub mod storage;
pub mod ai;
pub mod ads;
pub mod monetization;
pub mod offline;
pub mod queue;
pub mod cdn;
pub mod antispam;
pub mod mtproto;
#[cfg(test)]
pub mod e2e;

#[cfg(feature = "tauri-commands")]
pub mod commands;

// Web3 module — optional, requires `web3` feature
#[cfg(feature = "web3")]
pub mod web3;

// Re-export top-level types for convenience
pub use crypto::{
    HybridKeypair,
    PublicBundle,
    HybridCiphertext,
    CryptoError,
    encrypt,
    decrypt,
    sign,
    verify_signature,
};

pub use auth::{
    Keychain,
    KeychainError,
    SecurePassword,
};

pub use p2p::{
    P2PNode,
    P2PError,
    P2PMessage,
    MessageType,
    P2PEvent,
};

pub use transport::{
    CloudflareTransport,
    CloudflareMessage,
    DeliveryStatus,
    TransportError,
    TransportRouter,
    Route,
    RouteStats,
    RouterError,
    EncryptedRouteLog,
};

pub use chat::{
    ChatManager,
    ChatMessage,
    ChatError,
    ChatStatus,
    ChatManagerStats,
    PrivateChat,
    GroupChat,
    GroupMember,
    MemberRole,
    MemberStatus,
    GroupSettings,
    Channel,
    ChannelSettings,
    MessageContentType,
    DeliveryState,
    ChatTransport,
    IntegrationError,
    UserKeysRegistry,
};

pub use storage::{
    LocalStorage,
    StorageError,
    Contact,
    AppSettings,
    SettingEntry,
    ChatHistoryEntry,
    CachedMessage,
    StorageStats,
};

pub use ai::{
    AiClient,
    AiConfig,
    AiError,
    AiResult,
    AiProvider,
    AiProviderRegistry,
    ProviderConfig,
    ModelHint,
    ModelInfo,
    ProviderStatus,
};

// Ads module re-exports
pub use ads::{
    Ad,
    AdType,
    AdCategory,
    AdPreferences,
    AdImpression,
    AdStats,
    AdError,
    AdResult,
    engine::AdEngine,
    fetch::AdStorage,
    fetch::AdBundleFetcher,
    fetch::EncryptedAdBundle,
    fetch::BundleFetchRequest,
    fetch::BundleFetchResponse,
    fetch::fetch_and_store_ads,
};

// Ads commands (optional, requires `tauri-commands` feature)
#[cfg(feature = "tauri-commands")]
pub use ads::{
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

// Monetization module re-exports
pub use monetization::{
    Subscription,
    SubscriptionTier,
    PaymentMethod,
    Tip,
    TipStatus,
    CreditAccount,
    CreditTransaction,
    CreditSource,
    PremiumFeature,
    RevenueStats,
    MonetizationManager,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
