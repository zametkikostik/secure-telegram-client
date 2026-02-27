//! DNS Resolver с поддержкой DoH (заготовка)
//!
//! TODO: Интеграция DNS over HTTPS

use anyhow::Result;

pub struct DnsResolver {
    use_doh: bool,
}

impl DnsResolver {
    pub fn new() -> Self {
        Self { use_doh: false }
    }

    pub async fn resolve(&self, hostname: &str) -> Result<Vec<std::net::IpAddr>> {
        // Временная заглушка - использование системного DNS
        use tokio::net::lookup_host;
        
        let addrs: Vec<std::net::IpAddr> = lookup_host((hostname, 0))
            .await?
            .map(|sock_addr| sock_addr.ip())
            .collect();
        
        Ok(addrs)
    }
}
