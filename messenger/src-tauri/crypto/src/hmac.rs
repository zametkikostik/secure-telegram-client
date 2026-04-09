// HMAC utilities for message authentication
// SECURITY: требует аудита перед production
// TODO: pentest перед release

use hmac::{Hmac, Mac};
use sha3::Sha3_256;

type HmacSha3 = Hmac<Sha3_256>;

/// Вычисление HMAC для аутентификации сообщений
pub fn compute_hmac(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = <HmacSha3 as Mac>::new_from_slice(key)
        .expect("HMAC can take key of any size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

/// Верификация HMAC
pub fn verify_hmac(key: &[u8], data: &[u8], expected: &[u8]) -> bool {
    let mut mac = <HmacSha3 as Mac>::new_from_slice(key)
        .expect("HMAC can take key of any size");
    mac.update(data);
    mac.verify_slice(expected).is_ok()
}
