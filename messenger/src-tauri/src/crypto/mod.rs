// Crypto re-exports for Tauri app
// SECURITY: требует аудита перед production
// TODO: pentest перед release

// Реэкспорты из внешнего crypto crate (path = "crypto")
pub use crypto::{
    KeyPair,
    PublicBundle,
    HybridEncryptor,
    EncryptedMessage,
    PasswordHasher,
    Signer,
    CryptoError,
};
