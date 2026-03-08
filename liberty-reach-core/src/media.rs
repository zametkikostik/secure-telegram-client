//! Media Module
//! 
//! WebRTC для аудио/видео звонков с поддержкой Opus 6-32 kbps.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, info, warn, error};

/// Тип звонка
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CallType {
    /// Аудио звонок
    Audio,
    /// Видео звонок
    Video,
}

/// WebRTC менеджер
pub struct WebRtcManager {
    event_tx: mpsc::Sender<WebRtcEvent>,
}

/// Команды для WebRTC менеджера
#[derive(Debug, Clone)]
pub enum WebRtcCommand {
    CreateOffer { call_id: String },
    SetLocalDescription { call_id: String, sdp: String, sdp_type: String },
    SetRemoteDescription { call_id: String, sdp: String, sdp_type: String },
    AddIceCandidate { call_id: String, candidate: String, sdp_m_line_index: u16, sdp_mid: String },
    AcceptCall { call_id: String },
    RejectCall { call_id: String },
    EndCall { call_id: String },
    Shutdown,
}

/// События от WebRTC менеджера
#[derive(Debug, Clone)]
pub enum WebRtcEvent {
    OfferCreated { call_id: String, sdp: String },
    AnswerCreated { call_id: String, sdp: String },
    IceCandidateGenerated { call_id: String, candidate: String, sdp_m_line_index: u16, sdp_mid: String },
    CallAccepted { call_id: String },
    CallEnded { call_id: String },
    Error { call_id: String, error: String },
}

impl WebRtcManager {
    /// Создать новый WebRTC менеджер
    pub async fn new() -> Result<(Self, mpsc::Sender<WebRtcCommand>, mpsc::Receiver<WebRtcEvent>)> {
        let (cmd_tx, mut cmd_rx) = mpsc::channel(100);
        let (event_tx, event_rx) = mpsc::channel(100);

        let manager = Self { event_tx: event_tx.clone() };

        // Запускаем цикл обработки команд
        tokio::spawn(async move {
            while let Some(cmd) = cmd_rx.recv().await {
                match cmd {
                    WebRtcCommand::CreateOffer { call_id } => {
                        // TODO: Реализовать создание SDP offer
                        info!("Creating offer for call: {}", call_id);
                    }
                    WebRtcCommand::SetLocalDescription { call_id, sdp, sdp_type } => {
                        info!("Setting local description for call {}: {} ({})", call_id, sdp_type, sdp.len());
                    }
                    WebRtcCommand::SetRemoteDescription { call_id, sdp, sdp_type } => {
                        info!("Setting remote description for call {}: {} ({})", call_id, sdp_type, sdp.len());
                    }
                    WebRtcCommand::AddIceCandidate { call_id, candidate, .. } => {
                        debug!("Adding ICE candidate for call {}: {}", call_id, candidate);
                    }
                    WebRtcCommand::AcceptCall { call_id } => {
                        info!("Accepting call: {}", call_id);
                    }
                    WebRtcCommand::RejectCall { call_id } => {
                        info!("Rejecting call: {}", call_id);
                    }
                    WebRtcCommand::EndCall { call_id } => {
                        info!("Ending call: {}", call_id);
                    }
                    WebRtcCommand::Shutdown => {
                        info!("WebRTC manager shutdown");
                        break;
                    }
                }
            }
        });

        Ok((manager, cmd_tx, event_rx))
    }

    /// Создать WebRTC менеджер по умолчанию
    pub fn default() -> Self {
        Self { event_tx: mpsc::channel(100).0 }
    }
}
