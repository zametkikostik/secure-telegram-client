/**
 * IncomingCallModal — Modal for incoming calls with accept/reject buttons
 */

import React, { useEffect } from 'react';
import { FiPhone, FiPhoneOff } from 'react-icons/fi';
import { useCallStore } from '../services/callStore';
import { useTranslation } from 'react-i18next';

export const IncomingCallModal: React.FC = () => {
  const { t } = useTranslation();
  const { currentCall, acceptCall, rejectCall } = useCallStore();

  // Play ringtone when incoming call
  useEffect(() => {
    if (!currentCall || currentCall.state !== 'ringing') return;

    // Create ringtone using Web Audio API
    const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
    const oscillator = audioContext.createOscillator();
    const gainNode = audioContext.createGain();

    oscillator.connect(gainNode);
    gainNode.connect(audioContext.destination);

    oscillator.frequency.value = 440;
    oscillator.type = 'sine';
    gainNode.gain.value = 0.3;

    // Ring pattern: 2s on, 1s off
    const ringInterval = setInterval(() => {
      if (audioContext.state === 'running') {
        oscillator.start();
        setTimeout(() => oscillator.stop(), 2000);
      }
    }, 3000);

    oscillator.start();

    return () => {
      clearInterval(ringInterval);
      oscillator.stop();
      audioContext.close();
    };
  }, [currentCall?.id, currentCall?.state]);

  if (!currentCall || currentCall.state !== 'ringing') {
    return null;
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="bg-gradient-to-br from-gray-900 to-gray-800 rounded-3xl p-8 shadow-2xl max-w-sm w-full mx-4 animate-pulse-slow">
        {/* Caller Info */}
        <div className="text-center mb-8">
          <div className="w-24 h-24 rounded-full bg-gradient-to-br from-green-400 to-blue-500 mx-auto mb-4 flex items-center justify-center">
            <FiPhone className="w-12 h-12 text-white" />
          </div>
          <h2 className="text-2xl font-bold text-white mb-2">
            {currentCall.initiator.username}
          </h2>
          <p className="text-gray-400">
            {currentCall.type === 'video'
              ? t('call.incoming_video', 'Incoming Video Call')
              : t('call.incoming_audio', 'Incoming Audio Call')}
          </p>
        </div>

        {/* Accept/Reject Buttons */}
        <div className="flex justify-center gap-12">
          <button
            onClick={rejectCall}
            className="flex flex-col items-center gap-2 group"
            aria-label={t('call.reject', 'Reject Call')}
          >
            <div className="w-16 h-16 rounded-full bg-red-500 flex items-center justify-center group-hover:bg-red-600 transition-colors shadow-lg">
              <FiPhoneOff className="w-8 h-8 text-white" />
            </div>
            <span className="text-sm text-gray-400">{t('call.reject', 'Reject')}</span>
          </button>

          <button
            onClick={acceptCall}
            className="flex flex-col items-center gap-2 group"
            aria-label={t('call.accept', 'Accept Call')}
          >
            <div className="w-16 h-16 rounded-full bg-green-500 flex items-center justify-center group-hover:bg-green-600 transition-colors shadow-lg animate-bounce-slow">
              <FiPhone className="w-8 h-8 text-white" />
            </div>
            <span className="text-sm text-gray-400">{t('call.accept', 'Accept')}</span>
          </button>
        </div>
      </div>
    </div>
  );
};
