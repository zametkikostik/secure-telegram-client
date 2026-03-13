//! Liberty Reach — Resurrection Edition
//!
//! Полный функционал:
//! - P2P мессенджер с E2EE
//! - AI интеграция (Ollama + OpenRouter)
//! - Семейные статусы с подтверждением
//! - IPFS/Pinata хранилище
//! - Web3 кошелек (Polygon)
//! - Telegram Bridge
//! - WebSocket сервер для Web-фронтенда
//! - Биржевой агрегатор (Bitget/Bybit)
//! - Админ-панель с верификацией
//! - Stories (24h статусы)

use anyhow::Result;
use futures::StreamExt;
use libp2p::{SwarmBuilder, swarm::SwarmEvent, PeerId};
use tokio::io::{self, AsyncBufReadExt};
use colored::Colorize;
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::{mpsc, broadcast, RwLock};
use tokio::net::TcpStream;

mod identity;
mod crypto;
mod network;
mod ai;
mod storage;
mod bridge;
mod wallet;
mod exchange;
mod admin;
mod stories;
mod voice;
mod calls;
mod groups;
mod ratchet;
mod governance;
mod ai_fleet;

use crypto::CipherManager;
use ai_fleet::SovereignAIClient;
use identity::{ProfileManager, RelationshipType, RelationshipRequest};
use network::{LibertyBehaviour, LibertyBehaviourEvent, TOPIC_NAME, create_gossipsub_config};
use libp2p::gossipsub;
use rand::RngCore;
#[allow(unused_imports)]
use admin::PeerMetadata;
use stories::StoryMediaType;

#[cfg(feature = "voice")]
use voice::VoiceManager;
#[cfg(feature = "calls")]
use calls::CallManager;
use groups::GroupManager;

/// Сообщения для WebSocket клиентов
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum WSMessage {
    #[serde(rename = "chat_message")]
    ChatMessage {
        from: String,
        text: String,
        timestamp: u64,
    },
    #[serde(rename = "file_transfer")]
    FileTransfer {
        cid: String,
        filename: String,
        sender: String,
    },
    #[serde(rename = "status_update")]
    StatusUpdate {
        peer_id: String,
        status: String,
        relationship: String,
    },
    #[serde(rename = "wallet_info")]
    WalletInfo {
        address: String,
        balance: String,
    },
    #[serde(rename = "error")]
    Error {
        message: String,
    },
}

/// Состояние приложения для WebSocket
pub struct AppState {
    pub profile_manager: ProfileManager,
    pub wallet_manager: Option<wallet::WalletManager>,
    pub storage_manager: Option<storage::StorageManager>,
    pub exchange_manager: Option<exchange::ExchangeManager>,
    pub admin_manager: admin::AdminManager,
    pub story_manager: stories::StoryManager,
    #[cfg(feature = "voice")]
    pub voice_manager: Option<VoiceManager>,
    #[cfg(feature = "calls")]
    pub call_manager: Option<CallManager>,
    pub group_manager: GroupManager,
    pub connected_peers: Vec<PeerId>,
    pub message_history: Vec<String>,
}

impl AppState {
    pub fn new(peer_id: &PeerId) -> Self {
        Self {
            profile_manager: ProfileManager::new(peer_id),
            wallet_manager: None,
            storage_manager: None,
            exchange_manager: None,
            admin_manager: admin::AdminManager::new(admin::AdminConfig::new()),
            story_manager: stories::StoryManager::new(),
            #[cfg(feature = "voice")]
            voice_manager: None,
            #[cfg(feature = "calls")]
            call_manager: None,
            group_manager: GroupManager::new(&peer_id.to_string()),
            connected_peers: Vec::new(),
            message_history: Vec::new(),
        }
    }
}

#[tokio::main]
async fn main() {
    // Инициализация логирования
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("liberty_reach_messenger=info".parse().unwrap())
                .add_directive("libp2p=warn".parse().unwrap())
        )
        .init();

    if let Err(e) = run().await {
        eprintln!("{} {}", "✗ Критическая ошибка:".red().bold(), e);
        tracing::error!("Критическая ошибка: {}", e);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    // Загрузка переменных окружения
    let _ = dotenvy::from_filename(".env.local").ok();
    let _ = dotenvy::dotenv().ok();

    print_banner();

    // Загрузка идентичности
    let id_keys = identity::load_or_generate_identity("identity.key");
    let peer_id = identity::get_peer_id(&id_keys);
    println!("{} {}", "✓ Peer ID:".green(), peer_id.to_string().bright_white());

    // Инициализация AI
    let ai = ai::AIBridge::new();

    // Инициализация Sovereign AI Client (OpenRouter Free Fleet)
    let sovereign_ai: Arc<RwLock<Option<SovereignAIClient>>> = Arc::new(RwLock::new(None));
    
    if let Ok(api_key) = std::env::var("OPENROUTER_API_KEY") {
        if api_key.starts_with("sk-or-") {
            let client = SovereignAIClient::new(api_key);
            println!("{} {}", "✓ Sovereign AI:".green(), "OpenRouter Free-Fleet активирован");
            *sovereign_ai.write().await = Some(client);
        } else {
            println!("{} {}", "⚠ Sovereign AI:".yellow(), "Неверный формат OPENROUTER_API_KEY");
        }
    } else {
        println!("{} {}", "⚠ Sovereign AI:".yellow(), "OPENROUTER_API_KEY не настроен");
    }

    // Инициализация менеджера профилей
    let profile_manager = ProfileManager::new(&peer_id);
    let app_state = Arc::new(RwLock::new(AppState::new(&peer_id)));
    app_state.write().await.profile_manager = profile_manager;

    println!("{} {}", "✓ Профиль:".green(), "Семейные статусы активированы".bright_white());

    // Инициализация Web3 кошелька (опционально)
    let web3_config = wallet::Web3Config::new();
    if web3_config.is_configured() {
        match wallet::WalletManager::new(web3_config) {
            Ok(wallet_mgr) => {
                app_state.write().await.wallet_manager = Some(wallet_mgr);
                println!("{} {}", "✓ Web3:".green(), "Polygon кошелек подключен".bright_white());
            }
            Err(e) => {
                tracing::warn!("Web3 не подключен: {}", e);
            }
        }
    }

    // Инициализация Pinata storage (опционально)
    let pinata_config = storage::PinataConfig::new();
    if pinata_config.is_configured() {
        app_state.write().await.storage_manager = Some(storage::StorageManager::new(pinata_config));
        println!("{} {}", "✓ Storage:".green(), "Pinata IPFS подключен".bright_white());
    }

    // Инициализация биржевого менеджера (опционально)
    let exchange_config = exchange::ExchangeConfig::new();
    if exchange_config.is_configured() {
        app_state.write().await.exchange_manager = Some(exchange::ExchangeManager::new(exchange_config));
        println!("{} {}", "✓ Exchange:".green(), "Bitget/Bybit агрегатор подключен".bright_white());
    }

    // Инициализация админ-панели
    {
        let mut state = app_state.write().await;
        // Используем хэширование для безопасной настройки администратора
        state.admin_manager.config.set_admin_peer_id(&peer_id.to_string());
        if state.admin_manager.config.is_configured() {
            println!("{} {}", "✓ Admin:".green(), "Панель администратора активирована".bright_white());
        }
    }

    // Инициализация Voice Manager
    #[cfg(feature = "voice")]
    {
        let mut state = app_state.write().await;
        // Используем тот же ключ что и для CipherManager
        let cipher_key = state.profile_manager.get_cipher_key();
        // Получаем private key из identity keys для подписи
        let private_key = state.profile_manager.get_private_key();
        match VoiceManager::new(&cipher_key, &private_key) {
            Ok(voice_mgr) => {
                state.voice_manager = Some(voice_mgr);
                println!("{} {}", "✓ Voice:".green(), "Голосовые сообщения активированы".bright_white());
            }
            Err(e) => {
                tracing::warn!("Voice Manager не подключен: {}", e);
            }
        }
    }

    // Инициализация Call Manager (WebRTC)
    #[cfg(feature = "calls")]
    {
        let mut state = app_state.write().await;
        let signaling_url = std::env::var("SIGNALING_URL").unwrap_or_else(|_| "https://secure-messenger-push.kostik.workers.dev".to_string());
        // Получаем seed для Ed25519 ключей (32 байта)
        let seed = state.profile_manager.get_private_key();
        let seed_array: [u8; 32] = seed;
        match CallManager::new(&peer_id.to_string(), &signaling_url, &seed_array).await {
            Ok(call_mgr) => {
                state.call_manager = Some(call_mgr);
                println!("{} {}", "✓ Calls:".green(), "WebRTC звонки активированы".bright_white());
            }
            Err(e) => {
                tracing::warn!("Call Manager не подключен: {}", e);
            }
        }
    }

    // История сообщений
    let mut message_history: Vec<String> = Vec::with_capacity(10);

    // Создание конфигурации Gossipsub
    let g_config = create_gossipsub_config()
        .map_err(|e| anyhow::anyhow!("Gossipsub config error: {}", e))?;

    // Построение Swarm
    let mut swarm = SwarmBuilder::with_existing_identity(id_keys)
        .with_tokio()
        .with_tcp(
            libp2p::tcp::Config::default().nodelay(true),
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )?
        .with_relay_client(
            libp2p::noise::Config::new,
            libp2p::yamux::Config::default,
        )?
        .with_behaviour(|key, relay| {
            let g_config_clone = g_config.clone();
            Ok(LibertyBehaviour::new(key, g_config_clone, relay)
                .map_err(|e| anyhow::anyhow!("Behaviour error: {}", e))?)
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    // Слушаем все интерфейсы
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Подписка на топики
    swarm.behaviour_mut().gossipsub.subscribe(&gossipsub::IdentTopic::new(TOPIC_NAME))?;

    println!("{} {}", "✓ Сеть:".green(), "Запущена (TCP + Relay + AutoNAT + DCUtR + Identify)".bright_white());

    // Запуск WebSocket сервера
    let ws_state = app_state.clone();
    let (ws_shutdown_tx, ws_shutdown_rx) = broadcast::channel::<()>(1);
    
    tokio::spawn(async move {
        if let Err(e) = run_websocket_server(ws_state, ws_shutdown_rx).await {
            tracing::error!("WebSocket сервер остановлен: {}", e);
        }
    });

    println!("{} {}", "✓ WebSocket:".green(), "Порт 8080 (Web-фронтенд)".bright_white());
    println!();
    println!("{} {}", "ℹ Команды:".yellow().bold(), "/help".bright_white());
    println!();

    // Шифрование для демонстрации
    let cipher = CipherManager::generate_random();
    println!("{} {}", "🔒 E2EE:".green(), "AES-256-GCM активирован".bright_white());
    println!();

    let mut stdin = io::BufReader::new(io::stdin()).lines();

    loop {
        tokio::select! {
            // Ввод из консоли
            line = stdin.next_line() => {
                let line: String = match line {
                    Ok(Some(l)) => l,
                    Ok(None) => {
                        println!("{} {}", "👋 Выход...".yellow(), "До связи!".bright_white());
                        break;
                    }
                    Err(e) => {
                        eprintln!("{} {}", "✗ Ошибка ввода:".red(), e);
                        continue;
                    }
                };

                let input = line.trim();

                if input.starts_with('/') {
                    if let Err(e) = handle_command(
                        input,
                        &mut swarm,
                        &ai,
                        &mut message_history,
                        &peer_id,
                        &cipher,
                        &app_state,
                        &sovereign_ai,
                    ).await {
                        eprintln!("{} {}", "✗ Ошибка команды:".red(), e);
                    }
                    continue;
                }

                // Отправка сообщения (E2EE)
                if !input.is_empty() {
                    match cipher.encrypt(input.as_bytes()) {
                        Ok(encrypted) => {
                            if let Err(e) = swarm.behaviour_mut().gossipsub.publish(
                                gossipsub::IdentTopic::new(TOPIC_NAME),
                                encrypted,
                            ) {
                                eprintln!("{} {}", "✗ Ошибка публикации:".red(), e);
                            } else {
                                message_history.push(format!("Вы: {}", input));
                                tracing::info!("Сообщение отправлено");
                            }
                        }
                        Err(e) => eprintln!("{} {}", "✗ Ошибка шифрования:".red(), e),
                    }
                }
            }

            // События Swarm
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(LibertyBehaviourEvent::Gossipsub(
                    gossipsub::Event::Message { message, propagation_source: peer_id, .. }
                )) => {
                    handle_gossipsub_message(
                        message.clone(),
                        &peer_id,
                        &cipher,
                        &mut message_history,
                        &ai,
                    ).await;
                }

                SwarmEvent::Behaviour(LibertyBehaviourEvent::Kademlia(kad_event)) => {
                    // Синхронизация профилей через Kademlia
                    tracing::debug!("Kademlia событие: {:?}", kad_event);
                }

                SwarmEvent::Behaviour(LibertyBehaviourEvent::Mdns(mdns_event)) => {
                    match mdns_event {
                        libp2p::mdns::Event::Discovered(list) => {
                            for (peer_id, _addr) in list {
                                println!("{} {}", "🔍 mDNS найден:".green(), peer_id.to_string().bright_white());
                                app_state.write().await.connected_peers.push(peer_id);
                            }
                        }
                        libp2p::mdns::Event::Expired(list) => {
                            for (peer_id, _addr) in list {
                                app_state.write().await.connected_peers.retain(|p| p != &peer_id);
                            }
                        }
                    }
                }

                SwarmEvent::Behaviour(LibertyBehaviourEvent::Identify(identify_event)) => {
                    if let libp2p::identify::Event::Received { peer_id, info } = identify_event {
                        tracing::info!("Identify: {} использует {}", peer_id, info.protocol_version);
                        if !app_state.read().await.connected_peers.contains(&peer_id) {
                            app_state.write().await.connected_peers.push(peer_id);
                        }
                    }
                }

                SwarmEvent::Behaviour(LibertyBehaviourEvent::RelayClient(relay_event)) => {
                    tracing::info!("Relay событие: {:?}", relay_event);
                }

                SwarmEvent::Behaviour(LibertyBehaviourEvent::Autonat(autonat_event)) => {
                    if let libp2p::autonat::Event::StatusChanged { old, new } = autonat_event {
                        println!("{} {} → {}", "🌐 NAT статус:".cyan(),
                            format!("{:?}", old).yellow(),
                            format!("{:?}", new).green());
                    }
                }

                SwarmEvent::Behaviour(LibertyBehaviourEvent::Dcutr(dcutr_event)) => {
                    tracing::info!("DCUtR событие: {:?}", dcutr_event);
                }

                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("{} {}", "📍 Слушаю адрес:".green(), address.to_string().bright_white());
                }

                SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                    tracing::info!("Соединение установлено с {} через {:?}", peer_id, endpoint);
                    if !app_state.read().await.connected_peers.contains(&peer_id) {
                        app_state.write().await.connected_peers.push(peer_id);
                    }
                }

                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                    app_state.write().await.connected_peers.retain(|p| p != &peer_id);
                }

                _ => {}
            }
        }
    }

    Ok(())
}

/// WebSocket сервер для Web-фронтенда
async fn run_websocket_server(
    state: Arc<RwLock<AppState>>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<()> {
    use tokio::net::{TcpListener, TcpStream};
    use tokio_tungstenite::tungstenite::Message;
    #[allow(unused_imports)]
    use futures::{SinkExt, StreamExt};

    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("WebSocket сервер запущен на {}", addr);

    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                tracing::info!("WebSocket сервер остановлен");
                break;
            }

            result = listener.accept() => {
                match result {
                    Ok((stream, _)) => {
                        let state = state.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_websocket_connection(stream, state).await {
                                tracing::warn!("Ошибка WebSocket соединения: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Ошибка принятия соединения: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Обработка WebSocket соединения
async fn handle_websocket_connection(
    stream: TcpStream,
    state: Arc<RwLock<AppState>>,
) -> Result<()> {
    use tokio_tungstenite::tungstenite::Message;
    use futures::{SinkExt, StreamExt};

    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    let (mut writer, mut reader) = ws_stream.split();

    // Отправляем приветственное сообщение
    let welcome = WSMessage::StatusUpdate {
        peer_id: state.read().await.profile_manager.our_profile.peer_id.clone(),
        status: state.read().await.profile_manager.our_profile.status_text.clone(),
        relationship: state.read().await.profile_manager.our_profile.get_status_display(),
    };
    writer.send(Message::Text(serde_json::to_string(&welcome)?)).await?;

    while let Some(msg) = reader.next().await {
        let msg = msg?;
        
        match msg {
            Message::Text(text) => {
                // Парсим команду от клиента
                if let Ok(cmd) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(cmd_type) = cmd["type"].as_str() {
                        match cmd_type {
                            "get_profile" => {
                                let profile = &state.read().await.profile_manager.our_profile;
                                let response = serde_json::json!({
                                    "type": "profile",
                                    "name": profile.name,
                                    "status": profile.status_text,
                                    "relationship": profile.get_status_display(),
                                });
                                writer.send(Message::Text(response.to_string())).await?;
                            }
                            "get_wallet" => {
                                if let Some(wallet_mgr) = &state.read().await.wallet_manager {
                                    if let Ok(info) = wallet::WalletInfo::from_manager(wallet_mgr).await {
                                        let response = WSMessage::WalletInfo {
                                            address: info.address,
                                            balance: format!("{} MATIC", info.matic_balance),
                                        };
                                        writer.send(Message::Text(serde_json::to_string(&response)?)).await?;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    Ok(())
}

/// Обработка сообщений Gossipsub
async fn handle_gossipsub_message(
    message: gossipsub::Message,
    peer_id: &PeerId,
    cipher: &CipherManager,
    history: &mut Vec<String>,
    ai: &ai::AIBridge,
) {
    let topic = message.topic.as_str();
    let short_id = peer_id.to_string().chars().take(8).collect::<String>();

    if topic == TOPIC_NAME {
        match cipher.decrypt(&message.data) {
            Ok(decrypted) => {
                let text = String::from_utf8_lossy(&decrypted);
                println!("{} [{}]: {}", "💬".blue(), short_id.bright_blue(), text.bright_white());
                history.push(format!("[{}]: {}", short_id, text));

                if text.contains("@ai") {
                    println!("{} {}", "🤖 AI:".purple().bold(), "Думаю...".bright_white());
                    match ai.ask(text.replace("@ai", "").trim()).await {
                        Ok(answer) => println!("{} {}", "🤖 AI:".purple().bold(), answer.bright_white()),
                        Err(e) => eprintln!("{} {}", "✗ Ошибка AI:".red(), e),
                    }
                }
            }
            Err(_) => {
                println!("{} [{}]: {}", "🔒".yellow(), short_id.yellow(),
                    "Зашифрованное сообщение (нет ключа)".bright_yellow());
            }
        }
    }
}

/// Обработка команд консоли
async fn handle_command(
    cmd: &str,
    _swarm: &mut libp2p::Swarm<LibertyBehaviour>,
    ai: &ai::AIBridge,
    history: &mut Vec<String>,
    my_peer_id: &PeerId,
    _cipher: &CipherManager,
    app_state: &Arc<RwLock<AppState>>,
    sovereign_ai: &Arc<RwLock<Option<SovereignAIClient>>>,
) -> Result<()> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let command = parts[0].to_lowercase();

    match command.as_str() {
        "/help" => print_help(),

        "/info" => {
            let state = app_state.read().await;
            println!("\n{}", "Информация об узле:".cyan().bold());
            println!("  Peer ID: {}", my_peer_id.to_string().bright_white());
            println!("  Имя: {}", state.profile_manager.our_profile.name.bright_white());
            println!("  Статус: {}", state.profile_manager.our_profile.get_status_display().green());
            println!("  Подключено: {} пиров", state.connected_peers.len().to_string().bright_white());
            println!();
        }

        "/profile" => {
            let mut state = app_state.write().await;
            if parts.len() >= 3 && parts[1] == "set" {
                let new_name = parts[2..].join(" ");
                state.profile_manager.our_profile.name = new_name.clone();
                println!("{} {}", "✓ Имя обновлено:".green(), new_name.bright_white());
            } else {
                println!("\n{}", "Ваш профиль:".cyan().bold());
                let profile = &state.profile_manager.our_profile;
                println!("  Имя: {}", profile.name.bright_white());
                println!("  Статус: {}", profile.status_text.bright_white());
                println!("  Отношения: {}", profile.get_status_display().green());
                if let Some(hash) = &profile.avatar_ipfs_hash {
                    println!("  Аватар: ipfs://{}", hash.bright_white());
                }
                println!();
            }
        }

        "/status" => {
            let mut state = app_state.write().await;
            if parts.len() >= 2 {
                let new_status = parts[1..].join(" ");
                state.profile_manager.update_status(new_status.clone());
                println!("{} {}", "✓ Статус обновлен:".green(), new_status.bright_white());
            } else {
                let state = app_state.read().await;
                println!("{} {}", "Ваш статус:".cyan(), 
                    state.profile_manager.our_profile.status_text.bright_white());
            }
        }

        "/wallet" => {
            let state = app_state.read().await;
            if let Some(wallet_mgr) = &state.wallet_manager {
                match wallet::WalletInfo::from_manager(wallet_mgr).await {
                    Ok(info) => {
                        println!("\n{}", "Информация о кошельке:".cyan().bold());
                        println!("  Адрес: {}", info.address.bright_white());
                        println!("  Сеть: {}", info.network.bright_white());
                        println!("  Баланс: {} MATIC", info.matic_balance.green());
                        if let Some(usdc) = info.usdc_balance {
                            println!("  Баланс: {} USDC", usdc.green());
                        }
                        println!();
                    }
                    Err(e) => eprintln!("{} {}", "✗ Ошибка:".red(), e),
                }
            } else {
                println!("{} {}", "⚠ Кошелек:".yellow(), "Не подключен (настройте WEB3_RPC_URL)".bright_yellow());
            }
        }

        "/ai" => {
            if parts.len() < 2 {
                println!("{} {}", "⚠ Использование:".yellow(), "/ai [ваш вопрос]".bright_white());
                return Ok(());
            }

            let query = parts[1..].join(" ");
            println!("{} {}", "🤖 AI:".purple().bold(), "Анализирую...".bright_white());

            let last_messages = history.iter().rev().take(5).cloned().collect::<Vec<_>>();
            match ai.analyze_chat(&last_messages, &query).await {
                Ok(answer) => {
                    println!("{} {}", "🤖 AI ответ:".purple().bold(), answer.bright_white());
                    history.push(format!("AI: {}", answer));
                }
                Err(e) => eprintln!("{} {}", "✗ Ошибка AI:".red(), e),
            }
        }

        "/ask-free" => {
            // Проверка что sovereign_ai настроен
            let ai_guard = sovereign_ai.read().await;
            if ai_guard.is_none() {
                println!("{} {}", "⚠ Ошибка:".red(), "Sovereign AI не настроен (проверьте OPENROUTER_API_KEY)".bright_white());
                return Ok(());
            }

            if parts.len() < 2 {
                println!("{} {}", "⚠ Использование:".yellow(), "/ask-free [model_alias] [вопрос]".bright_white());
                println!("  Модели: qwen, llama, gemma, mistral");
                println!("  Пример: /ask-free qwen Что такое Rust?");
                return Ok(());
            }

            // Парсинг модели и вопроса
            let (model_alias, question) = if parts.len() > 2 && parts[1].len() < 20 {
                (parts[1].to_lowercase(), parts[2..].join(" "))
            } else {
                ("qwen".to_string(), parts[1..].join(" "))
            };

            // Выбор модели по алиасу
            let model = match model_alias.as_str() {
                "qwen" | "q" => "qwen/qwen-2.5-72b-instruct:free",
                "llama" | "l" => "meta-llama/llama-3.3-70b-instruct:free",
                "gemma" | "g" => "google/gemma-2-9b-it:free",
                "mistral" | "m" => "mistralai/mistral-small:free",
                _ => "qwen/qwen-2.5-72b-instruct:free",
            };

            println!("{} {} {}", "🤖 AI:".purple().bold(), "Модель:".bright_white(), model);
            println!("{} {}", "⏳ Запрос:".yellow(), question.bright_white());
            println!("{} {}", "⏳ Загрузка:".cyan(), "Используется OpenRouter Free-Fleet...".bright_white());

            // Запрос к AI
            if let Some(ref sovereign_ai_client) = *ai_guard {
                match sovereign_ai_client.ask_model(model, &question).await {
                    Ok(answer) => {
                        println!("{} {}", "✅ {} ответ:".green(), model);
                        println!("{}", answer.bright_white());
                        history.push(format!("AI ({}): {}", model, answer));
                    }
                    Err(e) => eprintln!("{} {}", "✗ Ошибка AI:".red(), e),
                }
            }
        }

        "/peers" => {
            let state = app_state.read().await;
            println!("\n{}", "Подключенные узлы:".cyan().bold());
            for peer_id in &state.connected_peers {
                println!("  • {}", peer_id.to_string().bright_white());
            }
            if state.connected_peers.is_empty() {
                println!("  {}", "Нет активных соединений".bright_yellow());
            }
            println!();
        }

        "/trade" => {
            if parts.len() < 3 {
                println!("{} {}", "⚠ Использование:".yellow(), "/trade [pair] [amount]".bright_white());
                println!("  Пример: /trade BTCUSDT 100");
                return Ok(());
            }

            let pair = parts[1].to_uppercase();
            let amount: f64 = parts[2].parse().unwrap_or(0.0);

            if amount <= 0.0 {
                println!("{} {}", "⚠ Ошибка:".red(), "Неверная сумма".bright_white());
                return Ok(());
            }

            let mut state = app_state.write().await;
            if let Some(exchange) = &mut state.exchange_manager {
                match exchange.create_order(&pair, exchange::TradeSide::Buy, amount).await {
                    Ok(order) => {
                        println!("\n{}", "📈 Ордер создан:".cyan().bold());
                        println!("  Пара: {}", order.pair.bright_white());
                        println!("  Тип: {}", "Покупка".green());
                        println!("  Сумма: {} {}", order.amount.to_string().bright_white(), order.pair.split('/').next().unwrap_or(""));
                        println!("  Цена: ${:.4}", order.price);
                        println!("  Комиссия ({}%): ${:.4}", exchange.get_fee_percent(), order.fee);
                        println!("  Итого: ${:.4}", order.total);
                        println!("  Кошелек комиссии: {}", exchange.get_admin_wallet().bright_white());
                    }
                    Err(e) => eprintln!("{} {}", "✗ Ошибка:".red(), e),
                }
            } else {
                println!("{} {}", "⚠ Exchange:".yellow(), "Не настроен (добавьте API ключи бирж)".bright_yellow());
            }
        }

        "/price" => {
            if parts.len() < 2 {
                println!("{} {}", "⚠ Использование:".yellow(), "/price [pair]".bright_white());
                return Ok(());
            }

            let pair = parts[1].to_uppercase();
            let mut state = app_state.write().await;

            if let Some(exchange) = &mut state.exchange_manager {
                match exchange.get_best_price(&pair).await {
                    Ok(price) => {
                        println!("{} {}: ${:.4}", "💹 Цена:".cyan(), pair.bright_white(), price);
                    }
                    Err(e) => eprintln!("{} {}", "✗ Ошибка:".red(), e),
                }
            } else {
                println!("{} {}", "⚠ Exchange:".yellow(), "Не настроен".bright_yellow());
            }
        }

        "/stories" => {
            let state = app_state.read().await;
            let all_stories = state.story_manager.get_all_active_stories();
            
            println!("\n{}", "📖 Активные истории:".cyan().bold());
            if all_stories.is_empty() {
                println!("  {}", "Нет активных историй".bright_yellow());
            } else {
                for story in all_stories {
                    println!("  • [{}] {} - {} (👁 {})", 
                        story.format_time_remaining().yellow(),
                        story.author_peer_id.chars().take(8).collect::<String>().bright_white(),
                        story.text_content.as_deref().unwrap_or("Медиа"),
                        story.view_count
                    );
                }
            }
            println!();
        }

        "/admin_requests" => {
            let state = app_state.read().await;
            let requests = state.admin_manager.get_pending_requests();
            
            println!("\n{}", "📋 Заявки на верификацию:".cyan().bold());
            if requests.is_empty() {
                println!("  {}", "Нет заявок".bright_yellow());
            } else {
                for req in requests {
                    println!("  • {} ({}) - {}", 
                        req.name.bright_white(),
                        req.peer_id.chars().take(8).collect::<String>(),
                        req.reason
                    );
                }
            }
            println!();
        }

        "/admin_verify" => {
            if parts.len() < 3 {
                println!("{} {}", "⚠ Использование:".yellow(), "/admin_verify [peer] [approve|decline]".bright_white());
                return Ok(());
            }

            let target_peer = parts[1];
            let action = parts[2].to_lowercase();
            let approve = action == "approve" || action == "a";

            let mut state = app_state.write().await;

            match state.admin_manager.review_verification_request(&my_peer_id.to_string(), target_peer, approve) {
                Ok(status) => {
                    println!("{} {}", "✓ Статус изменен:".green(), format!("{:?}", status).bright_white());
                }
                Err(e) => eprintln!("{} {}", "✗ Ошибка:".red(), e),
            }
        }

        "/admin_trace" => {
            if parts.len() < 2 {
                println!("{} {}", "⚠ Использование:".yellow(), "/admin_trace [peer]".bright_white());
                return Ok(());
            }

            let target_peer = parts[1];
            let state = app_state.read().await;

            // Geo-Trace требует прав администратора
            match state.admin_manager.trace_peer_location(target_peer).await {
                Ok(Some(metadata)) => {
                    println!("\n{}", "📍 Geo-Trace:".cyan().bold());
                    println!("  Peer ID: {}", metadata.peer_id.bright_white());
                    println!("  Последний раз: {} сек назад", metadata.last_seen);
                    if let Some(ip) = &metadata.last_ip {
                        println!("  IP: {}", ip.bright_white());
                    }
                    if let Some(country) = &metadata.country {
                        println!("  Страна: {}", country.bright_white());
                    }
                    if let Some(city) = &metadata.city {
                        println!("  Город: {}", city.bright_white());
                    }
                }
                Ok(None) => {
                    println!("{} {}", "⚠ Нет данных:".yellow(), "Пир офлайн или данные устарели".bright_yellow());
                }
                Err(e) => eprintln!("{} {}", "✗ Ошибка:".red(), e),
            }
        }

        "/admin_premium" => {
            if parts.len() < 3 {
                println!("{} {}", "⚠ Использование:".yellow(), "/admin_premium [peer] [days]".bright_white());
                return Ok(());
            }

            let target_peer = parts[1];
            let days: u32 = parts[2].parse().unwrap_or(0);

            if days == 0 {
                println!("{} {}", "⚠ Ошибка:".red(), "Неверное количество дней".bright_white());
                return Ok(());
            }

            let mut state = app_state.write().await;

            match state.admin_manager.grant_premium(&my_peer_id.to_string(), target_peer, days) {
                Ok(()) => {
                    println!("{} {} на {} дней", "✓ Premium выдан:".green(), target_peer.bright_white(), days);
                }
                Err(e) => eprintln!("{} {}", "✗ Ошибка:".red(), e),
            }
        }

        "/admin_status" => {
            if parts.len() < 2 {
                println!("{} {}", "⚠ Использование:".yellow(), "/admin_status [peer]".bright_white());
                return Ok(());
            }

            let target_peer = parts[1];
            let state = app_state.read().await;
            let status = state.admin_manager.get_status(target_peer);

            println!("{} {}: {} {}", "📊 Статус:".cyan(), target_peer.bright_white(), 
                status.as_emoji(), format!("{:?}", status).bright_white());
        }

        "/quit" | "/exit" => {
            println!("{} {}", "👋 Выход...".yellow(), "До связи!".bright_white());
            std::process::exit(0);
        }

        _ => {
            println!("{} {} {}", "⚠ Неизвестная команда:".yellow(), command, "(введите /help)".bright_white());
        }
    }

    Ok(())
}

fn print_banner() {
    println!("\n{}", "╔══════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{}", "║     Liberty Reach - P2P Messenger v0.5.0           ║".cyan().bold());
    println!("{}", "║        Admin Core & Exchange Edition                   ║".cyan().dimmed());
    println!("{}", "╚══════════════════════════════════════════════════════════╝".cyan().bold());
    println!();
}

fn print_help() {
    println!("\n{}", "Доступные команды:".cyan().bold());
    println!("  {}              - Показать эту справку", "/help".bright_white());
    println!("  {}              - Информация об узле", "/info".bright_white());
    println!("  {} [name]       - Показать/установить профиль", "/profile".bright_white());
    println!("  {} [текст]      - Установить статус", "/status".bright_white());
    println!("  {}              - Показать баланс кошелька", "/wallet".bright_white());
    println!("  {} [текст]      - Запрос к AI с контекстом", "/ai".bright_white());
    println!("  {} [model] [вопрос] - Запрос к Free-Fleet AI", "/ask-free".bright_white());
    println!("  {}              - Показать подключенные узлы", "/peers".bright_white());
    println!();
    println!("{} {}", "📈 Биржа:".yellow().bold(), "".bright_white());
    println!("  {} [pair] [sum] - Создать ордер (например: /trade BTCUSDT 100)", "/trade".bright_white());
    println!("  {} [pair]       - Узнать цену", "/price".bright_white());
    println!();
    println!("{} {}", "📖 Stories:".yellow().bold(), "".bright_white());
    println!("  {}              - Показать активные истории", "/stories".bright_white());
    println!();
    println!("{} {}", "🛡️ Админ-панель:".yellow().bold(), "".bright_white());
    println!("  {}              - Заявки на верификацию", "/admin_requests".bright_white());
    println!("  {} [peer] [a/d] - Принять/отклонить заявку", "/admin_verify".bright_white());
    println!("  {} [peer] [days]- Выдать Premium", "/admin_premium".bright_white());
    println!("  {} [peer]       - Geo-Trace пира", "/admin_trace".bright_white());
    println!("  {} [peer]       - Показать статус", "/admin_status".bright_white());
    println!();
    println!("{} {}", "🤖 AI Models:".yellow().bold(), "qwen, llama, gemma, mistral".bright_white());
    println!();
    println!("  {}              - Выйти из приложения", "/quit".bright_white());
    println!();
    println!("{} {}", "💡 Совет:".yellow(), "Откройте http://localhost:8080 для Web-интерфейса".bright_white());
    println!();
}
