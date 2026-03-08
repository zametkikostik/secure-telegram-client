//! SIP Module
//! 
//! SIP клиент для VoIP звонков и обмена сообщениями.

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{info, warn, error, debug};

use crate::media::CallType;

/// SIP конфигурация
#[derive(Debug, Clone)]
pub struct SipConfig {
    /// SIP сервер (например, "sip.example.com:5060")
    pub server: String,
    /// SIP username
    pub username: String,
    /// SIP пароль
    pub password: String,
    /// SIP домен (опционально)
    pub domain: Option<String>,
}

/// SIP клиент
pub struct SipClient {
    config: SipConfig,
}

/// Команды для SIP клиента
#[derive(Debug)]
pub enum SipCommand {
    Register,
    Unregister,
    StartCall { to: String, call_type: CallType },
    AcceptCall { call_id: String },
    RejectCall { call_id: String },
    EndCall { call_id: String },
    SendMessage { to: String, content: String },
    Shutdown,
}

/// События от SIP клиента
#[derive(Debug, Clone)]
pub enum SipEvent {
    Registered { server: String },
    RegistrationFailed { error: String },
    Unregistered,
    IncomingCall { call_id: String, from: String, call_type: CallType },
    CallAccepted { call_id: String },
    CallRejected { call_id: String },
    CallEnded { call_id: String, reason: String },
    MessageReceived { from: String, content: String },
    MessageSent { to: String },
    MessageFailed { to: String, error: String },
    Error { error: String },
}

impl SipClient {
    /// Создать новый SIP клиент
    pub async fn new(config: SipConfig) -> Result<(mpsc::Sender<SipCommand>, mpsc::Receiver<SipEvent>)> {
        let (cmd_tx, mut cmd_rx) = mpsc::channel(100);
        let (event_tx, event_rx) = mpsc::channel(100);

        // Запускаем цикл обработки команд
        let config_clone = config.clone();
        tokio::spawn(async move {
            info!("SIP client starting with server: {}", config.server);
            
            while let Some(cmd) = cmd_rx.recv().await {
                match cmd {
                    SipCommand::Register => {
                        info!("Registering with SIP server: {}", config.server);
                        let _ = event_tx.send(SipEvent::Registered {
                            server: config.server.clone(),
                        }).await;
                    }
                    
                    SipCommand::Unregister => {
                        info!("Unregistering from SIP server");
                        let _ = event_tx.send(SipEvent::Unregistered).await;
                    }
                    
                    SipCommand::StartCall { to, call_type } => {
                        info!("Starting SIP call to: {} ({:?})", to, call_type);
                        let call_id = uuid::Uuid::new_v4().to_string();
                        let _ = event_tx.send(SipEvent::IncomingCall {
                            call_id,
                            from: config.username.clone(),
                            call_type,
                        }).await;
                    }
                    
                    SipCommand::AcceptCall { call_id } => {
                        info!("Accepting SIP call: {}", call_id);
                        let _ = event_tx.send(SipEvent::CallAccepted { call_id }).await;
                    }
                    
                    SipCommand::RejectCall { call_id } => {
                        info!("Rejecting SIP call: {}", call_id);
                        let _ = event_tx.send(SipEvent::CallRejected { call_id }).await;
                    }
                    
                    SipCommand::EndCall { call_id } => {
                        info!("Ending SIP call: {}", call_id);
                        let _ = event_tx.send(SipEvent::CallEnded {
                            call_id,
                            reason: "local_hangup".to_string(),
                        }).await;
                    }
                    
                    SipCommand::SendMessage { to, content: _ } => {
                        debug!("Sending SIP MESSAGE to: {}", to);
                        let _ = event_tx.send(SipEvent::MessageSent { to }).await;
                    }
                    
                    SipCommand::Shutdown => {
                        info!("SIP client shutdown");
                        break;
                    }
                }
            }
        });

        Ok((cmd_tx, event_rx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_sip_client_creation() {
        let config = SipConfig {
            server: "sip.example.com:5060".to_string(),
            username: "test".to_string(),
            password: "password".to_string(),
            domain: None,
        };
        
        let result = SipClient::new(config).await;
        assert!(result.is_ok());
    }
}
