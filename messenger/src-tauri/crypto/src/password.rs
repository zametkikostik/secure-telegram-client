// Password hashing with Argon2id
// SECURITY: требует аудита перед production
// TODO: pentest перед release

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::PasswordHasher as _;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Хешер паролей с использованием Argon2id
pub struct PasswordHasher;

impl PasswordHasher {
    /// Хешрование пароля для локального хранения
    ///
    /// Параметры Argon2id:
    /// - m_cost: 64 MB (память)
    /// - t_cost: 3 итерации (время)
    /// - p_cost: 4 (параллелизм)
    /// SECURITY: параметры должны быть настроены под железо пользователя
    pub fn hash(password: &str) -> Result<String, CryptoError> {
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            argon2::Params::new(
                64 * 1024, // 64 MB memory
                3,         // 3 iterations
                4,         // 4 parallelism
                None,      // default output length
            ).map_err(|_| CryptoError::HashingFailed)?,
        );

        let salt = argon2::password_hash::SaltString::generate(&mut OsRng);
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| CryptoError::HashingFailed)?
            .to_string();

        tracing::debug!("Password hashed with Argon2id");

        Ok(password_hash)
    }

    /// Верификация пароля
    pub fn verify(password: &str, hash: &str) -> Result<bool, CryptoError> {
        let argon2 = Argon2::default();
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|_| CryptoError::HashingFailed)?;

        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
}

/// Безопасное хранение пароля в памяти
// SECURITY: использовать только для временного хранения, затем zeroize
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecurePassword {
    password: Vec<u8>,
}

impl SecurePassword {
    pub fn new(password: &str) -> Self {
        Self {
            password: password.as_bytes().to_vec(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.password
    }
}

pub use crate::error::CryptoError;
