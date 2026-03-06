//! Bot Builder - визуальный конструктор ботов

use serde::{Deserialize, Serialize};

/// Блок конструктора
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderBlock {
    pub id: String,
    pub block_type: String,
    pub position: (i32, i32),
    pub data: serde_json::Value,
}

/// Flow конструктора
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuilderFlow {
    pub id: String,
    pub name: String,
    pub blocks: Vec<BuilderBlock>,
    pub connections: Vec<Connection>,
}

/// Соединение между блоками
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

/// Экспорт flow в JSON
pub fn export_flow(flow: &BuilderFlow) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(flow)
}

/// Импорт flow из JSON
pub fn import_flow(json: &str) -> Result<BuilderFlow, serde_json::Error> {
    serde_json::from_str(json)
}
