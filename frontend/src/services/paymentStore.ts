/**
 * Payment Store — Zustand state management for payments & subscriptions
 */

import { create } from 'zustand';
import {
  SubscriptionPlan,
  UserSubscription,
  PaymentIntent,
  CreditTransaction,
  SubscriptionTier,
  PaymentMethod,
  TipPayload,
} from '../types/payment';
import * as paymentApi from '../services/paymentService';

interface PaymentState {
  // State
  subscription: UserSubscription | null;
  creditBalance: number;
  creditHistory: CreditTransaction[];
  paymentIntent: PaymentIntent | null;
  plans: SubscriptionPlan[];
  loading: boolean;
  error: string | null;
  checkoutUrl: string | null;

  // Subscription
  loadSubscription: () => Promise<void>;
  subscribeWithStripe: (planId: string) => Promise<void>;
  subscribeWithCrypto: (plan: SubscriptionPlan, token?: 'ETH' | 'USDC' | 'USDT') => Promise<void>;
  cancelSubscription: () => Promise<void>;

  // Credits
  loadCreditBalance: () => Promise<void>;
  loadCreditHistory: () => Promise<void>;
  purchaseCreditsStripe: (credits: number) => Promise<void>;
  purchaseCreditsWithCrypto: (credits: number, token?: 'ETH' | 'USDC' | 'USDT') => Promise<void>;

  // Tipping
  sendTipCredits: (payload: TipPayload) => Promise<void>;
  sendTipCrypto: (payload: TipPayload) => Promise<string>;

  // MetaMask
  connectMetaMask: () => Promise<void>;

  // Utility
  clearError: () => void;
}

export const usePaymentStore = create<PaymentState>((set, get) => ({
  subscription: null,
  creditBalance: 0,
  creditHistory: [],
  paymentIntent: null,
  plans: paymentApi.getSubscriptionPlans(),
  loading: false,
  error: null,
  checkoutUrl: null,

  loadSubscription: async () => {
    set({ loading: true, error: null });
    try {
      const [sub, credits, history] = await Promise.all([
        paymentApi.getStripeSubscription(),
        paymentApi.getCreditBalance(),
        paymentApi.getCreditHistory(),
      ]);
      set({
        subscription: sub,
        creditBalance: credits,
        creditHistory: history,
        loading: false,
      });
    } catch (e: any) {
      set({ error: e.message, loading: false });
    }
  },

  subscribeWithStripe: async (planId: string) => {
    set({ loading: true, error: null });
    try {
      const checkoutUrl = await paymentApi.createStripeCheckout(
        planId,
        window.location.origin + '/subscription/success',
        window.location.origin + '/subscription/cancel',
      );
      set({ checkoutUrl, loading: false });
      // Redirect to Stripe Checkout
      window.location.href = checkoutUrl;
    } catch (e: any) {
      set({ error: e.message, loading: false });
    }
  },

  subscribeWithCrypto: async (plan, token = 'ETH') => {
    set({ loading: true, error: null });
    try {
      const txHash = await paymentApi.payWithCrypto(plan, token);
      set({
        loading: false,
        paymentIntent: {
          id: txHash,
          amount: plan.priceCents,
          currency: token,
          method: PaymentMethod.Crypto,
          status: 'succeeded',
          created_at: new Date().toISOString(),
          crypto_tx_hash: txHash,
        },
      });
      // Reload subscription
      await get().loadSubscription();
    } catch (e: any) {
      set({ error: e.message, loading: false });
    }
  },

  cancelSubscription: async () => {
    set({ loading: true, error: null });
    try {
      await paymentApi.cancelSubscription();
      set({ loading: false });
      await get().loadSubscription();
    } catch (e: any) {
      set({ error: e.message, loading: false });
    }
  },

  loadCreditBalance: async () => {
    try {
      const balance = await paymentApi.getCreditBalance();
      set({ creditBalance: balance });
    } catch (e: any) {
      set({ error: e.message });
    }
  },

  loadCreditHistory: async () => {
    try {
      const history = await paymentApi.getCreditHistory();
      set({ creditHistory: history });
    } catch (e: any) {
      set({ error: e.message });
    }
  },

  purchaseCreditsStripe: async (credits: number) => {
    set({ loading: true, error: null });
    try {
      const checkoutUrl = await paymentApi.purchaseCreditsWithStripe(
        credits,
        window.location.origin + '/credits/success',
        window.location.origin + '/credits/cancel',
      );
      set({ checkoutUrl, loading: false });
      window.location.href = checkoutUrl;
    } catch (e: any) {
      set({ error: e.message, loading: false });
    }
  },

  purchaseCreditsWithCrypto: async (credits, token = "ETH" as const): Promise<void> => {
    set({ loading: true, error: null });
    try {
      const txHash = await paymentApi.purchaseCreditsWithCrypto(credits, token);
      set({ loading: false });
      await get().loadCreditBalance();
      return;
    } catch (e: any) {
      set({ error: e.message, loading: false });
    }
  },

  sendTipCredits: async (payload: TipPayload) => {
    set({ loading: true, error: null });
    try {
      await paymentApi.sendTipWithCredits(payload);
      set({ loading: false });
      await get().loadCreditBalance();
    } catch (e: any) {
      set({ error: e.message, loading: false });
    }
  },

  sendTipCrypto: async (payload: TipPayload): Promise<string> => {
    set({ loading: true, error: null });
    try {
      const txHash = await paymentApi.sendCryptoTip(payload);
      set({ loading: false });
      return;
    } catch (e: any) {
      set({ error: e.message, loading: false });
      return "";
    }
  },

  connectMetaMask: async () => {
    set({ loading: true, error: null });
    try {
      await paymentApi.connectMetaMask();
      set({ loading: false });
    } catch (e: any) {
      set({ error: e.message, loading: false });
    }
  },

  clearError: () => {
    set({ error: null });
  },
}));
