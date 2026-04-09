/**
 * WebRTC Calling Types
 *
 * Types for voice/video calling functionality.
 */

/** Type of call */
export enum CallType {
  Audio = 'audio',
  Video = 'video',
}

/** Call state machine */
export enum CallState {
  Idle = 'idle',
  Dialing = 'dialing',       // Outgoing call
  Ringing = 'ringing',       // Incoming call
  Connecting = 'connecting', // SDP exchange
  Connected = 'connected',   // Active call
  Ended = 'ended',
}

/** Call end reason */
export enum CallEndReason {
  Hangup = 'hangup',
  Missed = 'missed',
  Rejected = 'rejected',
  Error = 'error',
  ConnectionLost = 'connection_lost',
}

/** Information about a call participant */
export interface CallPeer {
  userId: string;
  username: string;
  avatarUrl?: string;
}

/** Active call state */
export interface ActiveCall {
  id: string;
  type: CallType;
  state: CallState;
  initiator: CallPeer;    // Who started the call
  recipient: CallPeer;    // Who is being called
  startTime?: number;     // Timestamp when connected
  endTime?: number;       // Timestamp when ended
  endReason?: CallEndReason;
  isMuted: boolean;
  isCameraOn: boolean;
  isSpeakerOn: boolean;
  error?: string;
}

/** WebRTC signaling message types */
export interface SignalingMessage {
  type: 'offer' | 'answer' | 'ice-candidate' | 'hangup';
  callId: string;
  from: string;
  to: string;
  data?: unknown;
}

/** SDP Offer/Answer payload */
export interface SDPPayload {
  type: 'offer' | 'answer';
  sdp: string;
}

/** ICE Candidate payload (matches WebSocketClient IceCandidate) */
export interface ICECandidatePayload {
  candidate: string;
  sdpMid: string | null;
  sdpMLineIndex: number | null;
}

/** Call statistics */
export interface CallStats {
  duration: number;           // seconds
  bytesSent: number;
  bytesReceived: number;
  packetsLost: number;
  jitter: number;             // ms
  resolution?: string;        // e.g. "1280x720"
  codec?: string;
}
