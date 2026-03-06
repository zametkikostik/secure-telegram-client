// messenger/src/web3/abcex.rs
//! ABCEX API интеграция (2-3% комиссия)

use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct AbcexIntegration {
    client: Client,
    api_key: String,
    fee_percentage: u64, // 200 = 2%
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AbcexOrder {
    pub id: String,
    pub from_currency: String,
    pub to_currency: String,
    pub amount: String,
    pub rate: String,
    pub fee: String,
    pub total: String,
}

impl AbcexIntegration {
    pub fn new(api_key: String, fee_percentage: u64) -> Self {
        Self {
            client: Client::new(),
            api_key,
            fee_percentage,
        }
    }
    
    pub async fn get_exchange_rate(
        &self,
        from: &str,
        to: &str,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let response = self.client
            .get("https://api.abcex.com/v1/rates")
            .query(&[
                ("from", from),
                ("to", to),
                ("api_key", &self.api_key),
            ])
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response)
    }
    
    pub async fn create_order(
        &self,
        from_currency: &str,
        to_currency: &str,
        amount: &str,
    ) -> Result<AbcexOrder, Box<dyn std::error::Error>> {
        // Получаем курс
        let rate_data = self.get_exchange_rate(from_currency, to_currency).await?;
        
        // Рассчитываем комиссию
        let amount_num: f64 = amount.parse()?;
        let fee = amount_num * (self.fee_percentage as f64 / 100.0);
        let total = amount_num - fee;
        
        let order = AbcexOrder {
            id: uuid::Uuid::new_v4().to_string(),
            from_currency: from_currency.to_string(),
            to_currency: to_currency.to_string(),
            amount: amount.to_string(),
            rate: rate_data["rate"].as_str().unwrap_or("1.0").to_string(),
            fee: fee.to_string(),
            total: total.to_string(),
        };
        
        Ok(order)
    }
}
