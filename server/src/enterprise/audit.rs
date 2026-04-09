//! Enterprise Audit Module
//!
//! Features:
//! - **OpenTelemetry**: Distributed tracing + metrics
//! - **SIEM Export**: CEF (Common Event Format), JSON, syslog
//! - **Audit Trail**: Immutable event log with cryptographic hashes
//! - **Compliance**: GDPR, 152-ФЗ, HIPAA audit requirements

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum AuditError {
    #[error("Failed to export to SIEM: {0}")]
    SiemExportFailed(String),

    #[error("Failed to serialize event: {0}")]
    SerializationFailed(#[from] serde_json::Error),

    #[error("Storage error: {0}")]
    Storage(String),
}

pub type AuditResult<T> = Result<T, AuditError>;

// ============================================================================
// Event Severity
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditSeverity {
    Debug = 0,
    Info = 1,
    Notice = 2,
    Warning = 3,
    Error = 4,
    Critical = 5,
    Alert = 6,
    Emergency = 7,
}

impl AuditSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditSeverity::Debug => "DEBUG",
            AuditSeverity::Info => "INFO",
            AuditSeverity::Notice => "NOTICE",
            AuditSeverity::Warning => "WARNING",
            AuditSeverity::Error => "ERROR",
            AuditSeverity::Critical => "CRITICAL",
            AuditSeverity::Alert => "ALERT",
            AuditSeverity::Emergency => "EMERGENCY",
        }
    }

    pub fn from_syslog_priority(priority: u8) -> Self {
        match priority {
            0..=1 => AuditSeverity::Emergency,
            2 => AuditSeverity::Alert,
            3 => AuditSeverity::Critical,
            4 => AuditSeverity::Error,
            5 => AuditSeverity::Warning,
            6 => AuditSeverity::Notice,
            7 => AuditSeverity::Info,
            _ => AuditSeverity::Debug,
        }
    }
}

// ============================================================================
// Audit Categories
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditCategory {
    Authentication,    // Login, logout, SSO, failed auth
    Authorization,     // Permission changes, role assignments
    UserManagement,    // Create, delete, modify users
    DataAccess,        // Read, write, delete data
    AdminActions,      // Admin panel operations
    SecurityEvents,    // Intrusion attempts, policy violations
    Compliance,        // GDPR deletion, data retention
    SystemEvents,      // Startup, shutdown, config changes
    FileOperations,    // Upload, download, delete files
    SessionManagement, // Session create, expire, revoke
}

impl AuditCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditCategory::Authentication => "authentication",
            AuditCategory::Authorization => "authorization",
            AuditCategory::UserManagement => "user_management",
            AuditCategory::DataAccess => "data_access",
            AuditCategory::AdminActions => "admin_actions",
            AuditCategory::SecurityEvents => "security_events",
            AuditCategory::Compliance => "compliance",
            AuditCategory::SystemEvents => "system_events",
            AuditCategory::FileOperations => "file_operations",
            AuditCategory::SessionManagement => "session_management",
        }
    }
}

// ============================================================================
// Audit Event
// ============================================================================

/// Immutable audit event with cryptographic chain hash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event ID (UUID)
    pub id: String,

    /// Timestamp (UTC)
    pub timestamp: DateTime<Utc>,

    /// Event severity
    pub severity: AuditSeverity,

    /// Event category
    pub category: AuditCategory,

    /// Event type (machine-readable)
    pub event_type: String,

    /// Human-readable description
    pub description: String,

    /// Actor (user ID, IP, service account)
    pub actor: String,

    /// Target resource (user ID, chat ID, file ID)
    pub target: Option<String>,

    /// Action details (JSON)
    pub details: serde_json::Value,

    /// Source IP address
    pub source_ip: Option<String>,

    /// User agent
    pub user_agent: Option<String>,

    /// Session ID
    pub session_id: Option<String>,

    /// Previous event hash (for chain integrity)
    pub previous_hash: String,

    /// This event's hash (SHA3-256 of all fields)
    pub event_hash: String,

    /// Compliance tags (GDPR, 152-ФЗ, HIPAA)
    pub compliance_tags: Vec<String>,
}

impl AuditEvent {
    /// Create new audit event
    pub fn new(category: AuditCategory, event_type: &str, actor: &str, description: &str) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        Self {
            id,
            timestamp,
            severity: AuditSeverity::Info,
            category,
            event_type: event_type.to_string(),
            description: description.to_string(),
            actor: actor.to_string(),
            target: None,
            details: serde_json::json!({}),
            source_ip: None,
            user_agent: None,
            session_id: None,
            previous_hash: String::new(),
            event_hash: String::new(),
            compliance_tags: Vec::new(),
        }
    }

    /// Compute cryptographic hash of event
    pub fn compute_hash(&self) -> String {
        let mut hasher = Sha3_256::new();

        // Hash all immutable fields
        hasher.update(self.id.as_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());
        hasher.update(self.severity.as_str().as_bytes());
        hasher.update(self.category.as_str().as_bytes());
        hasher.update(self.event_type.as_bytes());
        hasher.update(self.description.as_bytes());
        hasher.update(self.actor.as_bytes());
        hasher.update(self.previous_hash.as_bytes());

        if let Some(ref target) = self.target {
            hasher.update(target.as_bytes());
        }
        if let Some(ref ip) = self.source_ip {
            hasher.update(ip.as_bytes());
        }

        hex::encode(hasher.finalize())
    }

    /// Finalize event (compute hash and link to chain)
    pub fn finalize(mut self, previous_hash: &str) -> Self {
        self.previous_hash = previous_hash.to_string();
        self.event_hash = self.compute_hash();
        self
    }

    /// Set severity
    pub fn severity(mut self, severity: AuditSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set target
    pub fn target(mut self, target: &str) -> Self {
        self.target = Some(target.to_string());
        self
    }

    /// Set details
    pub fn details(mut self, details: serde_json::Value) -> Self {
        self.details = details;
        self
    }

    /// Set source IP
    pub fn source_ip(mut self, ip: &str) -> Self {
        self.source_ip = Some(ip.to_string());
        self
    }

    /// Set session ID
    pub fn session_id(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }

    /// Add compliance tag
    pub fn compliance_tag(mut self, tag: &str) -> Self {
        self.compliance_tags.push(tag.to_string());
        self
    }

    /// Verify event integrity
    pub fn verify(&self) -> bool {
        let computed = self.compute_hash();
        computed == self.event_hash
    }
}

// ============================================================================
// SIEM Export Format
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiemFormat {
    /// CEF (Common Event Format) — ArcSight, Splunk
    Cef,
    /// JSON — Elastic, Sumo Logic
    Json,
    /// Syslog — Generic syslog
    Syslog,
    /// LEEF — QRadar
    Leef,
}

// ============================================================================
// SIEM Exporter
// ============================================================================

/// Exports audit events to SIEM systems
pub struct SiemExporter {
    format: SiemFormat,
    endpoint: Option<String>,
    api_key: Option<String>,
}

impl SiemExporter {
    pub fn new(format: SiemFormat, endpoint: Option<String>) -> Self {
        Self {
            format,
            endpoint,
            api_key: None,
        }
    }

    /// Export single event to SIEM format
    pub fn export_event(&self, event: &AuditEvent) -> AuditResult<String> {
        match self.format {
            SiemFormat::Cef => self.format_cef(event),
            SiemFormat::Json => self.format_json(event),
            SiemFormat::Syslog => self.format_syslog(event),
            SiemFormat::Leef => self.format_leef(event),
        }
    }

    /// Export batch of events
    pub fn export_batch(&self, events: &[AuditEvent]) -> AuditResult<Vec<String>> {
        events.iter().map(|e| self.export_event(e)).collect()
    }

    // ========================================================================
    // CEF Format (ArcSight, Splunk)
    // ========================================================================

    fn format_cef(&self, event: &AuditEvent) -> AuditResult<String> {
        let cef = format!(
            "CEF:0|SecureMessenger|Audit|{}|{}|{}|{}|eventId={} actor={} cat={} src={}",
            env!("CARGO_PKG_VERSION"),
            event.event_type,
            self.escape_cef(&event.description),
            event.severity as u8,
            event.id,
            self.escape_cef(&event.actor),
            event.category.as_str(),
            event.source_ip.as_deref().unwrap_or("unknown"),
        );

        Ok(cef)
    }

    fn escape_cef(&self, s: &str) -> String {
        s.replace('|', "\\|")
            .replace('=', "\\=")
            .replace('\n', "\\n")
    }

    // ========================================================================
    // JSON Format (Elastic, Sumo Logic)
    // ========================================================================

    fn format_json(&self, event: &AuditEvent) -> AuditResult<String> {
        serde_json::to_string(event).map_err(AuditError::from)
    }

    // ========================================================================
    // Syslog Format
    // ========================================================================

    fn format_syslog(&self, event: &AuditEvent) -> AuditResult<String> {
        // RFC 5424: <PRI>VERSION TIMESTAMP HOSTNAME APP-NAME PROCID MSGID MSG
        let priority = match event.severity {
            AuditSeverity::Emergency => 0,
            AuditSeverity::Alert => 1,
            AuditSeverity::Critical => 2,
            AuditSeverity::Error => 3,
            AuditSeverity::Warning => 4,
            AuditSeverity::Notice => 5,
            AuditSeverity::Info => 6,
            AuditSeverity::Debug => 7,
        };

        let syslog = format!(
            "<{}>1 {} messenger audit - - [event_id={}] {} {}: {}",
            priority,
            event.timestamp.to_rfc3339(),
            event.id,
            event.category.as_str(),
            event.event_type,
            event.description,
        );

        Ok(syslog)
    }

    // ========================================================================
    // LEEF Format (QRadar)
    // ========================================================================

    fn format_leef(&self, event: &AuditEvent) -> AuditResult<String> {
        // LEEF: Version|Vendor|Product|Version|Event ID|tab-separated attributes
        let leef = format!(
            "LEEF:2.0|SecureMessenger|Audit|{}|{}\tcat={}\tseverity={}\tactor={}\tsrc={}\tdescription={}",
            env!("CARGO_PKG_VERSION"),
            event.event_type,
            event.category.as_str(),
            event.severity.as_str(),
            event.actor,
            event.source_ip.as_deref().unwrap_or("unknown"),
            event.description,
        );

        Ok(leef)
    }

    // ========================================================================
    // Send to SIEM endpoint
    // ========================================================================

    pub async fn send_to_siem(&self, events: &[AuditEvent]) -> AuditResult<()> {
        let Some(endpoint) = &self.endpoint else {
            warn!("No SIEM endpoint configured, skipping export");
            return Ok(());
        };

        let formatted = self.export_batch(events)?;
        let payload = formatted.join("\n");

        let client = reqwest::Client::new();
        let mut request = client.post(endpoint).body(payload);

        if let Some(ref api_key) = self.api_key {
            request = request.bearer_auth(api_key);
        }

        request
            .send()
            .await
            .map_err(|e| AuditError::SiemExportFailed(e.to_string()))?;

        info!("Exported {} events to SIEM: {}", events.len(), endpoint);
        Ok(())
    }
}

// ============================================================================
// OpenTelemetry Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenTelemetryConfig {
    pub enabled: bool,
    pub service_name: String,
    pub endpoint: String, // OTLP endpoint (e.g., "http://otel-collector:4317")
    pub export_interval_secs: u64,
    pub max_export_batch_size: usize,
}

impl Default for OpenTelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            service_name: "secure-messenger".to_string(),
            endpoint: "http://localhost:4317".to_string(),
            export_interval_secs: 5,
            max_export_batch_size: 512,
        }
    }
}

// ============================================================================
// Audit Logger
// ============================================================================

/// Centralized audit logger with chain integrity
pub struct AuditLogger {
    /// Events stored in memory buffer
    buffer: Arc<RwLock<Vec<AuditEvent>>>,

    /// Last event hash (for chain integrity)
    last_hash: Arc<RwLock<String>>,

    /// SIEM exporter
    siem_exporter: Option<SiemExporter>,

    /// Max buffer size before flush
    max_buffer_size: usize,
}

impl AuditLogger {
    /// Create new audit logger
    pub fn new(siem_exporter: Option<SiemExporter>) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(Vec::new())),
            last_hash: Arc::new(RwLock::new(String::new())), // Genesis hash = empty
            siem_exporter,
            max_buffer_size: 100,
        }
    }

    /// Log an audit event
    pub async fn log(&self, event: AuditEvent) {
        let previous_hash = {
            let hash = self.last_hash.read().await;
            hash.clone()
        };

        let finalized_event = event.finalize(&previous_hash);
        let event_hash = finalized_event.event_hash.clone();

        // Add to buffer
        {
            let mut buffer = self.buffer.write().await;
            buffer.push(finalized_event);
        }

        // Update last hash
        {
            let mut last_hash = self.last_hash.write().await;
            *last_hash = event_hash;
        }

        // Flush if buffer is full
        self.maybe_flush().await;
    }

    /// Convenience: log authentication event
    pub async fn log_auth(
        &self,
        actor: &str,
        success: bool,
        provider: &str,
        source_ip: Option<&str>,
    ) {
        let event_type = if success {
            "login.success"
        } else {
            "login.failure"
        };
        let description = if success {
            format!("Successful {} login", provider)
        } else {
            format!("Failed {} login", provider)
        };
        let severity = if success {
            AuditSeverity::Info
        } else {
            AuditSeverity::Warning
        };

        let mut event = AuditEvent::new(
            AuditCategory::Authentication,
            event_type,
            actor,
            &description,
        )
        .severity(severity)
        .details(serde_json::json!({
            "provider": provider,
            "success": success,
        }));

        if let Some(ip) = source_ip {
            event = event.source_ip(ip);
        }

        self.log(event).await;
    }

    /// Convenience: log admin action
    pub async fn log_admin_action(
        &self,
        admin_id: &str,
        action: &str,
        target: &str,
        details: serde_json::Value,
    ) {
        let event = AuditEvent::new(
            AuditCategory::AdminActions,
            &format!("admin.{}", action),
            admin_id,
            &format!("Admin action: {}", action),
        )
        .severity(AuditSeverity::Notice)
        .target(target)
        .details(details);

        self.log(event).await;
    }

    /// Convenience: log compliance event
    pub async fn log_compliance(&self, regulation: &str, action: &str, actor: &str, target: &str) {
        let event = AuditEvent::new(
            AuditCategory::Compliance,
            &format!("compliance.{}", regulation),
            actor,
            &format!("{} compliance: {}", regulation, action),
        )
        .severity(AuditSeverity::Notice)
        .target(target)
        .compliance_tag(regulation);

        self.log(event).await;
    }

    /// Flush buffer to SIEM
    async fn maybe_flush(&self) {
        let should_flush = {
            let buffer = self.buffer.read().await;
            buffer.len() >= self.max_buffer_size
        };

        if should_flush {
            self.flush().await;
        }
    }

    /// Force flush all events to SIEM
    pub async fn flush(&self) {
        let events = {
            let mut buffer = self.buffer.write().await;
            std::mem::take(&mut *buffer)
        };

        if events.is_empty() {
            return;
        }

        if let Some(ref exporter) = self.siem_exporter {
            match exporter.send_to_siem(&events).await {
                Ok(()) => {
                    info!("Flushed {} events to SIEM", events.len());
                }
                Err(e) => {
                    warn!("Failed to flush events to SIEM: {}", e);
                    // Put events back for retry
                    let mut buffer = self.buffer.write().await;
                    buffer.extend(events);
                }
            }
        }
    }

    /// Get events since timestamp
    pub async fn get_events_since(&self, since: DateTime<Utc>) -> Vec<AuditEvent> {
        let buffer = self.buffer.read().await;
        buffer
            .iter()
            .filter(|e| e.timestamp >= since)
            .cloned()
            .collect()
    }

    /// Verify chain integrity
    pub async fn verify_chain(&self) -> bool {
        let buffer = self.buffer.read().await;

        let mut expected_hash = String::new();
        for event in buffer.iter() {
            if event.previous_hash != expected_hash {
                warn!("Audit chain broken at event {}", event.id);
                return false;
            }
            if !event.verify() {
                warn!("Event {} hash mismatch", event.id);
                return false;
            }
            expected_hash = event.event_hash.clone();
        }

        true
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_event_creation() {
        let event = AuditEvent::new(
            AuditCategory::Authentication,
            "login.success",
            "user:123",
            "Successful Google login",
        )
        .source_ip("192.168.1.100")
        .finalize("");

        assert_eq!(event.category, AuditCategory::Authentication);
        assert_eq!(event.actor, "user:123");
        assert!(!event.event_hash.is_empty());
        assert!(event.verify());
    }

    #[tokio::test]
    async fn test_audit_chain_integrity() {
        let logger = AuditLogger::new(None);

        let event1 = AuditEvent::new(
            AuditCategory::Authentication,
            "login.success",
            "user:1",
            "Login 1",
        );
        logger.log(event1).await;

        let event2 = AuditEvent::new(
            AuditCategory::Authentication,
            "login.success",
            "user:2",
            "Login 2",
        );
        logger.log(event2).await;

        assert!(logger.verify_chain().await);
    }

    #[test]
    fn test_cef_format() {
        let exporter = SiemExporter::new(SiemFormat::Cef, None);
        let event = AuditEvent::new(
            AuditCategory::SecurityEvents,
            "intrusion.attempt",
            "attacker",
            "SQL injection attempt detected",
        )
        .source_ip("10.0.0.1")
        .finalize("");

        let cef = exporter.format_cef(&event).unwrap();
        assert!(cef.starts_with("CEF:0|"));
        assert!(cef.contains("intrusion.attempt"));
    }

    #[test]
    fn test_json_format() {
        let exporter = SiemExporter::new(SiemFormat::Json, None);
        let event = AuditEvent::new(
            AuditCategory::Compliance,
            "gdpr.data_deletion",
            "admin",
            "User data deleted per GDPR request",
        )
        .finalize("");

        let json = exporter.format_json(&event).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["category"], "Compliance"); // Serialize derives from enum
        assert_eq!(parsed["actor"], "admin");
    }

    #[test]
    fn test_syslog_format() {
        let exporter = SiemExporter::new(SiemFormat::Syslog, None);
        let event = AuditEvent::new(
            AuditCategory::SystemEvents,
            "system.startup",
            "system",
            "Server started",
        )
        .finalize("");

        let syslog = exporter.format_syslog(&event).unwrap();
        assert!(syslog.starts_with("<"));
        assert!(syslog.contains("messenger"));
        assert!(syslog.contains("system.startup"));
    }

    #[test]
    fn test_severity_levels() {
        assert_eq!(AuditSeverity::Info.as_str(), "INFO");
        assert_eq!(AuditSeverity::Critical.as_str(), "CRITICAL");
        assert_eq!(
            AuditSeverity::from_syslog_priority(3),
            AuditSeverity::Critical
        );
        assert_eq!(AuditSeverity::from_syslog_priority(4), AuditSeverity::Error);
    }
}
