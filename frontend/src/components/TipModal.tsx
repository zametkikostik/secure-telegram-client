/**
 * TipModal — Modal for sending tips to users
 */

import React, { useState } from 'react';
import { FiX, FiSend, FiCreditCard } from 'react-icons/fi';
import { SiEthereum } from 'react-icons/si';
import { usePaymentStore } from '../services/paymentStore';
import { PaymentMethod, TipPayload } from '../types/payment';
import { useTranslation } from 'react-i18next';

interface TipModalProps {
  recipientId: string;
  recipientName: string;
  onClose: () => void;
  onSuccess?: () => void;
}

export const TipModal: React.FC<TipModalProps> = ({
  recipientId,
  recipientName,
  onClose,
  onSuccess,
}) => {
  const { t } = useTranslation();
  const { sendTipCredits, sendTipCrypto, creditBalance, loading, error } = usePaymentStore();

  const [amount, setAmount] = useState('');
  const [message, setMessage] = useState('');
  const [paymentMethod, setPaymentMethod] = useState<PaymentMethod>(PaymentMethod.Credits);
  const [cryptoToken, setCryptoToken] = useState<'ETH' | 'USDC' | 'USDT'>('ETH');

  const presetAmounts = [10, 50, 100, 500];

  const handleSend = async () => {
    const numAmount = parseFloat(amount);
    if (!numAmount || numAmount <= 0) return;

    const payload: TipPayload = {
      recipient_id: recipientId,
      amount: numAmount,
      currency: paymentMethod === PaymentMethod.Credits ? 'credits' : cryptoToken,
      message: message || undefined,
    };

    try {
      if (paymentMethod === PaymentMethod.Credits) {
        if (numAmount > creditBalance) {
          alert(t('payment.insufficient_credits', 'Insufficient credits'));
          return;
        }
        await sendTipCredits(payload);
      } else {
        await sendTipCrypto(payload);
      }

      onSuccess?.();
      onClose();
    } catch {
      // Error handled by store
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="bg-gray-800 rounded-2xl p-6 max-w-md w-full mx-4">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <h3 className="text-xl font-bold text-white">
            {t('payment.tip_title', 'Send a Tip')}
          </h3>
          <button
            onClick={onClose}
            className="p-1 rounded hover:bg-gray-700 text-gray-400"
          >
            <FiX className="w-5 h-5" />
          </button>
        </div>

        <p className="text-gray-400 mb-4">
          {t('payment.tip_to', 'To')}: <span className="text-white font-semibold">{recipientName}</span>
        </p>

        {/* Error */}
        {error && (
          <div className="mb-4 p-3 bg-red-500/20 border border-red-500/50 rounded-lg text-red-300 text-sm">
            {error}
          </div>
        )}

        {/* Amount */}
        <div className="mb-4">
          <label className="block text-sm text-gray-400 mb-2">
            {t('payment.amount', 'Amount')}
          </label>

          {/* Preset Amounts */}
          {paymentMethod === PaymentMethod.Credits && (
            <div className="flex gap-2 mb-3">
              {presetAmounts.map((preset) => (
                <button
                  key={preset}
                  onClick={() => setAmount(preset.toString())}
                  className={`flex-1 py-1.5 text-sm rounded-lg transition ${
                    amount === preset.toString()
                      ? 'bg-blue-600 text-white'
                      : 'bg-gray-700 text-gray-400 hover:bg-gray-600'
                  }`}
                >
                  {preset}
                </button>
              ))}
            </div>
          )}

          <input
            type="number"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            placeholder={paymentMethod === PaymentMethod.Credits ? 'Credits' : cryptoToken}
            className="w-full px-4 py-2 bg-gray-700 text-white rounded-lg border border-gray-600 focus:border-blue-500 focus:outline-none"
            min="1"
          />

          {paymentMethod === PaymentMethod.Credits && (
            <p className="text-xs text-gray-500 mt-1">
              {t('payment.balance', 'Balance')}: {creditBalance} credits
            </p>
          )}
        </div>

        {/* Payment Method */}
        <div className="mb-4">
          <label className="block text-sm text-gray-400 mb-2">
            {t('payment.payment_method', 'Payment Method')}
          </label>

          <div className="flex gap-2 mb-3">
            <button
              onClick={() => setPaymentMethod(PaymentMethod.Credits)}
              className={`flex-1 flex items-center justify-center gap-2 py-2 rounded-lg transition ${
                paymentMethod === PaymentMethod.Credits
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-700 text-gray-400 hover:bg-gray-600'
              }`}
            >
              <FiCreditCard className="w-4 h-4" />
              Credits
            </button>

            <button
              onClick={() => setPaymentMethod(PaymentMethod.Crypto)}
              className={`flex-1 flex items-center justify-center gap-2 py-2 rounded-lg transition ${
                paymentMethod === PaymentMethod.Crypto
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-700 text-gray-400 hover:bg-gray-600'
              }`}
            >
              <SiEthereum className="w-4 h-4" />
              Crypto
            </button>
          </div>

          {/* Crypto Token Selection */}
          {paymentMethod === PaymentMethod.Crypto && (
            <div className="flex gap-2">
              {(['ETH', 'USDC', 'USDT'] as const).map((token) => (
                <button
                  key={token}
                  onClick={() => setCryptoToken(token)}
                  className={`flex-1 py-1.5 text-sm rounded-lg transition ${
                    cryptoToken === token
                      ? 'bg-purple-600 text-white'
                      : 'bg-gray-700 text-gray-400 hover:bg-gray-600'
                  }`}
                >
                  {token}
                </button>
              ))}
            </div>
          )}
        </div>

        {/* Message (Optional) */}
        <div className="mb-6">
          <label className="block text-sm text-gray-400 mb-2">
            {t('payment.message_optional', 'Message (Optional)')}
          </label>
          <textarea
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            placeholder={t('payment.tip_message', 'Thanks for your great content!')}
            rows={2}
            className="w-full px-4 py-2 bg-gray-700 text-white rounded-lg border border-gray-600 focus:border-blue-500 focus:outline-none resize-none"
          />
        </div>

        {/* Submit */}
        <button
          onClick={handleSend}
          disabled={loading || !amount || parseFloat(amount) <= 0}
          className="w-full flex items-center justify-center gap-2 py-3 bg-green-600 hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed text-white rounded-lg font-semibold transition"
        >
          <FiSend className="w-5 h-5" />
          {loading ? t('payment.sending', 'Sending...') : t('payment.send_tip', 'Send Tip')}
        </button>
      </div>
    </div>
  );
};
