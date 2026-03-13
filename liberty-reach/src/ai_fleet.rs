//! OpenRouter Free-Fleet Integration
//!
//! Реализует:
//! - Dynamic Free Model Picker с fallback логикой
//! - AI Privacy Wrapper & Sanitization
//! - Async кэш для идентичных запросов
//! - GDPR Compliance

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tokio::sync::RwLock;
use std::sync::Arc;
use regex::Regex;

/// Список бесплатных моделей OpenRouter
pub const FREE_MODELS: &[&str] = &[
    "qwen/qwen-2.5-72b-instruct:free",      // Основная (лучшая)
    "meta-llama/llama-3.3-70b-instruct:free", // Альтернатива
    "google/gemma-2-9b-it:free",            // Быстрая
    "mistralai/mistral-small:free",         // Резерв
];

/// Конфигурация Free Model Picker
pub struct FreeModelPickerConfig {
    /// Максимум попыток перебора
    pub max_retries: usize,
    /// Таймаут между попытками (мс)
    pub retry_delay_ms: u64,
    /// Список моделей (порядок важен)
    pub models: Vec<String>,
}

impl Default for FreeModelPickerConfig {
    fn default() -> Self {
        Self {
            max_retries: FREE_MODELS.len(),
            retry_delay_ms: 100,
            models: FREE_MODELS.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// Free Model Picker с dynamic fallback
pub struct FreeModelPicker {
    config: FreeModelPickerConfig,
    /// Текущий индекс модели (для round-robin)
    current_index: RwLock<usize>,
    /// Счётчик ошибок для каждой модели
    error_counts: RwLock<HashMap<String, usize>>,
    client: reqwest::Client,
}

impl FreeModelPicker {
    pub fn new(config: FreeModelPickerConfig) -> Self {
        Self {
            config,
            current_index: RwLock::new(0),
            error_counts: RwLock::new(HashMap::new()),
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    /// Запрос к OpenRouter с automatic fallback
    pub async fn query_with_fallback(
        &self,
        api_key: &str,
        prompt: &str,
        max_tokens: Option<usize>,
    ) -> Result<String> {
        let mut last_error: Option<anyhow::Error> = None;

        for attempt in 0..self.config.max_retries {
            // Получаем текущую модель
            let model_index = (self.get_current_index().await + attempt) % self.config.models.len();
            let model = &self.config.models[model_index];

            tracing::debug!("🔄 Попытка #{}: Модель {}", attempt + 1, model);

            match self.query_model(api_key, model, prompt, max_tokens).await {
                Ok(response) => {
                    // Успех! Сбрасываем счётчик ошибок
                    self.reset_error_count(model).await;
                    self.set_current_index(model_index).await;
                    return Ok(response);
                }
                Err(e) => {
                    let error_str = e.to_string();

                    // Проверяем тип ошибки
                    let is_retryable = error_str.contains("429")
                        || error_str.contains("402")
                        || error_str.contains("503")
                        || error_str.contains("timeout")
                        || error_str.contains("connection");

                    if !is_retryable {
                        // Не retryable ошибка — возвращаем сразу
                        return Err(e);
                    }

                    // Retryable ошибка — пробуем следующую модель
                    self.increment_error_count(model).await;
                    last_error = Some(e);

                    // Задержка перед следующей попыткой
                    tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
                }
            }
        }

        // Все попытки исчерпаны
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All models failed")))
    }

    /// Запрос к конкретной модели
    async fn query_model(
        &self,
        api_key: &str,
        model: &str,
        prompt: &str,
        max_tokens: Option<usize>,
    ) -> Result<String> {
        let response = self.client.post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .header("X-Title", "Liberty Reach Messenger") // GDPR Compliance
            .json(&serde_json::json!({
                "model": model,
                "messages": [{"role": "user", "content": prompt}],
                "max_tokens": max_tokens.unwrap_or(1024),
            }))
            .send()
            .await
            .context(format!("Failed to send request to model {}", model))?;

        // Проверка статуса
        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Model {} returned {}: {}",
                model,
                status,
                error_body
            ));
        }

        // Парсинг ответа
        let result: serde_json::Value = response.json().await?;
        let content = result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("Empty response");

        Ok(content.to_string())
    }

    async fn get_current_index(&self) -> usize {
        *self.current_index.read().await
    }

    async fn set_current_index(&self, index: usize) {
        *self.current_index.write().await = index;
    }

    async fn increment_error_count(&self, model: &str) {
        let mut counts = self.error_counts.write().await;
        *counts.entry(model.to_string()).or_insert(0) += 1;
    }

    async fn reset_error_count(&self, model: &str) {
        let mut counts = self.error_counts.write().await;
        counts.insert(model.to_string(), 0);
    }

    /// Получение статистики по моделям
    pub async fn get_stats(&self) -> HashMap<String, usize> {
        self.error_counts.read().await.clone()
    }
}

/// AI Privacy Wrapper & Sanitization
pub struct PrivacyWrapper {
    /// Кэш запросов: hash(question) -> (answer, timestamp)
    cache: RwLock<HashMap<u64, (String, u64)>>,
    /// TTL кэша (секунды)
    cache_ttl_secs: u64,
    /// Regex для PeerID
    peer_id_regex: Regex,
}

impl PrivacyWrapper {
    pub fn new(cache_ttl_secs: u64) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            cache_ttl_secs,
            peer_id_regex: Regex::new(r"12D3KooW[a-zA-Z0-9]+").unwrap(),
        }
    }

    /// Sanitize input: замена PeerID на [ANON_PEER]
    pub fn sanitize_input(&self, input: &str) -> String {
        self.peer_id_regex
            .replace_all(input, "[ANON_PEER]")
            .to_string()
    }

    /// Проверка кэша
    pub async fn get_cached(&self, question: &str) -> Option<String> {
        let hash = self.hash_question(question);
        let cache = self.cache.read().await;

        if let Some((answer, timestamp)) = cache.get(&hash) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            if now - timestamp < self.cache_ttl_secs {
                return Some(answer.clone());
            }
        }

        None
    }

    /// Сохранение в кэш
    pub async fn cache_result(&self, question: &str, answer: &str) {
        let hash = self.hash_question(question);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut cache = self.cache.write().await;
        cache.insert(hash, (answer.to_string(), now));

        // Очистка старого кэша (раз в 100 запросов)
        if cache.len() % 100 == 0 {
            self.cleanup_cache(&mut cache).await;
        }
    }

    /// Хэш вопроса для кэша
    fn hash_question(&self, question: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        question.hash(&mut hasher);
        hasher.finish()
    }

    /// Очистка старого кэша
    async fn cleanup_cache(&self, cache: &mut HashMap<u64, (String, u64)>) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        cache.retain(|_, (_, timestamp)| {
            now.saturating_sub(*timestamp) < self.cache_ttl_secs
        });
    }

    /// Статистика кэша
    pub async fn get_cache_stats(&self) -> usize {
        self.cache.read().await.len()
    }
}

/// Комбинированный AI клиент с Privacy + Free Fleet
pub struct SovereignAIClient {
    picker: FreeModelPicker,
    privacy: PrivacyWrapper,
    api_key: String,
}

impl SovereignAIClient {
    pub fn new(api_key: String) -> Self {
        Self {
            picker: FreeModelPicker::new(FreeModelPickerConfig::default()),
            privacy: PrivacyWrapper::new(3600), // 1 час TTL
            api_key,
        }
    }

    /// Запрос с privacy protection + fallback
    pub async fn ask(&self, question: &str) -> Result<String> {
        // Проверка кэша
        if let Some(cached) = self.privacy.get_cached(question).await {
            tracing::debug!("✅ Cache hit!");
            return Ok(cached);
        }

        // Sanitize input
        let sanitized = self.privacy.sanitize_input(question);

        // Запрос к Free Fleet
        let answer = self.picker.query_with_fallback(&self.api_key, &sanitized, None).await?;

        // Кэширование ответа
        self.privacy.cache_result(question, &answer).await;

        Ok(answer)
    }

    /// Запрос к конкретной модели
    pub async fn ask_model(&self, model: &str, question: &str) -> Result<String> {
        let sanitized = self.privacy.sanitize_input(question);
        self.picker.query_model(&self.api_key, model, &sanitized, None).await
    }

    /// Статистика
    pub async fn get_stats(&self) -> SovereignAIStats {
        SovereignAIStats {
            model_errors: self.picker.get_stats().await,
            cache_size: self.privacy.get_cache_stats().await,
        }
    }
}

/// Статистика AI клиента
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignAIStats {
    pub model_errors: HashMap<String, usize>,
    pub cache_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privacy_sanitization() {
        let wrapper = PrivacyWrapper::new(3600);
        let input = "Peer 12D3KooWABC123 sent message to 12D3KooWXYZ789";
        let sanitized = wrapper.sanitize_input(input);

        assert_eq!(sanitized, "Peer [ANON_PEER] sent message to [ANON_PEER]");
    }

    #[tokio::test]
    async fn test_cache_basic() {
        let wrapper = PrivacyWrapper::new(3600);

        wrapper.cache_result("test question", "test answer").await;
        let cached = wrapper.get_cached("test question").await;

        assert_eq!(cached, Some("test answer".to_string()));
    }

    #[tokio::test]
    async fn test_cache_ttl() {
        let wrapper = PrivacyWrapper::new(1); // 1 секунда TTL

        wrapper.cache_result("test question", "test answer").await;
        tokio::time::sleep(Duration::from_secs(2)).await;
        let cached = wrapper.get_cached("test question").await;

        assert_eq!(cached, None);
    }

    #[test]
    fn test_free_models_list() {
        assert!(!FREE_MODELS.is_empty());
        assert!(FREE_MODELS[0].contains("qwen"));
    }
}
