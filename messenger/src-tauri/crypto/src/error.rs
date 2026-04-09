// Unified crypto error type
// SECURITY: требует аудита перед production
// TODO: pentest перед release

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Not implemented yet")]
    NotImplemented,
    #[error("Invalid key length")]
    InvalidKeyLength,
    #[error("Encryption failed")]
    EncryptionFailed,
    #[error("Decryption failed")]
    DecryptionFailed,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Hashing failed")]
    HashingFailed,
    #[error("Key generation failed")]
    KeyGenerationFailed,
    #[error("Invalid data: {0}")]
    InvalidData(String),
}
