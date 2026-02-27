//! Менеджер прокси (заготовка)
//!
//! TODO: Реализация авто-загрузки прокси из IPFS

use anyhow::Result;

pub struct ProxyManager {
    enabled: bool,
}

impl ProxyManager {
    pub fn new() -> Self {
        Self { enabled: false }
    }

    pub async fn load_proxy_list(&mut self, _cid: &str) -> Result<()> {
        log::warn!("Загрузка прокси из IPFS требует реализации");
        Ok(())
    }
}
