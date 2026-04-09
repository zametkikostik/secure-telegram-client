//! Enterprise Features Module
//!
//! Production-ready enterprise integrations:
//! - **SSO**: OAuth2, SAML, LDAP, Kerberos authentication
//! - **Audit**: OpenTelemetry tracing, SIEM export (CEF, JSON)
//! - **Admin Panel**: User management, verification badges
//! - **Compliance**: GDPR/152-ФЗ data deletion policies

pub mod admin;
pub mod audit;
pub mod compliance;
pub mod sso;

// Re-export main types
pub use admin::{
    AdminAction, AdminError, AdminResult, AdminState, AdminUser, VerificationBadge,
    VerificationLevel,
};
pub use audit::{
    AuditCategory, AuditEvent, AuditLogger, AuditSeverity, OpenTelemetryConfig, SiemExporter,
    SiemFormat,
};
pub use compliance::{
    ComplianceManager, ComplianceReport, DataSubjectRequest, DeletionPolicy, Fz152Checklist,
    GdprChecklist, RetentionPolicy,
};
pub use sso::{SsoClient, SsoConfig, SsoError, SsoProvider, SsoResult, SsoSession};

/// Enterprise feature flags
#[derive(Debug, Clone, Default)]
pub struct EnterpriseConfig {
    pub sso_enabled: bool,
    pub audit_enabled: bool,
    pub admin_panel_enabled: bool,
    pub compliance_enabled: bool,
}

impl EnterpriseConfig {
    pub fn all_enabled() -> Self {
        Self {
            sso_enabled: true,
            audit_enabled: true,
            admin_panel_enabled: true,
            compliance_enabled: true,
        }
    }

    pub fn is_any_enabled(&self) -> bool {
        self.sso_enabled
            || self.audit_enabled
            || self.admin_panel_enabled
            || self.compliance_enabled
    }
}
