// messenger/src/chat/group.rs
//! Групповые чаты

use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMessage {
    pub id: String,
    pub from: String,
    pub group_id: String,
    pub text: String,
    pub timestamp: DateTime<Utc>,
    pub translated_text: Option<String>,
    pub reply_to: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GroupChat {
    pub id: String,
    pub name: String,
    pub description: String,
    pub members: HashSet<String>,
    pub admins: HashSet<String>,
    pub messages: Vec<GroupMessage>,
    pub created_at: DateTime<Utc>,
    pub max_members: usize,
}

impl GroupChat {
    pub fn new(name: String, creator: String, max_members: usize) -> Self {
        let mut members = HashSet::new();
        let mut admins = HashSet::new();
        members.insert(creator.clone());
        admins.insert(creator);
        
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: String::new(),
            members,
            admins,
            messages: Vec::new(),
            created_at: Utc::now(),
            max_members,
        }
    }
    
    pub fn add_member(&mut self, user_id: String) -> Result<(), String> {
        if self.members.len() >= self.max_members {
            return Err("Группа переполнена".to_string());
        }
        
        self.members.insert(user_id);
        Ok(())
    }
    
    pub fn remove_member(&mut self, user_id: &str) -> Result<(), String> {
        if !self.admins.iter().any(|a| a == user_id) {
            return Err("Только администраторы могут удалять участников".to_string());
        }
        
        self.members.remove(user_id);
        Ok(())
    }
    
    pub fn send_message(&mut self, from: String, text: String) -> Option<&GroupMessage> {
        if !self.members.contains(&from) {
            return None; // Возвращаем None вместо panic
        }
        
        let message = GroupMessage {
            id: Uuid::new_v4().to_string(),
            from,
            group_id: self.id.clone(),
            text,
            timestamp: Utc::now(),
            translated_text: None,
            reply_to: None,
        };
        
        self.messages.push(message);
        self.messages.last()
    }
    
    pub fn get_member_count(&self) -> usize {
        self.members.len()
    }
    
    pub fn is_admin(&self, user_id: &str) -> bool {
        self.admins.contains(user_id)
    }
}
