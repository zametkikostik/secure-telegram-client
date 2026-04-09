// Secure Messenger Tauri - Library entry point
// SECURITY: требует аудита перед production
// TODO: pentest перед release

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Файловые модули (НЕ inline!)
pub mod crypto;
pub mod state;
pub mod commands;
pub mod p2p;
pub mod storage;

// Публичный API
pub use crypto::{KeyPair, PublicBundle, HybridEncryptor, EncryptedMessage, PasswordHasher, Signer};
pub use state::AppState;
