//! DNS over HTTPS (DoH)
//!
//! Обход DNS блокировок через зашифрованные DNS запросы

use anyhow::{Result, Context};
use reqwest::Client;
use serde::Deserialize;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

/// DoH клиент
pub struct DohClient {
    http_client: Client,
    endpoints: Vec<String>,
    current_endpoint: usize,
}

/// DNS ответ
#[derive(Debug, Clone, Deserialize)]
pub struct DohResponse {
    pub Status: u32,
    pub TC: bool,
    pub RD: bool,
    pub RA: bool,
    pub AD: bool,
    pub CD: bool,
    pub Question: Vec<DnsQuestion>,
    pub Answer: Option<Vec<DnsRecord>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DnsQuestion {
    pub name: String,
    #[serde(rename = "type")]
    pub qtype: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DnsRecord {
    pub name: String,
    #[serde(rename = "type")]
    pub rtype: u16,
    pub TTL: u32,
    pub data: String,
}

impl DohClient {
    /// Создание нового клиента
    pub fn new() -> Self {
        Self {
            http_client: Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
            endpoints: vec![
                "https://cloudflare-dns.com/dns-query".to_string(),
                "https://dns.google/dns-query".to_string(),
                "https://dns.quad9.net/dns-query".to_string(),
                "https://doh.opendns.com/dns-query".to_string(),
            ],
            current_endpoint: 0,
        }
    }

    /// Создание с кастомными эндпоинтами
    pub fn with_endpoints(endpoints: Vec<String>) -> Self {
        Self {
            http_client: Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
            endpoints,
            current_endpoint: 0,
        }
    }

    /// DNS запрос
    pub async fn resolve(&mut self, domain: &str) -> Result<Vec<IpAddr>> {
        let mut last_error = None;

        for i in 0..self.endpoints.len() {
            let idx = (self.current_endpoint + i) % self.endpoints.len();
            let endpoint = &self.endpoints[idx];

            match self.resolve_with_endpoint(domain, endpoint).await {
                Ok(ips) => {
                    self.current_endpoint = idx;
                    return Ok(ips);
                }
                Err(e) => {
                    log::warn!("DoH endpoint {} failed: {}", endpoint, e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Все DoH эндпоинты недоступны")))
    }

    /// DNS запрос к конкретному эндпоинту
    async fn resolve_with_endpoint(&self, domain: &str, endpoint: &str) -> Result<Vec<IpAddr>> {
        let url = format!("{}?name={}&type=A", endpoint, domain);

        let response = self.http_client.get(&url)
            .header("Accept", "application/dns-json")
            .send()
            .await
            .with_context(|| format!("Ошибка запроса к {}", endpoint))?;

        if !response.status().is_success() {
            anyhow::bail!("DoH вернул статус: {}", response.status());
        }

        let doh_response: DohResponse = response.json()
            .await
            .context("Ошибка парсинга DoH ответа")?;

        // Проверка статуса
        if doh_response.Status != 0 {
            anyhow::bail!("DNS ошибка: статус {}", doh_response.Status);
        }

        // Извлечение IP адресов
        let mut ips = Vec::new();
        if let Some(answers) = &doh_response.Answer {
            for record in answers {
                if record.rtype == 1 { // A record
                    if let Ok(ip) = record.data.parse::<IpAddr>() {
                        ips.push(ip);
                    }
                }
            }
        }

        if ips.is_empty() {
            anyhow::bail!("Нет A записей для домена {}", domain);
        }

        Ok(ips)
    }

    /// Проверка доступности эндпоинтов
    pub async fn check_endpoints(&self) -> Vec<(String, bool)> {
        let mut results = Vec::new();

        for endpoint in &self.endpoints {
            let url = format!("{}?name=google.com&type=A", endpoint);
            let is_available = self.http_client.get(&url)
                .header("Accept", "application/dns-json")
                .send()
                .await
                .map(|r| r.status().is_success())
                .unwrap_or(false);

            results.push((endpoint.clone(), is_available));
        }

        results
    }

    /// Переключение на следующий эндпоинт
    pub fn rotate_endpoint(&mut self) {
        self.current_endpoint = (self.current_endpoint + 1) % self.endpoints.len();
        log::info!("DoH эндпоинт ротирован: {}", self.endpoints[self.current_endpoint]);
    }

    /// Добавление эндпоинта
    pub fn add_endpoint(&mut self, endpoint: String) {
        if !self.endpoints.contains(&endpoint) {
            self.endpoints.push(endpoint);
        }
    }
}

impl Default for DohClient {
    fn default() -> Self {
        Self::new()
    }
}

/// DNS блокировка детектор
pub struct DnsBlockageDetector {
    doh_client: DohClient,
    test_domains: Vec<String>,
}

impl DnsBlockageDetector {
    pub fn new() -> Self {
        Self {
            doh_client: DohClient::new(),
            test_domains: vec![
                "google.com".to_string(),
                "cloudflare.com".to_string(),
                "github.com".to_string(),
            ],
        }
    }

    /// Проверка DNS блокировки
    pub async fn is_blocked(&mut self, domain: &str) -> Result<bool> {
        // Попытка резолва через системный DNS
        let sys_dns = tokio::net::lookup_host((domain, 443)).await;

        // Попытка резолва через DoH
        let doh_dns = self.doh_client.resolve(domain).await;

        match (sys_dns, doh_dns) {
            (Ok(_), Ok(_)) => {
                // Оба работают - нет блокировки
                Ok(false)
            }
            (Err(_), Ok(_)) => {
                // Системный DNS не работает, DoH работает - DNS блокировка
                log::warn!("Обнаружена DNS блокировка для {}", domain);
                Ok(true)
            }
            (Err(_), Err(_)) => {
                // Оба не работают - возможная полная блокировка
                log::warn!("Возможна полная блокировка для {}", domain);
                Ok(true)
            }
            (Ok(_), Err(_)) => {
                // Системный DNS работает, DoH нет - DoH заблокирован
                log::warn!("DoH заблокирован, системный DNS работает");
                Ok(false)
            }
        }
    }

    /// Тестирование всех доменов
    pub async fn test_all_domains(&mut self) -> Vec<(String, bool)> {
        let mut results = Vec::new();

        for domain in &self.test_domains {
            let is_blocked = self.is_blocked(domain).await.unwrap_or(false);
            results.push((domain.clone(), is_blocked));
        }

        results
    }

    /// Добавление тестового домена
    pub fn add_test_domain(&mut self, domain: String) {
        self.test_domains.push(domain);
    }
}

impl Default for DnsBlockageDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_doh_resolution() {
        let mut client = DohClient::new();
        let result = client.resolve("google.com").await;
        
        // Google обычно доступен
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_endpoint_check() {
        let client = DohClient::new();
        let results = client.check_endpoints().await;
        
        // Хотя бы один эндпоинт должен работать
        let working = results.iter().filter(|(_, ok)| *ok).count();
        assert!(working > 0);
    }

    #[test]
    fn test_client_creation() {
        let client = DohClient::new();
        assert!(!client.endpoints.is_empty());
    }

    #[test]
    fn test_endpoint_rotation() {
        let mut client = DohClient::new();
        let initial = client.current_endpoint;
        client.rotate_endpoint();
        assert_ne!(initial, client.current_endpoint);
    }
}
