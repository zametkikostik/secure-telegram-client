/**
 * Payment Service
 *
 * Handles Stripe Checkout, crypto payments via MetaMask,
 * and credit system transactions.
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  PaymentIntent,
  PaymentMethod,
  SubscriptionPlan,
  UserSubscription,
  TipPayload,
  CreditTransaction,
} from '../types/payment';
import { DEFAULT_PLANS, DEFAULT_CREDIT_PACKAGES } from '../types/payment';

// ============================================================================
// Stripe Integration
// ============================================================================

/**
 * Create a Stripe Checkout Session and redirect to Stripe Checkout
 */
export async function createStripeCheckout(
  priceId: string,
  successUrl: string,
  cancelUrl: string,
): Promise<string> {
  // Returns checkout URL for redirect
  const url: string = await invoke('create_stripe_checkout', {
    priceId,
    successUrl,
    cancelUrl,
  });
  return url;
}

/**
 * Open Stripe Billing Portal for managing subscription
 */
export async function openStripeBillingPortal(): Promise<string> {
  const url: string = await invoke('open_stripe_billing_portal');
  return url;
}

/**
 * Handle Stripe webhook event (server-side)
 */
export async function handleStripeWebhook(payload: string, signature: string): Promise<void> {
  await invoke('handle_stripe_webhook', { payload, signature });
}

/**
 * Get user's current Stripe subscription
 */
export async function getStripeSubscription(): Promise<UserSubscription | null> {
  try {
    return await invoke<UserSubscription | null>('get_stripe_subscription');
  } catch {
    return null;
  }
}

// ============================================================================
// Crypto Payments via MetaMask
// ============================================================================

/**
 * Pay for subscription with crypto (ETH/USDC/USDT)
 */
export async function payWithCrypto(
  plan: SubscriptionPlan,
  token: 'ETH' | 'USDC' | 'USDT' = 'ETH',
): Promise<string> {
  // Returns transaction hash
  const txHash: string = await invoke('pay_crypto_subscription', {
    planId: plan.id,
    token,
  });
  return txHash;
}

/**
 * Send a crypto tip to another user
 */
export async function sendCryptoTip(payload: TipPayload): Promise<string> {
  const txHash: string = await invoke('send_crypto_tip', {
    recipientId: payload.recipient_id,
    amount: payload.amount,
    currency: payload.currency,
    message: payload.message,
  });
  return txHash;
}

/**
 * Connect MetaMask wallet
 */
export async function connectMetaMask(): Promise<{ address: string; chainId: number }> {
  return await invoke('connect_metamask');
}

/**
 * Get MetaMask balance
 */
export async function getMetaMaskBalance(): Promise<string> {
  return await invoke('get_metamask_balance');
}

/**
 * Switch network (e.g., to Sepolia for testing)
 */
export async function switchNetwork(chainId: number): Promise<void> {
  await invoke('switch_metamask_network', { chainId });
}

// ============================================================================
// Credit System
// ============================================================================

/**
 * Get user's credit balance
 */
export async function getCreditBalance(): Promise<number> {
  return await invoke<number>('get_credits');
}

/**
 * Purchase credits with Stripe
 */
export async function purchaseCreditsWithStripe(
  credits: number,
  successUrl: string,
  cancelUrl: string,
): Promise<string> {
  const url: string = await invoke('purchase_credits_stripe', {
    credits,
    successUrl,
    cancelUrl,
  });
  return url;
}

/**
 * Purchase credits with crypto
 */
export async function purchaseCreditsWithCrypto(
  credits: number,
  token: 'ETH' | 'USDC' | 'USDT' = 'ETH',
): Promise<string> {
  const txHash: string = await invoke('purchase_credits_crypto', {
    credits,
    token,
  });
  return txHash;
}

/**
 * Get credit transaction history
 */
export async function getCreditHistory(): Promise<CreditTransaction[]> {
  return await invoke<CreditTransaction[]>('get_credit_history');
}

// ============================================================================
// Subscription Management
// ============================================================================

/**
 * Get available subscription plans
 */
export function getSubscriptionPlans(): SubscriptionPlan[] {
  return DEFAULT_PLANS;
}

/**
 * Get available credit packages
 */
export function getCreditPackages() {
  return DEFAULT_CREDIT_PACKAGES;
}

/**
 * Check if user has premium
 */
export async function hasPremium(): Promise<boolean> {
  return await invoke<boolean>('has_premium');
}

/**
 * Cancel current subscription
 */
export async function cancelSubscription(): Promise<void> {
  await invoke('cancel_subscription');
}

/**
 * Upgrade subscription tier
 */
export async function upgradeSubscription(
  newTier: string,
  method: PaymentMethod,
): Promise<void> {
  await invoke('upgrade_subscription', { newTier, method });
}

// ============================================================================
// Tipping
// ============================================================================

/**
 * Send a tip with credits
 */
export async function sendTipWithCredits(payload: TipPayload): Promise<void> {
  await invoke('send_tip_credits', {
    recipientId: payload.recipient_id,
    amount: payload.amount,
    message: payload.message,
  });
}
