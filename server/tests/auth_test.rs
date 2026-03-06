// server/tests/auth_test.rs
//! Тесты для модуля аутентификации

use liberty_reach_server::auth;

#[test]
fn test_token_creation_and_verification() {
    let token = auth::create_token("user-123", "testuser").unwrap();
    let claims = auth::verify_token(&token).unwrap();
    
    assert_eq!(claims.sub, "user-123");
    assert_eq!(claims.username, "testuser");
    assert!(claims.exp > claims.iat);
}

#[test]
fn test_invalid_token() {
    let result = auth::verify_token("invalid-token");
    assert!(result.is_err());
}

#[test]
fn test_password_hashing() {
    let password = "secure_password123";
    let hash = auth::hash_password(password).unwrap();
    
    assert!(auth::verify_password(password, &hash));
    assert!(!auth::verify_password("wrong_password", &hash));
}

#[test]
fn test_keypair_generation() {
    let (signing_key, verifying_key) = auth::generate_keypair();
    
    // Проверка что ключи не пустые
    assert!(!signing_key.to_bytes().is_empty());
    assert!(!verifying_key.to_bytes().is_empty());
}

#[test]
fn test_message_signing() {
    let (signing_key, verifying_key) = auth::generate_keypair();
    let message = b"Hello, World!";
    
    let signature = auth::sign_message(&signing_key, message);
    assert!(auth::verify_signature(&verifying_key, message, &signature));
}

#[tokio::test]
async fn test_expired_token() {
    use jsonwebtoken::{encode, Header, Algorithm};
    use serde::{Deserialize, Serialize};
    use chrono::{Utc, Duration};
    
    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        username: String,
        exp: usize,
        iat: usize,
    }
    
    let claims = Claims {
        sub: "user-123".to_string(),
        username: "testuser".to_string(),
        exp: (Utc::now() - Duration::hours(1)).timestamp() as usize, // Истёк час назад
        iat: (Utc::now() - Duration::hours(2)).timestamp() as usize,
    };
    
    let token = encode(
        &Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(auth::get_jwt_secret().as_slice()),
    ).unwrap();
    
    // Токен должен быть невалидным из-за истечения срока
    let result = auth::verify_token(&token);
    assert!(result.is_err());
}
