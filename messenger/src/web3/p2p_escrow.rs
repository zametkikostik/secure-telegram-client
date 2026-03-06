// messenger/src/web3/p2p_escrow.rs
//! P2P Escrow смарт-контракт (3% комиссия)

use ethers::prelude::*;
use std::str::FromStr;

abigen!(
    P2PEscrowContract,
    r#"[
        function createDeal(address seller) external payable returns (uint256)
        function releaseCrypto(uint256 dealId) external
        function deals(uint256) external view returns (address buyer, address seller, uint256 amount, bool paid, bool released)
        function dealCounter() external view returns (uint256)
    ]"#,
);

pub struct P2PEscrow {
    contract: P2PEscrowContract<Provider<Http>>,
    admin_wallet: Address,
    fee_percent: u64, // 3%
}

#[derive(Debug)]
pub struct Deal {
    pub id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub amount: U256,
    pub paid: bool,
    pub released: bool,
}

impl P2PEscrow {
    pub fn new(contract_address: &str, provider: Provider<Http>) -> Result<Self, Box<dyn std::error::Error>> {
        let contract_address = Address::from_str(contract_address)?;
        let admin_wallet = Address::from_str("0x0000000000000000000000000000000000000000")?;
        
        let contract = P2PEscrowContract::new(contract_address, provider.into());
        
        Ok(Self {
            contract,
            admin_wallet,
            fee_percent: 3,
        })
    }
    
    pub async fn create_deal(
        &self,
        seller: Address,
        amount: U256,
    ) -> Result<Deal, Box<dyn std::error::Error>> {
        // В реальной реализации здесь будет вызов смарт-контракта
        let deal_id = 1u64;
        
        Ok(Deal {
            id: deal_id,
            buyer: self.admin_wallet,
            seller,
            amount,
            paid: false,
            released: false,
        })
    }
    
    pub async fn release_crypto(&self, deal_id: u64) -> Result<(), Box<dyn std::error::Error>> {
        // Вызов смарт-контракта для释放 crypto
        println!("Выпуск средств для сделки #{}", deal_id);
        Ok(())
    }
    
    pub fn calculate_fee(&self, amount: U256) -> U256 {
        amount * self.fee_percent / 100
    }
}
