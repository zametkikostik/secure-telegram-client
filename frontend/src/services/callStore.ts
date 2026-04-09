/**
 * Call Store — Zustand state management for WebRTC calls
 */

import { create } from 'zustand';
import { CallState, CallType, CallPeer, ActiveCall, CallEndReason } from '../types/call';
import { WebRTCService } from '../services/webrtcService';

interface CallStore {
  // State
  currentCall: ActiveCall | null;
  callState: CallState;
  remoteStream: MediaStream | null;
  localStream: MediaStream | null;
  isMuted: boolean;
  isCameraOn: boolean;
  isSpeakerOn: boolean;
  callDuration: number;

  // Actions
  startOutgoingCall: (type: CallType, recipient: CallPeer) => Promise<void>;
  acceptCall: () => Promise<void>;
  rejectCall: () => void;
  endCall: (reason?: string) => void;
  toggleMute: () => void;
  toggleCamera: () => Promise<void>;
  toggleSpeaker: () => void;
  resetCallState: () => void;
}

let durationInterval: ReturnType<typeof setInterval> | null = null;

export const useCallStore = create<CallStore>((set, get) => ({
  // Initial state
  currentCall: null,
  callState: CallState.Idle,
  remoteStream: null,
  localStream: null,
  isMuted: false,
  isCameraOn: false,
  isSpeakerOn: true,
  callDuration: 0,

  /** Start an outgoing call */
  startOutgoingCall: async (type: CallType, recipient: CallPeer) => {
    const service = WebRTCService.getInstance();
    const userId = localStorage.getItem('user_id') || 'unknown';
    const username = localStorage.getItem('username') || 'user';

    try {
      const call = await service.startCall(type, recipient, userId, username);

      set({
        currentCall: call,
        callState: call.state,
        isMuted: false,
        isCameraOn: call.isCameraOn,
        isSpeakerOn: true,
        callDuration: 0,
      });
    } catch (error) {
      console.error('Failed to start call:', error);
      set({
        callState: CallState.Idle,
        currentCall: null,
      });
    }
  },

  /** Accept incoming call */
  acceptCall: async () => {
    const { currentCall } = get();
    if (!currentCall) return;

    try {
      const service = WebRTCService.getInstance();
      await service.acceptCall(currentCall.id);

      set({
        callState: CallState.Connecting,
      });
    } catch (error) {
      console.error('Failed to accept call:', error);
    }
  },

  /** Reject incoming call */
  rejectCall: () => {
    const { currentCall } = get();
    if (!currentCall) return;

    const service = WebRTCService.getInstance();
    service.rejectCall(currentCall.id);

    set({
      callState: CallState.Ended,
      currentCall: null,
    });
  },

  /** End current call */
  endCall: (reason?: string) => {
    const service = WebRTCService.getInstance();
    service.endCall(reason);

    if (durationInterval) {
      clearInterval(durationInterval);
      durationInterval = null;
    }

    set({
      callState: CallState.Ended,
      currentCall: null,
      callDuration: 0,
      remoteStream: null,
      localStream: null,
    });
  },

  /** Toggle audio mute */
  toggleMute: () => {
    const service = WebRTCService.getInstance();
    service.toggleMute();

    set((state) => ({
      isMuted: !state.isMuted,
      currentCall: state.currentCall
        ? { ...state.currentCall, isMuted: !state.isMuted }
        : null,
    }));
  },

  /** Toggle camera */
  toggleCamera: async () => {
    const service = WebRTCService.getInstance();
    await service.toggleCamera();

    set((state) => ({
      isCameraOn: !state.isCameraOn,
      currentCall: state.currentCall
        ? { ...state.currentCall, isCameraOn: !state.isCameraOn }
        : null,
    }));
  },

  /** Toggle speaker */
  toggleSpeaker: () => {
    set((state) => ({
      isSpeakerOn: !state.isSpeakerOn,
      currentCall: state.currentCall
        ? { ...state.currentCall, isSpeakerOn: !state.isSpeakerOn }
        : null,
    }));
  },

  /** Reset call state */
  resetCallState: () => {
    if (durationInterval) {
      clearInterval(durationInterval);
      durationInterval = null;
    }

    set({
      currentCall: null,
      callState: CallState.Idle,
      callDuration: 0,
      remoteStream: null,
      localStream: null,
      isMuted: false,
      isCameraOn: false,
      isSpeakerOn: true,
    });
  },
}));
