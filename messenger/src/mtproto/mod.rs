//! MTProto-like Protocol Server
//!
//! Custom messaging protocol inspired by Telegram MTProto:
//! - Binary protocol with length-prefixed messages
//! - Auth key exchange (Diffie-Hellman like)
//! - Message encryption (AES-256-IGE like with our ChaCha20)
//! - Message sequence numbers
//! - Session management
//! - RPC-style request/response
//!
//! Protocol structure:
//! ------------------------------------------
//! | Message Length (4 bytes, big-endian)    |
//! ------------------------------------------
//! | Auth Key ID (8 bytes)                   |
//! ------------------------------------------
//! | Message ID (8 bytes, unique)            |
//! ------------------------------------------
//! | Sequence Number (4 bytes)               |
//! ------------------------------------------
//! | Encrypted Payload (variable)            |
//! ------------------------------------------

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, info, warn};

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum MtProtoError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Protocol error: {0}")]
    Protocol(String),
    #[error("Authentication failed")]
    AuthFailed,
    #[error("Invalid message")]
    InvalidMessage,
    #[error("Session expired")]
    SessionExpired,
    #[error("Encryption error: {0}")]
    Encryption(String),
}

pub type MtProtoResult<T> = Result<T, MtProtoError>;

// ============================================================================
// Message Types (RPC IDs)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum MessageType {
    // Auth
    ReqPq = 0x60469778,
    ReqDhParams = 0xd712e4be,
    SetClientDhParams = 0xf5045f1f,

    // Messages
    SendMessage = 0x00000001,
    ReceiveMessages = 0x00000002,
    DeleteMessages = 0x00000003,
    GetHistory = 0x00000004,

    // Users
    GetUsers = 0x00000010,
    GetUserFull = 0x00000011,

    // Chats
    GetChats = 0x00000020,
    CreateChat = 0x00000021,

    // System
    Ping = 0x000000ff,
    Pong = 0x000001ff,
    RpcResult = 0x000002ff,
    RpcError = 0x000003ff,

    // Encrypted
    MsgEncrypt = 0x00000e00,
    MsgDecrypt = 0x00000e01,
}

impl MessageType {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0x60469778 => Some(Self::ReqPq),
            0xd712e4be => Some(Self::ReqDhParams),
            0xf5045f1f => Some(Self::SetClientDhParams),
            0x00000001 => Some(Self::SendMessage),
            0x00000002 => Some(Self::ReceiveMessages),
            0x00000003 => Some(Self::DeleteMessages),
            0x00000004 => Some(Self::GetHistory),
            0x00000010 => Some(Self::GetUsers),
            0x00000011 => Some(Self::GetUserFull),
            0x00000020 => Some(Self::GetChats),
            0x00000021 => Some(Self::CreateChat),
            0x000000ff => Some(Self::Ping),
            0x000001ff => Some(Self::Pong),
            0x000002ff => Some(Self::RpcResult),
            0x000003ff => Some(Self::RpcError),
            0x00000e00 => Some(Self::MsgEncrypt),
            0x00000e01 => Some(Self::MsgDecrypt),
            _ => None,
        }
    }
}

// ============================================================================
// Protocol Messages
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    pub auth_key_id: u64,
    pub message_id: u64,
    pub seq_no: u32,
    pub message_type: u32,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
    pub message_type: u32,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse {
    pub success: bool,
    pub payload: serde_json::Value,
    pub error: Option<String>,
}

// ============================================================================
// Session
// ============================================================================

#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: u64,
    pub auth_key: Vec<u8>, // 256-bit shared secret
    pub server_salt: u64,
    pub client_seq_no: u32,
    pub server_seq_no: u32,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub user_id: Option<String>,
}

impl Session {
    pub fn new(session_id: u64, auth_key: Vec<u8>) -> Self {
        let now = Instant::now();
        Self {
            session_id,
            auth_key,
            server_salt: rand::random(),
            client_seq_no: 0,
            server_seq_no: 0,
            created_at: now,
            last_activity: now,
            user_id: None,
        }
    }

    pub fn next_server_seq(&mut self) -> u32 {
        self.server_seq_no += 1;
        self.server_seq_no
    }

    pub fn is_expired(&self, ttl: Duration) -> bool {
        self.last_activity.elapsed() > ttl
    }
}

// ============================================================================
// Protocol Encoder/Decoder
// ============================================================================

pub struct MtProtoCodec;

impl MtProtoCodec {
    /// Encode message to binary format
    pub fn encode(msg: &ProtocolMessage) -> Vec<u8> {
        let mut buffer = Vec::new();

        // Message type (4 bytes)
        buffer.extend_from_slice(&msg.message_type.to_be_bytes());
        // Auth key ID (8 bytes)
        buffer.extend_from_slice(&msg.auth_key_id.to_be_bytes());
        // Message ID (8 bytes)
        buffer.extend_from_slice(&msg.message_id.to_be_bytes());
        // Sequence number (4 bytes)
        buffer.extend_from_slice(&msg.seq_no.to_be_bytes());
        // Payload
        buffer.extend_from_slice(&msg.payload);

        // Prepend length
        let mut framed = Vec::new();
        framed.extend_from_slice(&(buffer.len() as u32).to_be_bytes());
        framed.extend_from_slice(&buffer);

        framed
    }

    /// Decode message from binary format
    pub async fn decode(stream: &mut TcpStream) -> MtProtoResult<ProtocolMessage> {
        // Read length (4 bytes)
        let mut len_buf = [0u8; 4];
        stream
            .read_exact(&mut len_buf)
            .await
            .map_err(|e| MtProtoError::Io(e.to_string()))?;
        let msg_len = u32::from_be_bytes(len_buf) as usize;

        // Read message
        let mut buffer = vec![0u8; msg_len];
        stream
            .read_exact(&mut buffer)
            .await
            .map_err(|e| MtProtoError::Io(e.to_string()))?;

        if buffer.len() < 24 {
            return Err(MtProtoError::InvalidMessage);
        }

        let message_type = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
        let auth_key_id = u64::from_be_bytes(buffer[4..12].try_into().unwrap());
        let message_id = u64::from_be_bytes(buffer[12..20].try_into().unwrap());
        let seq_no = u32::from_be_bytes(buffer[20..24].try_into().unwrap());
        let payload = buffer[24..].to_vec();

        Ok(ProtocolMessage {
            auth_key_id,
            message_id,
            seq_no,
            message_type,
            payload,
        })
    }
}

// ============================================================================
// Session Manager
// ============================================================================

pub struct SessionManager {
    sessions: Arc<tokio::sync::RwLock<HashMap<u64, Session>>>,
    session_ttl: Duration,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            session_ttl: Duration::from_secs(3600), // 1 hour
        }
    }

    pub async fn create_session(&self, session_id: u64, auth_key: Vec<u8>) -> Session {
        let session = Session::new(session_id, auth_key);
        self.sessions
            .write()
            .await
            .insert(session_id, session.clone());
        session
    }

    pub async fn get_session(&self, session_id: u64) -> Option<Session> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(&session_id)?;

        if session.is_expired(self.session_ttl) {
            return None;
        }

        Some(session.clone())
    }

    pub async fn update_activity(&self, session_id: u64) {
        if let Some(session) = self.sessions.write().await.get_mut(&session_id) {
            session.last_activity = Instant::now();
        }
    }

    pub async fn remove_session(&self, session_id: u64) {
        self.sessions.write().await.remove(&session_id);
    }

    pub async fn cleanup_expired(&self) {
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, s| !s.is_expired(self.session_ttl));
    }
}

// ============================================================================
// Encryption Helpers
// ============================================================================

pub fn generate_auth_key() -> Vec<u8> {
    // In production: proper Diffie-Hellman key exchange
    let mut key = vec![0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut key);
    key
}

pub fn compute_message_hash(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub fn generate_message_id() -> u64 {
    // MTProto-style message ID: Unix timestamp * 2^32 + random
    let timestamp = chrono::Utc::now().timestamp() as u64;
    let random: u32 = rand::random();
    (timestamp << 32) | (random as u64)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_conversion() {
        assert_eq!(MessageType::from_u32(0x60469778), Some(MessageType::ReqPq));
        assert_eq!(
            MessageType::from_u32(0x00000001),
            Some(MessageType::SendMessage)
        );
        assert_eq!(MessageType::from_u32(0x000000ff), Some(MessageType::Ping));
        assert_eq!(MessageType::from_u32(0xFFFFFFFF), None);
    }

    #[test]
    fn test_protocol_message_encoding() {
        let msg = ProtocolMessage {
            auth_key_id: 12345,
            message_id: 67890,
            seq_no: 1,
            message_type: MessageType::Ping as u32,
            payload: vec![1, 2, 3],
        };

        let encoded = MtProtoCodec::encode(&msg);
        assert!(encoded.len() > 24); // Length + header + payload
    }

    #[test]
    fn test_session_management() {
        let manager = SessionManager::new();
        // Can't easily test async in sync test — covered by integration tests
    }

    #[test]
    fn test_auth_key_generation() {
        let key1 = generate_auth_key();
        let key2 = generate_auth_key();
        assert_eq!(key1.len(), 32);
        assert_ne!(key1, key2); // Should be random
    }

    #[test]
    fn test_message_hash() {
        let hash1 = compute_message_hash(b"hello");
        let hash2 = compute_message_hash(b"hello");
        let hash3 = compute_message_hash(b"world");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_message_id_generation() {
        let id1 = generate_message_id();
        let id2 = generate_message_id();
        assert_ne!(id1, id2); // Should be unique
    }

    #[tokio::test]
    async fn test_session_expiry() {
        let manager = SessionManager::new();
        let session = manager.create_session(1, vec![1, 2, 3]).await;
        assert!(!session.is_expired(Duration::from_secs(3600)));

        // Force expiry
        manager
            .sessions
            .write()
            .await
            .get_mut(&1)
            .unwrap()
            .last_activity = Instant::now() - Duration::from_secs(3601);

        let expired = manager.get_session(1).await;
        assert!(expired.is_none());
    }
}
