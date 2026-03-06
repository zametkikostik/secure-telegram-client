// messenger/src/webrtc/conference.rs
//! Конференции (бесплатно)

use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Participant {
    pub id: String,
    pub name: String,
    pub is_speaking: bool,
}

pub struct ConferenceRoom {
    pub id: String,
    pub name: String,
    pub participants: HashMap<String, Participant>,
    pub max_participants: usize,
}

impl ConferenceRoom {
    pub fn new(name: String, max_participants: usize) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            participants: HashMap::new(),
            max_participants,
        }
    }
    
    pub fn join(&mut self, participant: Participant) -> Result<(), String> {
        if self.participants.len() >= self.max_participants {
            return Err("Конференция переполнена".to_string());
        }
        
        self.participants.insert(participant.id.clone(), participant);
        Ok(())
    }
    
    pub fn leave(&mut self, participant_id: &str) {
        self.participants.remove(participant_id);
    }
    
    pub fn get_participants(&self) -> Vec<&Participant> {
        self.participants.values().collect()
    }
    
    pub fn set_speaking(&mut self, participant_id: &str, is_speaking: bool) {
        if let Some(participant) = self.participants.get_mut(participant_id) {
            participant.is_speaking = is_speaking;
        }
    }
}

pub struct ConferenceManager {
    rooms: HashMap<String, ConferenceRoom>,
}

impl ConferenceManager {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }
    
    pub fn create_room(&mut self, name: String, max_participants: usize) -> &ConferenceRoom {
        let room = ConferenceRoom::new(name, max_participants);
        let id = room.id.clone();
        self.rooms.insert(id, room);
        self.rooms.get(&id).unwrap()
    }
    
    pub fn get_room(&self, room_id: &str) -> Option<&ConferenceRoom> {
        self.rooms.get(room_id)
    }
    
    pub fn get_room_mut(&mut self, room_id: &str) -> Option<&mut ConferenceRoom> {
        self.rooms.get_mut(room_id)
    }
    
    pub fn delete_room(&mut self, room_id: &str) {
        self.rooms.remove(room_id);
    }
}
