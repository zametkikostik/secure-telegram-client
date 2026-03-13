//! Модуль работы с децентрализованным хранилищем (Pinata IPFS)
//!
//! Реализует:
//! - Загрузку файлов через Pinata API
//! - Скачивание файлов по IPFS CID
//! - Content-Addressable Storage (CAS)
//! - Интеграцию с P2P-сетью для передачи CID
//! - IPFS Content Routing через Kademlia

use anyhow::{Result, Context};
use reqwest::{Client, multipart};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;
use std::collections::HashMap;

/// Конфигурация Pinata
pub struct PinataConfig {
    pub api_key: String,
    pub secret_key: String,
    pub gateway_url: String,
}

impl PinataConfig {
    pub fn new() -> Self {
        // Загрузка из переменных окружения
        let api_key = std::env::var("PINATA_API_KEY")
            .unwrap_or_else(|_| String::new());
        let secret_key = std::env::var("PINATA_SECRET_KEY")
            .unwrap_or_else(|_| String::new());

        Self {
            api_key,
            secret_key,
            gateway_url: "https://gateway.pinata.cloud".to_string(),
        }
    }

    pub fn with_keys(api_key: &str, secret_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            secret_key: secret_key.to_string(),
            gateway_url: "https://gateway.pinata.cloud".to_string(),
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty() && !self.secret_key.is_empty()
    }
}

impl Default for PinataConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Ответ от Pinata API при загрузке
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinataUploadResponse {
    #[serde(rename = "IpfsHash")]
    pub ipfs_hash: String,
    #[serde(rename = "PinSize")]
    pub pin_size: u64,
    #[serde(rename = "Timestamp")]
    pub timestamp: String,
}

/// Метаданные для загрузки
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinataMetadata {
    pub name: Option<String>,
    pub keyvalues: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// Менеджер хранилища
pub struct StorageManager {
    client: Client,
    config: PinataConfig,
}

impl StorageManager {
    pub fn new(config: PinataConfig) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(300)) // 5 минут для больших файлов
                .build()
                .unwrap_or_default(),
            config,
        }
    }

    /// Загрузка файла в Pinata
    pub async fn upload_file<P: AsRef<Path>>(&self, file_path: P, metadata: Option<PinataMetadata>) -> Result<String> {
        if !self.config.is_configured() {
            anyhow::bail!("Pinata не настроен: установите PINATA_API_KEY и PINATA_SECRET_KEY");
        }

        let file_path = file_path.as_ref();
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed")
            .to_string();

        let file_data = fs::read(file_path)
            .with_context(|| format!("Не удалось прочитать файл: {:?}", file_path))?;

        // Определяем MIME тип
        let mime_type = mime_guess::from_path(file_path)
            .first_or_octet_stream()
            .to_string();

        let part = multipart::Part::bytes(file_data)
            .file_name(file_name)
            .mime_str(&mime_type)?;

        let mut form = multipart::Form::new()
            .part("file", part);

        // Добавляем метаданные если есть
        if let Some(meta) = metadata {
            let meta_json = serde_json::to_string(&meta)?;
            form = form.text("pinataMetadata", meta_json);
        }

        let response = self.client.post("https://api.pinata.cloud/pinning/pinFileToIPFS")
            .header("pinata_api_key", &self.config.api_key)
            .header("pinata_secret_api_key", &self.config.secret_key)
            .multipart(form)
            .send()
            .await
            .context("Ошибка отправки файла в Pinata")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Pinata вернул ошибку ({}): {}", status, body);
        }

        let upload_response: PinataUploadResponse = response.json()
            .await
            .context("Ошибка парсинга ответа Pinata")?;

        Ok(upload_response.ipfs_hash)
    }

    /// Загрузка текста/JSON в Pinata
    pub async fn upload_json(&self, data: serde_json::Value, name: Option<&str>) -> Result<String> {
        if !self.config.is_configured() {
            anyhow::bail!("Pinata не настроен: установите PINATA_API_KEY и PINATA_SECRET_KEY");
        }

        let mut form = multipart::Form::new()
            .text("file", data.to_string());

        if let Some(file_name) = name {
            form = form.text("pinataOptions", serde_json::json!({
                "cidVersion": 1
            }).to_string());
            form = form.text("pinataMetadata", serde_json::json!({
                "name": file_name
            }).to_string());
        }

        let response = self.client.post("https://api.pinata.cloud/pinning/pinJSONToIPFS")
            .header("pinata_api_key", &self.config.api_key)
            .header("pinata_secret_api_key", &self.config.secret_key)
            .multipart(form)
            .send()
            .await
            .context("Ошибка отправки JSON в Pinata")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Pinata вернул ошибку ({}): {}", status, body);
        }

        let upload_response: PinataUploadResponse = response.json()
            .await
            .context("Ошибка парсинга ответа Pinata")?;

        Ok(upload_response.ipfs_hash)
    }

    /// Скачивание файла по IPFS CID
    pub async fn download_file(&self, cid: &str, save_path: &Path) -> Result<()> {
        let url = format!("{}/ipfs/{}", self.config.gateway_url, cid);

        let response = self.client.get(&url)
            .send()
            .await
            .context("Ошибка загрузки файла из IPFS")?;

        if !response.status().is_success() {
            anyhow::bail!("IPFS gateway вернул ошибку: {}", response.status());
        }

        let file_data = response.bytes()
            .await
            .context("Ошибка чтения данных файла")?;

        // Создаем директорию если нужно
        if let Some(parent) = save_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(save_path, &file_data)
            .with_context(|| format!("Не удалось сохранить файл: {:?}", save_path))?;

        Ok(())
    }

    /// Получение данных по IPFS CID (без сохранения)
    pub async fn fetch_ipfs_data(&self, cid: &str) -> Result<Vec<u8>> {
        let url = format!("{}/ipfs/{}", self.config.gateway_url, cid);

        let response = self.client.get(&url)
            .send()
            .await
            .context("Ошибка загрузки данных из IPFS")?;

        if !response.status().is_success() {
            anyhow::bail!("IPFS gateway вернул ошибку: {}", response.status());
        }

        let data = response.bytes()
            .await
            .context("Ошибка чтения данных")?;

        Ok(data.to_vec())
    }

    /// Проверка доступности файла в IPFS
    pub async fn check_availability(&self, cid: &str) -> Result<bool> {
        let url = format!("{}/ipfs/{}", self.config.gateway_url, cid);

        match self.client.head(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Формирование URL для доступа к файлу
    pub fn get_file_url(&self, cid: &str) -> String {
        format!("{}/ipfs/{}", self.config.gateway_url, cid)
    }
}

/// Структура для передачи файла в P2P сети
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PFileTransfer {
    /// IPFS CID файла
    pub cid: String,
    /// Имя файла
    pub filename: String,
    /// MIME тип
    pub mime_type: String,
    /// Размер файла в байтах
    pub size: u64,
    /// Отправитель
    pub sender_peer_id: String,
    /// Timestamp
    pub timestamp: u64,
    /// Опциональное описание
    pub description: Option<String>,
}

impl P2PFileTransfer {
    pub fn new(
        cid: String,
        filename: String,
        mime_type: String,
        size: u64,
        sender_peer_id: &str,
    ) -> Self {
        Self {
            cid,
            filename,
            mime_type,
            size,
            sender_peer_id: sender_peer_id.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            description: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .context("Ошибка сериализации P2PFileTransfer")
    }

    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .context("Ошибка десериализации P2PFileTransfer")
    }
}

/// Топик для передачи файлов в P2P сети
pub const FILE_TRANSFER_TOPIC: &str = "liberty-file-transfer";

/// IPFS Content Routing Manager
pub struct IpfsContentRouter {
    /// Кэш CID -> PeerID (кто имеет контент)
    cid_peers: HashMap<String, Vec<String>>,
}

impl IpfsContentRouter {
    pub fn new() -> Self {
        Self {
            cid_peers: HashMap::new(),
        }
    }

    /// Регистрация пира как провайдера контента
    pub fn add_provider(&mut self, cid: &str, peer_id: &str) {
        self.cid_peers
            .entry(cid.to_string())
            .or_insert_with(Vec::new)
            .push(peer_id.to_string());
    }

    /// Поиск пиров имеющих контент
    pub fn find_providers(&self, cid: &str) -> Vec<&str> {
        self.cid_peers
            .get(cid)
            .map(|peers| peers.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Удаление записи из кэша
    pub fn remove_provider(&mut self, cid: &str, peer_id: &str) {
        if let Some(peers) = self.cid_peers.get_mut(cid) {
            peers.retain(|p| p != peer_id);
            if peers.is_empty() {
                self.cid_peers.remove(cid);
            }
        }
    }

    /// Публикация записи в DHT (через Kademlia)
    pub fn publish_to_dht(&self, cid: &str) -> String {
        // В реальной реализации здесь был бы вызов kademlia_start_providing()
        format!("Published CID {} to DHT", cid)
    }

    /// Поиск записи в DHT
    pub fn search_dht(&self, cid: &str) -> Vec<String> {
        // В реальной реализации здесь был бы вызов kademlia_get_record()
        self.find_providers(cid).iter().map(|s| s.to_string()).collect()
    }
}

impl Default for IpfsContentRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p2p_file_transfer_serialization() {
        let transfer = P2PFileTransfer::new(
            "QmTest123".to_string(),
            "test.txt".to_string(),
            "text/plain".to_string(),
            1024,
            "12D3KooWTest",
        );

        let json = transfer.to_json().unwrap();
        let decoded = P2PFileTransfer::from_json(&json).unwrap();

        assert_eq!(transfer.cid, decoded.cid);
        assert_eq!(transfer.filename, decoded.filename);
    }
}
