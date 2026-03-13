//! AI-Governance Module - Автономная модерация
//!
//! Реализует:
//! - Проверка запросов через бесплатные AI модели
//! - Smart Quota: бесплатные модели для быстрой оценки
//! - Кэш проверенных адресов
//! - Временный бан-лист для спамеров

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tokio::sync::RwLock;
use std::sync::Arc;

/// Результат AI модерации
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModerationVerdict {
    /// Запрос безопасен
    Green,
    /// Подозрительный запрос (требуется проверка)
    Yellow,
    /// Спам/атака (блокировать)
    Red,
}

/// Запрос к AI для модерации
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModerationRequest {
    /// Peer ID отправителя
    pub peer_id: String,
    /// Текст запроса/сообщения
    pub content: String,
    /// Тип запроса
    pub request_type: ModerationType,
    /// Временная метка
    pub timestamp: u64,
}

/// Тип запроса на модерацию
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModerationType {
    /// AI запрос
    AIQuery,
    /// P2P сообщение
    P2PMessage,
    /// Web3 транзакция
    Web3Transaction,
    /// Файл/CID
    FileTransfer,
}

/// Кэш проверенных пиров
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedPeerCache {
    /// Peer ID
    pub peer_id: String,
    /// Количество успешных проверок
    pub trust_score: u32,
    /// Последняя проверка
    pub last_checked: u64,
    /// Истекает через (секунды)
    pub expires_in: u64,
}

/// Бан-лист пиров
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BannedPeer {
    /// Peer ID
    pub peer_id: String,
    /// Причина бана
    pub reason: String,
    /// Количество нарушений
    pub violation_count: u32,
    /// Забанен до (timestamp)
    pub banned_until: u64,
}

/// Конфигурация AI-Governance
pub struct GovernanceConfig {
    /// OpenRouter API ключ
    pub openrouter_api_key: Option<String>,
    /// Локальная Ollama модель
    pub ollama_model: String,
    /// Порог доверия (0.0 - 1.0)
    pub trust_threshold: f64,
    /// Время бана (секунды)
    pub ban_duration_secs: u64,
    /// Максимум нарушений перед перманентным баном
    pub max_violations: u32,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            openrouter_api_key: std::env::var("OPENROUTER_API_KEY").ok(),
            ollama_model: "qwen2.5-coder:3b".to_string(),
            trust_threshold: 0.7,
            ban_duration_secs: 3600, // 1 час
            max_violations: 5,
        }
    }
}

/// AI-Governance менеджер
pub struct GovernanceManager {
    config: GovernanceConfig,
    /// Кэш проверенных пиров
    trusted_cache: Arc<RwLock<HashMap<String, TrustedPeerCache>>>,
    /// Бан-лист
    ban_list: Arc<RwLock<HashMap<String, BannedPeer>>>,
    /// HTTP клиент
    client: reqwest::Client,
}

impl GovernanceManager {
    pub fn new(config: GovernanceConfig) -> Self {
        Self {
            config,
            trusted_cache: Arc::new(RwLock::new(HashMap::new())),
            ban_list: Arc::new(RwLock::new(HashMap::new())),
            client: reqwest::Client::new(),
        }
    }

    /// Проверка: забанен ли пир
    pub async fn is_banned(&self, peer_id: &str) -> bool {
        let ban_list = self.ban_list.read().await;
        
        if let Some(ban) = ban_list.get(peer_id) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if now < ban.banned_until {
                return true;
            } else {
                // Бан истек - нужно разбанить
                drop(ban_list);
                self.unban_peer(peer_id).await;
                return false;
            }
        }
        
        false
    }

    /// Добавление пира в бан-лист
    pub async fn ban_peer(&self, peer_id: &str, reason: &str) {
        let mut ban_list = self.ban_list.write().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let ban = if let Some(existing) = ban_list.get_mut(peer_id) {
            existing.violation_count += 1;
            existing.reason = reason.to_string();
            existing.banned_until = now + self.config.ban_duration_secs * (existing.violation_count as u64);
            existing.clone()
        } else {
            let ban = BannedPeer {
                peer_id: peer_id.to_string(),
                reason: reason.to_string(),
                violation_count: 1,
                banned_until: now + self.config.ban_duration_secs,
            };
            ban_list.insert(peer_id.to_string(), ban.clone());
            ban
        };

        tracing::warn!("🚫 Peer {} забанен: {} (нарушений: {}, до: {})", 
            peer_id, reason, ban.violation_count, ban.banned_until);
    }

    /// Разбан пира
    pub async fn unban_peer(&self, peer_id: &str) {
        let mut ban_list = self.ban_list.write().await;
        ban_list.remove(peer_id);
        tracing::info!("✅ Peer {} разбанен", peer_id);
    }

    /// Проверка запроса через AI-Governance
    pub async fn moderate_request(&self, request: &ModerationRequest) -> Result<ModerationVerdict> {
        // Проверка бана
        if self.is_banned(&request.peer_id).await {
            return Ok(ModerationVerdict::Red);
        }

        // Проверка кэша
        if let Some(verdict) = self.check_cache(&request.peer_id).await {
            return Ok(verdict);
        }

        // AI модерация через Smart Quota
        let verdict = self.smart_quota_moderation(request).await?;

        // Обновление кэша
        if verdict == ModerationVerdict::Green {
            self.update_cache(&request.peer_id, true).await;
        } else if verdict == ModerationVerdict::Red {
            self.ban_peer(&request.peer_id, &format!("AI Moderation: {:?}", request.request_type)).await;
        }

        Ok(verdict)
    }

    /// Smart Quota: быстрая проверка через бесплатные модели
    async fn smart_quota_moderation(&self, request: &ModerationRequest) -> Result<ModerationVerdict> {
        // Шаг 1: Быстрая проверка через бесплатную модель (OpenRouter free tier)
        if let Some(api_key) = &self.config.openrouter_api_key {
            match self.quick_moderation(request, api_key).await {
                Ok(ModerationVerdict::Red) => return Ok(ModerationVerdict::Red),
                Ok(ModerationVerdict::Green) => return Ok(ModerationVerdict::Green),
                Ok(ModerationVerdict::Yellow) | Err(_) => {
                    // Сомнения или ошибка → локальная проверка
                }
            }
        }

        // Шаг 2: Локальная Ollama для финального решения
        self.local_moderation(request).await
    }

    /// Быстрая модерация через OpenRouter (бесплатные модели)
    async fn quick_moderation(&self, request: &ModerationRequest, api_key: &str) -> Result<ModerationVerdict> {
        let prompt = format!(
            "You are a content moderator. Classify this request as SAFE, SUSPICIOUS, or DANGEROUS.\n\
             Request type: {:?}\n\
             Content: {}\n\
             Respond with only one word: SAFE, SUSPICIOUS, or DANGEROUS.",
            request.request_type,
            request.content
        );

        let response = self.client.post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": "meta-llama/llama-3-8b-instruct:free", // Бесплатная модель
                "messages": [{"role": "user", "content": prompt}],
                "max_tokens": 10,
            }))
            .send()
            .await
            .context("OpenRouter API error")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("OpenRouter API error: {}", response.status()));
        }

        let result: serde_json::Value = response.json().await?;
        let content = result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("SAFE");

        let verdict = match content.to_uppercase().trim() {
            "DANGEROUS" => ModerationVerdict::Red,
            "SUSPICIOUS" => ModerationVerdict::Yellow,
            _ => ModerationVerdict::Green,
        };

        Ok(verdict)
    }

    /// Локальная модерация через Ollama
    async fn local_moderation(&self, request: &ModerationRequest) -> Result<ModerationVerdict> {
        let prompt = format!(
            "Classify as SAFE, SUSPICIOUS, or DANGEROUS:\n\
             Type: {:?}\n\
             Content: {}\n\
             Response: one word only.",
            request.request_type,
            request.content
        );

        let response = self.client.post("http://localhost:11437/api/generate")
            .json(&serde_json::json!({
                "model": &self.config.ollama_model,
                "prompt": prompt,
                "stream": false,
            }))
            .send()
            .await
            .context("Ollama API error")?;

        let result: serde_json::Value = response.json().await?;
        let content = result["response"]
            .as_str()
            .unwrap_or("SAFE");

        let verdict = match content.to_uppercase().trim() {
            "DANGEROUS" => ModerationVerdict::Red,
            "SUSPICIOUS" => ModerationVerdict::Yellow,
            _ => ModerationVerdict::Green,
        };

        Ok(verdict)
    }

    /// Проверка кэша
    async fn check_cache(&self, peer_id: &str) -> Option<ModerationVerdict> {
        let cache = self.trusted_cache.read().await;
        
        if let Some(entry) = cache.get(peer_id) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if now < entry.last_checked + entry.expires_in {
                // Кэш валиден
                if entry.trust_score >= 10 {
                    return Some(ModerationVerdict::Green);
                }
            }
        }
        
        None
    }

    /// Обновление кэша
    async fn update_cache(&self, peer_id: &str, is_trusted: bool) {
        let mut cache = self.trusted_cache.write().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let entry = cache.entry(peer_id.to_string()).or_insert(TrustedPeerCache {
            peer_id: peer_id.to_string(),
            trust_score: 0,
            last_checked: now,
            expires_in: 3600, // 1 час
        });

        if is_trusted {
            entry.trust_score += 1;
        } else {
            entry.trust_score = entry.trust_score.saturating_sub(1);
        }

        entry.last_checked = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_governance_ban_unban() {
        let config = GovernanceConfig::default();
        let gov = GovernanceManager::new(config);

        // Бан
        gov.ban_peer("test_peer", "Spam").await;
        assert!(gov.is_banned("test_peer").await);

        // Разбан
        gov.unban_peer("test_peer").await;
        assert!(!gov.is_banned("test_peer").await);
    }

    #[tokio::test]
    async fn test_cache_update() {
        let config = GovernanceConfig::default();
        let gov = GovernanceManager::new(config);

        gov.update_cache("test_peer", true).await;
        gov.update_cache("test_peer", true).await;

        let cache = gov.trusted_cache.read().await;
        let entry = cache.get("test_peer").unwrap();
        assert_eq!(entry.trust_score, 2);
    }
}
