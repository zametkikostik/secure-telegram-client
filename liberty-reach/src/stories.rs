//! Модуль Stories (24-часовые истории)
//!
//! Реализует:
//! - Публикацию историй с медиа
//! - Хранение в IPFS (Pinata)
//! - Автоматическое удаление через 24 часа
//! - Просмотр историй контактов

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::collections::HashMap;
use libp2p::PeerId;
use chrono::{DateTime, Utc};

/// Тип медиа для истории
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StoryMediaType {
    Image,
    Video,
    Text,
    Audio,
}

/// История (Story)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Story {
    /// Уникальный ID истории
    pub id: String,
    /// Автор (Peer ID)
    pub author_peer_id: String,
    /// Тип медиа
    pub media_type: StoryMediaType,
    /// IPFS CID медиафайла
    pub media_cid: Option<String>,
    /// Текстовое содержимое (если есть)
    pub text_content: Option<String>,
    /// Timestamp создания
    pub created_at: u64,
    /// Timestamp истечения (created_at + 24h)
    pub expires_at: u64,
    /// Количество просмотров
    pub view_count: u32,
    /// Просмотревшие (Peer IDs)
    pub viewed_by: Vec<String>,
}

impl Story {
    const EXPIRY_SECONDS: u64 = 24 * 60 * 60; // 24 часа

    /// Создание новой истории
    pub fn new(
        author_peer_id: &str,
        media_type: StoryMediaType,
        media_cid: Option<String>,
        text_content: Option<String>,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let id = Self::generate_id(author_peer_id, now);

        Self {
            id,
            author_peer_id: author_peer_id.to_string(),
            media_type,
            media_cid,
            text_content,
            created_at: now,
            expires_at: now + Self::EXPIRY_SECONDS,
            view_count: 0,
            viewed_by: Vec::new(),
        }
    }

    /// Генерация уникального ID
    fn generate_id(peer_id: &str, timestamp: u64) -> String {
        format!("story_{}_{}", peer_id.chars().take(8).collect::<String>(), timestamp)
    }

    /// Проверка актуальности истории
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now >= self.expires_at
    }

    /// Оставшееся время в секундах
    pub fn time_remaining(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if now >= self.expires_at {
            0
        } else {
            self.expires_at - now
        }
    }

    /// Форматирование оставшегося времени
    pub fn format_time_remaining(&self) -> String {
        let remaining = self.time_remaining();
        
        if remaining < 60 {
            format!("{} сек", remaining)
        } else if remaining < 3600 {
            format!("{} мин", remaining / 60)
        } else if remaining < 86400 {
            format!("{} ч", remaining / 3600)
        } else {
            "Истекло".to_string()
        }
    }

    /// Отметка о просмотре
    pub fn mark_viewed(&mut self, viewer_peer_id: &str) {
        if !self.viewed_by.contains(&viewer_peer_id.to_string()) {
            self.viewed_by.push(viewer_peer_id.to_string());
            self.view_count += 1;
        }
    }

    /// Проверка, просмотрена ли история
    pub fn is_viewed_by(&self, viewer_peer_id: &str) -> bool {
        self.viewed_by.contains(&viewer_peer_id.to_string())
    }
}

/// Менеджер историй
pub struct StoryManager {
    /// Активные истории (peer_id -> Vec<Story>)
    stories: HashMap<String, Vec<Story>>,
    /// Кэш для быстрого доступа по ID
    story_cache: HashMap<String, String>, // story_id -> author_peer_id
}

impl StoryManager {
    pub fn new() -> Self {
        Self {
            stories: HashMap::new(),
            story_cache: HashMap::new(),
        }
    }

    /// Публикация новой истории
    pub fn publish_story(
        &mut self,
        author_peer_id: &str,
        media_type: StoryMediaType,
        media_cid: Option<String>,
        text_content: Option<String>,
    ) -> &Story {
        let story = Story::new(author_peer_id, media_type, media_cid, text_content);
        let story_id = story.id.clone();

        self.stories
            .entry(author_peer_id.to_string())
            .or_insert_with(Vec::new)
            .push(story);

        self.story_cache.insert(story_id, author_peer_id.to_string());

        self.cleanup_expired();

        self.stories.get(author_peer_id).unwrap().last().unwrap()
    }

    /// Получение историй пира
    pub fn get_peer_stories(&self, peer_id: &str) -> Vec<&Story> {
        self.stories
            .get(peer_id)
            .map(|stories| stories.iter().filter(|s| !s.is_expired()).collect())
            .unwrap_or_default()
    }

    /// Получение истории по ID
    pub fn get_story(&self, story_id: &str) -> Option<&Story> {
        if let Some(author_id) = self.story_cache.get(story_id) {
            if let Some(stories) = self.stories.get(author_id) {
                return stories.iter().find(|s| s.id == story_id);
            }
        }
        None
    }

    /// Получение истории по ID (mutable)
    pub fn get_story_mut(&mut self, story_id: &str) -> Option<&mut Story> {
        if let Some(author_id) = self.story_cache.get(story_id).cloned() {
            if let Some(stories) = self.stories.get_mut(&author_id) {
                return stories.iter_mut().find(|s| s.id == story_id);
            }
        }
        None
    }

    /// Отметка о просмотре
    pub fn mark_story_viewed(&mut self, story_id: &str, viewer_peer_id: &str) -> bool {
        if let Some(story) = self.get_story_mut(story_id) {
            if !story.is_expired() {
                story.mark_viewed(viewer_peer_id);
                return true;
            }
        }
        false
    }

    /// Очистка истекших историй
    pub fn cleanup_expired(&mut self) -> usize {
        let mut removed = 0;

        // Сбор ID историй для удаления
        let mut to_remove: Vec<(String, String)> = Vec::new();

        for (author_id, stories) in &self.stories {
            for story in stories {
                if story.is_expired() {
                    to_remove.push((author_id.clone(), story.id.clone()));
                }
            }
        }

        // Удаление
        for (author_id, story_id) in to_remove {
            if let Some(stories) = self.stories.get_mut(&author_id) {
                let initial_len = stories.len();
                stories.retain(|s| s.id != story_id);
                removed += initial_len - stories.len();
            }
            self.story_cache.remove(&story_id);
        }

        // Удаление пустых списков
        self.stories.retain(|_, stories| !stories.is_empty());

        removed
    }

    /// Получение всех активных историй от контактов
    pub fn get_all_active_stories(&self) -> Vec<&Story> {
        let mut all_stories: Vec<&Story> = Vec::new();

        for stories in self.stories.values() {
            for story in stories {
                if !story.is_expired() {
                    all_stories.push(story);
                }
            }
        }

        // Сортировка по времени создания (новые первые)
        all_stories.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        all_stories
    }

    /// Статистика
    pub fn get_stats(&self) -> StoryStats {
        let all_stories = self.get_all_active_stories();
        let total_views: u32 = all_stories.iter().map(|s| s.view_count).sum();

        StoryStats {
            active_stories: all_stories.len(),
            total_authors: self.stories.len(),
            total_views,
        }
    }
}

impl Default for StoryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Статистика историй
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryStats {
    pub active_stories: usize,
    pub total_authors: usize,
    pub total_views: u32,
}

/// Сообщение для передачи истории в P2P сети
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryAnnouncement {
    pub story_id: String,
    pub author_peer_id: String,
    pub media_type: StoryMediaType,
    pub media_cid: Option<String>,
    pub text_content: Option<String>,
    pub created_at: u64,
    pub expires_at: u64,
}

impl StoryAnnouncement {
    pub fn from_story(story: &Story) -> Self {
        Self {
            story_id: story.id.clone(),
            author_peer_id: story.author_peer_id.clone(),
            media_type: story.media_type.clone(),
            media_cid: story.media_cid.clone(),
            text_content: story.text_content.clone(),
            created_at: story.created_at,
            expires_at: story.expires_at,
        }
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .context("Ошибка сериализации StoryAnnouncement")
    }

    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .context("Ошибка десериализации StoryAnnouncement")
    }
}

/// Топик для анонсов историй в P2P сети
pub const STORY_ANNOUNCE_TOPIC: &str = "liberty-story-announce";

/// Форматирование времени для отображения
pub fn format_timestamp(timestamp: u64) -> String {
    let datetime = DateTime::<Utc>::from(
        UNIX_EPOCH + Duration::from_secs(timestamp)
    );
    datetime.format("%H:%M").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_story_creation() {
        let story = Story::new(
            "peer123",
            StoryMediaType::Image,
            Some("QmTest123".to_string()),
            Some("Привет!".to_string()),
        );

        assert_eq!(story.author_peer_id, "peer123");
        assert_eq!(story.media_type, StoryMediaType::Image);
        assert!(!story.is_expired());
        assert_eq!(story.view_count, 0);
    }

    #[test]
    fn test_story_expiry() {
        let mut story = Story::new(
            "peer123",
            StoryMediaType::Text,
            None,
            Some("Тест".to_string()),
        );

        // История не истекла
        assert!(!story.is_expired());
        assert!(story.time_remaining() > 0);

        // Искусственное истечение для теста
        story.expires_at = 0;
        assert!(story.is_expired());
        assert_eq!(story.time_remaining(), 0);
    }

    #[test]
    fn test_story_manager() {
        let mut manager = StoryManager::new();

        manager.publish_story(
            "peer123",
            StoryMediaType::Image,
            Some("QmTest".to_string()),
            None,
        );

        let stories = manager.get_peer_stories("peer123");
        assert_eq!(stories.len(), 1);

        let stats = manager.get_stats();
        assert_eq!(stats.active_stories, 1);
    }

    #[test]
    fn test_story_view() {
        let mut manager = StoryManager::new();

        manager.publish_story(
            "peer123",
            StoryMediaType::Text,
            None,
            Some("Тест".to_string()),
        );

        let stories = manager.get_peer_stories("peer123");
        let story = stories.first().unwrap();
        let story_id = story.id.clone();

        manager.mark_story_viewed(&story_id, "viewer456");

        let story = manager.get_story(&story_id).unwrap();
        assert_eq!(story.view_count, 1);
        assert!(story.is_viewed_by("viewer456"));
    }
}
