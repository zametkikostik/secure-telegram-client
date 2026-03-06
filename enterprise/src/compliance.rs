//! Compliance модуль
//! 
//! Функции:
//! - Политики безопасности
//! - DLP (Data Loss Prevention)
//! - Отчёты для регуляторов
//! - GDPR compliance

use serde::{Deserialize, Serialize};

/// Политика безопасности
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub rules: Vec<PolicyRule>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: String,
    pub name: String,
    pub condition: PolicyCondition,
    pub action: PolicyAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PolicyCondition {
    /// Сообщение содержит запрещённые слова
    ContainsKeywords { keywords: Vec<String> },
    /// Сообщение содержит конфиденциальные данные
    ContainsSensitiveData { patterns: Vec<String> },
    /// Пользователь不在 разрешённой группы
    UserNotInGroup { group_id: String },
    /// Время вне рабочего
    OutsideWorkingHours { start: String, end: String, timezone: String },
    /// Превышен лимит сообщений
    MessageRateLimit { limit: u64, period_seconds: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PolicyAction {
    /// Заблокировать сообщение
    Block,
    /// Предупредить пользователя
    Warn { message: String },
    /// Уведомить администратора
    NotifyAdmin { admin_id: String },
    /// Записать в аудит
    AuditLog { severity: String },
    /// Зашифровать сообщение
    Encrypt,
}

/// DLP сканер
pub struct DLPSanner {
    policies: Vec<SecurityPolicy>,
}

impl DLPSanner {
    pub fn new() -> Self {
        DLPSanner { policies: vec![] }
    }

    /// Добавить политику
    pub fn add_policy(&mut self, policy: SecurityPolicy) {
        self.policies.push(policy);
    }

    /// Сканирование сообщения
    pub fn scan_message(&self, content: &str, user_id: &str) -> ScanResult {
        let mut violations = vec![];

        for policy in &self.policies {
            if !policy.enabled {
                continue;
            }

            for rule in &policy.rules {
                if self.check_condition(&rule.condition, content, user_id) {
                    violations.push(PolicyViolation {
                        policy_id: policy.id.clone(),
                        policy_name: policy.name.clone(),
                        rule_id: rule.id.clone(),
                        rule_name: rule.name.clone(),
                        action: rule.action.clone(),
                    });
                }
            }
        }

        ScanResult {
            allowed: violations.is_empty(),
            violations,
        }
    }

    fn check_condition(
        &self,
        condition: &PolicyCondition,
        content: &str,
        user_id: &str,
    ) -> bool {
        match condition {
            PolicyCondition::ContainsKeywords { keywords } => {
                keywords.iter().any(|kw| content.contains(kw))
            }
            PolicyCondition::ContainsSensitiveData { patterns } => {
                patterns.iter().any(|pattern| {
                    let regex = regex::Regex::new(pattern).ok();
                    regex.map(|r| r.is_match(content)).unwrap_or(false)
                })
            }
            PolicyCondition::UserNotInGroup { group_id } => {
                // Проверка членства в группе
                false // TODO: реализовать
            }
            PolicyCondition::OutsideWorkingHours { .. } => {
                // Проверка времени
                false // TODO: реализовать
            }
            PolicyCondition::MessageRateLimit { .. } => {
                // Проверка лимита
                false // TODO: реализовать
            }
        }
    }
}

impl Default for DLPSanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Результат сканирования
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub allowed: bool,
    pub violations: Vec<PolicyViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub policy_id: String,
    pub policy_name: String,
    pub rule_id: String,
    pub rule_name: String,
    pub action: PolicyAction,
}

/// GDPR compliance
pub struct GDPRCompliance;

impl GDPRCompliance {
    /// Экспорт данных пользователя (Right to Access)
    pub fn export_user_data(user_id: &str) -> Result<UserDataExport, Box<dyn std::error::Error>> {
        // Сбор всех данных пользователя
        Ok(UserDataExport {
            user_id: user_id.to_string(),
            exported_at: chrono::Utc::now(),
            messages: vec![],
            contacts: vec![],
            settings: serde_json::json!({}),
        })
    }

    /// Удаление данных пользователя (Right to be Forgotten)
    pub fn delete_user_data(user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Анонимизация или удаление всех данных
        // - Сообщения
        // - Контакты
        // - Настройки
        // - Аудит логи (с сохранением для compliance)
        Ok(())
    }

    /// Проверка согласия на обработку данных
    pub fn check_consent(user_id: &str, purpose: &str) -> bool {
        // Проверка наличия согласия
        true // TODO: реализовать
    }

    /// Запись согласия
    pub fn record_consent(
        user_id: &str,
        purpose: &str,
        consent_text: &str,
    ) -> Result<ConsentRecord, Box<dyn std::error::Error>> {
        Ok(ConsentRecord {
            user_id: user_id.to_string(),
            purpose: purpose.to_string(),
            consent_text: consent_text.to_string(),
            granted_at: chrono::Utc::now(),
            ip_address: String::new(), // TODO: получить из запроса
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDataExport {
    pub user_id: String,
    pub exported_at: chrono::DateTime<chrono::Utc>,
    pub messages: Vec<MessageExport>,
    pub contacts: Vec<ContactExport>,
    pub settings: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageExport {
    pub id: String,
    pub chat_id: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactExport {
    pub user_id: String,
    pub name: String,
    pub added_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub user_id: String,
    pub purpose: String,
    pub consent_text: String,
    pub granted_at: chrono::DateTime<chrono::Utc>,
    pub ip_address: String,
}

/// Отчёт для регуляторов
#[derive(Debug, Serialize, Deserialize)]
pub struct RegulatoryReport {
    pub report_type: String,
    pub period_start: chrono::DateTime<chrono::Utc>,
    pub period_end: chrono::DateTime<chrono::Utc>,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub summary: ReportSummary,
    pub incidents: Vec<IncidentReport>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_users: u64,
    pub total_messages: u64,
    pub total_violations: u64,
    pub blocked_messages: u64,
    pub data_requests: u64,
    pub deletions: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IncidentReport {
    pub id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub severity: String,
    pub description: String,
    pub affected_users: Vec<String>,
    pub actions_taken: Vec<String>,
}

/// Шаблоны политик
pub mod templates {
    use super::*;

    /// Политика запрета отправки номеров карт
    pub fn credit_card_policy() -> SecurityPolicy {
        SecurityPolicy {
            id: "policy_cc_001".into(),
            name: "Запрет номеров кредитных карт".into(),
            description: "Блокировка сообщений с номерами кредитных карт".into(),
            enabled: true,
            rules: vec![PolicyRule {
                id: "rule_cc_001".into(),
                name: "Блокировка номеров карт".into(),
                condition: PolicyCondition::ContainsSensitiveData {
                    patterns: vec![
                        r"\b\d{4}[- ]?\d{4}[- ]?\d{4}[- ]?\d{4}\b".into(),
                    ],
                },
                action: PolicyAction::Block,
            }],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    /// Политика рабочего времени
    pub fn working_hours_policy() -> SecurityPolicy {
        SecurityPolicy {
            id: "policy_wh_001".into(),
            name: "Рабочее время".into(),
            description: "Запрет сообщений вне рабочего времени".into(),
            enabled: false,
            rules: vec![PolicyRule {
                id: "rule_wh_001".into(),
                name: "Только в рабочее время".into(),
                condition: PolicyCondition::OutsideWorkingHours {
                    start: "09:00".into(),
                    end: "18:00".into(),
                    timezone: "Europe/Moscow".into(),
                },
                action: PolicyAction::Warn {
                    message: "Отправка сообщений разрешена только в рабочее время (9:00-18:00 МСК)".into(),
                },
            }],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    /// Политика ключевых слов
    pub fn keywords_policy(keywords: Vec<&str>) -> SecurityPolicy {
        SecurityPolicy {
            id: "policy_kw_001".into(),
            name: "Запрещённые слова".into(),
            description: "Блокировка сообщений с запрещёнными словами".into(),
            enabled: true,
            rules: vec![PolicyRule {
                id: "rule_kw_001".into(),
                name: "Блокировка ключевых слов".into(),
                condition: PolicyCondition::ContainsKeywords {
                    keywords: keywords.into_iter().map(String::from).collect(),
                },
                action: PolicyAction::Block,
            }],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
}
