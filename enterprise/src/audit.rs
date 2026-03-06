//! Модуль аудита и логирования
//! 
//! Функции:
//! - Централизованное логирование всех событий
//! - Аудит действий пользователей и администраторов
//! - Отчёты для compliance
//! - Экспорт логов в SIEM системы

use sqlx::{PgPool, Row};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{info, warn, error};

/// Типы событий аудита
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum AuditEvent {
    /// Вход пользователя
    UserLogin {
        user_id: String,
        ip_address: String,
        user_agent: String,
        success: bool,
        method: String, // password, sso, ldap
    },
    /// Выход пользователя
    UserLogout {
        user_id: String,
        session_id: String,
    },
    /// Создание сообщения
    MessageCreated {
        message_id: String,
        chat_id: String,
        user_id: String,
        encrypted: bool,
    },
    /// Удаление сообщения
    MessageDeleted {
        message_id: String,
        chat_id: String,
        user_id: String,
        reason: Option<String>,
    },
    /// Изменение настроек
    SettingsChanged {
        user_id: String,
        setting_name: String,
        old_value: Option<String>,
        new_value: Option<String>,
    },
    /// Действие администратора
    AdminAction {
        admin_id: String,
        action: String,
        target_user_id: Option<String>,
        details: Option<String>,
    },
    /// Экспорт данных
    DataExport {
        user_id: String,
        export_type: String,
        records_count: u64,
    },
    /// Попытка нарушения политики
    PolicyViolation {
        user_id: String,
        policy_name: String,
        description: String,
        severity: Severity,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Запись аудита в БД
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuditRecord {
    pub id: i64,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub details: serde_json::Value,
    pub severity: String,
}

/// Логгер аудита
pub struct AuditLogger {
    db: PgPool,
}

impl AuditLogger {
    pub async fn new(db: &PgPool) -> Result<Self, Box<dyn std::error::Error>> {
        // Создание таблицы аудита
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS audit_logs (
                id BIGSERIAL PRIMARY KEY,
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                event_type VARCHAR(100) NOT NULL,
                user_id VARCHAR(255),
                ip_address INET,
                details JSONB NOT NULL,
                severity VARCHAR(20) NOT NULL DEFAULT 'info'
            )
            "#
        ).execute(db).await?;

        // Индексы для быстрого поиска
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_audit_user_id ON audit_logs(user_id);
            CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_logs(timestamp);
            CREATE INDEX IF NOT EXISTS idx_audit_event_type ON audit_logs(event_type);
            CREATE INDEX IF NOT EXISTS idx_audit_severity ON audit_logs(severity);
            "#
        ).execute(db).await?;

        Ok(AuditLogger { db: db.clone() })
    }

    /// Логирование события
    pub async fn log(&self, event: AuditEvent) -> Result<i64, Box<dyn std::error::Error>> {
        let (event_type, details, severity) = self.serialize_event(&event);
        
        let user_id = self.extract_user_id(&event);
        let severity_str = match severity {
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        };

        let result = sqlx::query(
            r#"
            INSERT INTO audit_logs (event_type, user_id, details, severity)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#
        )
        .bind(&event_type)
        .bind(&user_id)
        .bind(&details)
        .bind(severity_str)
        .fetch_one(&self.db)
        .await?;

        Ok(result.get(0))
    }

    /// Поиск записей аудита
    pub async fn query_logs(
        &self,
        user_id: Option<&str>,
        event_type: Option<&str>,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
        limit: i64,
    ) -> Result<Vec<AuditRecord>, Box<dyn std::error::Error>> {
        let mut query = String::from("SELECT * FROM audit_logs WHERE 1=1");
        
        if let Some(uid) = user_id {
            query.push_str(&format!(" AND user_id = '{}'", uid));
        }
        
        if let Some(et) = event_type {
            query.push_str(&format!(" AND event_type = '{}'", et));
        }
        
        if let Some(from_time) = from {
            query.push_str(&format!(" AND timestamp >= '{}'", from_time));
        }
        
        if let Some(to_time) = to {
            query.push_str(&format!(" AND timestamp <= '{}'", to_time));
        }
        
        query.push_str(&format!(" ORDER BY timestamp DESC LIMIT {}", limit));

        let records = sqlx::query_as::<_, AuditRecord>(&query)
            .fetch_all(&self.db)
            .await?;

        Ok(records)
    }

    /// Экспорт логов для compliance
    pub async fn export_compliance_report(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<ComplianceReport, Box<dyn std::error::Error>> {
        let records = self.query_logs(None, None, Some(from), Some(to), 100000).await?;
        
        let mut report = ComplianceReport {
            period_start: from,
            period_end: to,
            total_events: records.len() as u64,
            events_by_type: std::collections::HashMap::new(),
            events_by_user: std::collections::HashMap::new(),
            violations: vec![],
        };

        for record in &records {
            *report.events_by_type.entry(record.event_type.clone()).or_insert(0) += 1;
            
            if let Some(uid) = &record.user_id {
                *report.events_by_user.entry(uid.clone()).or_insert(0) += 1;
            }

            if record.severity == "high" || record.severity == "critical" {
                report.violations.push(record.clone());
            }
        }

        Ok(report)
    }

    fn serialize_event(&self, event: &AuditEvent) -> (String, serde_json::Value, Severity) {
        match event {
            AuditEvent::UserLogin { user_id, ip_address, user_agent, success, method } => {
                let severity = if *success { Severity::Low } else { Severity::Medium };
                ("user_login".into(), serde_json::json!({
                    "user_id": user_id,
                    "ip_address": ip_address,
                    "user_agent": user_agent,
                    "success": success,
                    "method": method,
                }), severity)
            }
            AuditEvent::MessageCreated { message_id, chat_id, user_id, encrypted } => {
                ("message_created".into(), serde_json::json!({
                    "message_id": message_id,
                    "chat_id": chat_id,
                    "user_id": user_id,
                    "encrypted": encrypted,
                }), Severity::Low)
            }
            AuditEvent::PolicyViolation { user_id, policy_name, description, severity } => {
                ("policy_violation".into(), serde_json::json!({
                    "user_id": user_id,
                    "policy_name": policy_name,
                    "description": description,
                }), severity.clone())
            }
            _ => ("other".into(), serde_json::json!({}), Severity::Low),
        }
    }

    fn extract_user_id(&self, event: &AuditEvent) -> Option<String> {
        match event {
            AuditEvent::UserLogin { user_id, .. } => Some(user_id.clone()),
            AuditEvent::MessageCreated { user_id, .. } => Some(user_id.clone()),
            AuditEvent::PolicyViolation { user_id, .. } => Some(user_id.clone()),
            _ => None,
        }
    }
}

/// Отчёт compliance
#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_events: u64,
    pub events_by_type: std::collections::HashMap<String, u64>,
    pub events_by_user: std::collections::HashMap<String, u64>,
    pub violations: Vec<AuditRecord>,
}

/// Middleware для логирования запросов
pub async fn log_request<B>(
    audit_logger: axum::extract::State<crate::AppState>,
    req: axum::http::Request<B>,
    next: axum::middleware::Next<B>,
) -> axum::http::Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = req.headers().clone();
    
    let response = next.run(req).await;
    
    let status = response.status();
    
    tracing::info!(
        method = %method,
        uri = %uri,
        status = %status,
        "HTTP request"
    );

    response
}
