/**
 * ActiveCallOverlay — Full-screen overlay for active voice/video calls
 */

import React, { useEffect, useRef, useState } from 'react';
import {
  FiPhoneOff,
  FiMic,
  FiMicOff,
  FiVideo,
  FiVideoOff,
  FiVolume2,
  FiVolumeX,
  FiMonitor,
  FiRadio,
} from 'react-icons/fi';
import { useCallStore } from '../services/callStore';
import { useTranslation } from 'react-i18next';
import { WebRTCService } from '../services/webrtcService';

export const ActiveCallOverlay: React.FC = () => {
  const { t } = useTranslation();
  const {
    currentCall,
    callState,
    callDuration,
    isMuted,
    isCameraOn,
    isSpeakerOn,
    toggleMute,
    toggleCamera,
    toggleSpeaker,
    endCall,
  } = useCallStore();

  const remoteVideoRef = useRef<HTMLVideoElement>(null);
  const localVideoRef = useRef<HTMLVideoElement>(null);
  const [showControls, setShowControls] = useState(true);

  // Auto-hide controls after 3 seconds
  useEffect(() => {
    if (callState !== 'connected') return;

    const timer = setTimeout(() => setShowControls(false), 3000);
    return () => clearTimeout(timer);
  }, [callState, showControls]);

  // Attach remote stream to video element
  useEffect(() => {
    const service = WebRTCService.getInstance();
    const remoteStream = service.getRemoteStream();
    const localStream = service.getLocalStream();

    if (remoteVideoRef.current && remoteStream) {
      remoteVideoRef.current.srcObject = remoteStream;
    }

    if (localVideoRef.current && localStream) {
      localVideoRef.current.srcObject = localStream;
    }
  }, [callState]);

  // Format duration
  const formatDuration = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  if (!currentCall || (callState !== 'connected' && callState !== 'connecting')) {
    return null;
  }

  const isVideoCall = currentCall.type === 'video';

  return (
    <div
      className="fixed inset-0 z-50 bg-gray-900"
      onMouseMove={() => setShowControls(true)}
      onClick={() => setShowControls(true)}
    >
      {/* Video Display */}
      {isVideoCall ? (
        <div className="relative w-full h-full">
          {/* Remote Video (full screen) */}
          <video
            ref={remoteVideoRef}
            autoPlay
            playsInline
            className="w-full h-full object-cover"
          />

          {/* Local Video (Picture-in-Picture) */}
          <div className="absolute bottom-24 right-4 w-32 h-48 rounded-2xl overflow-hidden shadow-2xl border-2 border-white/20 bg-gray-800">
            <video
              ref={localVideoRef}
              autoPlay
              playsInline
              muted
              className="w-full h-full object-cover"
            />
          </div>
        </div>
      ) : (
        /* Audio Call — Avatar Display */
        <div className="flex flex-col items-center justify-center h-full">
          <div className="w-32 h-32 rounded-full bg-gradient-to-br from-green-400 to-blue-500 flex items-center justify-center mb-4 animate-pulse">
            <FiMic className="w-16 h-16 text-white" />
          </div>
          <h2 className="text-2xl font-bold text-white mb-2">
            {currentCall.type === 'video'
              ? currentCall.recipient.username
              : currentCall.initiator.username}
          </h2>
          <p className="text-gray-400">
            {callState === 'connecting'
              ? t('call.connecting', 'Connecting...')
              : formatDuration(callDuration)}
          </p>
        </div>
      )}

      {/* Call Controls Overlay */}
      <div
        className={`absolute bottom-0 left-0 right-0 transition-opacity duration-300 ${
          showControls ? 'opacity-100' : 'opacity-0'
        }`}
      >
        {/* Call Info Bar */}
        <div className="bg-gradient-to-t from-black/80 to-transparent p-6">
          <div className="flex flex-col items-center gap-4">
            {/* Call Duration & Status */}
            <div className="text-center">
              <p className="text-white text-lg font-semibold">
                {currentCall.initiator.username} / {currentCall.recipient.username}
              </p>
              <p className="text-gray-400 text-sm">
                {callState === 'connected'
                  ? formatDuration(callDuration)
                  : t('call.connecting', 'Connecting...')}
              </p>
            </div>

            {/* Control Buttons */}
            <div className="flex items-center gap-6">
              {/* Mute Toggle */}
              <button
                onClick={toggleMute}
                className={`p-4 rounded-full transition-colors ${
                  isMuted ? 'bg-red-500 hover:bg-red-600' : 'bg-gray-700 hover:bg-gray-600'
                }`}
                aria-label={isMuted ? t('call.unmute', 'Unmute') : t('call.mute', 'Mute')}
              >
                {isMuted ? (
                  <FiMicOff className="w-6 h-6 text-white" />
                ) : (
                  <FiMic className="w-6 h-6 text-white" />
                )}
              </button>

              {/* Camera Toggle (video calls only) */}
              {isVideoCall && (
                <button
                  onClick={toggleCamera}
                  className={`p-4 rounded-full transition-colors ${
                    isCameraOn ? 'bg-gray-700 hover:bg-gray-600' : 'bg-red-500 hover:bg-red-600'
                  }`}
                  aria-label={isCameraOn ? t('call.camera_off', 'Turn Off Camera') : t('call.camera_on', 'Turn On Camera')}
                >
                  {isCameraOn ? (
                    <FiVideo className="w-6 h-6 text-white" />
                  ) : (
                    <FiVideoOff className="w-6 h-6 text-white" />
                  )}
                </button>
              )}

              {/* Screen Share Button */}
              <button
                onClick={() => WebRTCService.getInstance().toggleScreenShare()}
                className="p-4 rounded-full bg-gray-700 hover:bg-gray-600 transition-colors"
                aria-label="Share Screen"
                title="Демонстрация экрана"
              >
                <FiMonitor className="w-6 h-6 text-white" />
              </button>

              {/* Push-to-Talk Button */}
              <button
                onMouseDown={() => useCallStore.getState().toggleMute()}
                onMouseUp={() => useCallStore.getState().toggleMute()}
                onTouchStart={() => useCallStore.getState().toggleMute()}
                onTouchEnd={() => useCallStore.getState().toggleMute()}
                className={`p-4 rounded-full transition-colors ${
                  isMuted ? 'bg-red-500 hover:bg-red-600' : 'bg-green-500 hover:bg-green-600'
                }`}
                aria-label="Push to Talk"
                title="Рация (зажать для разговора)"
              >
                <FiRadio className="w-6 h-6 text-white" />
              </button>

              {/* Speaker Toggle */}
              <button
                onClick={toggleSpeaker}
                className={`p-4 rounded-full transition-colors ${
                  isSpeakerOn ? 'bg-blue-500 hover:bg-blue-600' : 'bg-gray-700 hover:bg-gray-600'
                }`}
                aria-label={isSpeakerOn ? t('call.speaker_off', 'Speaker Off') : t('call.speaker_on', 'Speaker On')}
              >
                {isSpeakerOn ? (
                  <FiVolume2 className="w-6 h-6 text-white" />
                ) : (
                  <FiVolumeX className="w-6 h-6 text-white" />
                )}
              </button>

              {/* End Call */}
              <button
                onClick={() => endCall('hangup')}
                className="p-4 rounded-full bg-red-500 hover:bg-red-600 transition-colors shadow-lg"
                aria-label={t('call.hangup', 'End Call')}
              >
                <FiPhoneOff className="w-8 h-8 text-white" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
