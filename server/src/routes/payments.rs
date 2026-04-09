//! Payment & Subscription Routes

use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::middleware::auth::get_user_id_from_header;
use crate::AppState;
use axum::http::HeaderMap;

// ============================================================================
// Helpers
// ============================================================================

fn require_auth(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    get_user_id_from_header(auth_header, &state.auth).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Unauthorized"})),
        )
    })
}

// ============================================================================
// Types
// ============================================================================

#[derive(Deserialize)]
pub struct TipRequest {
    pub recipient_id: String,
    pub amount: f64,
    pub currency: String,
    pub message: Option<String>,
}

#[derive(Serialize)]
pub struct TipResponse {
    pub tip_id: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct CreditBalance {
    pub balance: i64,
}

// ============================================================================
// Credits
// ============================================================================

pub async fn get_credits(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<CreditBalance>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = require_auth(&headers, &state)?;

    let row = sqlx::query(
        "SELECT COALESCE(SUM(amount), 0) as bal FROM credit_transactions WHERE user_id = ?",
    )
    .bind(&user_id)
    .fetch_one(&*state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;

    let balance: i64 = row.try_get("bal").map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;

    Ok(Json(CreditBalance { balance }))
}

pub async fn get_credit_history(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = require_auth(&headers, &state)?;

    let rows = sqlx::query(
        "SELECT id, amount, balance_after, source, description, created_at
         FROM credit_transactions WHERE user_id = ?
         ORDER BY created_at DESC LIMIT 50",
    )
    .bind(&user_id)
    .fetch_all(&*state.db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;

    let result: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            let id: String = row.get("id");
            let amount: i64 = row.get("amount");
            let balance_after: i64 = row.get("balance_after");
            let source: String = row.get("source");
            let description: String = row.get("description");
            let created_at: String = row.get("created_at");
            serde_json::json!({
                "id": id, "amount": amount, "balance_after": balance_after,
                "source": source, "description": description, "created_at": created_at,
            })
        })
        .collect();

    Ok(Json(result))
}

pub async fn add_credits(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = require_auth(&headers, &state)?;

    let credits = req["credits"].as_i64().ok_or((
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"error": "Missing credits"})),
    ))?;

    let now = Utc::now().to_rfc3339();

    let row = sqlx::query(
        "SELECT COALESCE(SUM(amount), 0) as bal FROM credit_transactions WHERE user_id = ?",
    )
    .bind(&user_id)
    .fetch_one(&*state.db)
    .await
    .unwrap_or_else(|_| panic!());

    let balance: i64 = row.try_get("bal").unwrap_or(0);
    let new_balance = balance + credits;

    sqlx::query(
        "INSERT INTO credit_transactions (id, user_id, amount, balance_after, source, description, created_at)
         VALUES (?, ?, ?, ?, 'purchase', ?, ?)",
    )
    .bind(Uuid::new_v4().simple().to_string())
    .bind(&user_id)
    .bind(credits)
    .bind(new_balance)
    .bind(&format!("Purchased {} credits", credits))
    .bind(&now)
    .execute(&*state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

    Ok(Json(
        serde_json::json!({"credits_added": credits, "new_balance": new_balance}),
    ))
}

// ============================================================================
// Tipping
// ============================================================================

pub async fn send_tip(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<TipRequest>,
) -> Result<Json<TipResponse>, (StatusCode, Json<serde_json::Value>)> {
    let sender_id = require_auth(&headers, &state)?;

    let recipient_exists: Option<(String,)> = sqlx::query_as("SELECT id FROM users WHERE id = ?")
        .bind(&req.recipient_id)
        .fetch_optional(&*state.db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;

    if recipient_exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Recipient not found"})),
        ));
    }

    let tip_id = Uuid::new_v4().simple().to_string();
    let now = Utc::now().to_rfc3339();
    let tip_amount_i64 = req.amount as i64;

    let sender_row = sqlx::query(
        "SELECT COALESCE(SUM(amount), 0) as bal FROM credit_transactions WHERE user_id = ?",
    )
    .bind(&sender_id)
    .fetch_one(&*state.db)
    .await
    .unwrap_or_else(|_| panic!());

    let sender_balance: i64 = sender_row.try_get("bal").unwrap_or(0);

    if sender_balance < tip_amount_i64 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Insufficient credits"})),
        ));
    }

    sqlx::query(
        "INSERT INTO credit_transactions (id, user_id, amount, balance_after, source, description, created_at)
         VALUES (?, ?, ?, ?, 'tip', ?, ?)",
    )
    .bind(&tip_id)
    .bind(&sender_id)
    .bind(-tip_amount_i64)
    .bind(sender_balance - tip_amount_i64)
    .bind(&format!("Tip to {}", req.recipient_id))
    .bind(&now)
    .execute(&*state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

    let recipient_row = sqlx::query(
        "SELECT COALESCE(SUM(amount), 0) as bal FROM credit_transactions WHERE user_id = ?",
    )
    .bind(&req.recipient_id)
    .fetch_one(&*state.db)
    .await
    .unwrap_or_else(|_| panic!());

    let recipient_balance: i64 = recipient_row.try_get("bal").unwrap_or(0);

    sqlx::query(
        "INSERT INTO credit_transactions (id, user_id, amount, balance_after, source, description, created_at)
         VALUES (?, ?, ?, ?, 'tip', ?, ?)",
    )
    .bind(Uuid::new_v4().simple().to_string())
    .bind(&req.recipient_id)
    .bind(tip_amount_i64)
    .bind(recipient_balance + tip_amount_i64)
    .bind(&format!("Tip from {}", sender_id))
    .bind(&now)
    .execute(&*state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()}))))?;

    Ok(Json(TipResponse {
        tip_id,
        status: "completed".to_string(),
    }))
}

// ============================================================================
// Stripe Webhook
// ============================================================================

pub async fn stripe_webhook(
    State(state): State<AppState>,
    body: String,
) -> Result<StatusCode, (StatusCode, String)> {
    tracing::info!("Received Stripe webhook: {}", body);

    let event: serde_json::Value = serde_json::from_str(&body)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid JSON".to_string()))?;

    let event_type = event["type"].as_str().unwrap_or("");

    match event_type {
        "checkout.session.completed" => {
            let session = &event["data"]["object"];
            let user_id = session["metadata"]["user_id"].as_str().unwrap_or("");
            let plan_id = session["metadata"]["plan_id"].as_str().unwrap_or("");

            if !user_id.is_empty() {
                let _ = handle_successful_payment(&state.db, user_id, plan_id, session).await;
            }
        }
        "customer.subscription.deleted" => {
            let subscription = &event["data"]["object"];
            let customer_id = subscription["customer"].as_str().unwrap_or("");

            let _ = sqlx::query(
                "UPDATE subscriptions SET status = 'cancelled' WHERE stripe_customer_id = ?",
            )
            .bind(customer_id)
            .execute(&*state.db)
            .await;
        }
        _ => {}
    }

    Ok(StatusCode::OK)
}

async fn handle_successful_payment(
    db: &sqlx::SqlitePool,
    user_id: &str,
    plan_id: &str,
    session: &serde_json::Value,
) -> Result<(), String> {
    let now = Utc::now().to_rfc3339();
    let period_end = Utc::now() + chrono::Duration::days(30);

    let tier = match plan_id {
        "premium" => "premium",
        "pro" => "pro",
        _ => "free",
    };

    let stripe_sub_id = session["subscription"].as_str().unwrap_or("");

    let _ = sqlx::query(
        "INSERT INTO subscriptions (id, user_id, tier, status, current_period_start, current_period_end, stripe_subscription_id, created_at)
         VALUES (?, ?, ?, 'active', ?, ?, ?, ?)
         ON CONFLICT(user_id) DO UPDATE SET tier=?, status='active', current_period_start=?, current_period_end=?, updated_at=?",
    )
    .bind(Uuid::new_v4().simple().to_string())
    .bind(user_id)
    .bind(tier)
    .bind(&now)
    .bind(&period_end.to_rfc3339())
    .bind(stripe_sub_id)
    .bind(&now)
    .bind(tier)
    .bind(&now)
    .bind(&period_end.to_rfc3339())
    .bind(&now)
    .execute(db)
    .await;

    Ok(())
}
use sqlx::Row;
