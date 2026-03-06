// messenger/src/chat/channel.rs
//! Каналы (broadcast)

use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
    pub id: String,
    pub channel_id: String,
    pub text: String,
    pub media: Option<Vec<String>>,
    pub timestamp: DateTime<Utc>,
    pub translated_text: Option<String>,
    pub views: u64,
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub owner: String,
    pub admins: HashSet<String>,
    pub subscribers: HashSet<String>,
    pub messages: Vec<ChannelMessage>,
    pub created_at: DateTime<Utc>,
    pub is_public: bool,
    pub invite_link: Option<String>,
}

impl Channel {
    pub fn new(name: String, owner: String, is_public: bool) -> Self {
        let mut admins = HashSet::new();
        admins.insert(owner.clone());
        
        let invite_link = if is_public {
            Some(format!("https://libertyreach.io/channel/{}", Uuid::new_v4()))
        } else {
            None
        };
        
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: String::new(),
            owner,
            admins,
            subscribers: HashSet::new(),
            messages: Vec::new(),
            created_at: Utc::now(),
            is_public,
            invite_link,
        }
    }
    
    pub fn subscribe(&mut self, user_id: String) {
        self.subscribers.insert(user_id);
    }
    
    pub fn unsubscribe(&mut self, user_id: &str) {
        self.subscribers.remove(user_id);
    }
    
    pub fn post_message(&mut self, text: String, media: Option<Vec<String>>) -> &ChannelMessage {
        let message = ChannelMessage {
            id: Uuid::new_v4().to_string(),
            channel_id: self.id.clone(),
            text,
            media,
            timestamp: Utc::now(),
            translated_text: None,
            views: 0,
        };
        
        self.messages.push(message);
        self.messages.last().unwrap()
    }
    
    pub fn get_subscriber_count(&self) -> usize {
        self.subscribers.len()
    }
    
    pub fn is_admin(&self, user_id: &str) -> bool {
        self.admins.contains(user_id)
    }
    
    pub fn add_admin(&mut self, user_id: String) {
        self.admins.insert(user_id);
    }
    
    pub fn remove_admin(&mut self, user_id: &str) {
        self.admins.remove(user_id);
    }
}
