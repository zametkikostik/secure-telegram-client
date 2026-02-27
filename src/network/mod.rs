//! Сетевой модуль
//!
//! Предоставляет:
//! - Pluggable Transports API (obfs4, Shadowsocks, SOCKS5)
//! - Детектор блокировок (DNS, TCP RST, TLS)
//! - Менеджер прокси
//! - DNS over HTTPS

pub mod transport;
pub mod proxy_manager;
pub mod blockage_detector;
pub mod dns_resolver;
pub mod obfs4;

pub use transport::{TransportManager, TransportConfig, TransportType};
pub use proxy_manager::ProxyManager;
pub use blockage_detector::{BlockageDetector, BlockageManager, BlockageType, BlockageResult};
pub use dns_resolver::DnsResolver;
pub use obfs4::{Obfs4Client, Obfs4Stream, Obfs4Bridge};

use anyhow::Result;

/// Инициализация сетевого модуля
pub fn init() -> Result<()> {
    log::info!("Инициализация сетевого модуля...");
    log::info!("  - Transport Manager: готов");
    log::info!("  - Blockage Detector: готов");
    log::info!("  - obfs4 транспорт: готов");
    log::info!("  - DNS Resolver: готов");
    Ok(())
}
