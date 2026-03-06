// messenger/src/web3/0x_swap.rs
use reqwest::Client;

pub struct ZeroExIntegration {
    client: Client,
    admin_wallet: String,
    fee_percentage: u64, // 200 = 2%
}

impl ZeroExIntegration {
    pub fn new(admin_wallet: String, fee_percentage: u64) -> Self {
        Self {
            client: Client::new(),
            admin_wallet,
            fee_percentage,
        }
    }
    
    pub async fn get_swap_quote(
        &self,
        from_token: &str,
        to_token: &str,
        amount: &str,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let response = self.client
            .get("https://api.0x.org/swap/v1/quote")
            .query(&[
                ("buyToken", to_token),
                ("sellToken", from_token),
                ("sellAmount", amount),
                ("affiliateAddress", &self.admin_wallet),
                ("feeRecipient", &self.admin_wallet),
                ("buyTokenPercentageFee", &self.fee_percentage.to_string()),
            ])
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response)
    }
}
