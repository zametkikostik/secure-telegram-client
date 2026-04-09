/**
 * Payment & Monetization Types
 */

// ============================================================================
// Subscriptions
// ============================================================================

export enum SubscriptionTier {
  Free = 'free',
  Premium = 'premium',
  Pro = 'pro',
}

export interface SubscriptionPlan {
  id: string;
  tier: SubscriptionTier;
  name: string;
  description: string;
  priceCents: number;
  priceUsd: number;
  cryptoPrice?: string;  // Price in ETH/USDC
  features: string[];
  stripePriceId?: string;
  popular?: boolean;
}

export interface UserSubscription {
  id: string;
  user_id: string;
  tier: SubscriptionTier;
  status: 'active' | 'cancelled' | 'expired' | 'past_due';
  current_period_start: string;
  current_period_end: string;
  cancel_at_period_end: boolean;
  payment_method: PaymentMethod;
  stripe_subscription_id?: string;
  crypto_tx_hash?: string;
}

// ============================================================================
// Payment Methods
// ============================================================================

export enum PaymentMethod {
  Stripe = 'stripe',
  Crypto = 'crypto',
  Credits = 'credits',
}

export interface PaymentIntent {
  id: string;
  amount: number;
  currency: string;
  method: PaymentMethod;
  status: 'pending' | 'processing' | 'succeeded' | 'failed' | 'refunded';
  created_at: string;
  metadata?: Record<string, unknown>;
  // Stripe specific
  stripe_client_secret?: string;
  // Crypto specific
  crypto_address?: string;
  crypto_amount?: string;
  crypto_token?: string;
  crypto_tx_hash?: string;
}

// ============================================================================
// Credits
// ============================================================================

export interface CreditPackage {
  id: string;
  name: string;
  credits: number;
  priceCents: number;
  priceUsd: number;
  bonus?: number;  // Bonus credits
  popular?: boolean;
}

export interface CreditTransaction {
  id: string;
  user_id: string;
  amount: number;  // Positive = earned, Negative = spent
  balance_after: number;
  source: CreditSource;
  description: string;
  created_at: string;
}

export enum CreditSource {
  Purchase = 'purchase',
  AdReward = 'ad_reward',
  Referral = 'referral',
  Tip = 'tip',
  Subscription = 'subscription',
  FeaturePurchase = 'feature_purchase',
}

// ============================================================================
// Tipping
// ============================================================================

export interface TipPayload {
  recipient_id: string;
  amount: number;  // In credits or crypto amount
  currency: 'credits' | 'ETH' | 'USDC' | 'USDT';
  message?: string;
}

export interface Tip {
  id: string;
  sender_id: string;
  sender_name: string;
  recipient_id: string;
  amount: number;
  currency: string;
  message?: string;
  status: 'pending' | 'completed' | 'failed';
  tx_hash?: string;  // For crypto tips
  created_at: string;
}

// ============================================================================
// Premium Features
// ============================================================================

export interface PremiumFeature {
  id: string;
  name: string;
  description: string;
  icon: string;
  tier_required: SubscriptionTier;
  price_credits?: number;  // One-time purchase price in credits
}

// ============================================================================
// Pricing
// ============================================================================

export const DEFAULT_PLANS: SubscriptionPlan[] = [
  {
    id: 'free',
    tier: SubscriptionTier.Free,
    name: 'Free',
    description: 'Basic messenger features',
    priceCents: 0,
    priceUsd: 0,
    features: [
      'Unlimited messages',
      'Basic E2EE encryption',
      'Up to 5 groups',
      'Standard stickers',
    ],
  },
  {
    id: 'premium',
    tier: SubscriptionTier.Premium,
    name: 'Premium',
    description: 'Enhanced features for power users',
    priceCents: 500,
    priceUsd: 5,
    cryptoPrice: '0.0015 ETH',
    features: [
      'Everything in Free',
      'Ad-free experience',
      'Custom themes',
      'Extended message history',
      'Priority delivery',
      'Premium stickers',
      'AI translation',
    ],
    popular: true,
  },
  {
    id: 'pro',
    tier: SubscriptionTier.Pro,
    name: 'Pro',
    description: 'Maximum privacy and power',
    priceCents: 1500,
    priceUsd: 15,
    cryptoPrice: '0.0045 ETH',
    features: [
      'Everything in Premium',
      'Unlimited groups',
      'Full AI assistant access',
      'Advanced steganography',
      'Custom bot creation',
      'API access',
      'Priority support',
      'Early access to features',
    ],
  },
];

export const DEFAULT_CREDIT_PACKAGES: CreditPackage[] = [
  {
    id: 'credits-small',
    name: 'Starter',
    credits: 100,
    priceCents: 99,
    priceUsd: 0.99,
  },
  {
    id: 'credits-medium',
    name: 'Standard',
    credits: 500,
    priceCents: 449,
    priceUsd: 4.49,
    bonus: 50,
    popular: true,
  },
  {
    id: 'credits-large',
    name: 'Power User',
    credits: 2000,
    priceCents: 1499,
    priceUsd: 14.99,
    bonus: 300,
  },
  {
    id: 'credits-mega',
    name: 'Whale',
    credits: 10000,
    priceCents: 5999,
    priceUsd: 59.99,
    bonus: 2000,
  },
];
