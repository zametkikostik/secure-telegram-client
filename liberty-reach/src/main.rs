//! Liberty Reach Messenger - CLI Application
//! 
//! Command-line интерфейс для тестирования ядра.

use anyhow::Result;
use clap::{Parser, Subcommand};
use liberty_reach_core::{
    LrCore, LrCoreConfig,
    bridge::{LrCommand, LrEvent, LrEventTx, create_default_channels},
    crypto::generate_random_key,
    init_logging,
};
use tracing::{info, Level};
use tokio::sync::mpsc;

#[derive(Parser)]
#[command(name = "liberty-reach")]
#[command(author = "Liberty Reach Team")]
#[command(version = "2.0.0")]
#[command(about = "Secure P2P Messenger with WebRTC and SIP support")]
struct Cli {
    /// Путь к базе данных
    #[arg(short, long, default_value = "liberty_reach.db")]
    db_path: String,
    
    /// User ID
    #[arg(short, long)]
    user_id: Option<String>,
    
    /// Username
    #[arg(short, long)]
    username: Option<String>,
    
    /// Bootstrap ноды для P2P
    #[arg(short, long)]
    bootstrap: Vec<String>,
    
    /// Включить QUIC
    #[arg(long)]
    enable_quic: bool,
    
    /// Включить обфускацию
    #[arg(long)]
    enable_obfuscation: bool,
    
    /// Уровень логирования
    #[arg(short, long, default_value = "info")]
    log_level: String,
    
    /// Команды
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Запустить ядро в интерактивном режиме
    Run,
    
    /// Отправить сообщение
    Send {
        /// Получатель
        recipient: String,
        /// Текст сообщения
        message: String,
    },
    
    /// Начать звонок
    Call {
        /// Получатель
        callee: String,
        /// Тип звонка (audio/video)
        #[arg(default_value = "audio")]
        call_type: String,
    },
    
    /// Получить историю сообщений
    History {
        /// Chat ID
        chat_id: String,
        /// Количество сообщений
        #[arg(short, long, default_value = "50")]
        limit: u32,
    },
    
    /// Добавить контакт
    AddContact {
        /// User ID
        user_id: String,
        /// Username
        username: String,
    },
    
    /// Получить статус
    Status,
    
    /// Остановить ядро
    Shutdown,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Инициализация логирования
    let level = match cli.log_level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };
    
    init_logging(level)?;
    
    info!("Liberty Reach Messenger v{}", env!("CARGO_PKG_VERSION"));
    
    // Генерируем user_id и username если не указаны
    let user_id = cli.user_id.unwrap_or_else(|| {
        format!("user_{}", uuid::Uuid::new_v4().to_string()[..8].to_string())
    });
    
    let username = cli.username.unwrap_or_else(|| {
        format!("user_{}", uuid::Uuid::new_v4().to_string()[..8].to_string())
    });
    
    info!("User ID: {}", user_id);
    info!("Username: {}", username);
    
    // Создаем конфигурацию
    let config = LrCoreConfig {
        db_path: cli.db_path,
        encryption_key: generate_random_key(),
        user_id: user_id.clone(),
        username: username.clone(),
        bootstrap_nodes: cli.bootstrap,
        enable_quic: cli.enable_quic,
        enable_obfuscation: cli.enable_obfuscation,
        sip_config: None,
    };
    
    // Создаем каналы
    let (cmd_tx, mut cmd_rx) = mpsc::channel(100);
    let (event_tx, mut event_rx) = mpsc::channel(100);
    
    // Создаем ядро
    let core = LrCore::new(config, LrEventTx::new(event_tx)).await?;
    
    // Запускаем ядро в отдельной задаче
    let core_handle = tokio::spawn(core.run());
    
    // Обработка событий в фоне
    let event_handle = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                LrEvent::Initialized { user_id, username, peer_id } => {
                    println!("✓ Initialized: {}@{} (peer: {})", username, user_id, peer_id);
                }
                LrEvent::MessageReceived { message_id, sender_id, content, timestamp, .. } => {
                    println!("📩 [{}] {}: {:?}", timestamp, sender_id, content);
                }
                LrEvent::MessageSent { message_id, chat_id, timestamp } => {
                    println!("✓ Message sent to {} (id: {})", chat_id, message_id);
                }
                LrEvent::ConnectionStatus { status, peer_count, topic_count } => {
                    println!("🌐 Connection: {:?} (peers: {}, topics: {})", status, peer_count, topic_count);
                }
                LrEvent::CallIncoming { call_id, caller_id, call_type, .. } => {
                    println!("📞 Incoming {} call from {} (id: {})", 
                        if call_type == liberty_reach_core::media::CallType::Audio { "audio" } else { "video" },
                        caller_id, call_id
                    );
                }
                LrEvent::CallStatus { call_id, status } => {
                    println!("📞 Call {} status: {:?}", call_id, status);
                }
                LrEvent::CallEnded { call_id, reason, duration_secs } => {
                    println!("📞 Call {} ended: {:?} (duration: {:?}s)", call_id, reason, duration_secs);
                }
                LrEvent::ContactOnline { user_id, last_seen } => {
                    println!("🟢 {} online (last seen: {:?})", user_id, last_seen);
                }
                LrEvent::ContactOffline { user_id } => {
                    println!("🔴 {} offline", user_id);
                }
                LrEvent::Notification { level, message } => {
                    println!("🔔 [{}]: {}", level, message);
                }
                LrEvent::ShutdownComplete => {
                    println!("✓ Shutdown complete");
                }
                _ => {}
            }
        }
    });
    
    // Даем ядру время на инициализацию
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Выполняем команду если указана
    if let Some(command) = cli.command {
        match command {
            Commands::Run => {
                println!("Running in interactive mode. Press Ctrl+C to stop.");
                
                // Ждем событий
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
            Commands::Send { recipient, message } => {
                println!("Sending message to {}: {}", recipient, message);
                
                cmd_tx.send(LrCommand::SendMessage {
                    recipient_id: recipient,
                    content: liberty_reach_core::bridge::MessageContent::Text(message),
                    encryption: liberty_reach_core::crypto::EncryptionAlgorithm::Aes256Gcm,
                    use_steganography: false,
                    stego_image_path: None,
                }).await?;
                
                // Ждем подтверждения
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
            Commands::Call { callee, call_type } => {
                let ct = if call_type == "video" {
                    liberty_reach_core::media::CallType::Video
                } else {
                    liberty_reach_core::media::CallType::Audio
                };
                
                println!("Starting {} call to {}", call_type, callee);
                
                cmd_tx.send(LrCommand::StartCall {
                    callee_id: callee,
                    call_type: ct,
                    use_relay: false,
                }).await?;
                
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
            Commands::History { chat_id, limit } => {
                println!("Getting message history for chat {} (limit: {})", chat_id, limit);
                
                cmd_tx.send(LrCommand::GetMessageHistory {
                    chat_id,
                    limit,
                    before_message_id: None,
                }).await?;
                
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
            Commands::AddContact { user_id, username } => {
                println!("Adding contact: {}@{}", username, user_id);
                
                cmd_tx.send(LrCommand::AddContact {
                    user_id,
                    username,
                    public_key: vec![],
                    nickname: None,
                }).await?;
                
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            Commands::Status => {
                println!("Getting connection status...");
                
                cmd_tx.send(LrCommand::GetConnectionStatus).await?;
                
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            Commands::Shutdown => {
                println!("Shutting down...");
                
                cmd_tx.send(LrCommand::Shutdown).await?;
                
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    } else {
        // Если команда не указана, запускаем интерактивный режим
        println!("No command specified. Use --help for usage.");
        println!("Starting in run mode...");
        
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
    
    // Останавливаем ядро
    cmd_tx.send(LrCommand::Shutdown).await?;
    
    // Ждем завершения
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // Отменяем задачи
    event_handle.abort();
    core_handle.abort();
    
    println!("Goodbye!");
    
    Ok(())
}
