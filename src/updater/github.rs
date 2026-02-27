//! GitHub Releases API
//!
//! Работа с GitHub Releases для автообновления.

use anyhow::{anyhow, Result};
use serde::Deserialize;

/// Информация о релизе
#[derive(Debug, Clone, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub draft: bool,
    pub prerelease: bool,
    pub published_at: String,
    pub assets: Vec<Asset>,
}

/// Ассет релиза
#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub name: String,
    pub size: u64,
    pub download_count: u64,
    pub browser_download_url: String,
}

/// Получение последнего релиза
pub async fn get_latest_release(owner: &str, repo: &str) -> Result<Release> {
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        owner, repo
    );

    log::debug!("Запрос к GitHub API: {}", url);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "secure-telegram-client")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| anyhow!("Ошибка запроса к GitHub: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow!("GitHub API вернул статус: {}", response.status()));
    }

    let release: Release = response
        .json()
        .await
        .map_err(|e| anyhow!("Ошибка парсинга ответа GitHub: {}", e))?;

    if release.draft {
        return Err(anyhow!("Последний релиз является черновиком"));
    }

    Ok(release)
}

/// Поиск подходящего ассета для текущей платформы
pub fn find_asset(release: &Release, target: &str) -> Result<Asset> {
    // Поиск по названию target
    let asset = release
        .assets
        .iter()
        .find(|a| a.name.contains(target))
        .cloned();

    if let Some(a) = asset {
        return Ok(a);
    }

    // Альтернативный поиск
    #[cfg(target_os = "linux")]
    {
        if target.contains("linux") {
            if let Some(a) = release
                .assets
                .iter()
                .find(|a| a.name.contains("linux") && a.name.contains("x86_64"))
                .cloned()
            {
                return Ok(a);
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if target.contains("windows") {
            if let Some(a) = release
                .assets
                .iter()
                .find(|a| a.name.contains("windows") || a.name.ends_with(".exe"))
                .cloned()
            {
                return Ok(a);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if target.contains("darwin") || target.contains("apple") {
            if let Some(a) = release
                .assets
                .iter()
                .find(|a| {
                    a.name.contains("macos")
                        || a.name.contains("darwin")
                        || a.name.contains("apple")
                })
                .cloned()
            {
                return Ok(a);
            }
        }
    }

    Err(anyhow!(
        "Не найден подходящий ассет для платформы: {}",
        target
    ))
}

/// Загрузка ассета
pub async fn download_asset(url: &str) -> Result<Vec<u8>> {
    log::debug!("Загрузка ассета: {}", url);

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "secure-telegram-client")
        .header("Accept", "application/octet-stream")
        .send()
        .await
        .map_err(|e| anyhow!("Ошибка загрузки ассета: {}", e))?;

    if !response.status().is_success() {
        return Err(anyhow!("Сервер вернул статус: {}", response.status()));
    }

    let data = response
        .bytes()
        .await
        .map_err(|e| anyhow!("Ошибка чтения данных: {}", e))?;

    Ok(data.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_asset_linux() {
        let release = Release {
            tag_name: "v0.1.0".to_string(),
            name: "v0.1.0".to_string(),
            body: String::new(),
            draft: false,
            prerelease: false,
            published_at: String::new(),
            assets: vec![
                Asset {
                    name: "secure-tg-x86_64-unknown-linux-gnu.tar.gz".to_string(),
                    size: 1000000,
                    download_count: 0,
                    browser_download_url: "https://example.com/linux".to_string(),
                },
                Asset {
                    name: "secure-tg-x86_64-apple-darwin.tar.gz".to_string(),
                    size: 1000000,
                    download_count: 0,
                    browser_download_url: "https://example.com/macos".to_string(),
                },
            ],
        };

        let asset = find_asset(&release, "x86_64-unknown-linux-gnu").unwrap();
        assert!(asset.name.contains("linux"));
    }
}
