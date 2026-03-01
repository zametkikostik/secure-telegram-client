//! Secure Telegram Client Library
//!
//! Децентрализованный Telegram клиент с постквантовым шифрованием,
//! anti-censorship и P2P fallback.

#![allow(dead_code)] // Многие компоненты - заглушки для будущего функционала

pub mod config;
pub mod crypto;
pub mod network;
pub mod obfs;
pub mod p2p;
pub mod stego;
pub mod storage;
pub mod tdlib_wrapper;
pub mod updater;

// Re-export main modules for easier access
pub use crypto::{dh, kyber, xchacha};
pub use network::{blockage_detector, dns_resolver, proxy_manager, transport};
pub use obfs::obfs4;
pub use stego::lsb;
pub use storage::message_queue;
pub use updater::{github, ipfs_updater};
