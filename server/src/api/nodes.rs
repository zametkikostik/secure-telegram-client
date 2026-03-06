// server/src/api/nodes.rs
//! P2P Ноды — децентрализованная сеть участников

use axum::{
    extract::State,
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, DateTime};
use crate::api::AppState;

/// Информация о P2P ноде
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerNode {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub public_key: String,
    pub peer_id: String, // libp2p peer ID
    pub multiaddr: Vec<String>,
    pub status: NodeStatus,
    pub joined_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub version: String,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Online,
    Offline,
    Syncing,
}

/// Запрос на регистрацию ноды
#[derive(Debug, Deserialize)]
pub struct RegisterNodeRequest {
    pub user_id: String,
    pub username: String,
    pub public_key: String,
    pub peer_id: String,
    pub multiaddr: Vec<String>,
    pub version: String,
    pub capabilities: Vec<String>,
}

/// Список нод
#[derive(Debug, Serialize)]
pub struct PeerListResponse {
    pub nodes: Vec<PeerNode>,
    pub total: usize,
    pub online: usize,
}

/// Регистрация новой ноды
pub async fn register_node(
    State(state): State<AppState>,
    Json(req): Json<RegisterNodeRequest>,
) -> Result<Json<PeerNode>, StatusCode> {
    use sqlx::Row;
    
    let node_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    
    // Проверка существует ли уже нода
    let existing = sqlx::query(
        "SELECT id FROM peer_nodes WHERE user_id = ?"
    )
    .bind(&req.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Ошибка проверки ноды: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    if existing.is_some() {
        // Обновление существующей ноды
        sqlx::query(
            "UPDATE peer_nodes SET 
             status = 'online', 
             last_seen = ?, 
             multiaddr = ?,
             version = ?,
             capabilities = ?
             WHERE user_id = ?"
        )
        .bind(now)
        .bind(serde_json::to_string(&req.multiaddr).unwrap())
        .bind(&req.version)
        .bind(serde_json::to_string(&req.capabilities).unwrap())
        .bind(&req.user_id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Ошибка обновления ноды: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        
        // Получить обновлённую ноду
        let node = sqlx::query_as(
            "SELECT * FROM peer_nodes WHERE user_id = ?"
        )
        .bind(&req.user_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        return Ok(Json(node));
    }
    
    // Создание новой ноды
    let node = sqlx::query_as(
        "INSERT INTO peer_nodes 
         (id, user_id, username, public_key, peer_id, multiaddr, status, joined_at, last_seen, version, capabilities)
         VALUES (?, ?, ?, ?, ?, ?, 'online', ?, ?, ?, ?)"
    )
    .bind(&node_id)
    .bind(&req.user_id)
    .bind(&req.username)
    .bind(&req.public_key)
    .bind(&req.peer_id)
    .bind(serde_json::to_string(&req.multiaddr).unwrap())
    .bind(now)
    .bind(now)
    .bind(&req.version)
    .bind(serde_json::to_string(&req.capabilities).unwrap())
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Ошибка регистрации ноды: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Синхронизация с GitHub
    sync_to_github(&req.user_id, &req.username, &req.public_key).await?;
    
    Ok(Json(PeerNode {
        id: node_id,
        user_id: req.user_id,
        username: req.username,
        public_key: req.public_key,
        peer_id: req.peer_id,
        multiaddr: req.multiaddr,
        status: NodeStatus::Online,
        joined_at: now,
        last_seen: now,
        version: req.version,
        capabilities: req.capabilities,
    }))
}

/// Получить список нод
pub async fn get_peer_list(
    State(state): State<AppState>,
) -> Result<Json<PeerListResponse>, StatusCode> {
    let nodes: Vec<PeerNode> = sqlx::query_as("SELECT * FROM peer_nodes ORDER BY last_seen DESC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Ошибка получения нод: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let total = nodes.len();
    let online = nodes.iter().filter(|n| n.status == NodeStatus::Online).count();
    
    Ok(Json(PeerListResponse {
        nodes,
        total,
        online,
    }))
}

/// Синхронизация с GitHub
async fn sync_to_github(
    user_id: &str,
    username: &str,
    public_key: &str,
) -> Result<(), StatusCode> {
    let github_token = std::env::var("GITHUB_TOKEN").map_err(|_| {
        tracing::warn!("GITHUB_TOKEN не настроен, синхронизация отключена");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    let repo = std::env::var("GITHUB_REPO").unwrap_or_else(|_| 
        "zametkikostik/secure-telegram-client".to_string()
    );
    
    // Добавление пользователя в PEERS.md
    let peer_entry = format!(
        "\n## {}\n- **User ID:** {}\n- **Public Key:** {}\n- **Joined:** {}\n",
        username,
        user_id,
        public_key,
        Utc::now().to_rfc3339()
    );
    
    // GitHub API для обновления файла
    let client = reqwest::Client::new();
    
    // Получить текущий файл
    let response = client
        .get(format!("https://api.github.com/repos/{}/contents/PEERS.md", repo))
        .header("Authorization", format!("Bearer {}", github_token))
        .header("User-Agent", "LibertyReach/1.0")
        .send()
        .await;
    
    let content = match response {
        Ok(resp) => {
            let json: serde_json::Value = resp.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
            let content = json["content"].as_str().unwrap_or("");
            let decoded = base64::decode(content).unwrap_or_default();
            String::from_utf8_lossy(&decoded).to_string()
        }
        Err(_) => "# Liberty Reach Peer Network\n\nСписок участников P2P сети:\n".to_string(),
    };
    
    let new_content = format!("{}{}", content, peer_entry);
    let encoded = base64::encode(&new_content);
    
    // Обновить файл
    let _ = client
        .put(format!("https://api.github.com/repos/{}/contents/PEERS.md", repo))
        .header("Authorization", format!("Bearer {}", github_token))
        .header("User-Agent", "LibertyReach/1.0")
        .json(&serde_json::json!({
            "message": format!("Add peer: {}", username),
            "content": encoded,
        }))
        .send()
        .await;
    
    Ok(())
}

/// Обновление статуса ноды (heartbeat)
pub async fn node_heartbeat(
    State(state): State<AppState>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id = req["user_id"].as_str().ok_or(StatusCode::BAD_REQUEST)?;
    
    sqlx::query("UPDATE peer_nodes SET last_seen = ?, status = 'online' WHERE user_id = ?")
        .bind(Utc::now())
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(Json(serde_json::json!({"status": "ok"})))
}
