//! Pluggable Transports API
//!
//! Поддерживаемые транспорты:
//! - obfs4 (obfuscation)
//! - Shadowsocks
//! - MTProto Proxy
//! - SOCKS5
//!
//! Архитектура:
//! ```text
//! Приложение → Transport API → Выбранный транспорт → Интернет
//!                  ↓
//!           Авто-переключение при блокировке
//! ```

use anyhow::{Result, Context, anyhow};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_socks::tcp::Socks5Stream;

/// Тип транспорта
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "type")]
pub enum TransportType {
    /// Прямое соединение
    Direct,
    /// obfs4 обфускация
    Obfs4 {
        /// Адрес моста
        bridge_addr: String,
        /// Публичный ключ моста
        public_key: String,
    },
    /// Shadowsocks
    Shadowsocks {
        /// Адрес сервера
        server_addr: SocketAddr,
        /// Метод шифрования
        method: String,
        /// Пароль
        password: String,
    },
    /// MTProto Proxy
    MtProto {
        /// Адрес прокси
        proxy_addr: SocketAddr,
        /// Secret для прокси
        secret: String,
    },
    /// SOCKS5 Proxy
    Socks5 {
        /// Адрес прокси
        proxy_addr: SocketAddr,
        /// Логин (опционально)
        username: Option<String>,
        /// Пароль (опционально)
        password: Option<String>,
    },
}

/// Конфигурация транспорта
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// Тип транспорта
    pub transport_type: TransportType,
    /// Таймаут подключения
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// Приоритет (меньше = выше приоритет)
    #[serde(default)]
    pub priority: u8,
    /// Включен ли транспорт
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_timeout() -> u64 {
    30
}

fn default_true() -> bool {
    true
}

/// Менеджер транспортов
pub struct TransportManager {
    /// Список доступных транспортов
    transports: Vec<TransportConfig>,
    /// Текущий активный транспорт
    current_transport_index: Option<usize>,
    /// Детектор блокировок
    blockage_detector: BlockageDetector,
}

impl TransportManager {
    /// Создание нового менеджера
    pub fn new(transports: Vec<TransportConfig>) -> Self {
        // Сортировка по приоритету
        let mut sorted = transports;
        sorted.sort_by_key(|t| t.priority);

        Self {
            transports: sorted.into_iter().filter(|t| t.enabled).collect(),
            current_transport_index: None,
            blockage_detector: BlockageDetector::new(),
        }
    }

    /// Подключение через лучший доступный транспорт
    pub async fn connect(&mut self, target_addr: &str) -> Result<Box<dyn AsyncTransport>> {
        let mut last_error = None;

        for (index, transport) in self.transports.iter().enumerate() {
            log::info!("Попытка подключения через {:?}", transport.transport_type);

            match self.connect_via_transport(transport, target_addr).await {
                Ok(stream) => {
                    log::info!("Успешное подключение через {:?}", transport.transport_type);
                    self.current_transport_index = Some(index);
                    return Ok(stream);
                }
                Err(e) => {
                    log::warn!("Транспорт {:?} не удался: {}", transport.transport_type, e);
                    
                    // Проверка на блокировку
                    if self.blockage_detector.is_blocked(&transport.transport_type).await {
                        log::warn!("Транспорт заблокирован, пробуем следующий");
                    }
                    
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("Нет доступных транспортов")))
    }

    /// Подключение через конкретный транспорт
    async fn connect_via_transport(
        &self,
        transport: &TransportConfig,
        target_addr: &str,
    ) -> Result<Box<dyn AsyncTransport>> {
        let timeout = Duration::from_secs(transport.timeout_secs);

        let stream: Box<dyn AsyncTransport> = match &transport.transport_type {
            TransportType::Direct => {
                let tcp_stream = tokio::time::timeout(timeout, TcpStream::connect(target_addr))
                    .await
                    .context("Таймаут прямого подключения")??;
                Box::new(tcp_stream)
            }

            TransportType::Socks5 { proxy_addr, username, password } => {
                let socks_stream = if username.is_some() && password.is_some() {
                    tokio::time::timeout(
                        timeout,
                        Socks5Stream::connect_with_password(
                            proxy_addr,
                            target_addr,
                            username.as_ref().unwrap(),
                            password.as_ref().unwrap(),
                        ),
                    )
                    .await
                    .context("Таймаут SOCKS5 подключения")??
                } else {
                    tokio::time::timeout(timeout, Socks5Stream::connect(proxy_addr, target_addr))
                        .await
                        .context("Таймаут SOCKS5 подключения")??
                };
                Box::new(ProxyStream::Socks5(socks_stream))
            }

            TransportType::Shadowsocks { server_addr, method, password } => {
                // В реальной реализации здесь будет подключение через shadowsocks-crypto
                log::warn!("Shadowsocks транспорт требует дополнительной реализации");
                return Err(anyhow!("Shadowsocks не реализован"));
            }

            TransportType::MtProto { proxy_addr, secret } => {
                // В реальной реализации здесь будет подключение через mtproto-proxy
                log::warn!("MTProto транспорт требует дополнительной реализации");
                return Err(anyhow!("MTProto не реализован"));
            }

            TransportType::Obfs4 { bridge_addr, public_key } => {
                // obfs4 обфускация
                let mut obfs4_client = crate::network::obfs4::Obfs4Client::new(
                    bridge_addr.clone(),
                    public_key.clone(),
                )?;
                
                let stream = obfs4_client.connect().await?;
                Box::new(ProxyStream::Obfs4(stream))
            }
        };

        Ok(stream)
    }

    /// Переключение на следующий транспорт
    pub fn switch_to_next(&mut self) -> Option<&TransportConfig> {
        if let Some(current) = self.current_transport_index {
            let next = (current + 1) % self.transports.len();
            self.current_transport_index = Some(next);
            self.transports.get(next)
        } else if !self.transports.is_empty() {
            self.current_transport_index = Some(0);
            self.transports.get(0)
        } else {
            None
        }
    }

    /// Получить текущий транспорт
    pub fn current_transport(&self) -> Option<&TransportConfig> {
        self.current_transport_index
            .and_then(|i| self.transports.get(i))
    }

    /// Загрузка списка прокси из IPFS
    pub async fn load_proxy_list_from_ipfs(&mut self, cid: &str) -> Result<()> {
        // В реальной реализации загрузка из IPFS
        log::info!("Загрузка списка прокси из IPFS: {}", cid);
        Ok(())
    }

    /// Тестирование всех транспортов
    pub async fn test_all_transports(&mut self, test_url: &str) -> Vec<(TransportType, bool, Duration)> {
        let mut results = Vec::new();

        for transport in &self.transports {
            let start = std::time::Instant::now();
            let success = self.connect_via_transport(transport, test_url).await.is_ok();
            let duration = start.elapsed();

            results.push((transport.transport_type.clone(), success, duration));
        }

        results
    }
}

/// Асинхронный транспорт (обобщённая обёртка)
pub trait AsyncTransport: AsyncRead + AsyncWrite + Unpin + Send {}

impl AsyncTransport for TcpStream {}

/// Прокси стрим (для транспортов с прокси)
pub enum ProxyStream {
    Socks5(Socks5Stream<TcpStream>),
    Obfs4(crate::network::obfs4::Obfs4Stream),
}

impl AsyncRead for ProxyStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            ProxyStream::Socks5(s) => std::pin::Pin::new(s).poll_read(cx, buf),
            ProxyStream::Obfs4(s) => std::pin::Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for ProxyStream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match self.get_mut() {
            ProxyStream::Socks5(s) => std::pin::Pin::new(s).poll_write(cx, buf),
            ProxyStream::Obfs4(s) => std::pin::Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            ProxyStream::Socks5(s) => std::pin::Pin::new(s).poll_flush(cx),
            ProxyStream::Obfs4(s) => std::pin::Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            ProxyStream::Socks5(s) => std::pin::Pin::new(s).poll_shutdown(cx),
            ProxyStream::Obfs4(s) => std::pin::Pin::new(s).poll_shutdown(cx),
        }
    }
}

impl AsyncTransport for ProxyStream {}

/// Детектор блокировок
pub struct BlockageDetector {
    /// Кэш заблокированных транспортов
    blocked_cache: std::collections::HashMap<TransportType, std::time::Instant>,
    /// TTL кэша
    cache_ttl: Duration,
}

impl BlockageDetector {
    pub fn new() -> Self {
        Self {
            blocked_cache: std::collections::HashMap::new(),
            cache_ttl: Duration::from_secs(300), // 5 минут
        }
    }

    /// Проверка блокировки транспорта
    pub async fn is_blocked(&self, transport: &TransportType) -> bool {
        if let Some(&time) = self.blocked_cache.get(transport) {
            if time.elapsed() < self.cache_ttl {
                return true;
            }
        }
        false
    }

    /// Отметка транспорта как заблокированного
    pub fn mark_as_blocked(&mut self, transport: TransportType) {
        self.blocked_cache.insert(transport, std::time::Instant::now());
    }

    /// Анализ типа блокировки
    pub async fn analyze_blockage(&self, target: &str) -> BlockageType {
        // 1. Проверка DNS
        if self.check_dns_blockage(target).await {
            return BlockageType::Dns;
        }

        // 2. Проверка TCP RST
        if self.check_tcp_reset(target).await {
            return BlockageType::TcpReset;
        }

        // 3. Проверка TLS handshake
        if self.check_tls_blockage(target).await {
            return BlockageType::TlsHandshake;
        }

        BlockageType::Unknown
    }

    async fn check_dns_blockage(&self, _target: &str) -> bool {
        // В реальной реализации: попытка резолва через DoH
        false
    }

    async fn check_tcp_reset(&self, _target: &str) -> bool {
        // В реальной реализации: анализ TCP RST пакетов
        false
    }

    async fn check_tls_blockage(&self, _target: &str) -> bool {
        // В реальной реализации: анализ TLS handshake failures
        false
    }
}

/// Тип блокировки
#[derive(Debug, Clone, PartialEq)]
pub enum BlockageType {
    /// DNS блокировка
    Dns,
    /// TCP RST инъекция
    TcpReset,
    /// TLS блокировка
    TlsHandshake,
    /// Полный IP бан
    IpBan,
    /// Неизвестный тип
    Unknown,
}

/// Конфигурация для загрузки из IPFS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyListConfig {
    /// Список прокси
    pub proxies: Vec<TransportConfig>,
    /// Версия списка
    pub version: String,
    /// Дата обновления
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_config_serialization() {
        let config = TransportConfig {
            transport_type: TransportType::Direct,
            timeout_secs: 30,
            priority: 1,
            enabled: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: TransportConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.transport_type, deserialized.transport_type);
    }

    #[tokio::test]
    async fn test_transport_manager_creation() {
        let transports = vec![
            TransportConfig {
                transport_type: TransportType::Direct,
                timeout_secs: 10,
                priority: 1,
                enabled: true,
            },
            TransportConfig {
                transport_type: TransportType::Socks5 {
                    proxy_addr: "127.0.0.1:1080".parse().unwrap(),
                    username: None,
                    password: None,
                },
                timeout_secs: 30,
                priority: 2,
                enabled: true,
            },
        ];

        let manager = TransportManager::new(transports);
        assert_eq!(manager.transports.len(), 2);
    }
}
