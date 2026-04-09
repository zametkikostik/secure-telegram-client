//! Secure Messenger — Tauri Desktop Application
//!
//! Main entry point for the desktop application.
//! Initializes logging, creates the Tauri app builder,
//! and registers all command handlers.
//!
//! SECURITY: требует аудита перед production
//! TODO: pentest перед release

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use base64::{engine::general_purpose::STANDARD, Engine};
use std::collections::HashMap;
use std::sync::Mutex;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Application state shared across Tauri commands
struct AppState {
    /// Maps user_id -> HybridKeypair (loaded from keychain on login)
    /// In production, this should be more sophisticated with proper session management
    active_keypairs: Mutex<HashMap<String, secure_messenger_lib::HybridKeypair>>,
    /// Maps user_id -> password (for keychain access)
    /// SECURITY: In production, use proper session tokens instead of storing passwords
    active_passwords: Mutex<HashMap<String, String>>,
}

/// Initialize application logging
fn init_logging() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "secure_messenger=info,tauri=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Tauri command: generate a new hybrid keypair
#[tauri::command]
async fn cmd_generate_keypair() -> Result<String, String> {
    tracing::info!("Generating new hybrid keypair");

    let keypair = secure_messenger_lib::HybridKeypair::generate().map_err(|e| e.to_string())?;

    let bundle = keypair.public_bundle();
    let json = serde_json::to_string(&bundle).map_err(|e| e.to_string())?;

    Ok(json)
}

/// Tauri command: encrypt a message
///
/// # Arguments
/// * `plaintext` — message to encrypt
/// * `sender_user_id` — ID of the sender (to load their keypair)
/// * `recipient_public_bundle_json` — JSON with recipient's public keys
///
/// # Returns
/// Base64-encoded encrypted message
#[tauri::command]
async fn cmd_encrypt(
    state: tauri::State<'_, AppState>,
    plaintext: String,
    sender_user_id: String,
    recipient_public_bundle_json: String,
) -> Result<String, String> {
    tracing::info!("Encrypting message for sender: {}", sender_user_id);

    // Get sender's keypair from active keypairs
    let keypairs = state.active_keypairs.lock().map_err(|e| e.to_string())?;
    let sender_keypair = keypairs
        .get(&sender_user_id)
        .ok_or_else(|| format!("No active keypair found for user: {}", sender_user_id))?;

    // Parse recipient's public bundle
    let recipient_bundle: secure_messenger_lib::PublicBundle =
        serde_json::from_str(&recipient_public_bundle_json)
            .map_err(|e| format!("Invalid recipient public bundle: {}", e))?;

    // Convert recipient keys to proper types
    let recipient_x25519 = x25519_dalek::PublicKey::from(recipient_bundle.x25519_public);

    // Reconstruct Kyber public key
    oqs::init();
    let kyber_kem =
        oqs::kem::Kem::new(oqs::kem::Algorithm::Kyber1024).map_err(|e| e.to_string())?;
    let recipient_kyber_ref = kyber_kem
        .public_key_from_bytes(&recipient_bundle.kyber_public)
        .ok_or_else(|| "Invalid Kyber public key length".to_string())?;
    let recipient_kyber = recipient_kyber_ref.to_owned();

    // Encrypt the message
    let ciphertext = secure_messenger_lib::encrypt(
        plaintext.as_bytes(),
        &recipient_x25519,
        &recipient_kyber,
        &sender_keypair.ed25519_secret,
    )
    .map_err(|e| format!("Encryption failed: {}", e))?;

    // Serialize and base64-encode
    let ciphertext_bytes = ciphertext
        .to_bytes()
        .map_err(|e| format!("Serialization failed: {}", e))?;
    let encoded = STANDARD.encode(&ciphertext_bytes);

    tracing::debug!("Message encrypted successfully");
    Ok(encoded)
}

/// Tauri command: decrypt a message
///
/// # Arguments
/// * `ciphertext_base64` — base64-encoded encrypted message
/// * `recipient_user_id` — ID of the recipient (to load their keypair)
/// * `sender_public_bundle_json` — JSON with sender's public keys (for signature verification)
///
/// # Returns
/// Decrypted plaintext message
#[tauri::command]
async fn cmd_decrypt(
    state: tauri::State<'_, AppState>,
    ciphertext_base64: String,
    recipient_user_id: String,
    sender_public_bundle_json: String,
) -> Result<String, String> {
    tracing::info!("Decrypting message for recipient: {}", recipient_user_id);

    // Get recipient's keypair from active keypairs
    let keypairs = state.active_keypairs.lock().map_err(|e| e.to_string())?;
    let recipient_keypair = keypairs
        .get(&recipient_user_id)
        .ok_or_else(|| format!("No active keypair found for user: {}", recipient_user_id))?;

    // Parse sender's public bundle (needed for signature verification)
    let sender_bundle: secure_messenger_lib::PublicBundle =
        serde_json::from_str(&sender_public_bundle_json)
            .map_err(|e| format!("Invalid sender public bundle: {}", e))?;

    // Convert sender's Ed25519 public key
    let sender_verifying_key = ed25519_dalek::VerifyingKey::from_bytes(
        &sender_bundle
            .ed25519_public
            .try_into()
            .map_err(|_| "Invalid Ed25519 public key length".to_string())?,
    )
    .map_err(|e| format!("Invalid sender verifying key: {}", e))?;

    // Decode base64 to get ciphertext
    let ciphertext_bytes = STANDARD
        .decode(&ciphertext_base64)
        .map_err(|e| format!("Invalid base64: {}", e))?;

    // Deserialize ciphertext
    let ciphertext = secure_messenger_lib::HybridCiphertext::from_bytes(&ciphertext_bytes)
        .map_err(|e| format!("Invalid ciphertext: {}", e))?;

    // Decrypt the message
    let plaintext = secure_messenger_lib::decrypt(
        &ciphertext,
        &recipient_keypair.x25519_secret,
        &recipient_keypair.kyber_secret,
        &sender_verifying_key,
    )
    .map_err(|e| format!("Decryption failed: {}", e))?;

    // Convert to string
    let plaintext_str =
        String::from_utf8(plaintext).map_err(|e| format!("Invalid UTF-8 in plaintext: {}", e))?;

    tracing::debug!("Message decrypted successfully");
    Ok(plaintext_str)
}

/// Tauri command: store keypair in keychain (on registration/key generation)
#[tauri::command]
async fn cmd_store_keypair(
    state: tauri::State<'_, AppState>,
    user_id: String,
    password: String,
) -> Result<String, String> {
    tracing::info!("Generating and storing keypair for user: {}", user_id);

    let keychain = secure_messenger_lib::Keychain::new();

    // Generate new keypair
    let keypair = secure_messenger_lib::HybridKeypair::generate()
        .map_err(|e| format!("Key generation failed: {}", e))?;

    // Store in keychain with password encryption
    keychain
        .store_keypair_with_password(&user_id, &keypair, &password)
        .map_err(|e| format!("Failed to store keypair: {}", e))?;

    // Also store password hash for login verification
    use argon2::password_hash::SaltString;
    use argon2::{Argon2, PasswordHasher};
    use rand::RngCore;

    let mut salt_bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt_bytes);
    let salt = SaltString::encode_b64(&salt_bytes).map_err(|e| e.to_string())?;

    let argon2_inst = Argon2::default();
    let ph = argon2_inst
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| e.to_string())?;
    let password_hash = ph.to_string();

    keychain
        .store_password_hash(&user_id, &password_hash)
        .map_err(|e| format!("Failed to store password hash: {}", e))?;

    // Get public bundle before moving keypair
    let bundle = keypair.public_bundle();

    // Add to active keypairs
    {
        let mut keypairs = state.active_keypairs.lock().map_err(|e| e.to_string())?;
        keypairs.insert(user_id.clone(), keypair);
    }
    {
        let mut passwords = state.active_passwords.lock().map_err(|e| e.to_string())?;
        passwords.insert(user_id.clone(), password);
    }

    // Return public bundle
    let bundle_json = serde_json::to_string(&bundle).map_err(|e| e.to_string())?;

    Ok(bundle_json)
}

/// Tauri command: load keypair from keychain (on login)
#[tauri::command]
async fn cmd_load_keypair(
    state: tauri::State<'_, AppState>,
    user_id: String,
    password: String,
) -> Result<String, String> {
    tracing::info!("Loading keypair for user: {}", user_id);

    let keychain = secure_messenger_lib::Keychain::new();

    // Load keypair with password
    let keypair = keychain
        .load_keypair_with_password(&user_id, &password)
        .map_err(|e| format!("Failed to load keypair: {}", e))?;

    // Verify password hash
    let stored_hash = keychain
        .load_password_hash(&user_id)
        .map_err(|e| format!("No password hash found: {}", e))?;

    use argon2::password_hash::PasswordHash;
    use argon2::{Argon2, PasswordVerifier};
    let argon2_inst = Argon2::default();
    let ph =
        PasswordHash::new(&stored_hash).map_err(|e| format!("Invalid password hash: {}", e))?;

    argon2_inst
        .verify_password(password.as_bytes(), &ph)
        .map_err(|_| "Invalid password".to_string())?;

    // Add to active keypairs
    {
        let mut keypairs = state.active_keypairs.lock().map_err(|e| e.to_string())?;
        keypairs.insert(user_id.clone(), keypair);
    }
    {
        let mut passwords = state.active_passwords.lock().map_err(|e| e.to_string())?;
        passwords.insert(user_id.clone(), password);
    }

    // Return public bundle
    let bundle = keychain
        .load_public_bundle(&user_id)
        .map_err(|e| e.to_string())?;
    let bundle_json = serde_json::to_string(&bundle).map_err(|e| e.to_string())?;

    Ok(bundle_json)
}

/// Tauri command: get library version
#[tauri::command]
fn cmd_version() -> String {
    secure_messenger_lib::VERSION.to_string()
}

/// Tauri command: initialize ad state with db path and worker URL
#[tauri::command]
async fn cmd_init_ad_state(
    state: tauri::State<'_, secure_messenger_lib::AdState>,
    db_path: String,
    worker_url: String,
    client_public_key: String,
) -> Result<(), String> {
    tracing::info!(
        "Initializing ad state: db={}, worker_url={}",
        db_path,
        worker_url
    );

    state
        .init(&db_path, &worker_url, &client_public_key)
        .await
        .map_err(|e| format!("Failed to init ad state: {}", e))
}

fn main() {
    init_logging();
    tracing::info!("Starting Secure Messenger v{}", env!("CARGO_PKG_VERSION"));

    // Initialize ad state
    #[cfg(feature = "tauri-commands")]
    let ad_state =
        secure_messenger_lib::AdState::new(secure_messenger_lib::AdPreferences::default());

    // Initialize application state
    let app_state = AppState {
        active_keypairs: Mutex::new(HashMap::new()),
        active_passwords: Mutex::new(HashMap::new()),
    };

    let mut builder = tauri::Builder::default()
        .manage(app_state)
        .manage(ad_state.clone())
        .invoke_handler(tauri::generate_handler![
            cmd_generate_keypair,
            cmd_encrypt,
            cmd_decrypt,
            cmd_store_keypair,
            cmd_load_keypair,
            cmd_version,
            cmd_init_ad_state,
        ]);

    // Register Web3 commands if web3 feature is enabled
    #[cfg(feature = "web3")]
    {
        use secure_messenger_lib::web3;

        builder = web3::swap_commands::register_swap_commands(builder);
        builder = web3::abcex_commands::register_abcex_commands(builder);
        builder = web3::bitget_commands::register_bitget_commands(builder);
        // Register wallet commands
        builder = web3::wallet::register_all_wallet_commands(builder);
        // Register MetaMask commands
        builder = web3::metamask::register_metamask_commands(builder);
        // Register transaction commands
        builder = web3::transaction_commands::register_transaction_commands(builder);
    }

    // Register ad commands if tauri-commands feature is enabled
    #[cfg(feature = "tauri-commands")]
    {
        builder = secure_messenger_lib::register_ad_commands(builder, ad_state);
    }

    builder
        .setup(|_app| {
            tracing::info!("App setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
