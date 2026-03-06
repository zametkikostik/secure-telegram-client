// messenger/src/chat/private.rs
//! 1-на-1 чаты

use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateMessage {
    pub id: String,
    pub from: String,
    pub to: String,
    pub text: String,
    pub timestamp: DateTime<Utc>,
    pub translated_text: Option<String>,
    pub is_read: bool,
}

#[derive(Debug, Clone)]
pub struct PrivateChat {
    pub id: String,
    pub participant1: String,
    pub participant2: String,
    pub messages: Vec<PrivateMessage>,
    pub last_message_at: Option<DateTime<Utc>>,
}

impl PrivateChat {
    pub fn new(participant1: String, participant2: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            participant1,
            participant2,
            messages: Vec::new(),
            last_message_at: None,
        }
    }
    
    pub fn send_message(&mut self, from: String, text: String) -> &PrivateMessage {
        let message = PrivateMessage {
            id: Uuid::new_v4().to_string(),
            from,
            to: if from == self.participant1 {
                self.participant2.clone()
            } else {
                self.participant1.clone()
            },
            text,
            timestamp: Utc::now(),
            translated_text: None,
            is_read: false,
        };
        
        self.last_message_at = Some(message.timestamp);
        self.messages.push(message);
        self.messages.last().unwrap()
    }
    
    pub fn mark_as_read(&mut self, message_id: &str) {
        if let Some(msg) = self.messages.iter_mut().find(|m| m.id == message_id) {
            msg.is_read = true;
        }
    }
    
    pub fn get_unread_count(&self, for_user: &str) -> usize {
        self.messages
            .iter()
            .filter(|m| m.to == for_user && !m.is_read)
            .count()
    }
}
