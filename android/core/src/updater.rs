//! Updater Module
//! 
//! Децентрализованные обновления через IPFS

use anyhow::{Result, Context, anyhow};
use serde::{Deserialize, Serialize};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

/// Информация о релизе в IPFS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub version: String,
    pub version_code: u32,
    pub apk_cid: String,  // IPFS CID APK файла
    pub signature: String,  // Ed25519 подпись (hex)
    pub changelog: String,
    pub published_at: String,
    pub min_sdk: u32,
}

/// Manifest с последними релизами
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseManifest {
    pub latest_version: String,
    pub latest_version_code: u32,
    pub release_cid: String,  // CID с информацией о релизе
    pub public_key: String,  // Публичный ключ разработчика (hex)
}

/// Проверка обновлений через IPFS
pub async fn check_for_updates() -> Result<Option<String>> {
    // В полной реализации:
    // 1. Запрос manifest.json из IPFS
    // 2. Сравнение версий
    // 3. Возврат информации об обновлении
    
    log::info!("Checking for updates via IPFS...");
    
    // Заглушка для демонстрации
    Ok(Some("Update available".to_string()))
}

/// Загрузка APK из IPFS
pub async fn download_apk(cid: &str) -> Result<Vec<u8>> {
    log::info!("Downloading APK from IPFS: {}", cid);
    
    // В полной реализации:
    // 1. Подключение к IPFS через libp2p или HTTP шлюз
    // 2. Загрузка файла по CID
    // 3. Возврат байтов APK
    
    Err(anyhow!("IPFS download not implemented"))
}

/// Верификация подписи релиза
pub fn verify_signature(
    release: &ReleaseInfo,
    data: &[u8],
    public_key_hex: &str,
) -> Result<bool> {
    // Декодирование публичного ключа
    let public_key_bytes = hex::decode(public_key_hex)
        .context("Invalid public key hex")?;
    
    if public_key_bytes.len() != 32 {
        return Err(anyhow!("Invalid public key length"));
    }
    
    let public_key_array: [u8; 32] = public_key_bytes
        .try_into()
        .map_err(|_| anyhow!("Invalid public key length"))?;
    
    let public_key = VerifyingKey::from_bytes(&public_key_array)
        .context("Invalid Ed25519 public key")?;
    
    // Декодирование подписи
    let signature_bytes = hex::decode(&release.signature)
        .context("Invalid signature hex")?;
    
    if signature_bytes.len() != 64 {
        return Err(anyhow!("Invalid signature length"));
    }
    
    let signature = Signature::from_slice(&signature_bytes)?;
    
    // Верификация
    let is_valid = public_key.verify(data, &signature).is_ok();
    
    if is_valid {
        log::info!("Signature verified successfully");
    } else {
        log::warn!("Signature verification failed!");
    }
    
    Ok(is_valid)
}

/// Парсинг manifest.json из IPFS
pub fn parse_manifest(json: &str) -> Result<ReleaseManifest> {
    let manifest: ReleaseManifest = serde_json::from_str(json)
        .context("Failed to parse manifest JSON")?;
    
    Ok(manifest)
}

/// Сравнение версий
pub fn compare_versions(current: u32, available: u32) -> bool {
    available > current
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    
    #[test]
    fn test_signature_generation_and_verification() {
        // Генерация пары ключей
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        
        // Данные для подписи
        let data = b"test apk data";
        
        // Подпись
        let signature = signing_key.sign(data);
        
        // Верификация
        assert!(verifying_key.verify(data, &signature).is_ok());
    }
    
    #[test]
    fn test_manifest_parsing() {
        let json = r#"{
            "latest_version": "0.2.0",
            "latest_version_code": 2,
            "release_cid": "QmTest123",
            "public_key": "0000000000000000000000000000000000000000000000000000000000000000"
        }"#;
        
        let manifest = parse_manifest(json).unwrap();
        assert_eq!(manifest.latest_version, "0.2.0");
        assert_eq!(manifest.latest_version_code, 2);
    }
    
    #[test]
    fn test_version_comparison() {
        assert!(compare_versions(1, 2));
        assert!(!compare_versions(2, 1));
        assert!(!compare_versions(1, 1));
    }
}
