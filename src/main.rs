//! Secure Telegram Client
//! 
//! Telegram клиент с постквантовым шифрованием, DPI обходом и стенографией.

mod crypto;
mod obfs;
mod stego;
mod tdlib_wrapper;
mod updater;
mod config;

use anyhow::Result;
use log::{info, error};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Инициализация логгера
    env_logger::init();
    
    info!("🔐 Secure Telegram Client v{}", env!("CARGO_PKG_VERSION"));
    info!("Запуск...");
    
    // Парсинг аргументов командной строки
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "--check-update" => {
                return updater::check_for_updates().await;
            }
            "--update" => {
                return updater::perform_update().await;
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            _ => {}
        }
    }
    
    // Загрузка конфигурации
    let config = config::Config::load()?;
    info!("Конфигурация загружена");
    
    // Инициализация криптографии
    crypto::init()?;
    info!("Криптография инициализирована");
    
    // Подключение к Telegram через TDLib
    let client = tdlib_wrapper::Client::new(&config).await?;
    info!("TDLib инициализирован");
    
    // Основной цикл
    run_client(client).await?;
    
    Ok(())
}

async fn run_client(_client: tdlib_wrapper::Client) -> Result<()> {
    info!("Клиент запущен. Ожидание команд...");
    
    // Здесь будет основной цикл обработки событий
    // Пока заглушка
    
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
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
    println!("  --help, -h        Показать эту справку");
    println!();
    println!("Примеры:");
    println!("  secure-tg                    # Запуск клиента");
    println!("  secure-tg --check-update     # Проверка обновлений");
    println!("  secure-tg --update           # Обновить клиент");
}
