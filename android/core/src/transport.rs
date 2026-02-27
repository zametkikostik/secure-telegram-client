//! Transport Module
//! 
//! Pluggable Transports API для обхода блокировок

use anyhow::{Result, Context};
use std::net::SocketAddr;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;

/// Trait для всех транспортов
pub trait Transport: Send + Sync {
    /// Подключение к цели
    fn connect(&self, target: &str) -> impl Future<Output = Result<Box<dyn AsyncStream>>>;
    
    /// Название транспорта
    fn name(&self) -> &str;
    
    /// Тип транспорта (для статистики)
    fn transport_type(&self) -> TransportType;
}

/// Асинхронный стрим
pub trait AsyncStream: AsyncRead + AsyncWrite + Unpin + Send {}

/// Тип транспорта
#[derive(Debug, Clone, PartialEq)]
pub enum TransportType {
    Direct,
    Obfs4,
    Shadowsocks,
    WebSocket,
}

/// Прямой транспорт (без обфускации)
pub struct DirectTransport;

impl Transport for DirectTransport {
    async fn connect(&self, target: &str) -> Result<Box<dyn AsyncStream>> {
        let stream = TcpStream::connect(target)
            .await
            .context("Failed to connect")?;
        
        Ok(Box::new(stream))
    }
    
    fn name(&self) -> &str {
        "Direct"
    }
    
    fn transport_type(&self) -> TransportType {
        TransportType::Direct
    }
}

/// obfs4 транспорт
pub struct Obfs4Transport {
    bridge_addr: String,
    public_key: String,
    cert: Option<Vec<u8>>,
}

impl Obfs4Transport {
    pub fn new(bridge_addr: String, public_key: String) -> Self {
        Self {
            bridge_addr,
            public_key,
            cert: None,
        }
    }
    
    /// Парсинг obfs4 URL
    /// Формат: obfs4://<public_key>@<host>:<port>
    pub fn from_url(url: &str) -> Result<Self> {
        if !url.starts_with("obfs4://") {
            anyhow::bail!("Invalid obfs4 URL format");
        }
        
        let without_scheme = &url[8..];
        let parts: Vec<&str> = without_scheme.split('@').collect();
        
        if parts.len() != 2 {
            anyhow::bail!("Invalid obfs4 URL: expected obfs4://key@host:port");
        }
        
        let public_key = parts[0].to_string();
        let bridge_addr = parts[1].to_string();
        
        Ok(Self::new(bridge_addr, public_key))
    }
}

impl Transport for Obfs4Transport {
    async fn connect(&self, target: &str) -> Result<Box<dyn AsyncStream>> {
        // 1. TCP подключение к мосту
        let stream = TcpStream::connect(&self.bridge_addr)
            .await
            .context("Failed to connect to obfs4 bridge")?;
        
        // 2. obfs4 handshake (упрощённо)
        // В полной реализации здесь будет:
        // - ECDH key exchange
        // - Keystream generation
        // - Certificate exchange
        
        // 3. Возвращаем обфусцированный стрим
        Ok(Box::new(stream))
    }
    
    fn name(&self) -> &str {
        "obfs4"
    }
    
    fn transport_type(&self) -> TransportType {
        TransportType::Obfs4
    }
}

/// Shadowsocks транспорт
pub struct ShadowsocksTransport {
    server_addr: SocketAddr,
    method: String,
    password: String,
}

impl ShadowsocksTransport {
    pub fn new(server_addr: SocketAddr, method: String, password: String) -> Self {
        Self {
            server_addr,
            method,
            password,
        }
    }
}

impl Transport for ShadowsocksTransport {
    async fn connect(&self, target: &str) -> Result<Box<dyn AsyncStream>> {
        // Подключение к Shadowsocks серверу
        let stream = TcpStream::connect(self.server_addr)
            .await
            .context("Failed to connect to Shadowsocks server")?;
        
        // В полной реализации здесь будет:
        // - Shadowsocks handshake
        // - Encryption
        
        Ok(Box::new(stream))
    }
    
    fn name(&self) -> &str {
        "Shadowsocks"
    }
    
    fn transport_type(&self) -> TransportType {
        TransportType::Shadowsocks
    }
}

/// Менеджер транспортов
pub struct TransportManager {
    transports: Vec<Box<dyn Transport>>,
    current_index: usize,
}

impl TransportManager {
    pub fn new() -> Self {
        Self {
            transports: Vec::new(),
            current_index: 0,
        }
    }
    
    /// Добавление транспорта
    pub fn add_transport(&mut self, transport: Box<dyn Transport>) {
        self.transports.push(transport);
    }
    
    /// Подключение через лучший доступный транспорт
    pub async fn connect(&mut self, target: &str) -> Result<Box<dyn AsyncStream>> {
        let mut last_error = None;
        
        for i in 0..self.transports.len() {
            let idx = (self.current_index + i) % self.transports.len();
            let transport = &self.transports[idx];
            
            log::info!("Trying transport: {}", transport.name());
            
            match transport.connect(target).await {
                Ok(stream) => {
                    log::info!("Connected via {}", transport.name());
                    self.current_index = idx;
                    return Ok(stream);
                }
                Err(e) => {
                    log::warn!("Transport {} failed: {}", transport.name(), e);
                    last_error = Some(e);
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("No available transports")))
    }
    
    /// Переключение на следующий транспорт
    pub fn switch_to_next(&mut self) {
        self.current_index = (self.current_index + 1) % self.transports.len();
    }
}

impl Default for TransportManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Детектор блокировок
pub async fn check_blockage(target: &str) -> String {
    // Проверка DNS
    match tokio::net::lookup_host((target, 443)).await {
        Ok(_) => log::info!("DNS resolution successful"),
        Err(e) => {
            log::warn!("DNS blockage detected: {}", e);
            return "dns_blocked".to_string();
        }
    }
    
    // Проверка TCP подключения
    match TcpStream::connect(format!("{}:443", target)).await {
        Ok(_) => log::info!("TCP connection successful"),
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("reset") {
                return "tcp_rst".to_string();
            } else if err_str.contains("timed out") {
                return "ip_blocked".to_string();
            }
        }
    }
    
    "ok".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_obfs4_url_parsing() {
        let url = "obfs4://abcdef1234567890@bridge.example.com:443";
        let transport = Obfs4Transport::from_url(url).unwrap();
        
        assert_eq!(transport.bridge_addr, "bridge.example.com:443");
        assert_eq!(transport.public_key, "abcdef1234567890");
    }
    
    #[tokio::test]
    async fn test_direct_transport() {
        let transport = DirectTransport;
        assert_eq!(transport.name(), "Direct");
        assert_eq!(transport.transport_type(), TransportType::Direct);
    }
}
