// messenger/src/web3/metamask.rs
//! MetaMask интеграция

use ethers::prelude::*;
use std::str::FromStr;

pub struct MetaMaskWallet {
    pub address: Address,
    pub provider: Provider<Http>,
}

impl MetaMaskWallet {
    pub fn new(rpc_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        
        // Адрес заглушка (реальный адрес получается из MetaMask)
        let address = Address::from_str("0x0000000000000000000000000000000000000000")?;
        
        Ok(Self { address, provider })
    }
    
    pub fn connect(&mut self, address: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.address = Address::from_str(address)?;
        Ok(())
    }
    
    pub async fn get_balance(&self) -> Result<U256, Box<dyn std::error::Error>> {
        let balance = self.provider.get_balance(self.address).await?;
        Ok(balance)
    }
    
    pub async fn send_transaction(
        &self,
        to: Address,
        amount: U256,
    ) -> Result<TxHash, Box<dyn std::error::Error>> {
        // В реальной реализации здесь будет подпись через MetaMask
        let tx = TransactionRequest::new()
            .to(to)
            .value(amount);
        
        let tx_hash = self.provider.send_transaction(tx, None).await?;
        Ok(tx_hash)
    }
}
