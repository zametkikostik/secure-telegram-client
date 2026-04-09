//! Tauri Commands — AI provider management
//!
//! These commands are called from the frontend (JavaScript/TypeScript)
//! to manage AI providers, switch between them, and check their status.

use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::ai::{AiClient, AiConfig, AiProvider, ModelHint, ModelInfo, ProviderStatus};

// ============================================================================
// State Management
// ============================================================================

/// Global AI client state (managed via Tauri state)
pub struct AiState {
    pub client: Mutex<AiClient>,
}

impl AiState {
    pub fn new() -> Self {
        let config = AiConfig::default();
        let client = AiClient::new(config);
        Self {
            client: Mutex::new(client),
        }
    }
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub name: String,
    pub available: bool,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfoDto {
    pub provider: String,
    pub model_id: String,
    pub name: String,
    pub is_free: bool,
}

// ============================================================================
// Tauri Commands
// ============================================================================

/// Get list of available AI providers
#[tauri::command]
pub async fn get_available_providers(
    state: tauri::State<'_, AiState>,
) -> Result<Vec<ProviderInfo>, String> {
    let client = state.client.lock().unwrap();
    let available = client.list_available_providers().await;
    let active = client.active_provider();

    Ok(vec![
        AiProvider::OpenRouter,
        AiProvider::Ollama,
        AiProvider::Anthropic,
        AiProvider::OpenAI,
        AiProvider::Groq,
        AiProvider::Mistral,
    ]
    .into_iter()
    .map(|p| ProviderInfo {
        name: p.to_string(),
        available: available.contains(&p),
        is_active: p == active,
    })
    .collect())
}

/// Switch the active AI provider
#[tauri::command]
pub async fn switch_ai_provider(
    provider: String,
    state: tauri::State<'_, AiState>,
) -> Result<(), String> {
    let provider = provider
        .parse::<AiProvider>()
        .map_err(|e| format!("Invalid provider: {}", e))?;

    let mut client = state.client.lock().unwrap();
    client.switch_provider(provider)
}

/// Get available models across all providers
#[tauri::command]
pub async fn get_available_models(
    state: tauri::State<'_, AiState>,
) -> Result<Vec<ModelInfoDto>, String> {
    let client = state.client.lock().unwrap();
    let models = client.list_available_models().await;

    Ok(models
        .into_iter()
        .map(|m| ModelInfoDto {
            provider: m.provider.to_string(),
            model_id: m.model_id,
            name: m.name,
            is_free: m.is_free,
        })
        .collect())
}

/// Check the status of a specific provider
#[tauri::command]
pub async fn check_provider_status(
    provider: String,
    state: tauri::State<'_, AiState>,
) -> Result<ProviderStatus, String> {
    let client = state.client.lock().unwrap();

    let provider_enum = provider
        .parse::<AiProvider>()
        .map_err(|e| format!("Invalid provider: {}", e))?;

    Ok(client.check_provider_status(provider_enum).await)
}

/// Get the currently active provider
#[tauri::command]
pub async fn get_active_provider(state: tauri::State<'_, AiState>) -> Result<String, String> {
    let client = state.client.lock().unwrap();
    Ok(client.active_provider().to_string())
}

/// Send a test message to verify the active provider works
#[tauri::command]
pub async fn test_ai_connection(state: tauri::State<'_, AiState>) -> Result<String, String> {
    let client = state.client.lock().unwrap();

    let result = client
        .send_with_hint(
            ModelHint::General,
            "You are a helpful assistant. Respond in 5 words or less.",
            "Say hello and confirm you're working.",
        )
        .await;

    result.map_err(|e| format!("AI request failed: {}", e))
}

/// Translate text using AI
#[tauri::command]
pub async fn ai_translate(
    text: String,
    from_lang: String,
    to_lang: String,
    state: tauri::State<'_, AiState>,
) -> Result<String, String> {
    let client = state.client.lock().unwrap();

    let result = client
        .send_with_hint(
            ModelHint::Translation,
            &format!(
                "You are a professional translator. Translate from {} to {}. \
                 Return ONLY the translation, no explanations.",
                from_lang, to_lang
            ),
            &text,
        )
        .await;

    result.map_err(|e| format!("Translation failed: {}", e))
}

/// Generate code from a description
#[tauri::command]
pub async fn ai_generate_code(
    description: String,
    language: String,
    state: tauri::State<'_, AiState>,
) -> Result<String, String> {
    let client = state.client.lock().unwrap();

    let result = client
        .send_with_hint(
            ModelHint::CodeGen,
            &format!(
                "You are an expert {} developer. Generate clean, well-documented code.",
                language
            ),
            &format!("Language: {}\n\n{}", language, description),
        )
        .await;

    result.map_err(|e| format!("Code generation failed: {}", e))
}

/// Summarize text
#[tauri::command]
pub async fn ai_summarize(
    text: String,
    state: tauri::State<'_, AiState>,
) -> Result<String, String> {
    let client = state.client.lock().unwrap();

    let result = client
        .send_with_hint(
            ModelHint::Summarization,
            "You are a chat summarizer. Provide a concise summary. \
             Use bullet points if appropriate.",
            &text,
        )
        .await;

    result.map_err(|e| format!("Summarization failed: {}", e))
}

// ============================================================================
// Web3 + Ads + Monetization Commands
// ============================================================================

use crate::monetization::{
    CreditSource, MonetizationManager, PaymentMethod, PremiumFeature, SubscriptionTier,
};
use crate::AdCategory;
use crate::AdEngine;
use crate::AdPreferences;
use crate::AdType;

#[cfg(feature = "web3")]
use crate::web3::metamask::MetaMaskManager;
#[cfg(feature = "web3")]
use crate::web3::transactions::TxStore;
#[cfg(feature = "web3")]
use crate::web3::types::Chain;

/// Shared state for Web3 + Monetization
#[cfg(feature = "web3")]
pub struct Web3State {
    pub metamask: MetaMaskManager,
    pub tx_store: TxStore,
    pub ad_engine: std::sync::Arc<AdEngine>,
    pub monetization: std::sync::Arc<MonetizationManager>,
    pub user_address: std::sync::Mutex<Option<String>>,
}

#[cfg(feature = "web3")]
impl Web3State {
    pub fn new() -> Self {
        Self {
            metamask: MetaMaskManager::new(),
            tx_store: TxStore::new(),
            ad_engine: std::sync::Arc::new(AdEngine::new(AdPreferences::default())),
            monetization: std::sync::Arc::new(MonetizationManager::new()),
            user_address: std::sync::Mutex::new(None),
        }
    }

    pub fn get_user_address(&self) -> Option<String> {
        self.user_address.lock().unwrap().clone()
    }

    pub fn set_user_address(&self, address: String) {
        *self.user_address.lock().unwrap() = Some(address);
    }
}

// ---------------------------------------------------------------------------
// Web3 Commands (gated behind web3 feature)
// ---------------------------------------------------------------------------

#[cfg(feature = "web3")]
mod web3_commands {
    use super::*;

    /// Get connected wallet info
    #[tauri::command]
    pub async fn get_wallet_info(
        state: tauri::State<'_, Web3State>,
    ) -> Result<serde_json::Value, String> {
        let mm_state = state.metamask.get_state();
        let address = state.get_user_address();

        Ok(serde_json::json!({
            "is_connected": mm_state.is_connected,
            "address": address.unwrap_or_default(),
            "chain_id": mm_state.chain_id,
        }))
    }

    /// Get native balance for connected wallet
    #[tauri::command]
    pub async fn get_balance(
        chain: String,
        state: tauri::State<'_, Web3State>,
    ) -> Result<serde_json::Value, String> {
        let chain_enum: Chain = chain.parse().map_err(|e: String| e)?;

        let address = state.get_user_address().ok_or("Wallet not connected")?;

        // In a real implementation, this would query the RPC
        Ok(serde_json::json!({
            "address": address,
            "chain": chain_enum.name(),
            "balance": "0.0",
            "symbol": chain_enum.native_symbol(),
        }))
    }

    /// Sign a message with connected wallet
    #[tauri::command]
    pub async fn sign_message(
        message: String,
        _state: tauri::State<'_, Web3State>,
    ) -> Result<String, String> {
        // In real implementation, this would use the JS bridge to sign
        Err("MetaMask signing requires frontend interaction".to_string())
    }

    /// Get transaction history
    #[tauri::command]
    pub async fn get_transactions(state: tauri::State<'_, Web3State>) -> Vec<serde_json::Value> {
        let txs = state.tx_store.list();
        txs.into_iter()
            .map(|tx| {
                serde_json::json!({
                    "id": tx.id,
                    "chain": tx.chain.name(),
                    "type": format!("{:?}", tx.tx_type),
                    "from": tx.from,
                    "to": tx.to,
                    "value": tx.value,
                    "status": format!("{:?}", tx.status),
                    "created_at": tx.created_at.to_rfc3339(),
                })
            })
            .collect()
    }

    // ---------------------------------------------------------------------------
    // Ads Commands
    // ---------------------------------------------------------------------------

    /// Get ad preferences
    #[tauri::command]
    pub async fn get_ad_preferences(state: tauri::State<'_, Web3State>) -> serde_json::Value {
        let prefs = state.ad_engine.get_preferences();
        serde_json::json!({
            "preferred_categories": prefs.preferred_categories.iter().map(|c| c.as_str()).collect::<Vec<_>>(),
            "blocked_categories": prefs.blocked_categories.iter().map(|c| c.as_str()).collect::<Vec<_>>(),
            "max_ads_per_hour": prefs.max_ads_per_hour,
            "enable_reward_ads": prefs.enable_reward_ads,
            "enable_banner_ads": prefs.enable_banner_ads,
            "enable_native_ads": prefs.enable_native_ads,
            "enable_interstitial_ads": prefs.enable_interstitial_ads,
        })
    }

    /// Update ad preferences
    #[tauri::command]
    pub async fn update_ad_preferences(
        preferences: serde_json::Value,
        state: tauri::State<'_, Web3State>,
    ) -> Result<(), String> {
        let mut prefs = state.ad_engine.get_preferences();

        if let Some(cats) = preferences
            .get("preferred_categories")
            .and_then(|v| v.as_array())
        {
            prefs.preferred_categories = cats
                .iter()
                .filter_map(|v| v.as_str().and_then(|s| s.parse::<AdCategory>().ok()))
                .collect();
        }

        if let Some(cats) = preferences
            .get("blocked_categories")
            .and_then(|v| v.as_array())
        {
            prefs.blocked_categories = cats
                .iter()
                .filter_map(|v| v.as_str().and_then(|s| s.parse::<AdCategory>().ok()))
                .collect();
        }

        if let Some(max) = preferences.get("max_ads_per_hour").and_then(|v| v.as_u64()) {
            prefs.max_ads_per_hour = max as u32;
        }

        state.ad_engine.update_preferences(prefs);
        Ok(())
    }

    /// Get available ads for a specific type
    #[tauri::command]
    pub async fn get_available_ads(
        ad_type: String,
        state: tauri::State<'_, Web3State>,
    ) -> Result<Vec<serde_json::Value>, String> {
        let type_enum: AdType = match ad_type.as_str() {
            "banner" => AdType::Banner,
            "native" => AdType::Native,
            "interstitial" => AdType::Interstitial,
            "reward" => AdType::Reward,
            _ => return Err(format!("Unknown ad type: {}", ad_type)),
        };

        match state.ad_engine.select_ad(type_enum) {
            Ok(ad) => Ok(vec![serde_json::json!({
                "id": ad.id,
                "advertiser": ad.advertiser,
                "title": ad.title,
                "body": ad.body,
                "image_url": ad.image_url,
                "url": ad.url,
                "cta": ad.cta,
                "credit_reward": ad.credit_reward,
            })]),
            Err(_) => Ok(vec![]),
        }
    }

    /// Record ad impression (user viewed the ad)
    #[tauri::command]
    pub async fn record_ad_impression(
        ad_id: String,
        duration_secs: u32,
        state: tauri::State<'_, Web3State>,
    ) -> Result<u32, String> {
        state
            .ad_engine
            .record_impression(&ad_id, duration_secs)
            .map_err(|e| format!("Failed to record impression: {:?}", e))?;

        Ok(state.ad_engine.get_credits())
    }

    /// Record ad click
    #[tauri::command]
    pub async fn record_ad_click(
        ad_id: String,
        state: tauri::State<'_, Web3State>,
    ) -> Result<String, String> {
        state
            .ad_engine
            .record_click(&ad_id)
            .map_err(|e| format!("Failed to record click: {:?}", e))
    }

    /// Get ad stats
    #[tauri::command]
    pub async fn get_ad_stats(state: tauri::State<'_, Web3State>) -> serde_json::Value {
        let stats = state.ad_engine.get_stats();
        serde_json::json!({
            "total_views": stats.total_views,
            "credits_earned": stats.credits_earned,
            "total_clicks": stats.total_clicks,
            "views_today": stats.views_today,
            "avg_view_duration_secs": stats.avg_view_duration_secs,
        })
    }

    // ---------------------------------------------------------------------------
    // Monetization Commands
    // ---------------------------------------------------------------------------

    /// Get credit balance
    #[tauri::command]
    pub async fn get_credits(state: tauri::State<'_, Web3State>) -> Result<u32, String> {
        let address = state.get_user_address().ok_or("Wallet not connected")?;

        Ok(state.monetization.get_credits(&address))
    }

    /// Get credit history
    #[tauri::command]
    pub async fn get_credit_history(
        state: tauri::State<'_, Web3State>,
    ) -> Result<Vec<serde_json::Value>, String> {
        let address = state.get_user_address().ok_or("Wallet not connected")?;

        let history = state.monetization.get_credit_history(&address);
        Ok(history
            .into_iter()
            .map(|tx| {
                serde_json::json!({
                    "id": tx.id,
                    "amount": tx.amount,
                    "source": format!("{:?}", tx.source),
                    "description": tx.description,
                    "created_at": tx.created_at.to_rfc3339(),
                })
            })
            .collect())
    }

    /// Get subscription info
    #[tauri::command]
    pub async fn get_subscription(
        state: tauri::State<'_, Web3State>,
    ) -> Result<serde_json::Value, String> {
        let address = state.get_user_address().ok_or("Wallet not connected")?;

        match state.monetization.get_subscription(&address) {
            Some(sub) => Ok(serde_json::json!({
                "tier": format!("{:?}", sub.tier),
                "end_date": sub.end_date.to_rfc3339(),
                "days_remaining": sub.days_remaining(),
                "auto_renew": sub.auto_renew,
                "is_active": sub.is_valid(),
            })),
            None => Ok(serde_json::json!({
                "tier": "Free",
                "end_date": null,
                "days_remaining": 0,
                "auto_renew": false,
                "is_active": false,
            })),
        }
    }

    /// Subscribe to a tier
    #[tauri::command]
    pub async fn subscribe(
        tier: String,
        payment_method: String,
        _tx_hash: Option<String>,
        state: tauri::State<'_, Web3State>,
    ) -> Result<serde_json::Value, String> {
        let address = state.get_user_address().ok_or("Wallet not connected")?;

        let tier_enum: SubscriptionTier = match tier.as_str() {
            "premium" => SubscriptionTier::Premium,
            "pro" => SubscriptionTier::Pro,
            _ => return Err(format!("Unknown tier: {}", tier)),
        };

        let pay_method: PaymentMethod = match payment_method.as_str() {
            "crypto" => PaymentMethod::Crypto,
            "credits" => PaymentMethod::Credits,
            _ => return Err(format!("Unknown payment method: {}", payment_method)),
        };

        // If paying with credits, check balance
        if pay_method == PaymentMethod::Credits {
            let cost = tier_enum.monthly_price_credits();
            let balance = state.monetization.get_credits(&address);
            if balance < cost {
                return Err(format!(
                    "Insufficient credits: need {}, have {}",
                    cost, balance
                ));
            }

            // Deduct credits
            state.monetization.spend_credits(
                &address,
                cost,
                CreditSource::Subscription,
                &format!("{} subscription", tier_enum.name()),
            );
        }

        let sub = state.monetization.create_subscription(
            address, tier_enum, pay_method, None, // tx_hash would come from actual payment
        );

        Ok(serde_json::json!({
            "id": sub.id,
            "tier": format!("{:?}", sub.tier),
            "end_date": sub.end_date.to_rfc3339(),
            "days_remaining": sub.days_remaining(),
        }))
    }

    /// Get available premium features
    #[tauri::command]
    pub async fn get_premium_features() -> Vec<serde_json::Value> {
        use crate::monetization::PremiumFeature;
        let features = [
            PremiumFeature::AdFreeMonth,
            PremiumFeature::CustomTheme,
            PremiumFeature::PremiumStickers,
            PremiumFeature::ExtendedHistory,
            PremiumFeature::PriorityDelivery,
            PremiumFeature::ProfileCustomization,
            PremiumFeature::BotMarketplace,
            PremiumFeature::EnsDisplay,
        ];

        features
            .iter()
            .map(|f| {
                serde_json::json!({
                    "name": f.name(),
                    "cost_credits": f.cost_credits(),
                })
            })
            .collect()
    }

    /// Purchase a premium feature
    #[tauri::command]
    pub async fn purchase_feature(
        feature: String,
        state: tauri::State<'_, Web3State>,
    ) -> Result<bool, String> {
        let address = state.get_user_address().ok_or("Wallet not connected")?;

        let feature_enum: PremiumFeature = match feature.as_str() {
            "Ad-Free Month" => PremiumFeature::AdFreeMonth,
            "Custom Theme" => PremiumFeature::CustomTheme,
            "Premium Stickers" => PremiumFeature::PremiumStickers,
            "Extended History" => PremiumFeature::ExtendedHistory,
            "Priority Delivery" => PremiumFeature::PriorityDelivery,
            "Profile Customization" => PremiumFeature::ProfileCustomization,
            "Bot Marketplace" => PremiumFeature::BotMarketplace,
            "ENS Display" => PremiumFeature::EnsDisplay,
            _ => return Err(format!("Unknown feature: {}", feature)),
        };

        Ok(state.monetization.purchase_feature(&address, feature_enum))
    }

    /// Get purchased features
    #[tauri::command]
    pub async fn get_purchased_features(
        state: tauri::State<'_, Web3State>,
    ) -> Result<Vec<String>, String> {
        let address = state.get_user_address().ok_or("Wallet not connected")?;

        let features = state.monetization.get_features(&address);
        Ok(features.into_iter().map(|f| f.name().to_string()).collect())
    }
} // end web3_commands mod
