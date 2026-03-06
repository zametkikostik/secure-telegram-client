// messenger/src/ai/assistant.rs
//! AI Ассистент на основе Qwen 3.5

use reqwest::Client;
use serde_json::json;

pub struct QwenAssistant {
    client: Client,
    api_key: String,
    model: String,
}

impl QwenAssistant {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }
    
    pub async fn chat(&self, message: &str) -> Result<String, Box<dyn std::error::Error>> {
        let response = self.client
            .post("https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "model": &self.model,
                "messages": [{
                    "role": "user",
                    "content": message
                }],
                "max_tokens": 2048
            }))
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response["output"]["text"].as_str().unwrap_or("").to_string())
    }
    
    pub async fn summarize(&self, text: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.chat(&format!("Саммаризируй следующий текст:\n{}", text)).await
    }
    
    pub async fn generate_code(&self, description: &str, language: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.chat(&format!("Напиши код на {}: {}", language, description)).await
    }
    
    pub async fn translate(&self, text: &str, from: &str, to: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.chat(&format!("Переведи с {} на {}: {}", from, to, text)).await
    }
}
