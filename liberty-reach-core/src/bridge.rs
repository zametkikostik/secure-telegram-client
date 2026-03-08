//! FFI Bridge Layer
//! 
//! Определяет команды (Commands) от UI к ядру и события (Events) от ядра к UI.
//! Все данные передаются через асинхронные каналы (tokio::sync::mpsc).

use serde::{Deserialize, Serialize};
use std::fmt;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::crypto::EncryptionAlgorithm;
use crate::media::CallType;

// ============================================================================
// COMMANDS: UI → Core
// ============================================================================

/// Команды от UI к ядру
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum LrCommand {
    // -------------------------------------------------------------------------
    // Инициализация и аутентификация
    // -------------------------------------------------------------------------
    /// Инициализация ядра с зашифрованной базой данных
    Initialize {
        db_path: String,
        encryption_key: Vec<u8>,
        user_id: String,
        username: String,
    },
    
    /// Регистрация SIP аккаунта
    RegisterSip {
        sip_server: String,
        sip_username: String,
        sip_password: String,
        sip_domain: Option<String>,
    },
    
    /// Вход в P2P сеть
    ConnectP2P {
        bootstrap_nodes: Vec<String>,
        enable_quic: bool,
        enable_obfuscation: bool,
    },
    
    // -------------------------------------------------------------------------
    // Сообщения
    // -------------------------------------------------------------------------
    /// Отправить сообщение через P2P
    SendMessage {
        recipient_id: String,
        content: MessageContent,
        encryption: EncryptionAlgorithm,
        use_steganography: bool,
        stego_image_path: Option<String>,
    },
    
    /// Отправить сообщение через SIP
    SendSipMessage {
        sip_uri: String,
        content: String,
    },
    
    /// Получить историю сообщений
    GetMessageHistory {
        chat_id: String,
        limit: u32,
        before_message_id: Option<String>,
    },
    
    /// Подтвердить прочтение сообщения
    MarkAsRead {
        message_id: String,
        chat_id: String,
    },
    
    // -------------------------------------------------------------------------
    // Звонки (WebRTC + SIP)
    // -------------------------------------------------------------------------
    /// Начать звонок (WebRTC P2P)
    StartCall {
        callee_id: String,
        call_type: CallType,
        use_relay: bool,
    },
    
    /// Принять входящий звонок
    AcceptCall {
        call_id: String,
    },
    
    /// Отклонить звонок
    RejectCall {
        call_id: String,
        reason: Option<String>,
    },
    
    /// Завершить звонок
    EndCall {
        call_id: String,
    },
    
    /// Отправить ICE кандидат
    SendIceCandidate {
        call_id: String,
        candidate: String,
        sdp_m_line_index: u16,
        sdp_mid: String,
    },
    
    /// SIP звонок
    StartSipCall {
        sip_uri: String,
        call_type: CallType,
    },
    
    // -------------------------------------------------------------------------
    // Контакты и чаты
    // -------------------------------------------------------------------------
    /// Добавить контакт
    AddContact {
        user_id: String,
        username: String,
        public_key: Vec<u8>,
        nickname: Option<String>,
    },
    
    /// Удалить контакт
    RemoveContact {
        user_id: String,
    },
    
    /// Создать группу
    CreateGroup {
        name: String,
        members: Vec<String>,
        group_key: Vec<u8>,
    },
    
    // -------------------------------------------------------------------------
    // Настройки и утилиты
    // -------------------------------------------------------------------------
    /// Обновить настройки обфускации трафика
    SetObfuscation {
        enabled: bool,
        mode: ObfuscationMode,
    },
    
    /// Экспорт ключей
    ExportKeys {
        path: String,
        password: String,
    },
    
    /// Импорт ключей
    ImportKeys {
        path: String,
        password: String,
    },
    
    /// Получить статус соединения
    GetConnectionStatus,
    
    /// Ping для проверки активности
    Ping {
        timestamp: u64,
    },
    
    /// Остановить ядро
    Shutdown,
    
    // -------------------------------------------------------------------------
    // Новые функции (Phase 2)
    // -------------------------------------------------------------------------
    /// Установить семейный статус
    SetFamilyStatus {
        status: crate::storage::FamilyStatus,
    },
    
    /// Установить партнёра
    SetPartner {
        partner_id: Option<String>,
    },
    
    /// Установить био
    SetBio {
        bio: String,
    },
    
    /// Установить обои
    SetWallpaper {
        wallpaper: String,
    },
    
    /// Синхронизировать обои с собеседником
    SyncWallpaperWith {
        user_id: String,
    },
    
    /// Закрепить сообщение
    PinMessage {
        chat_id: String,
        message_id: String,
    },
    
    /// Открепить сообщение
    UnpinMessage {
        pin_id: String,
    },
    
    /// Получить закреплённые сообщения
    GetPinnedMessages {
        chat_id: String,
    },
    
    /// Добавить в избранное
    AddToFavorites {
        message_id: String,
        tags: Vec<String>,
    },
    
    /// Получить избранное
    GetFavorites,
    
    /// Удалить из избранного
    RemoveFromFavorites {
        fav_id: String,
    },
    
    /// Запланировать сообщение
    ScheduleMessage {
        chat_id: String,
        content: MessageContent,
        send_at: u64,
    },
    
    /// Добавить реакцию
    AddReaction {
        message_id: String,
        emoji: String,
    },
    
    /// Удалить реакцию
    RemoveReaction {
        reaction_id: String,
    },
    
    /// Создать пак стикеров
    CreateStickerPack {
        name: String,
    },
    
    /// Добавить стикер в пак
    AddSticker {
        pack_id: String,
        data: Vec<u8>,
        emoji: Option<String>,
    },
    
    /// Получить стикеры пака
    GetStickers {
        pack_id: String,
    },
}

/// Режим обфускации трафика
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ObfuscationMode {
    /// Отключена
    #[default]
    Disabled,
    /// Маскировка под HTTPS
    Https,
    /// Obfs4 (как в Tor)
    Obfs4,
    /// Snowflake (как в Tor)
    Snowflake,
    /// DNS туннелирование
    DnsTunnel,
    /// Комбинированный режим
    Hybrid,
}

/// Тип содержимого сообщения
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "content_type", content = "content")]
pub enum MessageContent {
    Text(String),
    Image {
        data: Vec<u8>,
        mime_type: String,
        caption: Option<String>,
    },
    Video {
        data: Vec<u8>,
        mime_type: String,
        duration_secs: Option<u32>,
        caption: Option<String>,
    },
    Audio {
        data: Vec<u8>,
        mime_type: String,
        duration_secs: Option<u32>,
        title: Option<String>,
    },
    File {
        data: Vec<u8>,
        filename: String,
        mime_type: String,
        caption: Option<String>,
    },
    Location {
        latitude: f64,
        longitude: f64,
        accuracy: Option<f32>,
    },
    Contact {
        user_id: String,
        username: String,
        display_name: String,
    },
    Encrypted {
        ciphertext: Vec<u8>,
        nonce: Vec<u8>,
        algorithm: String,
    },
}

// ============================================================================
// EVENTS: Core → UI
// ============================================================================

/// События от ядра к UI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum LrEvent {
    // -------------------------------------------------------------------------
    // Состояние инициализации
    // -------------------------------------------------------------------------
    /// Ядро инициализировано
    Initialized {
        user_id: String,
        username: String,
        peer_id: String,
    },
    
    /// Ошибка инициализации
    InitializationError {
        error: String,
    },
    
    // -------------------------------------------------------------------------
    // Статус соединения
    // -------------------------------------------------------------------------
    /// Статус P2P соединения
    ConnectionStatus {
        status: ConnectionState,
        peer_count: usize,
        topic_count: usize,
    },
    
    /// Статус SIP регистрации
    SipStatus {
        registered: bool,
        server: String,
        error: Option<String>,
    },
    
    // -------------------------------------------------------------------------
    // Сообщения
    // -------------------------------------------------------------------------
    /// Получено новое сообщение
    MessageReceived {
        message_id: String,
        chat_id: String,
        sender_id: String,
        content: MessageContent,
        timestamp: u64,
        encrypted: bool,
    },
    
    /// Сообщение отправлено
    MessageSent {
        message_id: String,
        chat_id: String,
        timestamp: u64,
    },
    
    /// Ошибка отправки сообщения
    MessageError {
        message_id: String,
        error: String,
    },
    
    /// Сообщение прочитано
    MessageRead {
        message_id: String,
        chat_id: String,
        read_by: Vec<String>,
    },
    
    // -------------------------------------------------------------------------
    // Звонки
    // -------------------------------------------------------------------------
    /// Входящий звонок
    CallIncoming {
        call_id: String,
        caller_id: String,
        caller_username: Option<String>,
        call_type: CallType,
        timestamp: u64,
    },
    
    /// Исходящий звонок принят
    CallAccepted {
        call_id: String,
        callee_id: String,
    },
    
    /// Звонок завершен
    CallEnded {
        call_id: String,
        reason: CallEndReason,
        duration_secs: Option<u32>,
    },
    
    /// ICE кандидат получен
    IceCandidateReceived {
        call_id: String,
        candidate: String,
        sdp_m_line_index: u16,
        sdp_mid: String,
    },
    
    /// Статус звонка
    CallStatus {
        call_id: String,
        status: CallState,
    },
    
    // -------------------------------------------------------------------------
    // Контакты
    // -------------------------------------------------------------------------
    /// Контакт добавлен
    ContactAdded {
        user_id: String,
        username: String,
        nickname: Option<String>,
    },
    
    /// Контакт удален
    ContactRemoved {
        user_id: String,
    },
    
    /// Контакт онлайн
    ContactOnline {
        user_id: String,
        last_seen: Option<u64>,
    },
    
    /// Контакт офлайн
    ContactOffline {
        user_id: String,
    },
    
    // -------------------------------------------------------------------------
    // Уведомления
    // -------------------------------------------------------------------------
    /// Push уведомление
    PushNotification {
        title: String,
        body: String,
        data: serde_json::Value,
    },
    
    /// Системное уведомление
    Notification {
        level: NotificationLevel,
        message: String,
    },
    
    // -------------------------------------------------------------------------
    // Утилиты
    // -------------------------------------------------------------------------
    /// Ответ на Ping
    Pong {
        timestamp: u64,
        latency_ms: u64,
    },
    
    /// Ядро остановлено
    ShutdownComplete,
}

/// Состояние соединения
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error,
}

/// Состояние звонка
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CallState {
    Dialing,
    Ringing,
    Connected,
    OnHold,
    Ended,
    Error,
}

/// Причина завершения звонка
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallEndReason {
    LocalHangup,
    RemoteHangup,
    Busy,
    Timeout,
    Error(String),
    NetworkLost,
}

/// Уровень уведомления
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

// ============================================================================
// EVENT TRANSMITTER
// ============================================================================

/// Трансмиттер событий (Core → UI)
#[derive(Clone)]
pub struct LrEventTx {
    tx: mpsc::Sender<LrEvent>,
}

impl LrEventTx {
    pub fn new(tx: mpsc::Sender<LrEvent>) -> Self {
        Self { tx }
    }
    
    /// Отправить событие
    pub async fn send(&self, event: LrEvent) -> Result<(), mpsc::error::SendError<LrEvent>> {
        self.tx.send(event).await
    }
    
    /// Отправить событие без ожидания
    pub fn send_nowait(&self, event: LrEvent) -> Result<(), mpsc::error::TrySendError<LrEvent>> {
        self.tx.try_send(event)
    }
    
    /// Создать новый клон трансмиттера
    pub fn clone(&self) -> Self {
        Self { tx: self.tx.clone() }
    }
}

impl fmt::Debug for LrEventTx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LrEventTx").finish()
    }
}

// ============================================================================
// COMMAND RECEIVER
// ============================================================================

/// Ресивер команд (UI → Core)
pub struct LrCommandRx {
    rx: mpsc::Receiver<LrCommand>,
}

impl LrCommandRx {
    pub fn new(rx: mpsc::Receiver<LrCommand>) -> Self {
        Self { rx }
    }
    
    /// Получить следующую команду
    pub async fn recv(&mut self) -> Option<LrCommand> {
        self.rx.recv().await
    }
}

// ============================================================================
// CHANNEL FACTORY
// ============================================================================

/// Создать канал для команд и событий
pub fn create_channels(buffer_size: usize) -> (LrCommandRx, LrEventTx) {
    let (_cmd_tx, cmd_rx) = mpsc::channel(buffer_size);
    let (event_tx, _event_rx) = mpsc::channel(buffer_size);
    
    (LrCommandRx::new(cmd_rx), LrEventTx::new(event_tx))
}

/// Создать канал с буфером по умолчанию
pub fn create_default_channels() -> (LrCommandRx, LrEventTx) {
    create_channels(100)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_serialization() {
        let cmd = LrCommand::SendMessage {
            recipient_id: "user123".to_string(),
            content: MessageContent::Text("Hello".to_string()),
            encryption: EncryptionAlgorithm::Aes256Gcm,
            use_steganography: false,
            stego_image_path: None,
        };
        
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("SendMessage"));
        assert!(json.contains("user123"));
    }
    
    #[test]
    fn test_event_serialization() {
        let event = LrEvent::MessageReceived {
            message_id: "msg456".to_string(),
            chat_id: "chat789".to_string(),
            sender_id: "user123".to_string(),
            content: MessageContent::Text("Hello".to_string()),
            timestamp: 1234567890,
            encrypted: true,
        };
        
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("MessageReceived"));
        assert!(json.contains("msg456"));
    }
}
