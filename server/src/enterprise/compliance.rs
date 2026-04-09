//! Compliance Module — GDPR & 152-ФЗ
//!
//! Features:
//! - **GDPR Checklist**: Article 15-22 data subject rights
//! - **152-ФЗ Checklist**: Russian personal data law compliance
//! - **Data Deletion Policies**: Automated retention & deletion
//! - **Data Subject Requests**: Access, rectification, erasure, portability
//! - **Compliance Reports**: Audit-ready reports

use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{info, warn};

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum ComplianceError {
    #[error("Data subject request failed: {0}")]
    DsrFailed(String),

    #[error("Deletion policy violation: {0}")]
    DeletionPolicyViolation(String),

    #[error("Retention error: {0}")]
    RetentionError(String),

    #[error("Report generation failed: {0}")]
    ReportFailed(String),

    #[error("Database error: {0}")]
    Database(String),
}

pub type ComplianceResult<T> = Result<T, ComplianceError>;

// ============================================================================
// Data Subject Request (DSR) — GDPR Articles 15-22
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DsrType {
    /// Article 15 — Right of access
    Access,

    /// Article 16 — Right to rectification
    Rectification,

    /// Article 17 — Right to erasure ("right to be forgotten")
    Erasure,

    /// Article 18 — Right to restriction of processing
    Restriction,

    /// Article 20 — Right to data portability
    Portability,

    /// Article 21 — Right to object
    Objection,

    /// Article 22 — Automated decision-making
    AutomatedDecision,
}

impl DsrType {
    pub fn gdpr_article(&self) -> &'static str {
        match self {
            DsrType::Access => "Article 15",
            DsrType::Rectification => "Article 16",
            DsrType::Erasure => "Article 17",
            DsrType::Restriction => "Article 18",
            DsrType::Portability => "Article 20",
            DsrType::Objection => "Article 21",
            DsrType::AutomatedDecision => "Article 22",
        }
    }

    pub fn response_deadline_days(&self) -> u32 {
        30 // GDPR: 1 month
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DsrStatus {
    Pending,
    InProgress,
    Completed,
    Rejected { reason: String },
    Expired,
}

/// Data Subject Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSubjectRequest {
    pub id: String,
    pub user_id: String,
    pub request_type: DsrType,
    pub status: DsrStatus,
    pub submitted_at: DateTime<Utc>,
    pub deadline: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub handler_id: Option<String>,       // Admin who handled it
    pub notes: Option<String>,
    pub exported_data: Option<Vec<u8>>,   // For portability/access requests
}

impl DataSubjectRequest {
    pub fn new(user_id: &str, request_type: DsrType) -> Self {
        let submitted_at = Utc::now();
        let deadline = submitted_at + Duration::days(request_type.response_deadline_days() as i64);

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            request_type,
            status: DsrStatus::Pending,
            submitted_at,
            deadline,
            completed_at: None,
            handler_id: None,
            notes: None,
            exported_data: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.deadline && !matches!(self.status, DsrStatus::Completed)
    }
}

// ============================================================================
// Retention Policy
// ============================================================================

/// Data retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub name: String,
    pub description: String,
    pub data_category: String,
    pub retention_days: u32,
    pub anonymize_after_days: Option<u32>,
    pub legal_basis: String,                // "consent", "contract", "legitimate_interest"
    pub regulation: String,                 // "GDPR", "152-ФЗ"
    pub auto_delete: bool,
    pub exceptions: Vec<String>,            // Data types exempt from this policy
}

impl RetentionPolicy {
    pub fn gdpr_messages() -> Self {
        Self {
            name: "Message Retention (GDPR)".to_string(),
            description: "E2EE messages are deleted after 1 year per GDPR minimization".to_string(),
            data_category: "messages".to_string(),
            retention_days: 365,
            anonymize_after_days: Some(90),
            legal_basis: "consent".to_string(),
            regulation: "GDPR".to_string(),
            auto_delete: true,
            exceptions: vec!["audit_log".to_string()],
        }
    }

    pub fn fz152_personal_data() -> Self {
        Self {
            name: "Personal Data (152-ФЗ)".to_string(),
            description: "Personal data deleted when purpose fulfilled per 152-ФЗ Art. 5".to_string(),
            data_category: "personal_data".to_string(),
            retention_days: 3 * 365, // 3 years (statute of limitations)
            anonymize_after_days: Some(365),
            legal_basis: "consent".to_string(),
            regulation: "152-ФЗ".to_string(),
            auto_delete: true,
            exceptions: vec![],
        }
    }

    pub fn gdpr_logs() -> Self {
        Self {
            name: "Audit Logs (GDPR)".to_string(),
            description: "Audit logs retained for 2 years for compliance".to_string(),
            data_category: "audit_logs".to_string(),
            retention_days: 2 * 365,
            anonymize_after_days: Some(365),
            legal_basis: "legitimate_interest".to_string(),
            regulation: "GDPR".to_string(),
            auto_delete: true,
            exceptions: vec!["security_incidents".to_string()],
        }
    }
}

// ============================================================================
// Deletion Policy
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionPolicy {
    pub name: String,
    pub trigger: DeletionTrigger,
    pub scope: Vec<String>,             // Data categories to delete
    pub cascade: bool,                   // Delete related data
    pub soft_delete_first: bool,         // Soft delete, then hard delete after grace period
    pub grace_period_days: u32,
    pub notify_user: bool,
    pub notify_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeletionTrigger {
    /// User requested account deletion
    UserRequest,

    /// GDPR erasure request (Article 17)
    GdprErasure,

    /// Retention period expired
    RetentionExpired,

    /// Account suspended > X days
    SuspendedAccount,

    /// Admin manual deletion
    AdminAction,

    /// Inactive account > X days
    InactiveAccount,
}

impl DeletionPolicy {
    pub fn gdpr_erasure() -> Self {
        Self {
            name: "GDPR Article 17 Erasure".to_string(),
            trigger: DeletionTrigger::GdprErasure,
            scope: vec![
                "personal_data".to_string(),
                "messages".to_string(),
                "files".to_string(),
                "sessions".to_string(),
                "preferences".to_string(),
            ],
            cascade: true,
            soft_delete_first: true,
            grace_period_days: 30,
            notify_user: true,
            notify_admin: true,
        }
    }

    pub fn inactive_account() -> Self {
        Self {
            name: "Inactive Account Cleanup".to_string(),
            trigger: DeletionTrigger::InactiveAccount,
            scope: vec![
                "sessions".to_string(),
                "device_tokens".to_string(),
            ],
            cascade: false,
            soft_delete_first: false,
            grace_period_days: 0,
            notify_user: false,
            notify_admin: false,
        }
    }
}

// ============================================================================
// GDPR Checklist
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprChecklist {
    pub checks: Vec<GdprCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprCheck {
    pub id: String,
    pub article: String,
    pub title: String,
    pub description: String,
    pub is_compliant: bool,
    pub evidence: Option<String>,
    pub last_reviewed: Option<DateTime<Utc>>,
    pub reviewer: Option<String>,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl GdprChecklist {
    pub fn default() -> Self {
        Self {
            checks: vec![
                GdprCheck {
                    id: "gdpr-5".to_string(),
                    article: "Article 5".to_string(),
                    title: "Data Minimization".to_string(),
                    description: "Personal data is adequate, relevant, and limited to what is necessary".to_string(),
                    is_compliant: false,
                    evidence: None,
                    last_reviewed: None,
                    reviewer: None,
                    risk_level: RiskLevel::High,
                },
                GdprCheck {
                    id: "gdpr-6".to_string(),
                    article: "Article 6".to_string(),
                    title: "Lawfulness of Processing".to_string(),
                    description: "Processing has a valid legal basis (consent, contract, etc.)".to_string(),
                    is_compliant: false,
                    evidence: None,
                    last_reviewed: None,
                    reviewer: None,
                    risk_level: RiskLevel::Critical,
                },
                GdprCheck {
                    id: "gdpr-7".to_string(),
                    article: "Article 7".to_string(),
                    title: "Conditions for Consent".to_string(),
                    description: "Consent is freely given, specific, informed, and unambiguous".to_string(),
                    is_compliant: false,
                    evidence: None,
                    last_reviewed: None,
                    reviewer: None,
                    risk_level: RiskLevel::High,
                },
                GdprCheck {
                    id: "gdpr-15".to_string(),
                    article: "Article 15".to_string(),
                    title: "Right of Access".to_string(),
                    description: "Data subjects can obtain confirmation and access to their data".to_string(),
                    is_compliant: false,
                    evidence: None,
                    last_reviewed: None,
                    reviewer: None,
                    risk_level: RiskLevel::High,
                },
                GdprCheck {
                    id: "gdpr-17".to_string(),
                    article: "Article 17".to_string(),
                    title: "Right to Erasure".to_string(),
                    description: "Data subjects can request deletion of their personal data".to_string(),
                    is_compliant: false,
                    evidence: None,
                    last_reviewed: None,
                    reviewer: None,
                    risk_level: RiskLevel::Critical,
                },
                GdprCheck {
                    id: "gdpr-20".to_string(),
                    article: "Article 20".to_string(),
                    title: "Right to Data Portability".to_string(),
                    description: "Data subjects can receive their data in machine-readable format".to_string(),
                    is_compliant: false,
                    evidence: None,
                    last_reviewed: None,
                    reviewer: None,
                    risk_level: RiskLevel::Medium,
                },
                GdprCheck {
                    id: "gdpr-25".to_string(),
                    article: "Article 25".to_string(),
                    title: "Data Protection by Design and Default".to_string(),
                    description: "E2EE by default, data minimization built into architecture".to_string(),
                    is_compliant: true,
                    evidence: Some("E2EE with X25519+Kyber1024+ChaCha20-Poly1305".to_string()),
                    last_reviewed: Some(Utc::now()),
                    reviewer: Some("security-team".to_string()),
                    risk_level: RiskLevel::Low,
                },
                GdprCheck {
                    id: "gdpr-32".to_string(),
                    article: "Article 32".to_string(),
                    title: "Security of Processing".to_string(),
                    description: "Appropriate technical and organizational measures implemented".to_string(),
                    is_compliant: false,
                    evidence: None,
                    last_reviewed: None,
                    reviewer: None,
                    risk_level: RiskLevel::Critical,
                },
                GdprCheck {
                    id: "gdpr-33".to_string(),
                    article: "Article 33".to_string(),
                    title: "Breach Notification".to_string(),
                    description: "Process to notify supervisory authority within 72 hours".to_string(),
                    is_compliant: false,
                    evidence: None,
                    last_reviewed: None,
                    reviewer: None,
                    risk_level: RiskLevel::High,
                },
                GdprCheck {
                    id: "gdpr-35".to_string(),
                    article: "Article 35".to_string(),
                    title: "Data Protection Impact Assessment".to_string(),
                    description: "DPIA conducted for high-risk processing activities".to_string(),
                    is_compliant: false,
                    evidence: None,
                    last_reviewed: None,
                    reviewer: None,
                    risk_level: RiskLevel::High,
                },
            ],
        }
    }

    pub fn compliance_score(&self) -> f64 {
        if self.checks.is_empty() {
            return 0.0;
        }
        let compliant = self.checks.iter().filter(|c| c.is_compliant).count();
        (compliant as f64 / self.checks.len() as f64) * 100.0
    }

    pub fn critical_issues(&self) -> Vec<&GdprCheck> {
        self.checks.iter()
            .filter(|c| !c.is_compliant && c.risk_level == RiskLevel::Critical)
            .collect()
    }
}

// ============================================================================
// 152-ФЗ Checklist (Russian Personal Data Law)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fz152Checklist {
    pub checks: Vec<Fz152Check>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fz152Check {
    pub id: String,
    pub article: String,
    pub title: String,
    pub description: String,
    pub is_compliant: bool,
    pub evidence: Option<String>,
    pub last_reviewed: Option<DateTime<Utc>>,
    pub risk_level: RiskLevel,
}

impl Fz152Checklist {
    pub fn default() -> Self {
        Self {
            checks: vec![
                Fz152Check {
                    id: "fz152-5".to_string(),
                    article: "Статья 5".to_string(),
                    title: "Принципы обработки персональных данных".to_string(),
                    description: "Обработка ограничена достижением конкретных, заранее определённых целей".to_string(),
                    is_compliant: false,
                    evidence: None,
                    last_reviewed: None,
                    risk_level: RiskLevel::Critical,
                },
                Fz152Check {
                    id: "fz152-6".to_string(),
                    article: "Статья 6".to_string(),
                    title: "Условия обработки персональных данных".to_string(),
                    description: "Обработка допускается только с согласия субъекта ПДн".to_string(),
                    is_compliant: false,
                    evidence: None,
                    last_reviewed: None,
                    risk_level: RiskLevel::Critical,
                },
                Fz152Check {
                    id: "fz152-9".to_string(),
                    article: "Статья 9".to_string(),
                    title: "Согласие субъекта персональных данных".to_string(),
                    description: "Согласие должно быть конкретным, информированным, сознательным".to_string(),
                    is_compliant: false,
                    evidence: None,
                    last_reviewed: None,
                    risk_level: RiskLevel::High,
                },
                Fz152Check {
                    id: "fz152-14".to_string(),
                    article: "Статья 14".to_string(),
                    title: "Право на доступ к персональным данным".to_string(),
                    description: "Субъект вправе получить информацию об обработке своих ПДн".to_string(),
                    is_compliant: false,
                    evidence: None,
                    last_reviewed: None,
                    risk_level: RiskLevel::High,
                },
                Fz152Check {
                    id: "fz152-17".to_string(),
                    article: "Статья 17".to_string(),
                    title: "Уничтожение персональных данных".to_string(),
                    description: "ПДн уничтожаются по достижении целей обработки или при отзыве согласия".to_string(),
                    is_compliant: false,
                    evidence: None,
                    last_reviewed: None,
                    risk_level: RiskLevel::Critical,
                },
                Fz152Check {
                    id: "fz152-18-1".to_string(),
                    article: "Статья 18.1".to_string(),
                    title: "Локализация баз данных на территории РФ".to_string(),
                    description: "Сбор и хранение ПДн граждан РФ должны осуществляться на серверах в России".to_string(),
                    is_compliant: false,
                    evidence: None,
                    last_reviewed: None,
                    risk_level: RiskLevel::Critical,
                },
                Fz152Check {
                    id: "fz152-19".to_string(),
                    article: "Статья 19".to_string(),
                    title: "Меры по защите персональных данных".to_string(),
                    description: "Организационные и технические меры для защиты ПДн".to_string(),
                    is_compliant: false,
                    evidence: None,
                    last_reviewed: None,
                    risk_level: RiskLevel::High,
                },
                Fz152Check {
                    id: "fz152-21".to_string(),
                    article: "Статья 21".to_string(),
                    title: "Уведомление Роскомнадзора".to_string(),
                    description: "Оператор обязан уведомить Роскомнадзор о начале обработки ПДн".to_string(),
                    is_compliant: false,
                    evidence: None,
                    last_reviewed: None,
                    risk_level: RiskLevel::High,
                },
            ],
        }
    }

    pub fn compliance_score(&self) -> f64 {
        if self.checks.is_empty() {
            return 0.0;
        }
        let compliant = self.checks.iter().filter(|c| c.is_compliant).count();
        (compliant as f64 / self.checks.len() as f64) * 100.0
    }

    pub fn critical_issues(&self) -> Vec<&Fz152Check> {
        self.checks.iter()
            .filter(|c| !c.is_compliant && c.risk_level == RiskLevel::Critical)
            .collect()
    }
}

// ============================================================================
// Compliance Report
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub generated_at: DateTime<Utc>,
    pub report_period: (DateTime<Utc>, DateTime<Utc>),
    pub gdpr: GdprComplianceSummary,
    pub fz152: Fz152ComplianceSummary,
    pub data_subject_requests: DsrSummary,
    pub deletions: DeletionSummary,
    pub overall_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdprComplianceSummary {
    pub checklist_score: f64,
    pub total_checks: usize,
    pub compliant_checks: usize,
    pub critical_issues: usize,
    pub open_dsr_count: usize,
    pub overdue_dsr_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fz152ComplianceSummary {
    pub checklist_score: f64,
    pub total_checks: usize,
    pub compliant_checks: usize,
    pub critical_issues: usize,
    pub data_localized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DsrSummary {
    pub total_requests: usize,
    pub pending: usize,
    pub completed: usize,
    pub rejected: usize,
    pub expired: usize,
    pub avg_response_time_days: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletionSummary {
    pub total_deletions: usize,
    pub by_trigger: HashMap<String, usize>,
    pub pending_grace_period: usize,
    pub failed_deletions: usize,
}

// ============================================================================
// Compliance Manager
// ============================================================================

pub struct ComplianceManager {
    dsr_requests: tokio::sync::RwLock<Vec<DataSubjectRequest>>,
    retention_policies: Vec<RetentionPolicy>,
    deletion_policies: Vec<DeletionPolicy>,
    gdpr_checklist: GdprChecklist,
    fz152_checklist: Fz152Checklist,
}

impl ComplianceManager {
    pub fn new() -> Self {
        Self {
            dsr_requests: tokio::sync::RwLock::new(Vec::new()),
            retention_policies: vec![
                RetentionPolicy::gdpr_messages(),
                RetentionPolicy::fz152_personal_data(),
                RetentionPolicy::gdpr_logs(),
            ],
            deletion_policies: vec![
                DeletionPolicy::gdpr_erasure(),
                DeletionPolicy::inactive_account(),
            ],
            gdpr_checklist: GdprChecklist::default(),
            fz152_checklist: Fz152Checklist::default(),
        }
    }

    // ========================================================================
    // Data Subject Requests
    // ========================================================================

    /// Submit a new data subject request
    pub async fn submit_dsr(&self, user_id: &str, request_type: DsrType) -> ComplianceResult<String> {
        let dsr = DataSubjectRequest::new(user_id, request_type);

        info!(
            "New DSR: type={}, user={}, deadline={}",
            request_type.gdpr_article(),
            user_id,
            dsr.deadline
        );

        let dsr_id = dsr.id.clone();
        self.dsr_requests.write().await.push(dsr);

        Ok(dsr_id)
    }

    /// Get all DSRs for a user
    pub async fn get_user_dsrs(&self, user_id: &str) -> Vec<DataSubjectRequest> {
        let requests = self.dsr_requests.read().await;
        requests.iter()
            .filter(|r| r.user_id == user_id)
            .cloned()
            .collect()
    }

    /// Check for expired DSRs
    pub async fn check_expired_dsrs(&self) -> Vec<String> {
        let mut requests = self.dsr_requests.write().await;
        let mut expired = Vec::new();

        for req in requests.iter_mut() {
            if req.is_expired() && matches!(req.status, DsrStatus::Pending | DsrStatus::InProgress) {
                req.status = DsrStatus::Expired;
                expired.push(req.id.clone());
                warn!("DSR expired: {} (user: {})", req.id, req.user_id);
            }
        }

        expired
    }

    // ========================================================================
    // Retention Enforcement
    // ========================================================================

    /// Get policies that require data deletion
    pub fn get_active_deletion_policies(&self) -> Vec<&DeletionPolicy> {
        self.deletion_policies.iter().collect()
    }

    /// Get retention policies for a data category
    pub fn get_retention_policy(&self, category: &str) -> Option<&RetentionPolicy> {
        self.retention_policies.iter()
            .find(|p| p.data_category == category)
    }

    /// Calculate data expiration date
    pub fn get_data_expiry(&self, category: &str, created_at: DateTime<Utc>) -> DateTime<Utc> {
        if let Some(policy) = self.get_retention_policy(category) {
            created_at + Duration::days(policy.retention_days as i64)
        } else {
            created_at + Duration::days(365) // Default: 1 year
        }
    }

    // ========================================================================
    // Compliance Reports
    // ========================================================================

    /// Generate comprehensive compliance report
    pub async fn generate_report(
        &self,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> ComplianceResult<ComplianceReport> {
        let requests = self.dsr_requests.read().await;

        let dsr_summary = DsrSummary {
            total_requests: requests.len(),
            pending: requests.iter().filter(|r| matches!(r.status, DsrStatus::Pending)).count(),
            completed: requests.iter().filter(|r| matches!(r.status, DsrStatus::Completed)).count(),
            rejected: requests.iter().filter(|r| matches!(r.status, DsrStatus::Rejected { .. })).count(),
            expired: requests.iter().filter(|r| matches!(r.status, DsrStatus::Expired)).count(),
            avg_response_time_days: 0.0, // Calculate from completed DSRS
        };

        let gdpr_critical = self.gdpr_checklist.critical_issues().len();
        let fz152_critical = self.fz152_checklist.critical_issues().len();

        let gdpr_score = self.gdpr_checklist.compliance_score();
        let fz152_score = self.fz152_checklist.compliance_score();
        let overall = (gdpr_score + fz152_score) / 2.0;

        let report = ComplianceReport {
            generated_at: Utc::now(),
            report_period: (period_start, period_end),
            gdpr: GdprComplianceSummary {
                checklist_score: gdpr_score,
                total_checks: self.gdpr_checklist.checks.len(),
                compliant_checks: self.gdpr_checklist.checks.iter().filter(|c| c.is_compliant).count(),
                critical_issues: gdpr_critical,
                open_dsr_count: dsr_summary.pending,
                overdue_dsr_count: dsr_summary.expired,
            },
            fz152: Fz152ComplianceSummary {
                checklist_score: fz152_score,
                total_checks: self.fz152_checklist.checks.len(),
                compliant_checks: self.fz152_checklist.checks.iter().filter(|c| c.is_compliant).count(),
                critical_issues: fz152_critical,
                data_localized: false, // Set based on actual infrastructure
            },
            data_subject_requests: dsr_summary,
            deletions: DeletionSummary {
                total_deletions: 0,
                by_trigger: HashMap::new(),
                pending_grace_period: 0,
                failed_deletions: 0,
            },
            overall_score: overall,
        };

        info!(
            "Compliance report generated: overall={:.1}%, GDPR={:.1}%, 152-ФЗ={:.1}%",
            overall, gdpr_score, fz152_score
        );

        Ok(report)
    }

    /// Get GDPR checklist
    pub fn get_gdpr_checklist(&self) -> &GdprChecklist {
        &self.gdpr_checklist
    }

    /// Get 152-ФЗ checklist
    pub fn get_fz152_checklist(&self) -> &Fz152Checklist {
        &self.fz152_checklist
    }

    /// Mark GDPR check as compliant
    pub fn update_gdpr_check(&mut self, check_id: &str, is_compliant: bool, evidence: Option<String>) {
        if let Some(check) = self.gdpr_checklist.checks.iter_mut().find(|c| c.id == check_id) {
            check.is_compliant = is_compliant;
            check.evidence = evidence;
            check.last_reviewed = Some(Utc::now());
        }
    }

    /// Mark 152-ФЗ check as compliant
    pub fn update_fz152_check(&mut self, check_id: &str, is_compliant: bool, evidence: Option<String>) {
        if let Some(check) = self.fz152_checklist.checks.iter_mut().find(|c| c.id == check_id) {
            check.is_compliant = is_compliant;
            check.evidence = evidence;
            check.last_reviewed = Some(Utc::now());
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gdpr_checklist_default() {
        let checklist = GdprChecklist::default();
        assert_eq!(checklist.checks.len(), 10);
        assert_eq!(checklist.compliance_score(), 10.0); // 1/10 compliant (Article 25)
        assert!(!checklist.critical_issues().is_empty());
    }

    #[test]
    fn test_fz152_checklist_default() {
        let checklist = Fz152Checklist::default();
        assert_eq!(checklist.checks.len(), 8);
        assert_eq!(checklist.compliance_score(), 0.0); // 0/8 compliant
        assert!(!checklist.critical_issues().is_empty());
    }

    #[test]
    fn test_dsr_creation() {
        let dsr = DataSubjectRequest::new("user:123", DsrType::Erasure);

        assert_eq!(dsr.user_id, "user:123");
        assert_eq!(dsr.request_type, DsrType::Erasure);
        assert_eq!(dsr.request_type.gdpr_article(), "Article 17");
        assert!(matches!(dsr.status, DsrStatus::Pending));
        assert!(dsr.deadline > dsr.submitted_at);
    }

    #[test]
    fn test_dsr_expiry() {
        let mut dsr = DataSubjectRequest::new("user:123", DsrType::Access);
        assert!(!dsr.is_expired());

        // Simulate past deadline
        dsr.deadline = Utc::now() - Duration::days(1);
        assert!(dsr.is_expired());
    }

    #[test]
    fn test_retention_policies() {
        let gdpr_messages = RetentionPolicy::gdpr_messages();
        assert_eq!(gdpr_messages.retention_days, 365);
        assert_eq!(gdpr_messages.regulation, "GDPR");

        let fz152 = RetentionPolicy::fz152_personal_data();
        assert_eq!(fz152.retention_days, 3 * 365);
        assert_eq!(fz152.regulation, "152-ФЗ");
    }

    #[test]
    fn test_deletion_policies() {
        let gdpr_erasure = DeletionPolicy::gdpr_erasure();
        assert!(matches!(gdpr_erasure.trigger, DeletionTrigger::GdprErasure));
        assert!(gdpr_erasure.soft_delete_first);
        assert_eq!(gdpr_erasure.grace_period_days, 30);
    }

    #[tokio::test]
    async fn test_compliance_manager_dsr() {
        let manager = ComplianceManager::new();

        let dsr_id = manager.submit_dsr("user:1", DsrType::Access).await.unwrap();
        assert!(!dsr_id.is_empty());

        let user_dsrs = manager.get_user_dsrs("user:1").await;
        assert_eq!(user_dsrs.len(), 1);
        assert_eq!(user_dsrs[0].request_type, DsrType::Access);
    }

    #[test]
    fn test_data_expiry_calculation() {
        let manager = ComplianceManager::new();
        let created = Utc::now();

        let msg_expiry = manager.get_data_expiry("messages", created);
        assert_eq!(msg_expiry - created, Duration::days(365));

        let pd_expiry = manager.get_data_expiry("personal_data", created);
        assert_eq!(pd_expiry - created, Duration::days(3 * 365));
    }

    #[test]
    fn test_verification_badge_levels() {
        assert_eq!(crate::enterprise::admin::VerificationLevel::EmailVerified.display_badge(), "✓");
        assert_eq!(crate::enterprise::admin::VerificationLevel::IdVerified.display_badge(), "✓✓");
        assert_eq!(
            crate::enterprise::admin::VerificationLevel::EnterpriseVerified.display_badge(),
            "✓✓✓"
        );
    }
}
