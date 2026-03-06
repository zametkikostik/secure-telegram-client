// server/src/auth.rs
//! Аутентификация JWT + Ed25519

use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer};
use jsonwebtoken::{encode, decode, Header, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use rand::rngs::OsRng;

/// Секрет для JWT
pub fn get_jwt_secret() -> Vec<u8> {
    std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "liberty-reach-secret-key-2024".to_string())
        .into_bytes()
}

/// Claims для JWT токена
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub username: String,
    pub exp: usize,  // expiration time
    pub iat: usize,  // issued at
}

/// Генерация пары ключей Ed25519
pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
    let mut csprng = OsRng {};
    let signing_key: SigningKey = SigningKey::generate(&mut csprng);
    let verifying_key: VerifyingKey = signing_key.verifying_key();
    (signing_key, verifying_key)
}

/// Создание JWT токена
pub fn create_token(user_id: &str, username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expiration = now + Duration::days(30);

    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_string(),
        exp: expiration.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(&get_jwt_secret()),
    )
}

/// Верификация JWT токена
pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(&get_jwt_secret()),
        &Validation::new(Algorithm::HS256),
    )?;

    Ok(token_data.claims)
}

/// Хэширование пароля
pub fn hash_password(password: &str) -> Result<String, argon2::Error> {
    use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
    
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    
    Ok(password_hash.to_string())
}

/// Верификация пароля
pub fn verify_password(password: &str, hash: &str) -> bool {
    use argon2::{password_hash::PasswordHash, Argon2, PasswordVerifier};
    
    let parsed_hash = PasswordHash::new(hash).ok()?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

/// Подпись сообщения приватным ключом
pub fn sign_message(private_key: &SigningKey, message: &[u8]) -> Signature {
    private_key.sign(message)
}

/// Верификация подписи сообщения
pub fn verify_signature(
    public_key: &VerifyingKey,
    message: &[u8],
    signature: &Signature,
) -> bool {
    public_key.verify(message, signature).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation_and_verification() {
        let token = create_token("user-123", "testuser").unwrap();
        let claims = verify_token(&token).unwrap();
        
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.username, "testuser");
    }

    #[test]
    fn test_password_hashing() {
        let password = "secure_password123";
        let hash = hash_password(password).unwrap();
        
        assert!(verify_password(password, &hash));
        assert!(!verify_password("wrong_password", &hash));
    }
}
