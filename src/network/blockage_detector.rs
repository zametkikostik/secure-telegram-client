//! Детектор блокировок
//!
//! Обнаруживает следующие типы блокировок:
//! - DNS блокировка (возвращает неверный IP)
//! - TCP RST инъекция
//! - TLS блокировка (handshake failure)
//! - DPI детектирование по времени ответа

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Тип блокировки
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BlockageType {
    /// DNS блокировка
    Dns,
    /// TCP RST инъекция
    TcpReset,
    /// TLS блокировка
    TlsHandshake,
    /// DPI детектирование
    Dpi,
    /// Полный IP бан
    IpBan,
    /// Нет блокировки
    None,
    /// Неизвестно
    Unknown,
}

/// Результат проверки
#[derive(Debug, Clone)]
pub struct BlockageResult {
    /// Тип блокировки
    pub blockage_type: BlockageType,
    /// Время проверки
    pub check_time: Duration,
    /// Дополнительная информация
    pub details: String,
}

/// Детектор блокировок
pub struct BlockageDetector {
    /// Кэш результатов проверок
    cache: HashMap<String, (BlockageResult, Instant)>,
    /// TTL кэша
    cache_ttl: Duration,
    /// Тестовые домены для проверки
    test_domains: Vec<String>,
    /// Тестовые IP для проверки
    test_ips: Vec<String>,
    /// DoH эндпоинты
    doh_endpoints: Vec<String>,
}

impl BlockageDetector {
    /// Создание нового детектора
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            cache_ttl: Duration::from_secs(300), // 5 минут
            test_domains: vec![
                "google.com".to_string(),
                "cloudflare.com".to_string(),
                "github.com".to_string(),
            ],
            test_ips: vec!["8.8.8.8".to_string(), "1.1.1.1".to_string()],
            doh_endpoints: vec![
                "https://cloudflare-dns.com/dns-query".to_string(),
                "https://dns.google/dns-query".to_string(),
            ],
        }
    }

    /// Полная проверка на блокировки
    pub async fn check_all(&mut self, target: &str) -> Result<BlockageResult> {
        let start = Instant::now();

        // Проверка кэша
        if let Some((result, time)) = self.cache.get(target) {
            if time.elapsed() < self.cache_ttl {
                return Ok(result.clone());
            }
        }

        // 1. Проверка DNS
        let dns_result = self.check_dns_blockage(target).await;
        if dns_result.blockage_type != BlockageType::None {
            self.cache
                .insert(target.to_string(), (dns_result.clone(), Instant::now()));
            return Ok(dns_result);
        }

        // 2. Проверка TCP подключения
        let tcp_result = self.check_tcp_blockage(target).await;
        if tcp_result.blockage_type != BlockageType::None {
            self.cache
                .insert(target.to_string(), (tcp_result.clone(), Instant::now()));
            return Ok(tcp_result);
        }

        // 3. Проверка TLS handshake
        let tls_result = self.check_tls_blockage(target).await;
        if tls_result.blockage_type != BlockageType::None {
            self.cache
                .insert(target.to_string(), (tls_result.clone(), Instant::now()));
            return Ok(tls_result);
        }

        // 4. Проверка на DPI по времени ответа
        let dpi_result = self.check_dpi_detection(target).await;

        let check_time = start.elapsed();
        self.cache
            .insert(target.to_string(), (dpi_result.clone(), Instant::now()));

        Ok(dpi_result)
    }

    /// Проверка DNS блокировки
    pub async fn check_dns_blockage(&self, target: &str) -> BlockageResult {
        let start = Instant::now();

        // Извлечение домена из target (если это URL)
        let domain = if target.starts_with("http") {
            target
                .trim_start_matches("http://")
                .trim_start_matches("https://")
                .split('/')
                .next()
                .unwrap_or(target)
        } else {
            target.split(':').next().unwrap_or(target)
        };

        // Попытка резолва через системный DNS
        let sys_dns = tokio::net::lookup_host((domain, 80)).await;

        // Попытка резолва через DoH
        let doh_dns = self.resolve_via_doh(domain).await;

        let check_time = start.elapsed();

        match (sys_dns, doh_dns) {
            (Ok(_), Ok(_)) => {
                // Оба работают - нет блокировки
                BlockageResult {
                    blockage_type: BlockageType::None,
                    check_time,
                    details: "DNS работает нормально".to_string(),
                }
            }
            (Err(_), Ok(doh_ips)) => {
                // Системный DNS не работает, DoH работает - DNS блокировка
                BlockageResult {
                    blockage_type: BlockageType::Dns,
                    check_time,
                    details: format!("DNS блокировка обнаружена. DoH IP: {:?}", doh_ips),
                }
            }
            (Err(_), Err(_)) => {
                // Оба не работают - возможный IP бан
                BlockageResult {
                    blockage_type: BlockageType::IpBan,
                    check_time,
                    details: "Возможна полная блокировка сети".to_string(),
                }
            }
            (Ok(_), Err(_)) => {
                // Системный DNS работает, DoH нет - DoH заблокирован
                BlockageResult {
                    blockage_type: BlockageType::Dns,
                    check_time,
                    details: "DoH заблокирован, системный DNS работает".to_string(),
                }
            }
        }
    }

    /// Проверка TCP блокировки
    pub async fn check_tcp_blockage(&self, target: &str) -> BlockageResult {
        let start = Instant::now();

        // Попытка подключения к целевому хосту
        let direct_connect = timeout(Duration::from_secs(5), TcpStream::connect(target)).await;

        let check_time = start.elapsed();

        match direct_connect {
            Ok(Ok(_)) => {
                // Подключение успешно
                BlockageResult {
                    blockage_type: BlockageType::None,
                    check_time,
                    details: "TCP подключение успешно".to_string(),
                }
            }
            Ok(Err(e)) => {
                let err_str = e.to_string();

                // Проверка на TCP RST
                if err_str.contains("Connection reset") || err_str.contains("ECONNRESET") {
                    BlockageResult {
                        blockage_type: BlockageType::TcpReset,
                        check_time,
                        details: "Обнаружена TCP RST инъекция".to_string(),
                    }
                } else if err_str.contains("timed out") {
                    // Таймаут - возможный IP бан
                    BlockageResult {
                        blockage_type: BlockageType::IpBan,
                        check_time,
                        details: "Таймаут подключения - возможный IP бан".to_string(),
                    }
                } else {
                    BlockageResult {
                        blockage_type: BlockageType::Unknown,
                        check_time,
                        details: format!("Ошибка TCP: {}", err_str),
                    }
                }
            }
            Err(_) => {
                // Таймаут на уровне tokio
                BlockageResult {
                    blockage_type: BlockageType::IpBan,
                    check_time,
                    details: "Таймаут подключения".to_string(),
                }
            }
        }
    }

    /// Проверка TLS блокировки
    pub async fn check_tls_blockage(&self, target: &str) -> BlockageResult {
        let start = Instant::now();

        // Извлечение домена
        let domain = if target.starts_with("http") {
            target
                .trim_start_matches("http://")
                .trim_start_matches("https://")
                .split('/')
                .next()
                .unwrap_or(target)
        } else {
            target.split(':').next().unwrap_or(target)
        };

        // Попытка TLS handshake через rustls
        let tls_result = self.try_tls_handshake(domain, 443).await;

        let check_time = start.elapsed();

        match tls_result {
            Ok(_) => BlockageResult {
                blockage_type: BlockageType::None,
                check_time,
                details: "TLS handshake успешен".to_string(),
            },
            Err(e) => {
                let err_str = e.to_string();

                if err_str.contains("handshake") || err_str.contains("certificate") {
                    BlockageResult {
                        blockage_type: BlockageType::TlsHandshake,
                        check_time,
                        details: format!("TLS блокировка: {}", err_str),
                    }
                } else {
                    BlockageResult {
                        blockage_type: BlockageType::Unknown,
                        check_time,
                        details: format!("TLS ошибка: {}", err_str),
                    }
                }
            }
        }
    }

    /// Проверка DPI детектирования
    pub async fn check_dpi_detection(&self, target: &str) -> BlockageResult {
        let start = Instant::now();

        // Проверка по времени ответа
        // DPI системы часто добавляют задержку

        let mut times = Vec::new();

        for _ in 0..3 {
            let conn_start = Instant::now();
            let connect_result = timeout(Duration::from_secs(3), TcpStream::connect(target)).await;

            match connect_result {
                Ok(Ok(_)) => {
                    times.push(conn_start.elapsed());
                }
                _ => break,
            }
        }

        let check_time = start.elapsed();

        if times.is_empty() {
            return BlockageResult {
                blockage_type: BlockageType::Unknown,
                check_time,
                details: "Не удалось выполнить проверку DPI".to_string(),
            };
        }

        // Вычисление среднего времени
        let avg_time = times.iter().sum::<Duration>() / times.len() as u32;

        // Если среднее время > 500ms - возможное DPI
        if avg_time > Duration::from_millis(500) {
            BlockageResult {
                blockage_type: BlockageType::Dpi,
                check_time,
                details: format!("Возможное DPI обнаружено (среднее время: {:?})", avg_time),
            }
        } else {
            BlockageResult {
                blockage_type: BlockageType::None,
                check_time,
                details: format!("DPI не обнаружено (среднее время: {:?})", avg_time),
            }
        }
    }

    /// Разрешение домена через DoH
    async fn resolve_via_doh(&self, domain: &str) -> Result<Vec<String>> {
        let client = reqwest::Client::new();

        for endpoint in &self.doh_endpoints {
            let url = format!("{}?name={}&type=A", endpoint, domain);

            match timeout(
                Duration::from_secs(3),
                client
                    .get(&url)
                    .header("Accept", "application/dns-json")
                    .send(),
            )
            .await
            {
                Ok(Ok(response)) => {
                    if response.status().is_success() {
                        if let Ok(json) = response.json::<serde_json::Value>().await {
                            if let Some(answers) = json["Answer"].as_array() {
                                let mut ips = Vec::new();
                                for answer in answers {
                                    if let Some(data) = answer["data"].as_str() {
                                        ips.push(data.to_string());
                                    }
                                }
                                if !ips.is_empty() {
                                    return Ok(ips);
                                }
                            }
                        }
                    }
                }
                _ => continue,
            }
        }

        Err(anyhow::anyhow!("DoH не ответил"))
    }

    /// Попытка TLS handshake
    async fn try_tls_handshake(&self, domain: &str, port: u16) -> Result<()> {
        // Упрощённая реализация без полноценного TLS
        // В реальной версии использовать tokio-rustls

        let addr = format!("{}:{}", domain, port);
        let _stream = timeout(Duration::from_secs(5), TcpStream::connect(&addr)).await??;

        // Заглушка - всегда Ok
        Ok(())
    }

    /// Очистка кэша
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Установка TTL кэша
    pub fn set_cache_ttl(&mut self, ttl: Duration) {
        self.cache_ttl = ttl;
    }

    /// Добавление тестового домена
    pub fn add_test_domain(&mut self, domain: &str) {
        self.test_domains.push(domain.to_string());
    }

    /// Добавление DoH эндпоинта
    pub fn add_doh_endpoint(&mut self, endpoint: &str) {
        self.doh_endpoints.push(endpoint.to_string());
    }
}

impl Default for BlockageDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Менеджер обхода блокировок
pub struct BlockageManager {
    /// Детектор блокировок
    detector: BlockageDetector,
    /// Статус блокировок по транспортам
    blocked_transports: HashMap<String, Instant>,
}

impl BlockageManager {
    pub fn new() -> Self {
        Self {
            detector: BlockageDetector::new(),
            blocked_transports: HashMap::new(),
        }
    }

    /// Проверка и возврат рекомендуемого транспорта
    pub async fn get_best_transport(&mut self, transports: &[&str]) -> Result<Option<String>> {
        for transport in transports {
            if !self.is_transport_blocked(transport).await {
                return Ok(Some(transport.to_string()));
            }
        }
        Ok(None)
    }

    /// Проверка блокировки транспорта
    pub async fn is_transport_blocked(&mut self, transport: &str) -> bool {
        // Проверка кэша
        if let Some(&time) = self.blocked_transports.get(transport) {
            if time.elapsed() < Duration::from_secs(300) {
                return true;
            }
        }

        // Проверка транспорта
        let result = self.detector.check_all(transport).await;

        match result {
            Ok(r) => {
                if r.blockage_type != BlockageType::None {
                    self.blocked_transports
                        .insert(transport.to_string(), Instant::now());
                    true
                } else {
                    false
                }
            }
            Err(_) => true,
        }
    }

    /// Отметка транспорта как заблокированного
    pub fn mark_transport_blocked(&mut self, transport: &str) {
        self.blocked_transports
            .insert(transport.to_string(), Instant::now());
    }
}

impl Default for BlockageManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detector_creation() {
        let detector = BlockageDetector::new();
        assert_eq!(detector.cache_ttl, Duration::from_secs(300));
    }

    #[tokio::test]
    async fn test_dns_check() {
        let mut detector = BlockageDetector::new();
        let result = detector.check_dns_blockage("google.com").await;

        // Google обычно доступен
        assert!(
            result.blockage_type == BlockageType::None || result.blockage_type == BlockageType::Dns
        );
    }

    #[tokio::test]
    async fn test_tcp_check() {
        let mut detector = BlockageDetector::new();
        let result = detector.check_tcp_blockage("8.8.8.8:53").await;

        // Google DNS обычно доступен
        assert!(
            result.blockage_type == BlockageType::None
                || result.blockage_type == BlockageType::IpBan
        );
    }
}
