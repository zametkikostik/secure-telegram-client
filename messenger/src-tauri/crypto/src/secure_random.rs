// Secure random number generation
// SECURITY: требует аудита перед production
// TODO: pentest перед release

use rand::rngs::OsRng;
use rand::RngCore;

/// Генерация криптографически безопасных случайных байтов
pub fn generate_random_bytes(len: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; len];
    OsRng.fill_bytes(&mut bytes);
    bytes
}

/// Генерация случайной nonce
pub fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

/// Генерация случайного salt
pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}
