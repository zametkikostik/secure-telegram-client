use base64::{engine::general_purpose::STANDARD as base64_standard, Engine as _};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use x25519_dalek::{PublicKey, StaticSecret};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2EEPayload {
    pub ephemeral_public_key: String,
    pub ciphertext: String,
    pub nonce: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyBundle {
    pub x25519_public_key: String,
    pub ed25519_public_key: String,
}

#[allow(dead_code)]
fn derive_chacha_key(secret: &StaticSecret, public: &PublicKey) -> [u8; 32] {
    let shared = secret.diffie_hellman(public);
    let mut hasher = Sha3_256::new();
    hasher.update(shared.as_bytes());
    hasher.finalize().into()
}

#[allow(dead_code)]
pub fn encrypt_message(
    message: &[u8],
    recipient_public_key: &[u8; 32],
) -> Result<E2EEPayload, String> {
    let mut secret_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut secret_bytes);
    let ephemeral_secret = StaticSecret::from(secret_bytes);
    let ephemeral_public = PublicKey::from(&ephemeral_secret);

    let recipient_pk = PublicKey::from(*recipient_public_key);
    let chacha_key = derive_chacha_key(&ephemeral_secret, &recipient_pk);
    let chacha_key = Key::from_slice(&chacha_key);

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let cipher = ChaCha20Poly1305::new(chacha_key);
    let ciphertext = cipher
        .encrypt(nonce, message.as_ref())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    Ok(E2EEPayload {
        ephemeral_public_key: base64_standard.encode(ephemeral_public.as_bytes()),
        ciphertext: base64_standard.encode(&ciphertext),
        nonce: base64_standard.encode(&nonce_bytes),
    })
}

#[allow(dead_code)]
pub fn decrypt_message(payload: &E2EEPayload, my_secret_key: &[u8; 32]) -> Result<Vec<u8>, String> {
    let ephemeral_pk_bytes = base64_standard
        .decode(&payload.ephemeral_public_key)
        .map_err(|e| format!("Invalid ephemeral key: {}", e))?;
    let ephemeral_pk = PublicKey::from(*arrayref::array_ref![ephemeral_pk_bytes, 0, 32]);

    let my_secret = StaticSecret::from(*my_secret_key);
    let chacha_key = derive_chacha_key(&my_secret, &ephemeral_pk);
    let chacha_key = Key::from_slice(&chacha_key);

    let nonce_bytes = base64_standard
        .decode(&payload.nonce)
        .map_err(|e| format!("Invalid nonce: {}", e))?;
    let ciphertext = base64_standard
        .decode(&payload.ciphertext)
        .map_err(|e| format!("Invalid ciphertext: {}", e))?;

    let nonce = Nonce::from_slice(&nonce_bytes);
    let cipher = ChaCha20Poly1305::new(chacha_key);

    cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let mut bob_secret = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bob_secret);

        let bob_public = PublicKey::from(bob_secret);

        let message = b"Hello Bob!";
        let payload = encrypt_message(message, bob_public.as_bytes()).unwrap();

        let decrypted = decrypt_message(&payload, &bob_secret).unwrap();
        assert_eq!(decrypted, message);
    }

    #[test]
    fn test_wrong_key_fails() {
        let mut bob_secret = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bob_secret);
        let mut eve_secret = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut eve_secret);

        let bob_public = PublicKey::from(bob_secret);
        let message = b"Secret message";
        let payload = encrypt_message(message, bob_public.as_bytes()).unwrap();

        let result = decrypt_message(&payload, &eve_secret);
        assert!(result.is_err());
    }
}
