//! Secure Telegram Client v2.0
//!
//! Децентрализованный Telegram клиент с постквантовым шифрованием,
//! anti-censorship и P2P fallback.

mod crypto;
mod obfs;
mod stego;
mod tdlib_wrapper;
mod updater;
mod config;
mod cli;
mod network;
mod p2p;
mod storage;

use anyhow::{Result, Context};
use log::{info, error, warn};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::signal;

/// Глобальный флаг для graceful shutdown
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

#[tokio::main]
async fn main() -> Result<()> {
    // Инициализация логгера с форматированием
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    )
    .format_timestamp_secs()
    .format_target(true)
    .init();

    info!("🔐 Secure Telegram Client v{}", env!("CARGO_PKG_VERSION"));
    info!("Запуск...");

    // Парсинг аргументов командной строки
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--check-update" => {
                return updater::check_for_updates()
                    .await
                    .context("Ошибка проверки обновлений");
            }
            "--update" => {
                return updater::perform_update()
                    .await
                    .context("Ошибка обновления");
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "--version" | "-V" => {
                println!("secure-tg {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            "--init-config" => {
                match config::save_config_template() {
                    Ok(path) => {
                        println!("✅ Конфигурация создана: {:?}", path);
                        println!("📝 Отредактируйте файл и установите api_id и api_hash");
                    }
                    Err(e) => {
                        eprintln!("❌ Ошибка: {}", e);
                    }
                }
                return Ok(());
            }
            "--verbose" | "-v" => {
                // Перезапуск с verbose логированием
                env::set_var("RUST_LOG", "debug");
                println!("Перезапуск с debug логированием...");
                return Ok(());
            }
            _ => {}
        }
    }

    // Загрузка конфигурации
    let config = config::Config::load()
        .context("Не удалось загрузить конфигурацию")?;
    
    // Валидация конфигурации
    if let Err(e) = config.validate() {
        warn!("Конфигурация невалидна: {}", e);
        warn!("Используется конфигурация по умолчанию. Отредактируйте config.json");
    }
    
    info!("Конфигурация загружена");

    // Инициализация криптографии
    crypto::init()
        .context("Ошибка инициализации криптографии")?;
    info!("Криптография инициализирована");

    // Инициализация обфускации
    obfs::init()
        .context("Ошибка инициализации обфускации")?;
    
    // Инициализация стенографии (если включена)
    if config.encryption.steganography_enabled {
        stego::init()
            .context("Ошибка инициализации стенографии")?;
    }

    // Инициализация сетевого модуля
    network::init()
        .context("Ошибка инициализации сетевого модуля")?;
    info!("Сетевой модуль инициализирован");

    // Инициализация P2P модуля (если включен)
    if config.p2p.enabled {
        p2p::init()
            .context("Ошибка инициализации P2P модуля")?;
        info!("P2P модуль инициализирован (fallback режим)");
    }

    // Инициализация хранилища
    storage::init()
        .context("Ошибка инициализации хранилища")?;
    info!("Хранилище инициализировано");

    // Подключение к Telegram через TDLib
    let mut client = tdlib_wrapper::client::TdClient::new(&config).await
        .context("Ошибка инициализации TDLib")?;
    info!("TDLib инициализирован");

    // Обработка сигналов (Ctrl+C)
    let shutdown_sender = client.get_event_sender();
    tokio::spawn(async move {
        if signal::ctrl_c().await.is_ok() {
            info!("Получен сигнал завершения (Ctrl+C)");
            SHUTDOWN.store(true, Ordering::Relaxed);
            // Отправляем событие завершения
            let _ = shutdown_sender.send(tdlib_wrapper::client::TdEvent::ConnectionState { connected: false }).await;
        }
    });

    // Запуск CLI
    info!("Запуск CLI...");
    cli::run_cli(&mut client).await
        .context("Ошибка работы CLI")?;

    info!("Клиент остановлен");

    Ok(())
}

fn print_help() {
    println!("Secure Telegram Client v{}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Использование:");
    println!("  secure-tg [OPTIONS]");
    println!();
    println!("Опции:");
    println!("  --check-update    Проверить наличие обновлений");
    println!("  --update          Выполнить обновление");
    println!("  --init-config     Создать шаблон конфигурации");
    println!("  --verbose, -v     Включить подробное логирование");
    println!("  --version, -V     Показать версию");
    println!("  --help, -h        Показать эту справку");
    println!();
    println!("Примеры:");
    println!("  secure-tg                    # Запуск клиента");
    println!("  secure-tg --check-update     # Проверка обновлений");
    println!("  secure-tg --update           # Обновить клиент");
    println!("  secure-tg -v                 # Запуск с debug логированием");
    println!("  secure-tg --init-config      # Создать шаблон config.json");
}
