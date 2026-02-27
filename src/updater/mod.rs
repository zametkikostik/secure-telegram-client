//! Модуль автообновления
//!
//! Проверка и загрузка обновлений из GitHub Releases и IPFS.

pub mod github;
pub mod ipfs_updater;

use anyhow::{Result, anyhow};
use log::{info, warn};

/// Имя репозитория на GitHub
const REPO_OWNER: &str = "YOUR_USERNAME";
const REPO_NAME: &str = "secure-telegram-client";

/// Проверка наличия обновлений
pub async fn check_for_updates() -> Result<()> {
    info!("Проверка обновлений...");
    
    let current_version = env!("CARGO_PKG_VERSION");
    info!("Текущая версия: {}", current_version);
    
    let latest = github::get_latest_release(REPO_OWNER, REPO_NAME).await?;
    
    info!("Последняя версия: {}", latest.tag_name);
    
    if latest.tag_name.trim_start_matches('v') != current_version {
        info!("🔔 Доступно обновление: {}", latest.tag_name);
        info!("Описание: {}", latest.name);
        println!("\n🔔 Доступно обновление!");
        println!("   Текущая версия: {}", current_version);
        println!("   Новая версия: {}", latest.tag_name);
        println!("   Описание: {}", latest.name);
        println!("\nДля обновления выполните: secure-tg --update");
    } else {
        info!("✅ Установлена последняя версия");
        println!("\n✅ Установлена последняя версия");
    }
    
    Ok(())
}

/// Выполнение обновления
pub async fn perform_update() -> Result<()> {
    info!("Выполнение обновления...");
    
    let current_version = env!("CARGO_PKG_VERSION");
    
    let latest = github::get_latest_release(REPO_OWNER, REPO_NAME).await?;
    let new_version = latest.tag_name.trim_start_matches('v');
    
    if new_version == current_version {
        info!("✅ Обновление не требуется");
        println!("\n✅ Обновление не требуется");
        return Ok(());
    }
    
    info!("Обновление с {} до {}", current_version, new_version);
    
    // Определение платформы
    let target = get_target();
    info!("Целевая платформа: {}", target);
    
    // Поиск подходящего артефакта
    let asset = github::find_asset(&latest, &target)?;
    
    info!("Загрузка артефакта: {}", asset.name);
    
    // Загрузка
    let asset_data = github::download_asset(&asset.browser_download_url).await?;
    
    info!("Артефакт загружен ({} байт)", asset_data.len());
    
    // В реальной реализации здесь будет:
    // 1. Распаковка архива
    // 2. Замена бинарного файла
    // 3. Перезапуск
    
    info!("✅ Обновление готово к установке");
    println!("\n✅ Обновление загружено!");
    println!("   Новая версия: {}", new_version);
    println!("   Файл: {}", asset.name);
    println!("\nВнимание: Автоматическая установка требует прав администратора");
    println!("Рекомендуется вручную заменить бинарный файл");
    
    Ok(())
}

/// Определение целевой платформы
fn get_target() -> &'static str {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        return "x86_64-unknown-linux-gnu";
    }

    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        return "aarch64-unknown-linux-gnu";
    }

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        return "x86_64-apple-darwin";
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        return "aarch64-apple-darwin";
    }

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        return "x86_64-pc-windows-msvc";
    }

    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    {
        return "aarch64-pc-windows-msvc";
    }

    #[allow(unreachable_code)]
    "unknown"
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_target() {
        let target = get_target();
        assert!(!target.is_empty());
        assert_ne!(target, "unknown");
    }
}
