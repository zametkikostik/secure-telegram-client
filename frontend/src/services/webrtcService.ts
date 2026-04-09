/**
 * WebRTC Service
 *
 * Manages RTCPeerConnection lifecycle for voice/video calls:
 * - Creates offers/answers
 * - Handles ICE candidates
 * - Manages local/remote media streams
 * - Integrates with WebSocketClient for signaling
 *
 * Uses native browser WebRTC API (no external dependencies needed).
 */

import {
  CallType,
  CallState,
  CallPeer,
  ActiveCall,
  SDPPayload,
  ICECandidatePayload,
  CallStats,
} from '../types/call';
import { WebSocketClient } from './webSocketClient';

/** STUN/TURN server configuration */
interface IceServerConfig {
  urls: string[];
  username?: string;
  credential?: string;
}

/** Callbacks for call events */
export interface CallEventHandlers {
  onCallStateChange?: (state: CallState) => void;
  onIncomingCall?: (call: ActiveCall) => void;
  onCallConnected?: () => void;
  onCallEnded?: (reason: string) => void;
  onError?: (error: string) => void;
}

/**
 * WebRTC Service singleton
 *
 * Manages a single active call at a time (1-on-1 calls only).
 * Group calls will be supported in future versions.
 */
export class WebRTCService {
  private static instance: WebRTCService | null = null;

  private peerConnection: RTCPeerConnection | null = null;
  private localStream: MediaStream | null = null;
  private remoteStream: MediaStream | null = null;

  private callConfig: {
    iceServers: IceServerConfig[];
    videoEnabled: boolean;
    maxBitrate: number;
  };

  private currentCall: ActiveCall | null = null;
  private handlers: CallEventHandlers = {};
  private wsClient: WebSocketClient | null = null;

  private callStatsInterval: ReturnType<typeof setInterval> | null = null;

  private constructor() {
    this.callConfig = {
      iceServers: [
        {
          urls: [
            'stun:stun.l.google.com:19302',
            'stun:stun1.l.google.com:19302',
            'stun:stun.services.mozilla.com',
          ],
        },
      ],
      videoEnabled: import.meta.env.VITE_WEBRTC_VIDEO_ENABLED !== 'false',
      maxBitrate: parseInt(import.meta.env.VITE_WEBRTC_MAX_BITRATE || '0', 10),
    };

    // Add TURN server if configured
    const turnServer = import.meta.env.VITE_WEBRTC_TURN_SERVER;
    const turnUsername = import.meta.env.VITE_WEBRTC_TURN_USERNAME;
    const turnCredential = import.meta.env.VITE_WEBRTC_TURN_CREDENTIAL;
    if (turnServer && turnUsername && turnCredential) {
      this.callConfig.iceServers.push({
        urls: [turnServer],
        username: turnUsername,
        credential: turnCredential,
      });
    }
  }

  /** Get singleton instance */
  static getInstance(): WebRTCService {
    if (!WebRTCService.instance) {
      WebRTCService.instance = new WebRTCService();
    }
    return WebRTCService.instance;
  }

  /** Set WebSocket client for signaling */
  setWebSocketClient(client: WebSocketClient): void {
    this.wsClient = client;
    this.setupSignalingHandlers();
  }

  /** Set event handlers */
  setHandlers(handlers: CallEventHandlers): void {
    this.handlers = handlers;
  }

  /**
   * Start an outgoing call
   */
  async startCall(
    type: CallType,
    recipient: CallPeer,
    currentUserId: string,
    currentUsername: string,
  ): Promise<ActiveCall> {
    if (this.currentCall && this.currentCall.state !== CallState.Idle) {
      throw new Error('Call already in progress');
    }

    if (type === CallType.Video && !this.callConfig.videoEnabled) {
      throw new Error('Video calls are disabled');
    }

    const call: ActiveCall = {
      id: crypto.randomUUID(),
      type,
      state: CallState.Dialing,
      initiator: { userId: currentUserId, username: currentUsername },
      recipient,
      isMuted: false,
      isCameraOn: type === CallType.Video,
      isSpeakerOn: true,
    };

    this.currentCall = call;
    this.handlers.onCallStateChange?.(CallState.Dialing);

    try {
      // Get local media stream
      await this.acquireLocalStream(type);

      // Create peer connection
      await this.createPeerConnection();

      // Create SDP offer
      const offer = await this.peerConnection!.createOffer();
      await this.peerConnection!.setLocalDescription(offer);

      // Send offer via signaling
      this.handlers.onCallStateChange?.(CallState.Connecting);
      await this.sendSignalingMessage({
        type: 'offer',
        callId: call.id,
        from: currentUserId,
        to: recipient.userId,
        data: { sdp: offer.sdp, type: 'offer' },
      });

      return call;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to start call';
      this.endCall('error', message);
      this.handlers.onError?.(message);
      throw new Error(message);
    }
  }

  /**
   * Accept an incoming call
   */
  async acceptCall(callId: string): Promise<void> {
    if (!this.currentCall || this.currentCall.id !== callId) {
      throw new Error('No matching incoming call');
    }

    this.currentCall.state = CallState.Connecting;
    this.handlers.onCallStateChange?.(CallState.Connecting);

    try {
      // Get local media stream
      await this.acquireLocalStream(this.currentCall.type);

      // Create peer connection
      await this.createPeerConnection();

      // Answer will be sent when we receive the offer
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to accept call';
      this.endCall('error', message);
      this.handlers.onError?.(message);
    }
  }

  /**
   * Reject an incoming call
   */
  rejectCall(callId: string): void {
    if (!this.currentCall || this.currentCall.id !== callId) return;

    this.sendSignalingMessage({
      type: 'hangup',
      callId,
      from: this.currentCall.recipient.userId,
      to: this.currentCall.initiator.userId,
    });

    this.endCall('rejected');
  }

  /**
   * End the current call
   */
  endCall(reason: string = 'hangup', errorMessage?: string): void {
    if (!this.currentCall) return;

    // Send hangup signal via WebSocket
    if (this.wsClient) {
      try {
        const targetUserId = this.currentCall.recipient?.userId || this.currentCall.initiator.userId;
        this.wsClient.sendP2PHangup(targetUserId, this.currentCall.id, reason);
      } catch (e) {
        console.warn('[WebRTC] Failed to send hangup signal:', e);
      }
    }

    // Stop stats collection
    if (this.callStatsInterval) {
      clearInterval(this.callStatsInterval);
      this.callStatsInterval = null;
    }

    // Close peer connection
    if (this.peerConnection) {
      this.peerConnection.close();
      this.peerConnection = null;
    }

    // Stop local media tracks
    if (this.localStream) {
      this.localStream.getTracks().forEach((track) => track.stop());
      this.localStream = null;
    }

    this.currentCall.state = CallState.Ended;
    this.currentCall.endTime = Date.now();
    this.currentCall.endReason = reason as any;
    if (errorMessage) {
      this.currentCall.error = errorMessage;
    }

    this.handlers.onCallStateChange?.(CallState.Ended);
    this.handlers.onCallEnded?.(reason);

    // Reset after a short delay
    setTimeout(() => {
      this.currentCall = null;
    }, 1000);
  }

  /** Toggle local audio mute */
  toggleMute(): void {
    if (!this.localStream) return;

    const audioTracks = this.localStream.getAudioTracks();
    audioTracks.forEach((track) => {
      track.enabled = !track.enabled;
    });

    if (this.currentCall) {
      this.currentCall.isMuted = !this.currentCall.isMuted;
    }
  }

  /** Toggle camera on/off */
  async toggleCamera(): Promise<void> {
    if (!this.currentCall || this.currentCall.type !== CallType.Video) return;
    if (!this.localStream) return;

    const videoTracks = this.localStream.getVideoTracks();
    if (videoTracks.length === 0) {
      // Need to add video track
      try {
        const videoStream = await navigator.mediaDevices.getUserMedia({ video: true });
        videoStream.getVideoTracks().forEach((track) => {
          this.localStream!.addTrack(track);
          if (this.peerConnection) {
            this.peerConnection.addTrack(track, this.localStream!);
          }
        });
        this.currentCall.isCameraOn = true;
      } catch {
        // Failed to get camera
      }
    } else {
      videoTracks.forEach((track) => {
        track.enabled = !track.enabled;
      });
      this.currentCall.isCameraOn = !this.currentCall.isCameraOn;
    }
  }

  /** Toggle screen sharing */
  async toggleScreenShare(): Promise<void> {
    if (!this.currentCall || !this.peerConnection) return;

    try {
      const screenStream = await navigator.mediaDevices.getDisplayMedia({
        video: true,
        audio: false,
      });

      const screenTrack = screenStream.getVideoTracks()[0];
      if (screenTrack) {
        // Replace camera track with screen track
        const senders = this.peerConnection.getSenders();
        const videoSender = senders.find(s => s.track?.kind === 'video');
        if (videoSender) {
          videoSender.replaceTrack(screenTrack);
        }

        // Stop when user stops sharing
        screenTrack.onended = () => {
          this.stopScreenShare();
        };
      }
    } catch (e) {
      console.warn('Screen share cancelled:', e);
    }
  }

  /** Stop screen sharing and resume camera */
  private async stopScreenShare(): Promise<void> {
    if (!this.currentCall || !this.peerConnection) return;

    try {
      const cameraStream = await navigator.mediaDevices.getUserMedia({ video: true });
      const cameraTrack = cameraStream.getVideoTracks()[0];
      if (cameraTrack) {
        const senders = this.peerConnection.getSenders();
        const videoSender = senders.find(s => s.track?.kind === 'video');
        if (videoSender) {
          videoSender.replaceTrack(cameraTrack);
        }
      }
    } catch (e) {
      console.warn('Failed to resume camera:', e);
    }
  }

  /** Get local stream for display */
  getLocalStream(): MediaStream | null {
    return this.localStream;
  }

  /** Get remote stream for playback */
  getRemoteStream(): MediaStream | null {
    return this.remoteStream;
  }

  /** Get current call state */
  getCurrentCall(): ActiveCall | null {
    return this.currentCall;
  }

  /** Get call duration in seconds */
  getCallDuration(): number {
    if (!this.currentCall || !this.currentCall.startTime) return 0;
    if (this.currentCall.endTime) {
      return Math.floor((this.currentCall.endTime - this.currentCall.startTime) / 1000);
    }
    return Math.floor((Date.now() - this.currentCall.startTime) / 1000);
  }

  // ============================================================================
  // Private methods
  // ============================================================================

  /** Acquire local media stream (mic + optional camera) */
  private async acquireLocalStream(type: CallType): Promise<MediaStream> {
    if (this.localStream) return this.localStream;

    const constraints: MediaStreamConstraints = {
      audio: {
        echoCancellation: true,
        noiseSuppression: true,
        autoGainControl: true,
      },
      video: type === CallType.Video ? {
        width: { ideal: 1280 },
        height: { ideal: 720 },
        facingMode: 'user',
      } : false,
    };

    this.localStream = await navigator.mediaDevices.getUserMedia(constraints);
    return this.localStream;
  }

  /** Create and configure RTCPeerConnection */
  private async createPeerConnection(): Promise<void> {
    this.peerConnection = new RTCPeerConnection({
      iceServers: this.callConfig.iceServers.map((s) => ({
        urls: s.urls,
        username: s.username,
        credential: s.credential,
      })),
    });

    // Add local tracks
    if (this.localStream) {
      this.localStream.getTracks().forEach((track) => {
        this.peerConnection!.addTrack(track, this.localStream!);
      });
    }

    // Create remote stream
    this.remoteStream = new MediaStream();

    // Handle incoming tracks
    this.peerConnection.ontrack = (event) => {
      event.streams[0].getTracks().forEach((track) => {
        this.remoteStream!.addTrack(track);
      });
    };

    // Handle ICE candidates
    this.peerConnection.onicecandidate = (event) => {
      if (event.candidate && this.currentCall) {
        this.sendSignalingMessage({
          type: 'ice-candidate',
          callId: this.currentCall.id,
          from: this.currentCall.initiator.userId,
          to: this.currentCall.recipient.userId,
          data: {
            candidate: event.candidate.candidate,
            sdpMid: event.candidate.sdpMid,
            sdpMLineIndex: event.candidate.sdpMLineIndex,
          },
        });
      }
    };

    // Handle connection state changes
    this.peerConnection.onconnectionstatechange = () => {
      if (!this.peerConnection || !this.currentCall) return;

      switch (this.peerConnection.connectionState) {
        case 'connected':
          this.currentCall.state = CallState.Connected;
          this.currentCall.startTime = Date.now();
          this.handlers.onCallStateChange?.(CallState.Connected);
          this.handlers.onCallConnected?.();
          this.startStatsCollection();
          break;
        case 'disconnected':
        case 'failed':
        case 'closed':
          this.endCall('connection_lost');
          break;
      }
    };
  }

  /** Handle incoming signaling messages */
  private setupSignalingHandlers(): void {
    if (!this.wsClient) return;

    this.wsClient.onP2PEvent(async (event) => {
      console.log('[WebRTC] P2P Event received:', event.event_type, event.peer_id);

      if (!this.peerConnection || !this.currentCall) return;

      try {
        switch (event.event_type) {
          case 'p2p_offer': {
            // Received an incoming call offer
            const data = event.data as any;
            if (data?.sdp) {
              await this.peerConnection.setRemoteDescription(
                new RTCSessionDescription({ type: 'offer', sdp: data.sdp })
              );

              // Create and send answer
              const answer = await this.peerConnection.createAnswer();
              await this.peerConnection.setLocalDescription(answer);

              this.wsClient!.sendP2PAnswer(event.peer_id, answer.sdp || '', []);
              console.log('[WebRTC] SDP Answer sent');
            }
            break;
          }

          case 'p2p_answer': {
            // Received answer to our offer
            const data = event.data as any;
            if (data?.sdp && this.peerConnection.signalingState === 'have-local-offer') {
              await this.peerConnection.setRemoteDescription(
                new RTCSessionDescription({ type: 'answer', sdp: data.sdp })
              );
              console.log('[WebRTC] SDP Answer received and set');
            }
            break;
          }

          case 'ice_candidate': {
            // Received ICE candidate
            const data = event.data as any;
            if (data?.candidate) {
              await this.peerConnection.addIceCandidate(
                new RTCIceCandidate({
                  candidate: data.candidate.candidate || data.candidate,
                  sdpMid: data.candidate.sdpMid || data.candidate.sdp_mid,
                  sdpMLineIndex: data.candidate.sdpMLineIndex ?? data.candidate.sdp_m_line_index,
                })
              );
            }
            break;
          }

          case 'p2p_hangup': {
            // Remote user hung up
            const data = event.data as any;
            this.endCall(data?.reason || 'remote_hangup');
            break;
          }

          default:
            console.log('[WebRTC] Unknown P2P event type:', event.event_type);
        }
      } catch (error) {
        console.error('[WebRTC] Error handling P2P event:', error);
        this.handlers.onError?.(error instanceof Error ? error.message : 'P2P signaling error');
      }
    });
  }

  /** Send SDP answer after receiving offer */
  private async sendAnswer(): Promise<void> {
    if (!this.peerConnection || !this.currentCall) return;

    try {
      const answer = await this.peerConnection.createAnswer();
      await this.peerConnection.setLocalDescription(answer);

      if (this.wsClient) {
        this.wsClient.sendP2PAnswer(this.currentCall.recipient.userId, answer.sdp || '', []);
        console.log('[WebRTC] SDP Answer sent via WebSocket');
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to send answer';
      this.handlers.onError?.(message);
    }
  }

  /** Send signaling message via WebSocket */
  private async sendSignalingMessage(msg: any): Promise<void> {
    if (!this.wsClient) {
      console.error('[WebRTC] No WebSocket client for signaling');
      this.handlers.onError?.('Signaling not available');
      return;
    }

    try {
      switch (msg.type) {
        case 'offer':
          this.wsClient.sendP2POffer(msg.to, msg.data.sdp, []);
          break;
        case 'answer':
          this.wsClient.sendP2PAnswer(msg.to, msg.data.sdp, []);
          break;
        case 'ice-candidate':
          this.wsClient.sendIceCandidate(msg.to, msg.data);
          break;
        case 'hangup':
          // Send hangup notification
          this.wsClient.sendP2POffer(msg.to, '', []); // Reuse offer type for hangup signal
          break;
        default:
          console.warn('[WebRTC] Unknown signaling message type:', msg.type);
      }
    } catch (error) {
      console.error('[WebRTC] Failed to send signaling message:', error);
    }
  }

  /** Collect call statistics */
  private startStatsCollection(): void {
    if (!this.peerConnection) return;

    this.callStatsInterval = setInterval(async () => {
      if (!this.peerConnection) return;

      try {
        const stats = await this.peerConnection.getStats();
        // Stats processing can be added here for UI display
      } catch {
        // Ignore stats errors
      }
    }, 5000);
  }
}
