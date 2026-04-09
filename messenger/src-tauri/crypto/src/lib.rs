// Cryptographic primitives library
// SECURITY: требует аудита перед production
// TODO: pentest перед release

pub mod error;
pub mod keypair;
pub mod hybrid_encrypt;
pub mod password;
pub mod signature;
pub mod hmac;
pub mod secure_random;

pub use error::CryptoError;
pub use keypair::{KeyPair, PublicBundle};
pub use hybrid_encrypt::{HybridEncryptor, EncryptedMessage};
pub use password::PasswordHasher;
pub use signature::Signer;
pub use secure_random::{generate_random_bytes, generate_nonce, generate_salt};
