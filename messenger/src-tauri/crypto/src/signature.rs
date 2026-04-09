// Digital signatures with Ed25519
// SECURITY: требует аудита перед production
// TODO: pentest перед release

use ed25519_dalek::{SigningKey, VerifyingKey, Signature};
use ed25519_dalek::{Signer as _, Verifier as _};
use rand::rngs::OsRng;

/// Подписывание сообщений
pub struct Signer;

impl Signer {
    /// Генерация новой пары ключей для подписи
    pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = VerifyingKey::from(&signing_key);

        tracing::debug!("Generated Ed25519 signing keypair");

        (signing_key, verifying_key)
    }

    /// Подпись сообщения
    pub fn sign(message: &[u8], signing_key: &SigningKey) -> Signature {
        signing_key.sign(message)
    }

    /// Верификация подписи
    pub fn verify(
        message: &[u8],
        signature: &Signature,
        verifying_key: &VerifyingKey,
    ) -> Result<(), ed25519_dalek::SignatureError> {
        verifying_key.verify(message, signature)
    }

    /// Сериализация подписи
    pub fn signature_to_bytes(signature: &Signature) -> [u8; 64] {
        signature.to_bytes()
    }

    /// Десериализация подписи
    pub fn signature_from_bytes(bytes: &[u8; 64]) -> Result<Signature, ed25519_dalek::SignatureError> {
        Ok(Signature::from_bytes(bytes))
    }
}
