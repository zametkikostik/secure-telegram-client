use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade, Message},
        State,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, Mutex};
use tracing::{info, debug};
use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    Auth { token: String },
    MessageReceived { id: String, chat_id: String, sender_id: String, sender_name: Option<String>, content: String, msg_type: String, created_at: String },
    MessageEdited { id: String, chat_id: String, sender_id: String, content: String, edited_at: Option<String> },
    MessageDeleted { id: String, chat_id: String },
    SubscribeChat { chat_id: String },
    PresenceUpdate { user_id: String, online: bool },
    TypingUpdate { chat_id: String, user_id: String, username: String, is_typing: bool },
    // P2P Signaling for WebRTC
    P2POffer { target_user_id: String, call_id: String, sdp: String, candidates: Vec<IceCandidate> },
    P2PAnswer { target_user_id: String, call_id: String, sdp: String, candidates: Vec<IceCandidate> },
    IceCandidate { target_user_id: String, call_id: String, candidate: IceCandidate },
    // Hangup notification
    P2PHangup { target_user_id: String, call_id: String, reason: String },
    // Ping/Pong
    Ping,
    Pong,
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceCandidate {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_m_line_index: Option<u16>,
}

pub struct WsManager {
    connections: Arc<RwLock<HashMap<String, Vec<mpsc::UnboundedSender<Message>>>>>,
    online_users: Arc<RwLock<HashMap<String, bool>>>,
}

impl WsManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            online_users: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn send_to_user(&self, user_id: &str, ws_msg: WsMessage) {
        let conns = self.connections.read().await;
        if let Some(tx_list) = conns.get(user_id) {
            if let Ok(msg) = serde_json::to_string(&ws_msg) {
                for tx in tx_list {
                    let _ = tx.send(Message::Text(msg.clone()));
                }
            }
        }
    }

    pub async fn add_connection(&self, user_id: &str, tx: mpsc::UnboundedSender<Message>) {
        self.connections.write().await.entry(user_id.to_string()).or_default().push(tx);
        self.online_users.write().await.insert(user_id.to_string(), true);
    }

    pub async fn remove_connection(&self, user_id: &str) {
        let mut conns = self.connections.write().await;
        if let Some(tx_list) = conns.get_mut(user_id) {
            tx_list.retain(|tx| !tx.is_closed());
            if tx_list.is_empty() {
                conns.remove(user_id);
                self.online_users.write().await.remove(user_id);
            }
        }
    }

    pub async fn is_online(&self, user_id: &str) -> bool {
        self.online_users.read().await.get(user_id).copied().unwrap_or(false)
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let ws_manager = state.ws_manager.clone();
    let auth = state.auth.clone();
    ws.on_upgrade(move |socket| handle_ws(socket, auth, ws_manager))
}

async fn handle_ws(
    socket: WebSocket,
    auth: Arc<crate::middleware::auth::AuthState>,
    ws_manager: Arc<WsManager>,
) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    let user_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    // Forward task
    let user_id_clone = user_id.clone();
    let ws_manager_clone = ws_manager.clone();
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
        // Cleanup on disconnect - broadcast offline
        if let Some(uid) = user_id_clone.lock().await.clone() {
            let presence = WsMessage::PresenceUpdate { user_id: uid.clone(), online: false };
            let conns = ws_manager_clone.connections.read().await;
            for other_uid in conns.keys() {
                ws_manager_clone.send_to_user(other_uid, presence.clone()).await;
            }
            ws_manager_clone.remove_connection(&uid).await;
            info!("WS disconnected: {}", uid);
        }
    });

    // Receive task
    let tx_clone = tx.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                    match ws_msg {
                        WsMessage::Auth { token } => {
                            if let Ok(claims) = auth.validate_token(&token) {
                                let uid = claims.sub.clone();
                                *user_id.lock().await = Some(uid.clone());
                                let _ = ws_manager.add_connection(&uid, tx_clone.clone()).await;
                                info!("WS authenticated: {}", uid);

                                // Broadcast online status to all users
                                let presence = WsMessage::PresenceUpdate { user_id: uid.clone(), online: true };
                                let conns = ws_manager.connections.read().await;
                                for other_uid in conns.keys() {
                                    if other_uid != &uid {
                                        ws_manager.send_to_user(other_uid, presence.clone()).await;
                                    }
                                }
                            }
                        }
                        WsMessage::PresenceUpdate { user_id: target, online } => {
                            // Forward presence to target user
                            if let Some(sender_id) = user_id.lock().await.clone() {
                                let presence = WsMessage::PresenceUpdate { user_id: sender_id, online };
                                ws_manager.send_to_user(&target, presence).await;
                            }
                        }
                        WsMessage::SubscribeChat { chat_id } => {
                            debug!("Subscribe: {}", chat_id);
                        }
                        // P2P Signaling - forward to target user
                        WsMessage::P2POffer { target_user_id, call_id, sdp, candidates } => {
                            if let Some(sender_id) = user_id.lock().await.clone() {
                                info!("P2P Offer: {} -> {}", sender_id, target_user_id);
                                let forward_msg = WsMessage::P2POffer {
                                    target_user_id: sender_id,
                                    call_id: call_id.clone(),
                                    sdp: sdp.clone(),
                                    candidates: candidates.clone(),
                                };
                                ws_manager.send_to_user(&target_user_id, forward_msg).await;
                            }
                        }
                        WsMessage::P2PAnswer { target_user_id, call_id, sdp, candidates } => {
                            if let Some(sender_id) = user_id.lock().await.clone() {
                                info!("P2P Answer: {} -> {}", sender_id, target_user_id);
                                let forward_msg = WsMessage::P2PAnswer {
                                    target_user_id: sender_id,
                                    call_id: call_id.clone(),
                                    sdp: sdp.clone(),
                                    candidates: candidates.clone(),
                                };
                                ws_manager.send_to_user(&target_user_id, forward_msg).await;
                            }
                        }
                        WsMessage::IceCandidate { target_user_id, call_id, candidate } => {
                            if let Some(sender_id) = user_id.lock().await.clone() {
                                debug!("ICE Candidate: {} -> {}", sender_id, target_user_id);
                                let forward_msg = WsMessage::IceCandidate {
                                    target_user_id: sender_id,
                                    call_id: call_id.clone(),
                                    candidate: candidate.clone(),
                                };
                                ws_manager.send_to_user(&target_user_id, forward_msg).await;
                            }
                        }
                        WsMessage::P2PHangup { target_user_id, call_id, reason } => {
                            if let Some(sender_id) = user_id.lock().await.clone() {
                                info!("P2P Hangup: {} -> {} (reason: {})", sender_id, target_user_id, reason);
                                let forward_msg = WsMessage::P2PHangup {
                                    target_user_id: sender_id,
                                    call_id: call_id.clone(),
                                    reason: reason.clone(),
                                };
                                ws_manager.send_to_user(&target_user_id, forward_msg).await;
                            }
                        }
                        WsMessage::Ping => {
                            let _ = tx_clone.send(Message::Text(
                                serde_json::to_string(&WsMessage::Pong).unwrap_or_default()
                            ));
                        }
                        _ => debug!("WS msg: {:?}", ws_msg),
                    }
                }
            } else if let Message::Close(_) = msg {
                break;
            }
        }
    });

    tokio::select! {
        _ = send_task => info!("WS send done"),
        _ = recv_task => info!("WS recv done"),
    }
}
