use aes_gcm::{Aes256Gcm, Key, Nonce, aead::Aead};
use aes_gcm::aead::OsRng;
use chacha20poly1305::ChaCha20Poly1305;
use x25519_dalek::{EphemeralSecret, PublicKey};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature};
use pqcrypto_kyber::kyber1024;

pub struct Crypto {
    signing_key: SigningKey,
    encryption_key: EphemeralSecret,
}

impl Crypto {
    pub fn new() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let encryption_key = EphemeralSecret::random();
        
        Crypto {
            signing_key,
            encryption_key,
        }
    }

    /// Post-quantum KEM (Kyber1024)
    pub fn generate_kyber_keys() -> (kyber1024::PublicKey, kyber1024::SecretKey) {
        kyber1024::keypair()
    }

    /// X25519 Key Exchange
    pub fn key_exchange(&self, peer_public: &PublicKey) -> [u8; 32] {
        self.encryption_key.diffie_hellman(peer_public)
            .as_bytes()
            .to_owned()
    }

    /// AES-256-GCM шифрование
    pub fn encrypt_aes(&self, plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let nonce = Nonce::from_slice(&[0u8; 12]); // В продакшене использовать случайный nonce!
        
        cipher.encrypt(nonce, plaintext)
            .map_err(|e| e.to_string())
    }

    /// AES-256-GCM дешифрование
    pub fn decrypt_aes(&self, ciphertext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
        let nonce = Nonce::from_slice(&[0u8; 12]);
        
        cipher.decrypt(nonce, ciphertext)
            .map_err(|e| e.to_string())
    }

    /// ChaCha20-Poly1305 шифрование
    pub fn encrypt_chacha20(&self, plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
        let cipher = ChaCha20Poly1305::new(Key::<ChaCha20Poly1305>::from_slice(key));
        let nonce = Nonce::from_slice(&[0u8; 12]);
        
        cipher.encrypt(nonce, plaintext)
            .map_err(|e| e.to_string())
    }

    /// Ed25519 подпись
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.signing_key.sign(message)
    }

    /// Ed25519 верификация
    pub fn verify(&self, message: &[u8], signature: &Signature, public_key: &VerifyingKey) -> bool {
        public_key.verify(message, signature).is_ok()
    }

    pub fn get_public_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }
}

impl Default for Crypto {
    fn default() -> Self {
        Self::new()
    }
}
