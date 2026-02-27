//! Secure Telegram Client Library
//!
//! Децентрализованный Telegram клиент с постквантовым шифрованием,
//! anti-censorship и P2P fallback.

pub mod crypto;
pub mod obfs;
pub mod stego;
pub mod network;
pub mod p2p;
pub mod storage;
pub mod updater;
pub mod config;
pub mod tdlib_wrapper;

// Re-export main modules for easier access
pub use crypto::{kyber, xchacha, dh};
pub use obfs::obfs4;
pub use stego::lsb;
pub use network::{transport, blockage_detector, dns_resolver, proxy_manager};
pub use storage::message_queue;
pub use updater::{ipfs_updater, github};
