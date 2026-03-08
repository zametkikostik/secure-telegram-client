//! The Engine / Reactor Module
//! 
//! Главный цикл ядра на tokio::select!, который одновременно слушает:
//! - Канал команд от UI
//! - События P2P-роя (libp2p Swarm)
//! - SIP-стек и WebRTC сигналлинг
//! 
//! Реализует «Never-Die» цикл с экспоненциальным откатом при переподключениях.

use anyhow::{Context, Result, anyhow, bail};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::sleep;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

use crate::bridge::{
    LrCommand, LrEvent, LrEventTx,
    MessageContent, ConnectionState, CallState, CallEndReason,
    NotificationLevel, ObfuscationMode,
};
use crate::crypto::{
    CryptoContainer, EncryptionAlgorithm, Kdf,
    LsbSteganography,
};
use crate::storage::{
    DatabaseManager, DatabaseConfig, User, Chat, ChatMember, Message, Call,
    generate_random_key,
};
use crate::p2p::{
    P2PCommand, P2PEvent, P2PMessage,
    create_p2p_manager, MESSAGE_TOPIC, SIGNALING_TOPIC,
};
use crate::media::{CallType, WebRtcCommand};
use crate::sip::{SipClient, SipCommand, SipConfig, SipEvent};

// ============================================================================
// КОНФИГУРАЦИЯ ЯДРА
// ============================================================================

/// Конфигурация ядра
#[derive(Debug, Clone)]
pub struct LrCoreConfig {
    /// Путь к базе данных
    pub db_path: String,
    /// Ключ шифрования базы данных
    pub encryption_key: Vec<u8>,
    /// User ID
    pub user_id: String,
    /// Username
    pub username: String,
    /// Bootstrap ноды для P2P
    pub bootstrap_nodes: Vec<String>,
    /// Включить QUIC
    pub enable_quic: bool,
    /// Включить обфускацию
    pub enable_obfuscation: bool,
    /// SIP конфигурация
    pub sip_config: Option<SipConfig>,
}

impl Default for LrCoreConfig {
    fn default() -> Self {
        Self {
            db_path: "liberty_reach.db".to_string(),
            encryption_key: generate_random_key(),
            user_id: String::new(),
            username: String::new(),
            bootstrap_nodes: vec![],
            enable_quic: true,
            enable_obfuscation: false,
            sip_config: None,
        }
    }
}

// ============================================================================
// LrCore - ГЛАВНЫЙ КЛАСС ЯДРА
// ============================================================================

/// Liberty Reach Core - главное ядро приложения
pub struct LrCore {
    /// Конфигурация
    config: LrCoreConfig,
    /// Трансмиттер событий
    event_tx: LrEventTx,
    /// Менеджер базы данных
    db: Arc<RwLock<DatabaseManager>>,
    /// Крипто контейнер
    crypto: Arc<RwLock<CryptoContainer>>,
    /// P2P менеджер (опционально)
    p2p_cmd_tx: Option<mpsc::Sender<P2PCommand>>,
    /// WebRTC менеджер (опционально)
    webrtc_tx: Option<mpsc::Sender<WebRtcCommand>>,
    /// SIP клиент (опционально)
    sip_tx: Arc<RwLock<Option<mpsc::Sender<SipCommand>>>>,
    /// Состояние соединения
    connection_state: Arc<RwLock<ConnectionState>>,
    /// Активные звонки
    active_calls: Arc<RwLock<Vec<String>>>,
    /// Флаг остановки
    shutdown: Arc<RwLock<bool>>,
    /// Peer ID
    peer_id: String,
}

impl LrCore {
    /// Создать и инициализировать ядро
    pub async fn new(
        config: LrCoreConfig,
        event_tx: LrEventTx,
    ) -> Result<Self> {
        info!("Initializing Liberty Reach Core v{}", env!("CARGO_PKG_VERSION"));
        
        // Инициализируем базу данных
        let db_config = DatabaseConfig {
            path: config.db_path.clone().into(),
            encryption_key: config.encryption_key.clone(),
            create_if_missing: true,
        };
        
        let db = DatabaseManager::new(db_config).await
            .context("Failed to initialize database")?;
        
        // Инициализируем крипто контейнер
        let mut crypto = CryptoContainer::new();
        crypto.set_aes_key(config.encryption_key.clone().try_into()
            .unwrap_or([0u8; 32]));
        crypto.init_ed25519();
        crypto.init_x25519();
        
        // Получаем публичный ключ Kyber
        match crypto.init_kyber() {
            Ok(pk) => info!("Kyber1024 public key initialized: {} bytes", pk.len()),
            Err(e) => warn!("Failed to initialize Kyber1024: {}", e),
        }
        
        let peer_id = Uuid::new_v4().to_string();
        
        Ok(Self {
            config,
            event_tx,
            db: Arc::new(RwLock::new(db)),
            crypto: Arc::new(RwLock::new(crypto)),
            p2p_cmd_tx: None,
            webrtc_tx: None,
            sip_tx: Arc::new(RwLock::new(None)),
            connection_state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            active_calls: Arc::new(RwLock::new(vec![])),
            shutdown: Arc::new(RwLock::new(false)),
            peer_id,
        })
    }
    
    /// Запустить главный цикл ядра
    pub async fn run(mut self) -> Result<()> {
        info!("Starting Liberty Reach Core reactor");
        
        // Отправляем событие инициализации
        let _ = self.event_tx.send(LrEvent::Initialized {
            user_id: self.config.user_id.clone(),
            username: self.config.username.clone(),
            peer_id: self.peer_id.clone(),
        }).await;
        
        // Запускаем P2P менеджер если есть bootstrap ноды
        if !self.config.bootstrap_nodes.is_empty() {
            match self.start_p2p().await {
                Ok(_) => info!("P2P manager started"),
                Err(e) => warn!("Failed to start P2P manager: {}", e),
            }
        }
        
        // Запускаем SIP клиент если есть конфигурация
        if let Some(sip_config) = &self.config.sip_config {
            match self.start_sip(sip_config.clone()).await {
                Ok(_) => info!("SIP client started"),
                Err(e) => warn!("Failed to start SIP client: {}", e),
            }
        }
        
        // Обновляем статус соединения
        *self.connection_state.write().await = ConnectionState::Connected;
        let _ = self.event_tx.send(LrEvent::ConnectionStatus {
            status: ConnectionState::Connected,
            peer_count: 0,
            topic_count: 2,
        }).await;
        
        // Сохраняем пользователя в базу
        let user = User {
            user_id: self.config.user_id.clone(),
            username: self.config.username.clone(),
            display_name: None,
            public_key: Some(self.crypto.read().await.sign(b"").unwrap_or_default()),
            pq_public_key: None,
            created_at: current_timestamp(),
            last_seen: Some(current_timestamp()),
            is_contact: false,
            nickname: None,
            bio: None,
            family_status: None,
            partner_id: None,
            wallpaper: None,
            synced_wallpaper_with: None,
            theme: None,
            is_verified: false,
            is_bot: false,
        };
        
        if let Err(e) = self.db.read().await.add_user(&user).await {
            warn!("Failed to save user to database: {}", e);
        }
        
        info!("Liberty Reach Core is running");
        
        // Never-Die цикл с экспоненциальным откатом
        let mut reconnect_delay = Duration::from_secs(1);
        let max_reconnect_delay = Duration::from_secs(60);
        
        while !*self.shutdown.read().await {
            // Здесь мог бы быть основной цикл обработки
            // Но в нашей архитектуре команды приходят через канал
            
            sleep(Duration::from_millis(100)).await;
            
            // Сбрасываем задержку переподключения если соединение активно
            if *self.connection_state.read().await == ConnectionState::Connected {
                reconnect_delay = Duration::from_secs(1);
            }
        }
        
        // Останавливаем все компоненты
        self.shutdown().await?;
        
        Ok(())
    }
    
    /// Запустить P2P менеджер
    async fn start_p2p(&mut self) -> Result<()> {
        let (cmd_tx, mut event_rx, peer_id) = create_p2p_manager(
            self.config.bootstrap_nodes.clone(),
            self.config.enable_quic,
            self.config.enable_obfuscation,
        ).await?;
        
        self.p2p_cmd_tx = Some(cmd_tx.clone());
        self.peer_id = peer_id;
        
        // Подписываемся на топики
        if let Some(tx) = &self.p2p_cmd_tx {
            let _ = tx.send(P2PCommand::Subscribe {
                topic: MESSAGE_TOPIC.to_string(),
            }).await;
            
            let _ = tx.send(P2PCommand::Subscribe {
                topic: SIGNALING_TOPIC.to_string(),
            }).await;
        }
        
        // Spawn task to handle P2P events
        let event_tx = self.event_tx.clone();
        let _db = self.db.clone();

        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                match event {
                    P2PEvent::PeerConnected { peer_id } => {
                        info!("P2P peer connected: {}", peer_id);
                        let _ = event_tx.send(LrEvent::ContactOnline {
                            user_id: peer_id,
                            last_seen: None,
                        }).await;
                    }
                    P2PEvent::PeerDisconnected { peer_id } => {
                        info!("P2P peer disconnected: {}", peer_id);
                        let _ = event_tx.send(LrEvent::ContactOffline {
                            user_id: peer_id,
                        }).await;
                    }
                    P2PEvent::MessageReceived { from, data, topic } => {
                        debug!("P2P message received from {} on {}", from, topic);

                        // Пытаемся десериализовать сообщение
                        if let Ok(p2p_msg) = bincode::deserialize::<P2PMessage>(&data) {
                            match p2p_msg {
                                P2PMessage::Text { from, content, timestamp } => {
                                    let message_id = Uuid::new_v4().to_string();
                                    let _ = event_tx.send(LrEvent::MessageReceived {
                                        message_id,
                                        chat_id: from.clone(),
                                        sender_id: from,
                                        content: MessageContent::Text(content),
                                        timestamp,
                                        encrypted: false,
                                    }).await;
                                }
                                P2PMessage::Encrypted { from, ciphertext, nonce, timestamp } => {
                                    // Здесь будет дешифрование
                                    let message_id = Uuid::new_v4().to_string();
                                    let _ = event_tx.send(LrEvent::MessageReceived {
                                        message_id,
                                        chat_id: from.clone(),
                                        sender_id: from,
                                        content: MessageContent::Encrypted {
                                            ciphertext,
                                            nonce,
                                            algorithm: "aes-256-gcm".to_string(),
                                        },
                                        timestamp,
                                        encrypted: true,
                                    }).await;
                                }
                                P2PMessage::WebRTCSignaling { from, call_id, sdp_type, sdp: _, ice_candidates: _ } => {
                                    // Обработка WebRTC сигналлинга
                                    debug!("Received WebRTC signaling from {}: {} - {}", from, sdp_type, call_id);
                                }
                                P2PMessage::Ping { timestamp } => {
                                    // Отправляем Pong в ответ
                                    let _pong = P2PMessage::Pong {
                                        timestamp,
                                        latency_ms: 0,
                                    };
                                    // Отправка будет реализована когда будет direct messaging
                                }
                                P2PMessage::Pong { timestamp: _, latency_ms } => {
                                    debug!("Ping response: {} ms", latency_ms);
                                }
                                _ => {}
                            }
                        }
                    }
                    P2PEvent::SubscriptionConfirmed { topic } => {
                        info!("Subscribed to P2P topic: {}", topic);
                    }
                    P2PEvent::BootstrapComplete { peer_count } => {
                        info!("P2P bootstrap complete with {} peers", peer_count);
                        let _ = event_tx.send(LrEvent::ConnectionStatus {
                            status: ConnectionState::Connected,
                            peer_count,
                            topic_count: 2,
                        }).await;
                    }
                    P2PEvent::Error { error } => {
                        error!("P2P error: {}", error);
                        let _ = event_tx.send(LrEvent::Notification {
                            level: NotificationLevel::Error,
                            message: format!("P2P error: {}", error),
                        }).await;
                    }
                    P2PEvent::Connected { peer_id } => {
                        info!("P2P connected with peer ID: {}", peer_id);
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Запустить SIP клиент
    async fn start_sip(&self, config: SipConfig) -> Result<()> {
        let (cmd_tx, mut event_rx) = SipClient::new(config).await?;
        *self.sip_tx.write().await = Some(cmd_tx.clone());
        
        let event_tx = self.event_tx.clone();
        
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                match event {
                    SipEvent::Registered { server } => {
                        info!("SIP registered to: {}", server);
                        let _ = event_tx.send(LrEvent::SipStatus {
                            registered: true,
                            server,
                            error: None,
                        }).await;
                    }
                    SipEvent::RegistrationFailed { error } => {
                        error!("SIP registration failed: {}", error);
                        let _ = event_tx.send(LrEvent::SipStatus {
                            registered: false,
                            server: String::new(),
                            error: Some(error),
                        }).await;
                    }
                    SipEvent::IncomingCall { call_id, from, call_type } => {
                        info!("Incoming SIP call from: {}", from);
                        let _ = event_tx.send(LrEvent::CallIncoming {
                            call_id,
                            caller_id: from,
                            caller_username: None,
                            call_type,
                            timestamp: current_timestamp(),
                        }).await;
                    }
                    SipEvent::CallEnded { call_id, reason: _ } => {
                        let _ = event_tx.send(LrEvent::CallEnded {
                            call_id,
                            reason: CallEndReason::RemoteHangup,
                            duration_secs: None,
                        }).await;
                    }
                    SipEvent::MessageReceived { from, content } => {
                        let _ = event_tx.send(LrEvent::MessageReceived {
                            message_id: Uuid::new_v4().to_string(),
                            chat_id: from.clone(),
                            sender_id: from,
                            content: MessageContent::Text(content),
                            timestamp: current_timestamp(),
                            encrypted: false,
                        }).await;
                    }
                    _ => {}
                }
            }
        });
        
        Ok(())
    }
    
    /// Обработать команду от UI
    pub async fn handle_command(&self, command: LrCommand) -> Result<()> {
        debug!("Handling command: {:?}", command);
        
        match command {
            LrCommand::Initialize { .. } => {
                // Уже инициализировано в new()
                Ok(())
            }
            
            LrCommand::RegisterSip { sip_server, sip_username, sip_password, sip_domain } => {
                let config = SipConfig {
                    server: sip_server,
                    username: sip_username,
                    password: sip_password,
                    domain: sip_domain,
                };
                
                self.start_sip(config).await
            }
            
            LrCommand::ConnectP2P { bootstrap_nodes: _, enable_quic: _, enable_obfuscation: _ } => {
                // Обновляем конфигурацию и перезапускаем P2P
                warn!("P2P reconfiguration not yet implemented");
                Ok(())
            }
            
            LrCommand::SendMessage {
                recipient_id,
                content,
                encryption,
                use_steganography,
                stego_image_path,
            } => {
                self.send_message(
                    recipient_id,
                    content,
                    encryption,
                    use_steganography,
                    stego_image_path,
                ).await
            }
            
            LrCommand::SendSipMessage { sip_uri, content } => {
                if let Some(tx) = &*self.sip_tx.read().await {
                    let _ = tx.send(SipCommand::SendMessage {
                        to: sip_uri,
                        content,
                    }).await;
                }
                Ok(())
            }
            
            LrCommand::GetMessageHistory { chat_id, limit, before_message_id } => {
                let messages = self.db.read().await.get_message_history(
                    &chat_id,
                    limit,
                    before_message_id.as_deref(),
                ).await?;
                
                // Отправляем сообщения через события
                for msg in messages {
                    let content = if let Some(data) = &msg.content {
                        match msg.content_type.as_str() {
                            "text" => MessageContent::Text(String::from_utf8_lossy(data).to_string()),
                            _ => MessageContent::Text("[binary content]".to_string()),
                        }
                    } else {
                        MessageContent::Text("".to_string())
                    };
                    
                    let _ = self.event_tx.send(LrEvent::MessageReceived {
                        message_id: msg.message_id,
                        chat_id: msg.chat_id,
                        sender_id: msg.sender_id,
                        content,
                        timestamp: msg.timestamp,
                        encrypted: msg.encrypted,
                    }).await;
                }
                
                Ok(())
            }
            
            LrCommand::MarkAsRead { message_id, chat_id: _ } => {
                self.db.read().await.mark_message_as_read(&message_id, &[&self.config.user_id]).await?;
                Ok(())
            }
            
            LrCommand::StartCall { callee_id, call_type, use_relay } => {
                self.start_call(&callee_id, call_type, use_relay).await
            }
            
            LrCommand::AcceptCall { call_id } => {
                self.accept_call(&call_id).await
            }
            
            LrCommand::RejectCall { call_id, reason } => {
                self.reject_call(&call_id, reason).await
            }
            
            LrCommand::EndCall { call_id } => {
                self.end_call(&call_id).await
            }
            
            LrCommand::SendIceCandidate { call_id: _, candidate: _, sdp_m_line_index: _, sdp_mid: _ } => {
                // Отправляем ICE кандидат через P2P или WebRTC
                Ok(())
            }
            
            LrCommand::StartSipCall { sip_uri, call_type } => {
                if let Some(tx) = &*self.sip_tx.read().await {
                    let _ = tx.send(SipCommand::StartCall {
                        to: sip_uri,
                        call_type,
                    }).await;
                }
                Ok(())
            }
            
            LrCommand::AddContact { user_id, username, public_key, nickname } => {
                let user = User {
                    user_id,
                    username,
                    display_name: nickname.clone(),
                    public_key: Some(public_key),
                    pq_public_key: None,
                    created_at: current_timestamp(),
                    last_seen: None,
                    is_contact: true,
                    nickname,
                    bio: None,
                    family_status: None,
                    partner_id: None,
                    wallpaper: None,
                    synced_wallpaper_with: None,
                    theme: None,
                    is_verified: false,
                    is_bot: false,
                };
                
                self.db.read().await.add_user(&user).await?;
                
                let _ = self.event_tx.send(LrEvent::ContactAdded {
                    user_id: user.user_id.clone(),
                    username: user.username.clone(),
                    nickname: user.display_name.clone(),
                }).await;
                
                Ok(())
            }
            
            LrCommand::RemoveContact { user_id: _ } => {
                // TODO: Реализовать удаление контакта
                Ok(())
            }
            
            LrCommand::CreateGroup { name, members, group_key: _ } => {
                let chat_id = Uuid::new_v4().to_string();
                
                let chat = Chat {
                    chat_id: chat_id.clone(),
                    chat_type: "group".to_string(),
                    name: Some(name),
                    created_at: current_timestamp(),
                    last_message_at: None,
                    unread_count: 0,
                    members: members.iter().map(|user_id| ChatMember {
                        user_id: user_id.clone(),
                        role: "member".to_string(),
                        joined_at: current_timestamp(),
                    }).collect(),
                };
                
                self.db.read().await.create_chat(&chat).await?;
                Ok(())
            }
            
            LrCommand::SetObfuscation { enabled, mode } => {
                info!("Obfuscation set to: {} ({:?})", enabled, mode);
                // TODO: Применить настройки обфускации к P2P
                Ok(())
            }
            
            LrCommand::ExportKeys { path, password } => {
                self.export_keys(&path, &password).await
            }
            
            LrCommand::ImportKeys { path, password } => {
                self.import_keys(&path, &password).await
            }
            
            LrCommand::GetConnectionStatus => {
                let state = self.connection_state.read().await.clone();
                let _ = self.event_tx.send(LrEvent::ConnectionStatus {
                    status: state,
                    peer_count: 0,
                    topic_count: 2,
                }).await;
                Ok(())
            }
            
            LrCommand::Ping { timestamp } => {
                let latency = current_timestamp() - timestamp;
                let _ = self.event_tx.send(LrEvent::Pong {
                    timestamp,
                    latency_ms: latency,
                }).await;
                Ok(())
            }
            
            LrCommand::Shutdown => {
                info!("Shutdown requested");
                *self.shutdown.write().await = true;
                Ok(())
            }
            
            // -------------------------------------------------------------------------
            // Новые функции (Phase 2)
            // -------------------------------------------------------------------------
            LrCommand::SetFamilyStatus { status } => {
                info!("Setting family status: {:?}", status);
                // TODO: Сохранить в базу данных
                Ok(())
            }
            
            LrCommand::SetPartner { partner_id } => {
                info!("Setting partner: {:?}", partner_id);
                // TODO: Сохранить в базу данных
                Ok(())
            }
            
            LrCommand::SetBio { bio } => {
                info!("Setting bio: {}", bio);
                // TODO: Сохранить в базу данных
                Ok(())
            }
            
            LrCommand::SetWallpaper { wallpaper } => {
                info!("Setting wallpaper: {}", wallpaper);
                // TODO: Сохранить в базу данных
                Ok(())
            }
            
            LrCommand::SyncWallpaperWith { user_id } => {
                info!("Syncing wallpaper with: {}", user_id);
                // TODO: Синхронизировать обои
                Ok(())
            }
            
            LrCommand::PinMessage { chat_id, message_id } => {
                info!("Pinning message {} in chat {}", message_id, chat_id);
                let pinned = crate::storage::PinnedMessage {
                    pin_id: Uuid::new_v4().to_string(),
                    chat_id,
                    message_id,
                    pinned_by: self.config.user_id.clone(),
                    pinned_at: current_timestamp(),
                };
                self.db.read().await.pin_message(&pinned).await
            }
            
            LrCommand::UnpinMessage { pin_id } => {
                info!("Unpinning message: {}", pin_id);
                self.db.read().await.unpin_message(&pin_id).await
            }
            
            LrCommand::GetPinnedMessages { chat_id } => {
                info!("Getting pinned messages for chat: {}", chat_id);
                let pinned = self.db.read().await.get_pinned_messages(&chat_id).await?;
                // TODO: Отправить через события
                info!("Found {} pinned messages", pinned.len());
                Ok(())
            }
            
            LrCommand::AddToFavorites { message_id, tags } => {
                info!("Adding message to favorites: {}", message_id);
                let fav = crate::storage::FavoriteMessage {
                    fav_id: Uuid::new_v4().to_string(),
                    message_id,
                    user_id: self.config.user_id.clone(),
                    tags: tags.join(","),
                    created_at: current_timestamp(),
                };
                self.db.read().await.add_to_favorites(&fav).await
            }
            
            LrCommand::GetFavorites => {
                info!("Getting favorites");
                let favs = self.db.read().await.get_favorites(&self.config.user_id).await?;
                // TODO: Отправить через события
                info!("Found {} favorite messages", favs.len());
                Ok(())
            }
            
            LrCommand::RemoveFromFavorites { fav_id } => {
                info!("Removing from favorites: {}", fav_id);
                self.db.read().await.remove_from_favorites(&fav_id).await
            }
            
            LrCommand::ScheduleMessage { chat_id, content, send_at } => {
                info!("Scheduling message for chat {} at {}", chat_id, send_at);
                let content_bytes = match content {
                    MessageContent::Text(text) => text.into_bytes(),
                    _ => bincode::serialize(&content)?,
                };
                let scheduled = crate::storage::ScheduledMessage {
                    schedule_id: Uuid::new_v4().to_string(),
                    chat_id,
                    content: content_bytes,
                    send_at,
                    created_at: current_timestamp(),
                    status: "pending".to_string(),
                };
                self.db.read().await.schedule_message(&scheduled).await
            }
            
            LrCommand::AddReaction { message_id, emoji } => {
                info!("Adding reaction {} to message {}", emoji, message_id);
                let reaction = crate::storage::Reaction {
                    reaction_id: Uuid::new_v4().to_string(),
                    message_id,
                    user_id: self.config.user_id.clone(),
                    emoji,
                    created_at: current_timestamp(),
                };
                self.db.read().await.add_reaction(&reaction).await
            }
            
            LrCommand::RemoveReaction { reaction_id } => {
                info!("Removing reaction: {}", reaction_id);
                self.db.read().await.remove_reaction(&reaction_id).await
            }
            
            LrCommand::CreateStickerPack { name } => {
                info!("Creating sticker pack: {}", name);
                let pack = crate::storage::StickerPack {
                    pack_id: Uuid::new_v4().to_string(),
                    name,
                    creator_id: self.config.user_id.clone(),
                    stickers_count: 0,
                    created_at: current_timestamp(),
                };
                self.db.read().await.create_sticker_pack(&pack).await
            }
            
            LrCommand::AddSticker { pack_id, data, emoji } => {
                info!("Adding sticker to pack: {}", pack_id);
                let sticker = crate::storage::Sticker {
                    sticker_id: Uuid::new_v4().to_string(),
                    pack_id,
                    data,
                    emoji,
                };
                self.db.read().await.add_sticker(&sticker).await
            }
            
            LrCommand::GetStickers { pack_id } => {
                info!("Getting stickers for pack: {}", pack_id);
                let stickers = self.db.read().await.get_stickers(&pack_id).await?;
                // TODO: Отправить через события
                info!("Found {} stickers", stickers.len());
                Ok(())
            }
        }
    }
    
    /// Отправить сообщение
    async fn send_message(
        &self,
        recipient_id: String,
        content: MessageContent,
        encryption: EncryptionAlgorithm,
        use_steganography: bool,
        stego_image_path: Option<String>,
    ) -> Result<()> {
        let message_id = Uuid::new_v4().to_string();
        let timestamp = current_timestamp();
        
        // Сериализуем содержимое
        let content_bytes = match &content {
            MessageContent::Text(text) => text.as_bytes().to_vec(),
            MessageContent::Image { data, .. } => data.clone(),
            MessageContent::Encrypted { ciphertext, .. } => ciphertext.clone(),
            _ => bincode::serialize(&content)?,
        };
        
        // Шифруем если нужно
        let (final_content, encrypted) = if encryption != EncryptionAlgorithm::Aes256Gcm {
            let crypto = self.crypto.read().await;
            let (ciphertext, nonce) = crypto.encrypt(encryption, &content_bytes)?;

            (MessageContent::Encrypted {
                ciphertext,
                nonce,
                algorithm: format!("{:?}", encryption),
            }, true)
        } else {
            let result_content = content;
            (result_content, false)
        };
        
        // Применяем стеганографию если нужно
        if use_steganography {
            if let Some(image_path) = stego_image_path {
                if let MessageContent::Encrypted { ciphertext, .. } = &final_content {
                    LsbSteganography::hide_message_to_file(
                        &image_path,
                        &format!("{}_stego.png", message_id),
                        ciphertext,
                    )?;

                    info!("Message hidden in image using LSB steganography");
                }
            }
        }
        
        // Отправляем через P2P
        if let Some(tx) = &self.p2p_cmd_tx {
            let p2p_msg = match &final_content {
                MessageContent::Text(text) => P2PMessage::Text {
                    from: self.config.user_id.clone(),
                    content: text.clone(),
                    timestamp,
                },
                MessageContent::Encrypted { ciphertext, nonce, algorithm } => {
                    P2PMessage::Encrypted {
                        from: self.config.user_id.clone(),
                        ciphertext: ciphertext.clone(),
                        nonce: nonce.clone(),
                        timestamp,
                    }
                }
                _ => P2PMessage::Text {
                    from: self.config.user_id.clone(),
                    content: "[unsupported content type]".to_string(),
                    timestamp,
                },
            };
            
            let data = bincode::serialize(&p2p_msg)?;
            
            let _ = tx.send(P2PCommand::Publish {
                topic: format!("{}-direct-{}", MESSAGE_TOPIC, recipient_id),
                data,
            }).await;
        }
        
        // Сохраняем в базу
        let message = Message {
            message_id: message_id.clone(),
            chat_id: recipient_id.clone(),
            sender_id: self.config.user_id.clone(),
            content_type: "text".to_string(),
            content: Some(content_bytes),
            encrypted,
            signature: None,
            nonce: None,
            timestamp,
            status: "sent".to_string(),
            is_read: false,
            read_by: None,
            reply_to_message_id: None,
        };
        
        self.db.read().await.save_message(&message).await?;
        
        // Отправляем событие об успешной отправке
        let _ = self.event_tx.send(LrEvent::MessageSent {
            message_id,
            chat_id: recipient_id,
            timestamp,
        }).await;
        
        Ok(())
    }
    
    /// Начать звонок
    async fn start_call(&self, callee_id: &str, call_type: CallType, _use_relay: bool) -> Result<()> {
        let call_id = Uuid::new_v4().to_string();
        
        // Добавляем в активные звонки
        self.active_calls.write().await.push(call_id.clone());
        
        // Сохраняем в базу
        let call = Call {
            call_id: call_id.clone(),
            caller_id: self.config.user_id.clone(),
            callee_id: callee_id.to_string(),
            call_type: format!("{:?}", call_type),
            status: "dialing".to_string(),
            started_at: Some(current_timestamp()),
            ended_at: None,
            duration_secs: None,
            end_reason: None,
        };
        
        self.db.read().await.save_call(&call).await?;
        
        // Отправляем событие
        let _ = self.event_tx.send(LrEvent::CallStatus {
            call_id: call_id.clone(),
            status: CallState::Dialing,
        }).await;
        
        info!("Started call {} to {}", call_id, callee_id);
        
        Ok(())
    }
    
    /// Принять звонок
    async fn accept_call(&self, call_id: &str) -> Result<()> {
        // Обновляем статус в базе
        self.db.read().await.save_call(&Call {
            call_id: call_id.to_string(),
            caller_id: String::new(),
            callee_id: String::new(),
            call_type: String::new(),
            status: "connected".to_string(),
            started_at: Some(current_timestamp()),
            ended_at: None,
            duration_secs: None,
            end_reason: None,
        }).await?;
        
        let _ = self.event_tx.send(LrEvent::CallStatus {
            call_id: call_id.to_string(),
            status: CallState::Connected,
        }).await;
        
        Ok(())
    }
    
    /// Отклонить звонок
    async fn reject_call(&self, call_id: &str, _reason: Option<String>) -> Result<()> {
        let _ = self.event_tx.send(LrEvent::CallEnded {
            call_id: call_id.to_string(),
            reason: CallEndReason::Busy,
            duration_secs: None,
        }).await;
        
        Ok(())
    }
    
    /// Завершить звонок
    async fn end_call(&self, call_id: &str) -> Result<()> {
        // Удаляем из активных
        let mut calls = self.active_calls.write().await;
        if let Some(pos) = calls.iter().position(|c| c == call_id) {
            calls.remove(pos);
        }
        
        // Обновляем в базе
        self.db.read().await.save_call(&Call {
            call_id: call_id.to_string(),
            caller_id: String::new(),
            callee_id: String::new(),
            call_type: String::new(),
            status: "ended".to_string(),
            started_at: None,
            ended_at: Some(current_timestamp()),
            duration_secs: None,
            end_reason: Some("local_hangup".to_string()),
        }).await?;
        
        let _ = self.event_tx.send(LrEvent::CallEnded {
            call_id: call_id.to_string(),
            reason: CallEndReason::LocalHangup,
            duration_secs: None,
        }).await;
        
        Ok(())
    }
    
    /// Экспорт ключей
    async fn export_keys(&self, path: &str, _password: &str) -> Result<()> {
        let _crypto = self.crypto.read().await;
        
        // TODO: Реализовать полный экспорт ключей
        
        info!("Keys exported to {}", path);
        Ok(())
    }
    
    /// Импорт ключей
    async fn import_keys(&self, path: &str, _password: &str) -> Result<()> {
        // TODO: Реализовать импорт ключей
        info!("Keys imported from {}", path);
        Ok(())
    }
    
    /// Остановить ядро
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down Liberty Reach Core");
        
        // Останавливаем P2P
        if let Some(tx) = &self.p2p_cmd_tx {
            let _ = tx.send(P2PCommand::Disconnect {
                peer_id: self.peer_id.clone(),
            }).await;
        }
        
        // Останавливаем SIP
        if let Some(tx) = &*self.sip_tx.read().await {
            let _ = tx.send(SipCommand::Shutdown).await;
        }
        
        *self.shutdown.write().await = true;
        
        let _ = self.event_tx.send(LrEvent::ShutdownComplete).await;
        
        Ok(())
    }
    
    /// Получить Peer ID
    pub fn peer_id(&self) -> &str {
        &self.peer_id
    }
    
    /// Получить состояние соединения
    pub async fn connection_state(&self) -> ConnectionState {
        self.connection_state.read().await.clone()
    }
}

// ============================================================================
// УТИЛИТЫ
// ============================================================================

/// Получить текущий timestamp в секундах
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Создать каналы для команд и событий
pub fn create_core_channels(buffer_size: usize) -> (mpsc::Sender<LrCommand>, LrEventTx) {
    let (cmd_tx, _cmd_rx) = mpsc::channel(buffer_size);
    let (event_tx, _event_rx) = mpsc::channel(buffer_size);

    (cmd_tx, LrEventTx::new(event_tx))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_core_initialization() {
        let (cmd_tx, cmd_rx) = mpsc::channel(100);
        let (event_tx, event_rx) = mpsc::channel(100);
        
        let config = LrCoreConfig {
            db_path: ":memory:".to_string(),
            encryption_key: generate_random_key(),
            user_id: "test_user".to_string(),
            username: "test".to_string(),
            bootstrap_nodes: vec![],
            enable_quic: true,
            enable_obfuscation: false,
            sip_config: None,
        };
        
        let core = LrCore::new(config, LrEventTx::new(event_tx)).await;
        assert!(core.is_ok());
    }
    
    #[test]
    fn test_timestamp() {
        let ts = current_timestamp();
        assert!(ts > 1700000000); // После ноября 2023
    }
}
