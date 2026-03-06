// messenger/src/ai/translator.rs
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct AITranslator {
    client: Client,
    qwen_api_key: String,
}

impl AITranslator {
    pub fn new(qwen_api_key: String) -> Self {
        Self {
            client: Client::new(),
            qwen_api_key,
        }
    }
    
    pub async fn translate_text(
        &self,
        text: &str,
        from: &str,
        to: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let response = self.client
            .post("https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation")
            .header("Authorization", format!("Bearer {}", self.qwen_api_key))
            .json(&serde_json::json!({
                "model": "qwen-turbo",
                "messages": [{
                    "role": "user",
                    "content": format!("Translate from {} to {}: {}", from, to, text)
                }],
                "max_tokens": 2048
            }))
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response["output"]["text"].as_str().unwrap_or("").to_string())
    }
    
    pub async fn detect_language(&self, text: &str) -> Result<String, Box<dyn std::error::Error>> {
        let response = self.client
            .post("https://dashscope.aliyuncs.com/api/v1/services/aigc/text-generation/generation")
            .header("Authorization", format!("Bearer {}", self.qwen_api_key))
            .json(&serde_json::json!({
                "model": "qwen-turbo",
                "messages": [{
                    "role": "user",
                    "content": format!("Detect language of: {}", text)
                }],
                "max_tokens": 100
            }))
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response["output"]["text"].as_str().unwrap_or("en").to_string())
    }
}
