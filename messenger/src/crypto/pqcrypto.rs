// messenger/src/crypto/pqcrypto.rs
//! Post-quantum шифрование на основе Kyber1024

use pqcrypto_kyber::kyber1024;
use pqcrypto_traits::kem::{PublicKey, SecretKey, SharedSecret, Ciphertext};

pub struct Kyber1024 {
    public_key: kyber1024::PublicKey,
    secret_key: kyber1024::SecretKey,
}

impl Kyber1024 {
    pub fn new() -> Self {
        let (pk, sk) = kyber1024::keypair();
        Self {
            public_key: pk,
            secret_key: sk,
        }
    }
    
    pub fn encapsulate(&self) -> Result<(Vec<u8>, SharedSecret), Box<dyn std::error::Error>> {
        let (ct, ss) = kyber1024::encapsulate(&self.public_key)?;
        Ok((ct.as_bytes().to_vec(), ss))
    }
    
    pub fn decapsulate(&self, ciphertext: &[u8]) -> Result<SharedSecret, Box<dyn std::error::Error>> {
        let ct = kyber1024::Ciphertext::from_bytes(ciphertext)?;
        let ss = kyber1024::decapsulate(&ct, &self.secret_key)?;
        Ok(ss)
    }
    
    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.public_key.as_bytes().to_vec()
    }
}
