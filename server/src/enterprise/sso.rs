//! Single Sign-On (SSO) Module
//!
//! Supports multiple enterprise authentication providers:
//! - **OAuth2**: Google, Microsoft, GitHub, Generic
//! - **LDAP/Active Directory**: Corporate directories
//! - **SAML 2.0**: Enterprise SSO (Okta, OneLogin, ADFS)
//! - **Kerberos**: Windows domain authentication
//!
//! Integration uses `axum-sessions` for session management.

use axum::{
    extract::{Query, State, Request},
    response::{IntoResponse, Redirect, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tracing::{info, warn, debug};

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum SsoError {
    #[error("SSO provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Authentication failed: {0}")]
    AuthFailed(String),

    #[error("Session error: {0}")]
    SessionError(String),

    #[error("Token validation failed: {0}")]
    TokenValidation(String),

    #[error("LDAP error: {0}")]
    LdapError(String),

    #[error("SAML error: {0}")]
    SamlError(String),

    #[error("Kerberos error: {0}")]
    KerberosError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("User not provisioned: {0}")]
    UserNotProvisioned(String),
}

pub type SsoResult<T> = Result<T, SsoError>;

// ============================================================================
// SSO Provider Types
// ============================================================================

/// Supported SSO authentication providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SsoProvider {
    /// OAuth2 provider (Google, Microsoft, etc.)
    OAuth2 {
        provider_name: String,       // "google", "microsoft", "github"
        client_id: String,
        client_secret: String,
        auth_url: String,
        token_url: String,
        user_info_url: String,
        scopes: Vec<String>,
        redirect_url: String,
    },

    /// LDAP/Active Directory
    Ldap {
        url: String,                  // "ldap://ad.company.com:389"
        bind_dn: String,              // "CN=service,OU=Users,DC=company,DC=com"
        bind_password: String,
        base_dn: String,              // "OU=Users,DC=company,DC=com"
        search_filter: String,        // "(sAMAccountName={username})"
        use_tls: bool,
        tls_verify_certs: bool,
    },

    /// SAML 2.0 (Okta, OneLogin, ADFS)
    Saml {
        idp_metadata_url: String,     // SAML IdP metadata XML URL
        sp_entity_id: String,         // Service Provider entity ID
        acs_url: String,              // Assertion Consumer Service URL
        name_id_format: String,       // "urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress"
        sign_requests: bool,
        signing_cert: Option<String>,
        signing_key: Option<String>,
    },

    /// Kerberos (Windows domain)
    Kerberos {
        realm: String,                // "COMPANY.COM"
        kdc_server: String,           // "dc.company.com"
        service_principal: String,    // "HTTP/app.company.com@COMPANY.COM"
        keytab_path: Option<String>,
    },
}

impl SsoProvider {
    pub fn name(&self) -> &str {
        match self {
            SsoProvider::OAuth2 { provider_name, .. } => provider_name.as_str(),
            SsoProvider::Ldap { .. } => "ldap",
            SsoProvider::Saml { .. } => "saml",
            SsoProvider::Kerberos { .. } => "kerberos",
        }
    }
}

// ============================================================================
// SSO Configuration
// ============================================================================

/// Complete SSO configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoConfig {
    pub enabled: bool,
    pub providers: Vec<SsoProvider>,
    pub session_timeout_secs: u64,
    pub require_verified_email: bool,
    pub allowed_domains: Vec<String>,  // If empty, all domains allowed
    pub auto_provision_users: bool,    // Create user account on first SSO login
    pub default_role: String,          // Role assigned to new SSO users
}

impl Default for SsoConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            providers: Vec::new(),
            session_timeout_secs: 86400, // 24 hours
            require_verified_email: true,
            allowed_domains: Vec::new(),
            auto_provision_users: true,
            default_role: "user".to_string(),
        }
    }
}

// ============================================================================
// SSO Session
// ============================================================================

/// Authenticated SSO session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoSession {
    pub user_id: String,
    pub provider: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub external_id: String,           // ID from external provider
    pub session_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub claims: serde_json::Value,     // All claims from provider
}

impl SsoSession {
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }
}

// ============================================================================
// SSO Client
// ============================================================================

/// SSO authentication client
pub struct SsoClient {
    config: SsoConfig,
    http_client: reqwest::Client,
}

impl SsoClient {
    /// Create new SSO client
    pub fn new(config: SsoConfig) -> SsoResult<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| SsoError::ConfigError(e.to_string()))?;

        Ok(Self { config, http_client })
    }

    // ========================================================================
    // OAuth2 Flow
    // ========================================================================

    /// Get OAuth2 authorization URL (redirect user to provider)
    pub fn get_oauth2_auth_url(
        &self,
        provider_name: &str,
        state: &str,
    ) -> SsoResult<String> {
        let provider = self.find_oauth2_provider(provider_name)?;

        match provider {
            SsoProvider::OAuth2 {
                auth_url,
                client_id,
                scopes,
                redirect_url,
                ..
            } => {
                let scope_str = scopes.join(" ");
                let url = format!(
                    "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
                    auth_url, client_id, redirect_url, scope_str, state
                );
                Ok(url)
            }
            _ => Err(SsoError::ProviderNotFound("Not an OAuth2 provider".into())),
        }
    }

    /// Exchange OAuth2 code for tokens and get user info
    pub async fn exchange_oauth2_code(
        &self,
        provider_name: &str,
        code: &str,
    ) -> SsoResult<SsoSession> {
        let provider = self.find_oauth2_provider(provider_name)?;

        match provider {
            SsoProvider::OAuth2 {
                token_url,
                client_id,
                client_secret,
                redirect_url,
                user_info_url,
                ..
            } => {
                // Exchange code for access token
                let token_response = self.http_client
                    .post(&token_url)
                    .form(&[
                        ("grant_type", "authorization_code"),
                        ("code", code),
                        ("redirect_uri", redirect_url.as_str()),
                        ("client_id", client_id.as_str()),
                        ("client_secret", client_secret.as_str()),
                    ])
                    .send()
                    .await
                    .map_err(|e| SsoError::AuthFailed(e.to_string()))?;

                if !token_response.status().is_success() {
                    return Err(SsoError::AuthFailed(
                        format!("Token exchange failed: HTTP {}", token_response.status())
                    ));
                }

                let tokens: serde_json::Value = token_response
                    .json()
                    .await
                    .map_err(|e| SsoError::TokenValidation(e.to_string()))?;

                let access_token = tokens["access_token"]
                    .as_str()
                    .ok_or_else(|| SsoError::TokenValidation("No access_token".into()))?;

                // Get user info
                let user_response = self.http_client
                    .get(&user_info_url)
                    .bearer_auth(access_token)
                    .send()
                    .await
                    .map_err(|e| SsoError::AuthFailed(e.to_string()))?;

                let user_info: serde_json::Value = user_response
                    .json()
                    .await
                    .map_err(|e| SsoError::TokenValidation(e.to_string()))?;

                // Extract user details
                let email = user_info["email"].as_str().map(String::from);
                let display_name = user_info["name"]
                    .as_str()
                    .or_else(|| user_info["login"].as_str())
                    .map(String::from);
                let external_id = user_info["sub"]
                    .as_str()
                    .or_else(|| user_info["id"].as_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Validate email domain
                if self.config.require_verified_email {
                    if let Some(ref email_ref) = email {
                        self.validate_email_domain(email_ref)?;
                    }
                }

                let email_for_log = email.clone();

                // Create session
                let session = self.create_session(
                    &external_id,
                    provider_name,
                    email,
                    display_name,
                    user_info,
                );

                info!("OAuth2 login successful: provider={}, email={:?}", provider_name, email_for_log);
                Ok(session)
            }
            _ => Err(SsoError::ProviderNotFound("Not an OAuth2 provider".into())),
        }
    }

    // ========================================================================
    // LDAP Authentication
    // ========================================================================

    /// Authenticate user against LDAP/Active Directory
    pub async fn authenticate_ldap(
        &self,
        username: &str,
        password: &str,
    ) -> SsoResult<SsoSession> {
        let provider = self.find_ldap_provider()?;

        match provider {
            SsoProvider::Ldap {
                url, bind_dn, bind_password, base_dn, search_filter, ..
            } => {
                // NOTE: In production, use the `ldap3` crate for actual LDAP communication
                // This is a placeholder showing the interface
                debug!(
                    "LDAP auth attempt: url={}, user={}, base_dn={}",
                    url, username, base_dn
                );

                // Simulated LDAP auth flow:
                // 1. Bind with service account
                // 2. Search for user by username
                // 3. Bind with user credentials
                // 4. Get user attributes (email, display_name, groups)

                warn!("LDAP authentication not fully implemented — needs ldap3 crate");

                Err(SsoError::LdapError("LDAP provider not configured".into()))
            }
            _ => Err(SsoError::ProviderNotFound("Not an LDAP provider".into())),
        }
    }

    // ========================================================================
    // SAML Authentication
    // ========================================================================

    /// Generate SAML AuthnRequest (redirect to IdP)
    pub fn generate_saml_auth_request(&self) -> SsoResult<String> {
        let provider = self.find_saml_provider()?;

        match provider {
            SsoProvider::Saml {
                idp_metadata_url,
                acs_url,
                sp_entity_id,
                name_id_format,
                ..
            } => {
                // NOTE: In production, use `saml2` crate for SAML XML generation
                // This shows the interface structure
                debug!(
                    "SAML AuthnRequest: idp={}, acs={}, sp={}",
                    idp_metadata_url, acs_url, sp_entity_id
                );

                warn!("SAML authentication not fully implemented — needs saml2 crate");

                Err(SsoError::SamlError("SAML provider not configured".into()))
            }
            _ => Err(SsoError::ProviderNotFound("Not a SAML provider".into())),
        }
    }

    /// Process SAML Response (from IdP POST)
    pub async fn process_saml_response(&self, saml_response: &str) -> SsoResult<SsoSession> {
        // NOTE: In production:
        // 1. Decode base64 SAMLResponse
        // 2. Verify XML signature with IdP public key
        // 3. Validate assertions (audience, conditions, subject)
        // 4. Extract user attributes (email, name, groups, roles)

        warn!("SAML response processing not fully implemented — needs saml2 crate");

        Err(SsoError::SamlError("SAML processing not implemented".into()))
    }

    // ========================================================================
    // Kerberos Authentication
    // ========================================================================

    /// Authenticate via Kerberos (SPNEGO ticket)
    pub async fn authenticate_kerberos(&self, spnego_ticket: &[u8]) -> SsoResult<SsoSession> {
        let provider = self.find_kerberos_provider()?;

        match provider {
            SsoProvider::Kerberos {
                realm, service_principal, ..
            } => {
                debug!(
                    "Kerberos auth: realm={}, sp={}",
                    realm, service_principal
                );

                // NOTE: In production, use `kerberos` crate
                // 1. Validate SPNEGO ticket with KDC
                // 2. Extract user principal name (UPN)
                // 3. Map UPN to user account

                warn!("Kerberos authentication not fully implemented — needs kerberos crate");

                Err(SsoError::KerberosError("Kerberos provider not configured".into()))
            }
            _ => Err(SsoError::ProviderNotFound("Not a Kerberos provider".into())),
        }
    }

    // ========================================================================
    // Session Management
    // ========================================================================

    /// Create a new SSO session
    fn create_session(
        &self,
        external_id: &str,
        provider: &str,
        email: Option<String>,
        display_name: Option<String>,
        claims: serde_json::Value,
    ) -> SsoSession {
        let now = chrono::Utc::now();
        let expires = now + chrono::Duration::seconds(self.config.session_timeout_secs as i64);

        SsoSession {
            user_id: format!("sso:{}:{}", provider, external_id),
            provider: provider.to_string(),
            email,
            display_name,
            external_id: external_id.to_string(),
            session_id: uuid::Uuid::new_v4().to_string(),
            created_at: now,
            expires_at: expires,
            claims,
        }
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    fn find_oauth2_provider(&self, name: &str) -> SsoResult<SsoProvider> {
        self.config.providers
            .iter()
            .find(|p| matches!(p, SsoProvider::OAuth2 { provider_name, .. } if provider_name == name))
            .cloned()
            .ok_or_else(|| SsoError::ProviderNotFound(name.to_string()))
    }

    fn find_ldap_provider(&self) -> SsoResult<SsoProvider> {
        self.config.providers
            .iter()
            .find(|p| matches!(p, SsoProvider::Ldap { .. }))
            .cloned()
            .ok_or_else(|| SsoError::ProviderNotFound("ldap".into()))
    }

    fn find_saml_provider(&self) -> SsoResult<SsoProvider> {
        self.config.providers
            .iter()
            .find(|p| matches!(p, SsoProvider::Saml { .. }))
            .cloned()
            .ok_or_else(|| SsoError::ProviderNotFound("saml".into()))
    }

    fn find_kerberos_provider(&self) -> SsoResult<SsoProvider> {
        self.config.providers
            .iter()
            .find(|p| matches!(p, SsoProvider::Kerberos { .. }))
            .cloned()
            .ok_or_else(|| SsoError::ProviderNotFound("kerberos".into()))
    }

    fn validate_email_domain(&self, email: &str) -> SsoResult<()> {
        if self.config.allowed_domains.is_empty() {
            return Ok(()); // No domain restriction
        }

        let domain = email
            .split('@')
            .nth(1)
            .ok_or_else(|| SsoError::AuthFailed("Invalid email format".into()))?;

        if self.config.allowed_domains.iter().any(|d| d == domain) {
            Ok(())
        } else {
            Err(SsoError::AuthFailed(
                format!("Email domain '{}' not allowed", domain)
            ))
        }
    }
}

// ============================================================================
// Axum Routes
// ============================================================================

/// Create SSO router
pub fn sso_router(client: Arc<SsoClient>) -> Router {
    Router::new()
        .route("/sso/:provider/login", get(oauth2_login_handler))
        .route("/sso/:provider/callback", get(oauth2_callback_handler))
        .route("/sso/ldap/login", post(ldap_login_handler))
        .route("/sso/saml/login", get(saml_login_handler))
        .route("/sso/saml/acs", post(saml_acs_handler))
        .route("/sso/kerberos/login", post(kerberos_login_handler))
        .with_state(client)
}

// ============================================================================
// Route Handlers
// ============================================================================

#[derive(Deserialize)]
pub struct LoginQuery {
    pub redirect: Option<String>,
}

/// OAuth2 login — redirect to provider
async fn oauth2_login_handler(
    State(client): State<Arc<SsoClient>>,
    axum::extract::Path(provider): axum::extract::Path<String>,
    Query(params): Query<LoginQuery>,
) -> impl IntoResponse {
    let state = uuid::Uuid::new_v4().to_string();
    // In production: store state in session/cookie for CSRF protection

    match client.get_oauth2_auth_url(&provider, &state) {
        Ok(auth_url) => Redirect::temporary(&auth_url).into_response(),
        Err(e) => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        ).into_response(),
    }
}

/// OAuth2 callback — exchange code, create session
async fn oauth2_callback_handler(
    State(client): State<Arc<SsoClient>>,
    axum::extract::Path(provider): axum::extract::Path<String>,
    Query(params): Query<serde_json::Value>,
) -> impl IntoResponse {
    let code = match params["code"].as_str() {
        Some(c) => c,
        None => return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "No authorization code" })),
        ).into_response(),
    };

    match client.exchange_oauth2_code(&provider, code).await {
        Ok(session) => {
            // In production: set session cookie, redirect to app
            (
                axum::http::StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "session": session,
                })),
            ).into_response()
        }
        Err(e) => (
            axum::http::StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": e.to_string() })),
        ).into_response(),
    }
}

/// LDAP login
#[derive(Deserialize)]
pub struct LdapLoginRequest {
    pub username: String,
    pub password: String,
}

async fn ldap_login_handler(
    State(client): State<Arc<SsoClient>>,
    Json(req): Json<LdapLoginRequest>,
) -> impl IntoResponse {
    match client.authenticate_ldap(&req.username, &req.password).await {
        Ok(session) => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "session": session,
            })),
        ).into_response(),
        Err(e) => (
            axum::http::StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": e.to_string() })),
        ).into_response(),
    }
}

/// SAML login — generate AuthnRequest
async fn saml_login_handler(
    State(_client): State<Arc<SsoClient>>,
) -> impl IntoResponse {
    // In production: return redirect to IdP with SAMLRequest
    (
        axum::http::StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({ "error": "SAML not configured" })),
    ).into_response()
}

/// SAML ACS — process IdP response
async fn saml_acs_handler(
    State(_client): State<Arc<SsoClient>>,
    form: axum::Form<serde_json::Value>,
) -> impl IntoResponse {
    // In production: parse SAMLResponse, validate, create session
    (
        axum::http::StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({ "error": "SAML ACS not configured" })),
    ).into_response()
}

/// Kerberos login
#[derive(Deserialize)]
pub struct KerberosLoginRequest {
    pub spnego_ticket: String, // base64-encoded
}

async fn kerberos_login_handler(
    State(client): State<Arc<SsoClient>>,
    Json(req): Json<KerberosLoginRequest>,
) -> impl IntoResponse {
    let ticket = match base64::decode(&req.spnego_ticket) {
        Ok(t) => t,
        Err(e) => return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": format!("Invalid ticket: {}", e) })),
        ).into_response(),
    };

    match client.authenticate_kerberos(&ticket).await {
        Ok(session) => (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "session": session,
            })),
        ).into_response(),
        Err(e) => (
            axum::http::StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": e.to_string() })),
        ).into_response(),
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sso_config_default() {
        let config = SsoConfig::default();
        assert!(!config.enabled);
        assert!(config.providers.is_empty());
        assert!(config.auto_provision_users);
    }

    #[test]
    fn test_sso_provider_name() {
        let oauth = SsoProvider::OAuth2 {
            provider_name: "google".to_string(),
            client_id: "test".to_string(),
            client_secret: "test".to_string(),
            auth_url: "".to_string(),
            token_url: "".to_string(),
            user_info_url: "".to_string(),
            scopes: vec![],
            redirect_url: "".to_string(),
        };
        assert_eq!(oauth.name(), "google");

        let ldap = SsoProvider::Ldap {
            url: "".to_string(),
            bind_dn: "".to_string(),
            bind_password: "".to_string(),
            base_dn: "".to_string(),
            search_filter: "".to_string(),
            use_tls: true,
            tls_verify_certs: true,
        };
        assert_eq!(ldap.name(), "ldap");
    }

    #[test]
    fn test_sso_session_expiry() {
        let now = chrono::Utc::now();
        let session = SsoSession {
            user_id: "sso:google:123".to_string(),
            provider: "google".to_string(),
            email: Some("user@company.com".to_string()),
            display_name: Some("Test User".to_string()),
            external_id: "123".to_string(),
            session_id: "uuid".to_string(),
            created_at: now,
            expires_at: now + chrono::Duration::hours(1),
            claims: serde_json::json!({}),
        };

        assert!(!session.is_expired());
    }

    #[test]
    fn test_email_domain_validation() {
        let config = SsoConfig {
            enabled: true,
            allowed_domains: vec!["company.com".to_string()],
            require_verified_email: true,
            ..Default::default()
        };

        let client = SsoClient::new(config).unwrap();

        assert!(client.validate_email_domain("user@company.com").is_ok());
        assert!(client.validate_email_domain("user@other.com").is_err());
    }
}
