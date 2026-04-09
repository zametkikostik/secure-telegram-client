/**
 * SubscriptionPage — Page for viewing and managing subscriptions
 */

import React, { useEffect, useState } from 'react';
import { FiCheck, FiX, FiCreditCard } from 'react-icons/fi';
import { SiEthereum } from 'react-icons/si';
import { usePaymentStore } from '../services/paymentStore';
import { SubscriptionPlan, SubscriptionTier, PaymentMethod } from '../types/payment';
import { useTranslation } from 'react-i18next';

export const SubscriptionPage: React.FC = () => {
  const { t } = useTranslation();
  const { plans, subscription, loading, error, loadSubscription, subscribeWithStripe, subscribeWithCrypto, cancelSubscription } = usePaymentStore();
  const [selectedPlan, setSelectedPlan] = useState<SubscriptionPlan | null>(null);
  const [showPaymentModal, setShowPaymentModal] = useState(false);
  const [paymentMethod, setPaymentMethod] = useState<PaymentMethod>(PaymentMethod.Stripe);

  useEffect(() => {
    loadSubscription();
  }, [loadSubscription]);

  const handleSubscribe = (plan: SubscriptionPlan) => {
    if (plan.tier === SubscriptionTier.Free) return;
    setSelectedPlan(plan);
    setShowPaymentModal(true);
  };

  const handlePayment = async () => {
    if (!selectedPlan) return;

    if (paymentMethod === PaymentMethod.Stripe) {
      await subscribeWithStripe(selectedPlan.id);
    } else {
      await subscribeWithCrypto(selectedPlan, 'ETH');
    }
    setShowPaymentModal(false);
  };

  const handleCancel = async () => {
    if (confirm(t('payment.confirm_cancel', 'Are you sure you want to cancel?'))) {
      await cancelSubscription();
    }
  };

  return (
    <div className="max-w-6xl mx-auto px-4 py-8">
      {/* Header */}
      <div className="text-center mb-12">
        <h1 className="text-4xl font-bold text-white mb-4">
          {t('payment.title', 'Upgrade Your Plan')}
        </h1>
        <p className="text-gray-400 text-lg">
          {t('payment.subtitle', 'Choose the plan that works best for you')}
        </p>
      </div>

      {/* Error */}
      {error && (
        <div className="mb-6 p-4 bg-red-500/20 border border-red-500/50 rounded-lg text-red-300">
          {error}
        </div>
      )}

      {/* Current Subscription */}
      {subscription && subscription.status === 'active' && (
        <div className="mb-8 p-4 bg-green-500/10 border border-green-500/30 rounded-lg">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-green-400 font-semibold">
                {t('payment.current_plan', 'Current Plan')}: {subscription.tier}
              </p>
              <p className="text-gray-400 text-sm">
                {t('payment.renews', 'Renews')} {new Date(subscription.current_period_end).toLocaleDateString()}
              </p>
            </div>
            {!subscription.cancel_at_period_end && (
              <button
                onClick={handleCancel}
                className="px-4 py-2 bg-red-500/20 hover:bg-red-500/30 text-red-400 rounded-lg transition"
              >
                {t('payment.cancel_subscription', 'Cancel')}
              </button>
            )}
          </div>
        </div>
      )}

      {/* Plans Grid */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {plans.map((plan) => (
          <div
            key={plan.id}
            className={`relative p-6 rounded-2xl border transition-all hover:scale-105 ${
              plan.tier === SubscriptionTier.Premium
                ? 'bg-gradient-to-br from-blue-600/20 to-purple-600/20 border-blue-500/50'
                : plan.tier === SubscriptionTier.Pro
                ? 'bg-gradient-to-br from-yellow-600/20 to-orange-600/20 border-yellow-500/50'
                : 'bg-gray-800 border-gray-700'
            }`}
          >
            {/* Popular Badge */}
            {plan.popular && (
              <div className="absolute -top-3 left-1/2 -translate-x-1/2 px-3 py-1 bg-blue-600 text-white text-xs font-semibold rounded-full">
                {t('payment.popular', 'Most Popular')}
              </div>
            )}

            {/* Plan Header */}
            <h3 className="text-2xl font-bold text-white mb-2">{plan.name}</h3>
            <p className="text-gray-400 text-sm mb-4">{plan.description}</p>

            {/* Price */}
            <div className="mb-6">
              <span className="text-4xl font-bold text-white">
                ${plan.priceUsd}
              </span>
              <span className="text-gray-400">/month</span>
              {plan.cryptoPrice && (
                <p className="text-xs text-gray-500 mt-1">or {plan.cryptoPrice}</p>
              )}
            </div>

            {/* Features */}
            <ul className="space-y-3 mb-6">
              {plan.features.map((feature, i) => (
                <li key={i} className="flex items-start gap-2 text-sm text-gray-300">
                  <FiCheck className="w-4 h-4 text-green-400 mt-0.5 flex-shrink-0" />
                  <span>{feature}</span>
                </li>
              ))}
            </ul>

            {/* CTA Button */}
            <button
              onClick={() => handleSubscribe(plan)}
              disabled={plan.tier === SubscriptionTier.Free || loading}
              className={`w-full py-3 rounded-lg font-semibold transition ${
                plan.tier === SubscriptionTier.Free
                  ? 'bg-gray-700 text-gray-500 cursor-not-allowed'
                  : plan.popular
                  ? 'bg-blue-600 hover:bg-blue-700 text-white'
                  : 'bg-gray-700 hover:bg-gray-600 text-white'
              }`}
            >
              {plan.tier === SubscriptionTier.Free
                ? t('payment.current', 'Current')
                : t('payment.subscribe', 'Subscribe')}
            </button>
          </div>
        ))}
      </div>

      {/* Payment Method Modal */}
      {showPaymentModal && selectedPlan && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
          <div className="bg-gray-800 rounded-2xl p-6 max-w-md w-full mx-4">
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-xl font-bold text-white">
                {t('payment.choose_method', 'Choose Payment Method')}
              </h3>
              <button
                onClick={() => setShowPaymentModal(false)}
                className="p-1 rounded hover:bg-gray-700 text-gray-400"
              >
                <FiX className="w-5 h-5" />
              </button>
            </div>

            <p className="text-gray-400 mb-6">
              {t('payment.plan_summary', 'Plan')}: {selectedPlan.name} — ${selectedPlan.priceUsd}/month
            </p>

            {/* Payment Methods */}
            <div className="space-y-3 mb-6">
              <button
                onClick={() => setPaymentMethod(PaymentMethod.Stripe)}
                className={`w-full flex items-center gap-3 p-4 rounded-lg border transition ${
                  paymentMethod === PaymentMethod.Stripe
                    ? 'border-blue-500 bg-blue-500/10'
                    : 'border-gray-700 hover:bg-gray-700'
                }`}
              >
                <FiCreditCard className="w-6 h-6 text-blue-400" />
                <div className="text-left">
                  <p className="text-white font-semibold">Stripe</p>
                  <p className="text-gray-400 text-sm">Credit card, Apple Pay, Google Pay</p>
                </div>
              </button>

              <button
                onClick={() => setPaymentMethod(PaymentMethod.Crypto)}
                className={`w-full flex items-center gap-3 p-4 rounded-lg border transition ${
                  paymentMethod === PaymentMethod.Crypto
                    ? 'border-blue-500 bg-blue-500/10'
                    : 'border-gray-700 hover:bg-gray-700'
                }`}
              >
                <SiEthereum className="w-6 h-6 text-purple-400" />
                <div className="text-left">
                  <p className="text-white font-semibold">Crypto (MetaMask)</p>
                  <p className="text-gray-400 text-sm">ETH, USDC, USDT</p>
                </div>
              </button>
            </div>

            {/* Confirm Button */}
            <button
              onClick={handlePayment}
              disabled={loading}
              className="w-full py-3 bg-green-600 hover:bg-green-700 disabled:opacity-50 text-white rounded-lg font-semibold transition"
            >
              {loading
                ? t('payment.processing', 'Processing...')
                : t('payment.confirm', 'Confirm Payment')}
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
