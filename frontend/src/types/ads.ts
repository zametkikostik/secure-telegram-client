/**
 * Ad Module Types
 * 
 * Type definitions for the privacy-first advertising system.
 * All ad selection happens ON-DEVICE based on user-chosen categories.
 * NO user data is sent to ad servers.
 */

// ============================================================================
// Ad Types
// ============================================================================

/** Ad display type */
export type AdType = 'banner' | 'native' | 'interstitial' | 'reward' | 'sponsored'

/** Ad category (user selects which categories they want to see) */
export type AdCategory =
  | 'crypto'
  | 'privacy'
  | 'security'
  | 'open_source'
  | 'tech'
  | 'gaming'
  | 'finance'
  | 'education'
  | 'health'
  | 'entertainment'
  | 'shopping'
  | 'social'
  | 'general'

/** Ad content */
export interface Ad {
  /** Unique ad ID (from advertiser) */
  id: string
  /** Advertiser name */
  advertiser: string
  /** Ad type */
  ad_type: AdType
  /** Category */
  category: AdCategory
  /** Title / headline */
  title: string
  /** Body text */
  body: string
  /** Image URL (local path after download) */
  image_url: string | null
  /** Click-through URL */
  url: string | null
  /** Call-to-action button text */
  cta: string | null
  /** Credit reward for viewing (reward ads only) */
  credit_reward: number
  /** Impressions cap (how many times this ad should be shown) */
  impression_cap: number
  /** Start date */
  start_date: string
  /** End date */
  end_date: string
  /** Priority (higher = more likely to be shown) */
  priority: number
  /** Whether the ad has been viewed */
  viewed: boolean
  /** Click count */
  click_count: number
  /** Local impressions count */
  impression_count: number
}

// ============================================================================
// Ad Preferences
// ============================================================================

/** User's ad preferences (what categories they want to see) */
export interface AdPreferences {
  /** Categories the user is interested in */
  preferred_categories: AdCategory[]
  /** Categories the user wants to BLOCK */
  blocked_categories: AdCategory[]
  /** Maximum ads per hour */
  max_ads_per_hour: number
  /** Enable reward ads (watch-to-earn) */
  enable_reward_ads: boolean
  /** Enable banner ads */
  enable_banner_ads: boolean
  /** Enable native ads */
  enable_native_ads: boolean
  /** Enable interstitial ads (opt-in) */
  enable_interstitial_ads: boolean
  /** Dark mode ad style */
  dark_mode: boolean
}

// ============================================================================
// Ad Stats
// ============================================================================

/** Ad stats summary */
export interface AdStats {
  /** Total ads viewed */
  total_views: number
  /** Total credits earned from ads */
  credits_earned: number
  /** Total ad clicks */
  total_clicks: number
  /** Ads viewed today */
  views_today: number
  /** Average view duration (seconds) */
  avg_view_duration_secs: number
  /** Top category viewed */
  top_category: AdCategory | null
}

// ============================================================================
// Ad Settings
// ============================================================================

/** Ad settings response */
export interface AdSettings {
  /** User preferences */
  preferences: AdPreferences
  /** Current credit balance */
  credits: number
  /** Ad stats */
  stats: AdStats
}

// ============================================================================
// Request/Response Types
// ============================================================================

/** Request to fetch ads */
export interface FetchAdsRequest {
  /** Maximum number of ads to fetch */
  max_ads?: number
}

/** Response from fetching ads */
export interface FetchAdsResponse {
  /** Number of ads fetched */
  fetched_count: number
  /** Total ads available */
  total_count: number
}

/** Request to select an ad */
export interface SelectAdRequest {
  /** Ad type to select */
  ad_type: AdType
}

/** Response from selecting an ad */
export interface SelectAdResponse {
  /** Selected ad (if available) */
  ad: Ad | null
}

/** Request to record impression */
export interface RecordImpressionRequest {
  /** Ad ID */
  ad_id: string
  /** View duration in seconds */
  duration_secs: number
}

/** Response from recording impression */
export interface RecordImpressionResponse {
  /** Success status */
  success: boolean
  /** Credits earned (if reward ad) */
  credits_earned: number
  /** Error message (if failed) */
  error: string | null
}

/** Request to record click */
export interface RecordClickRequest {
  /** Ad ID */
  ad_id: string
}

/** Response from recording click */
export interface RecordClickResponse {
  /** Success status */
  success: boolean
  /** Click URL (to open) */
  url: string | null
  /** Error message (if failed) */
  error: string | null
}

// ============================================================================
// Constants
// ============================================================================

/** All available ad categories */
export const ALL_AD_CATEGORIES: AdCategory[] = [
  'crypto',
  'privacy',
  'security',
  'open_source',
  'tech',
  'gaming',
  'finance',
  'education',
  'health',
  'entertainment',
  'shopping',
  'social',
  'general',
]

/** Category display labels */
export const CATEGORY_LABELS: Record<AdCategory, string> = {
  crypto: 'Криптовалюты',
  privacy: 'Приватность',
  security: 'Безопасность',
  open_source: 'Открытый код',
  tech: 'Технологии',
  gaming: 'Игры',
  finance: 'Финансы',
  education: 'Образование',
  health: 'Здоровье',
  entertainment: 'Развлечения',
  shopping: 'Покупки',
  social: 'Социальные сети',
  general: 'Общее',
}

/** Default ad preferences */
export const DEFAULT_AD_PREFERENCES: AdPreferences = {
  preferred_categories: ['crypto', 'privacy', 'security', 'tech', 'open_source'],
  blocked_categories: ['shopping', 'social'],
  max_ads_per_hour: 10,
  enable_reward_ads: true,
  enable_banner_ads: true,
  enable_native_ads: true,
  enable_interstitial_ads: false,
  dark_mode: false,
}

// ============================================================================
// Tauri Command Constants
// ============================================================================

/** Fetch encrypted ad bundle from Cloudflare Worker */
export const TAURI_CMD_FETCH_ADS = 'fetch_ads'

/** Select an ad based on type and preferences */
export const TAURI_CMD_SELECT_AD = 'select_ad'

/** Record an ad impression */
export const TAURI_CMD_RECORD_IMPRESSION = 'record_impression'

/** Record an ad click */
export const TAURI_CMD_RECORD_CLICK = 'record_click'

/** Get ad settings and stats */
export const TAURI_CMD_GET_AD_SETTINGS = 'get_ad_settings'

/** Update ad preferences */
export const TAURI_CMD_UPDATE_AD_PREFERENCES = 'update_ad_preferences'

/** Get current credit balance */
export const TAURI_CMD_GET_AD_CREDITS = 'get_ad_credits'

/** Spend credits */
export const TAURI_CMD_SPEND_AD_CREDITS = 'spend_ad_credits'

/** Get all available ads (for settings page) */
export const TAURI_CMD_LIST_ADS = 'list_ads'

/** Report impressions to server (anonymous batch) */
export const TAURI_CMD_REPORT_IMPRESSIONS = 'report_impressions'

/** Clean up expired ads */
export const TAURI_CMD_CLEANUP_ADS = 'cleanup_ads'

/** All ad module Tauri commands */
export const TAURI_AD_COMMANDS = {
  FETCH_ADS: TAURI_CMD_FETCH_ADS,
  SELECT_AD: TAURI_CMD_SELECT_AD,
  RECORD_IMPRESSION: TAURI_CMD_RECORD_IMPRESSION,
  RECORD_CLICK: TAURI_CMD_RECORD_CLICK,
  GET_AD_SETTINGS: TAURI_CMD_GET_AD_SETTINGS,
  UPDATE_AD_PREFERENCES: TAURI_CMD_UPDATE_AD_PREFERENCES,
  GET_AD_CREDITS: TAURI_CMD_GET_AD_CREDITS,
  SPEND_AD_CREDITS: TAURI_CMD_SPEND_AD_CREDITS,
  LIST_ADS: TAURI_CMD_LIST_ADS,
  REPORT_IMPRESSIONS: TAURI_CMD_REPORT_IMPRESSIONS,
  CLEANUP_ADS: TAURI_CMD_CLEANUP_ADS,
} as const
