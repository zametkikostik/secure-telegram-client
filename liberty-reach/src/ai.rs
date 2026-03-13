//! Модуль интеграции с AI + Web3 Monetization
//!
//! Поддерживает:
//! - Локальный Ollama (порт 11437) — приоритет
//! - OpenRouter API (ключ из .env.local) — fallback
//! - Автоматический fallback при недоступности Ollama (2 сек таймаут)
//! - Проверка баланса Polygon перед AI запросами (Monetization)
//! - ERC20/NFT проверка подписки для доступа к AI
//! - Контекстный анализ истории чата
//! - Вся нагрузка при fallback на внешнем API, не на CPU сервера

use reqwest::Client;
use serde_json::json;
use anyhow::{Result, Context};
use std::time::Duration;
use ethers::prelude::*;
use ethers::providers::{Http, Provider};
use ethers::utils::format_units;
use std::sync::Arc;

/// Конфигурация AI-провайдера
#[derive(Debug, Clone)]
pub struct AIConfig {
    /// Локальный Ollama
    pub ollama_endpoint: String,
    pub ollama_model: String,
    /// OpenRouter API
    pub openrouter_api_key: Option<String>,
    pub openrouter_model: String,
    pub openrouter_endpoint: String,
    /// Web3 Monetization (Polygon)
    pub web3_rpc_url: Option<String>,
    pub web3_wallet_address: Option<String>,
    /// Минимальный баланс MATIC для доступа к AI
    pub min_balance_matic: f64,
    /// ERC20 токен для подписки (опционально)
    pub erc20_token_address: Option<String>,
    /// Минимальное количество токенов для доступа
    pub min_erc20_balance: f64,
    /// NFT коллекция для подписки (опционально)
    pub nft_contract_address: Option<String>,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            ollama_endpoint: "http://localhost:11437/api/generate".to_string(),
            ollama_model: "qwen2.5-coder:3b".to_string(),
            openrouter_api_key: std::env::var("OPENROUTER_API_KEY").ok(),
            openrouter_model: "qwen/qwen-2.5-coder-32b-instruct".to_string(),
            openrouter_endpoint: "https://openrouter.ai/api/v1/chat/completions".to_string(),
            web3_rpc_url: std::env::var("WEB3_RPC_URL").ok(),
            web3_wallet_address: std::env::var("WEB3_WALLET_ADDRESS").ok(),
            min_balance_matic: 0.01,
            erc20_token_address: std::env::var("AI_ERC20_TOKEN").ok(),
            min_erc20_balance: 1.0,
            nft_contract_address: std::env::var("AI_NFT_CONTRACT").ok(),
        }
    }
}

/// Мост для интеграции с AI
pub struct AIBridge {
    /// Клиент для Ollama (короткие таймауты)
    ollama_client: Client,
    /// Клиент для OpenRouter (стандартные таймауты)
    openrouter_client: Client,
    /// Конфигурация
    config: AIConfig,
}

impl AIBridge {
    /// Создание AI моста с гибридной логикой + Web3 проверкой
    /// Приоритет: Ollama (локальный) → OpenRouter (fallback)
    pub fn new() -> Self {
        // Загрузка переменных окружения
        let _ = dotenvy::from_filename(".env.local").ok();
        let _ = dotenvy::dotenv().ok();

        let config = AIConfig::default();

        // Клиент для Ollama с короткими таймаутами (2 сек коннект)
        let ollama_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(2))
            .build()
            .unwrap_or_default();

        // Клиент для OpenRouter
        let openrouter_client = Client::builder()
            .timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_default();

        // Информация о режиме работы
        if config.openrouter_api_key.is_some() {
            println!("✓ AI: Ollama (приоритет) + OpenRouter (fallback)");
        } else {
            println!("✓ AI: Ollama (локальный, без fallback)");
        }

        if config.web3_wallet_address.is_some() {
            println!("💰 Web3 Monetization: включена (Polygon)");
        }

        Self {
            ollama_client,
            openrouter_client,
            config,
        }
    }

    /// Проверка баланса Polygon кошелька
    pub async fn check_polygon_balance(&self) -> Result<f64> {
        let rpc_url = self.config.web3_rpc_url.as_ref()
            .ok_or_else(|| anyhow::anyhow!("WEB3_RPC_URL не настроен"))?;

        let wallet_address = self.config.web3_wallet_address.as_ref()
            .ok_or_else(|| anyhow::anyhow!("WEB3_WALLET_ADDRESS не настроен"))?;

        // Подключение к Polygon RPC
        let provider = Provider::<Http>::try_from(rpc_url)
            .context("Ошибка подключения к Polygon RPC")?;
        let provider = Arc::new(provider);

        // Парсинг адреса кошелька
        let address = wallet_address.parse::<Address>()
            .context("Неверный формат адреса кошелька")?;

        // Получение баланса MATIC
        let balance_wei = provider.get_balance(address, None).await
            .context("Ошибка получения баланса")?;

        // Конвертация из Wei в MATIC (1 MATIC = 10^18 Wei)
        let balance_matic = format_units(balance_wei, "ether")
            .unwrap_or_else(|_| "0".to_string())
            .parse::<f64>()
            .unwrap_or(0.0);

        Ok(balance_matic)
    }

    /// Проверка минимального баланса для доступа к AI
    pub async fn verify_ai_access(&self) -> Result<bool> {
        // Если Web3 не настроен — разрешаем доступ
        if self.config.web3_wallet_address.is_none() {
            tracing::debug!("Web3 не настроен, разрешаем доступ к AI");
            return Ok(true);
        }

        // Проверка баланса MATIC
        let balance = self.check_polygon_balance().await?;
        let min_balance = self.config.min_balance_matic;

        if balance >= min_balance {
            tracing::info!("💰 Баланс Polygon: {:.4} MATIC (доступ разрешён)", balance);
            return Ok(true);
        }

        // Проверка ERC20 токена (если настроен)
        if let Some(_) = &self.config.erc20_token_address {
            match self.check_erc20_balance().await {
                Ok(erc20_balance) => {
                    if erc20_balance >= self.config.min_erc20_balance {
                        tracing::info!("💰 ERC20 баланс: {:.4} (доступ разрешён)", erc20_balance);
                        return Ok(true);
                    }
                }
                Err(e) => tracing::warn!("⚠️ Ошибка проверки ERC20: {}", e),
            }
        }

        // Проверка NFT (если настроен)
        if let Some(_) = &self.config.nft_contract_address {
            match self.check_nft_ownership().await {
                Ok(has_nft) => {
                    if has_nft {
                        tracing::info!("🖼️ NFT владелец подтверждён (доступ разрешён)");
                        return Ok(true);
                    }
                }
                Err(e) => tracing::warn!("⚠️ Ошибка проверки NFT: {}", e),
            }
        }

        // Все проверки не пройдены
        tracing::warn!("⚠️ Недостаточный баланс: {:.4} MATIC (минимум {:.4})", balance, min_balance);
        Err(anyhow::anyhow!(
            "💰 Недостаточно MATIC для доступа к AI. Требуется минимум {:.4} MATIC. Ваш баланс: {:.4}",
            min_balance, balance
        ))
    }

    /// Проверка баланса ERC20 токена (WIP - requires ethers contract fix)
    pub async fn check_erc20_balance(&self) -> Result<f64> {
        // TODO: Implement proper ERC20 balance check
        // For now, return 0 to allow fallback to other methods
        tracing::warn!("ERC20 balance check not yet implemented");
        Ok(0.0)
    }

    /// Проверка владения NFT (WIP - requires ethers contract fix)
    pub async fn check_nft_ownership(&self) -> Result<bool> {
        // TODO: Implement proper NFT ownership check
        // For now, return false to allow fallback to other methods
        tracing::warn!("NFT ownership check not yet implemented");
        Ok(false)
    }

    /// Простой запрос к AI
    pub async fn ask(&self, prompt: &str) -> Result<String> {
        self.ask_with_context(prompt, &[]).await
    }

    /// Запрос к AI с Web3 проверкой + гибридной логикой:
    /// 1. Проверка баланса Polygon (если настроено)
    /// 2. Попытка #1: Локальный Ollama (приоритет)
    /// 3. Попытка #2: OpenRouter (fallback)
    /// 4. Попытка #3: Offline AI message (retry strategy)
    pub async fn ask_with_context(&self, prompt: &str, history: &[String]) -> Result<String> {
        // Шаг 1: Web3 проверка баланса (Monetization)
        if let Err(e) = self.verify_ai_access().await {
            tracing::error!("❌ Web3 проверка не пройдена: {}", e);
            return Err(e);
        }

        // Шаг 2: Попытка #1 — Локальный Ollama (приоритет)
        match self.ask_ollama(prompt, history).await {
            Ok(response) => {
                tracing::debug!("✅ AI ответ от Ollama (локальный)");
                Ok(response)
            }
            Err(ollama_err) => {
                // Логирование ошибки Ollama
                tracing::warn!("⚠️ Ollama недоступен: {}. Переключаюсь на OpenRouter...", ollama_err);

                // Шаг 3: Попытка #2 — OpenRouter (fallback)
                if let Some(_) = &self.config.openrouter_api_key {
                    tracing::info!("🔄 AI fallback: OpenRouter API (попытка #2)");
                    match self.ask_openrouter(prompt, history).await {
                        Ok(response) => {
                            tracing::info!("✅ AI ответ от OpenRouter (fallback)");
                            Ok(response)
                        }
                        Err(openrouter_err) => {
                            // Логирование ошибки OpenRouter
                            tracing::warn!("⚠️ OpenRouter недоступен: {}. Возвращаю Offline AI message", openrouter_err);

                            // Шаг 4: Попытка #3 — Предопределённое сообщение (retry strategy)
                            Ok("🤖 AI временно недоступен. Пожалуйста, попробуйте позже или проверьте интернет-соединение.".to_string())
                        }
                    }
                } else {
                    // Нет OpenRouter — возвращаем понятную ошибку
                    tracing::error!("❌ AI недоступен: OpenRouter не настроен");
                    Err(anyhow::anyhow!("🤖 AI недоступен: Ollama не отвечает, OpenRouter не настроен"))
                }
            }
        }
    }

    /// Запрос к локальному Ollama
    async fn ask_ollama(&self, prompt: &str, history: &[String]) -> Result<String> {
        let context = self.format_context(history);
        let full_prompt = format!("{}\n\nВопрос пользователя: {}\n\nОтвечай кратко и по делу.", context, prompt);

        let res = self.ollama_client.post(&self.config.ollama_endpoint)
            .json(&json!({
                "model": &self.config.ollama_model,
                "prompt": full_prompt,
                "stream": false
            }))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    anyhow::anyhow!("⏱️ Ollama таймаут (30 сек): домашний ПК недоступен или AI перегружен")
                } else if e.is_connect() {
                    anyhow::anyhow!("🔌 Ollama недоступен: проверьте, запущен ли Ollama на домашнем ПК")
                } else {
                    anyhow::anyhow!("Ошибка подключения к Ollama: {}", e)
                }
            })?;

        if !res.status().is_success() {
            anyhow::bail!("Ollama вернул ошибку: {}", res.status());
        }

        let value = res.json::<serde_json::Value>()
            .await
            .context("Ошибка парсинга ответа Ollama")?;

        if let Some(error) = value.get("error").and_then(|e| e.as_str()) {
            anyhow::bail!("Ошибка AI: {}", error);
        }

        Ok(value["response"]
            .as_str()
            .unwrap_or("Пустой ответ AI")
            .to_string())
    }

    /// Запрос к OpenRouter API (fallback)
    async fn ask_openrouter(&self, prompt: &str, history: &[String]) -> Result<String> {
        let api_key = self.config.openrouter_api_key.as_ref()
            .ok_or_else(|| anyhow::anyhow!("OpenRouter API ключ не настроен"))?;

        let messages = self.build_messages(prompt, history);

        let res = self.openrouter_client.post(&self.config.openrouter_endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": &self.config.openrouter_model,
                "messages": messages,
                "max_tokens": 1024,
                "temperature": 0.7,
            }))
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    anyhow::anyhow!("⏱️ OpenRouter таймаут (60 сек): сервер перегружен")
                } else if e.is_connect() {
                    anyhow::anyhow!("🔌 OpenRouter недоступен: проверьте интернет-соединение")
                } else {
                    anyhow::anyhow!("Ошибка подключения к OpenRouter: {}", e)
                }
            })?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            anyhow::bail!("OpenRouter вернул ошибку ({}): {}", status, body);
        }

        let value = res.json::<serde_json::Value>()
            .await
            .context("Ошибка парсинга ответа OpenRouter")?;

        // Проверка на ошибки в ответе
        if let Some(error) = value.get("error").and_then(|e| e.as_str()) {
            anyhow::bail!("Ошибка AI: {}", error);
        }

        // Извлечение ответа из структуры OpenRouter
        let response = value["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("Пустой ответ AI");

        Ok(response.to_string())
    }

    /// Форматирование контекста истории
    fn format_context(&self, history: &[String]) -> String {
        if history.is_empty() {
            "Нет предыдущего контекста.".to_string()
        } else {
            format!("История последних сообщений:\n{}", history.join("\n"))
        }
    }

    /// Построение списка сообщений для OpenRouter
    fn build_messages(&self, prompt: &str, history: &[String]) -> Vec<serde_json::Value> {
        let mut messages = Vec::new();

        // Системное сообщение
        messages.push(json!({
            "role": "system",
            "content": "Ты — Liberty Architect, Senior Rust Engineer. Твоя специализация: разработка децентрализованного мессенджера Liberty Reach. Отвечай кратко и по делу."
        }));

        // История чата
        if !history.is_empty() {
            let context = self.format_context(history);
            messages.push(json!({
                "role": "user",
                "content": context
            }));
        }

        // Текущий вопрос
        messages.push(json!({
            "role": "user",
            "content": prompt
        }));

        messages
    }

    /// Анализ последних сообщений чата
    pub async fn analyze_chat(&self, messages: &[String], query: &str) -> Result<String> {
        if messages.is_empty() {
            return Ok("Нет сообщений для анализа.".to_string());
        }

        let analysis_prompt = format!(
            "Проанализируй следующие сообщения чата и ответь на вопрос.\n\n\
             Сообщения:\n{}\n\n\
             Вопрос: {}",
            messages.join("\n"),
            query
        );

        self.ask(&analysis_prompt).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_web3_access_empty_wallet() {
        // Тест имитирует пустой кошелёк (без Web3 конфигурации)
        let config = AIConfig {
            web3_wallet_address: None,
            web3_rpc_url: None,
            min_balance_matic: 0.01,
            ..Default::default()
        };

        let ollama_client = Client::new();
        let openrouter_client = Client::new();

        let bridge = AIBridge {
            ollama_client,
            openrouter_client,
            config,
        };

        // Проверка: доступ разрешён если Web3 не настроен
        let result = bridge.verify_ai_access().await;
        assert!(result.is_ok(), "Доступ должен быть разрешён без Web3");
    }

    #[tokio::test]
    async fn test_ai_config_defaults() {
        let config = AIConfig::default();

        assert_eq!(config.ollama_endpoint, "http://localhost:11437/api/generate");
        assert_eq!(config.ollama_model, "qwen2.5-coder:3b");
        assert_eq!(config.min_balance_matic, 0.01);
        assert!(config.openrouter_model.contains("qwen"));
    }
}

impl Default for AIBridge {
    fn default() -> Self {
        Self::new()
    }
}
