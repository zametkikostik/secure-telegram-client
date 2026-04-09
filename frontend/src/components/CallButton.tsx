/**
 * CallButton — Audio/Video call buttons in chat header
 */

import React from 'react';
import { FiPhone, FiVideo } from 'react-icons/fi';
import { CallType } from '../types/call';
import { useCallStore } from '../services/callStore';
import { useTranslation } from 'react-i18next';

interface CallButtonProps {
  contactId: string;
  contactName: string;
  disabled?: boolean;
}

export const CallButton: React.FC<CallButtonProps> = ({
  contactId,
  contactName,
  disabled = false,
}) => {
  const { t } = useTranslation();
  const { startOutgoingCall, callState } = useCallStore();

  const isCallActive = callState !== 'idle' && callState !== 'ended';

  const handleCall = async (type: CallType) => {
    if (isCallActive || disabled) return;

    await startOutgoingCall(type, {
      userId: contactId,
      username: contactName,
    });
  };

  return (
    <div className="flex items-center gap-2">
      {/* Audio Call Button */}
      <button
        onClick={() => handleCall(CallType.Audio)}
        disabled={isCallActive || disabled}
        className="p-2 rounded-full transition-colors hover:bg-green-500/10 disabled:opacity-40 disabled:cursor-not-allowed"
        title={t('call.audio', 'Audio Call')}
        aria-label={t('call.audio', 'Audio Call')}
      >
        <FiPhone className="w-5 h-5 text-green-500" />
      </button>

      {/* Video Call Button */}
      {import.meta.env.VITE_WEBRTC_VIDEO_ENABLED !== 'false' && (
        <button
          onClick={() => handleCall(CallType.Video)}
          disabled={isCallActive || disabled}
          className="p-2 rounded-full transition-colors hover:bg-blue-500/10 disabled:opacity-40 disabled:cursor-not-allowed"
          title={t('call.video', 'Video Call')}
          aria-label={t('call.video', 'Video Call')}
        >
          <FiVideo className="w-5 h-5 text-blue-500" />
        </button>
      )}
    </div>
  );
};
