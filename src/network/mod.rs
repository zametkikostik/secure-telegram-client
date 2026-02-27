//! Сетевой модуль
//!
//! Предоставляет:
//! - Pluggable Transports API (obfs4, Shadowsocks, SOCKS5)
//! - Детектор блокировок (DNS, TCP RST, TLS)
//! - TLS Fingerprint Evasion
//! - DNS over HTTPS
//! - Менеджер прокси

pub mod blockage_detector;
pub mod dns_over_https;
pub mod dns_resolver;
pub mod obfs4;
pub mod proxy_manager;
pub mod shadowsocks;
pub mod tls_fingerprint;
pub mod transport;

pub use blockage_detector::{BlockageDetector, BlockageManager, BlockageResult, BlockageType};
pub use dns_over_https::{DnsBlockageDetector, DohClient};
pub use dns_resolver::DnsResolver;
pub use obfs4::{Obfs4Bridge, Obfs4Client, Obfs4Stream};
pub use proxy_manager::ProxyManager;
pub use shadowsocks::{ShadowsocksStream, ShadowsocksTransport};
pub use tls_fingerprint::{TlsFingerprint, TlsFingerprintManager};
pub use transport::{TransportConfig, TransportManager, TransportType};

use anyhow::Result;

/// Инициализация сетевого модуля
pub fn init() -> Result<()> {
    log::info!("Инициализация сетевого модуля...");
    log::info!("  - Transport Manager: готов");
    log::info!("  - Blockage Detector: готов");
    log::info!("  - obfs4 транспорт: готов");
    log::info!("  - Shadowsocks транспорт: готов");
    log::info!("  - TLS Fingerprint: готов");
    log::info!("  - DNS over HTTPS: готов");
    Ok(())
}
