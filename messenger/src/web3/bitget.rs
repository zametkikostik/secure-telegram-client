// messenger/src/web3/bitget.rs
//! Bitget API интеграция (2-3% комиссия)

use reqwest::Client;
use serde::{Deserialize, Serialize};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64;
use chrono::Utc;

type HmacSha256 = Hmac<Sha256>;

pub struct BitgetIntegration {
    client: Client,
    api_key: String,
    secret_key: String,
    passphrase: String,
    fee_percentage: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BitgetOrder {
    pub order_id: String,
    pub symbol: String,
    pub side: String,
    pub amount: String,
    pub price: String,
    pub fee: String,
}

impl BitgetIntegration {
    pub fn new(
        api_key: String,
        secret_key: String,
        passphrase: String,
        fee_percentage: u64,
    ) -> Self {
        Self {
            client: Client::new(),
            api_key,
            secret_key,
            passphrase,
            fee_percentage,
        }
    }
    
    fn generate_signature(&self, timestamp: &str, method: &str, path: &str, body: &str) -> String {
        let message = format!("{}{}{}{}", timestamp, method, path, body);
        
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes()).unwrap();
        mac.update(message.as_bytes());
        let result = mac.finalize();
        
        base64::encode(result.into_bytes())
    }
    
    pub async fn get_ticker(&self, symbol: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let timestamp = Utc::now().timestamp().to_string();
        let signature = self.generate_signature(&timestamp, "GET", "/api/v2/market/ticker", "");
        
        let response = self.client
            .get("https://api.bitget.com/api/v2/market/ticker")
            .query(&[("symbol", symbol)])
            .header("ACCESS-KEY", &self.api_key)
            .header("ACCESS-SIGN", &signature)
            .header("ACCESS-TIMESTAMP", &timestamp)
            .header("ACCESS-PASSPHRASE", &self.passphrase)
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response)
    }
    
    pub async fn place_order(
        &self,
        symbol: &str,
        side: &str,
        amount: &str,
        price: &str,
    ) -> Result<BitgetOrder, Box<dyn std::error::Error>> {
        let timestamp = Utc::now().timestamp().to_string();
        
        let body = serde_json::json!({
            "symbol": symbol,
            "side": side,
            "amount": amount,
            "price": price,
        }).to_string();
        
        let signature = self.generate_signature(&timestamp, "POST", "/api/v2/spot/trade/order", &body);
        
        let response = self.client
            .post("https://api.bitget.com/api/v2/spot/trade/order")
            .header("ACCESS-KEY", &self.api_key)
            .header("ACCESS-SIGN", &signature)
            .header("ACCESS-TIMESTAMP", &timestamp)
            .header("ACCESS-PASSPHRASE", &self.passphrase)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "symbol": symbol,
                "side": side,
                "amount": amount,
                "price": price,
            }))
            .send()
            .await?
            .json()
            .await?;
        
        let amount_num: f64 = amount.parse()?;
        let fee = amount_num * (self.fee_percentage as f64 / 100.0);
        
        let order = BitgetOrder {
            order_id: response["data"]["order_id"].as_str().unwrap_or("").to_string(),
            symbol: symbol.to_string(),
            side: side.to_string(),
            amount: amount.to_string(),
            price: price.to_string(),
            fee: fee.to_string(),
        };
        
        Ok(order)
    }
}
