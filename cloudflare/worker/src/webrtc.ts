// cloudflare/worker/src/webrtc.ts
/**
 * WebRTC/STUN сервер для P2P звонков
 */

export interface WebRTCOffer {
  type: 'offer' | 'answer';
  sdp: string;
  caller: string;
  callee: string;
  callId: string;
}

export class WebRTCHandler {
  private kv: KVNamespace;

  constructor(kv: KVNamespace) {
    this.kv = kv;
  }

  async createOffer(caller: string, callee: string, sdp: string): Promise<string> {
    const callId = crypto.randomUUID();
    
    const offer: WebRTCOffer = {
      type: 'offer',
      sdp,
      caller,
      callee,
      callId,
    };

    await this.kv.put(`call:${callId}`, JSON.stringify(offer), { expirationTtl: 300 });

    return callId;
  }

  async getOffer(callId: string): Promise<WebRTCOffer | null> {
    const data = await this.kv.get(`call:${callId}`);
    if (!data) return null;
    return JSON.parse(data);
  }

  async createAnswer(callId: string, sdp: string): Promise<void> {
    const offer = await this.getOffer(callId);
    if (!offer) throw new Error('Offer not found');

    const answer: WebRTCOffer = {
      type: 'answer',
      sdp,
      caller: offer.caller,
      callee: offer.callee,
      callId,
    };

    await this.kv.put(`call:${callId}:answer`, JSON.stringify(answer), { expirationTtl: 300 });
  }

  async getAnswer(callId: string): Promise<WebRTCOffer | null> {
    const data = await this.kv.get(`call:${callId}:answer`);
    if (!data) return null;
    return JSON.parse(data);
  }

  async addIceCandidate(callId: string, candidate: RTCIceCandidateInit, from: string): Promise<void> {
    const key = `call:${callId}:ice:${from}`;
    const existing = await this.kv.get(key);
    const candidates: RTCIceCandidateInit[] = existing ? JSON.parse(existing) : [];
    candidates.push(candidate);
    await this.kv.put(key, JSON.stringify(candidates), { expirationTtl: 300 });
  }

  async getIceCandidates(callId: string, forUser: string): Promise<RTCIceCandidateInit[]> {
    const otherUser = forUser === 'caller' ? 'callee' : 'caller';
    const key = `call:${callId}:ice:${otherUser}`;
    const data = await this.kv.get(key);
    return data ? JSON.parse(data) : [];
  }

  async endCall(callId: string): Promise<void> {
    await this.kv.delete(`call:${callId}`);
    await this.kv.delete(`call:${callId}:answer`);
    await this.kv.delete(`call:${callId}:ice:caller`);
    await this.kv.delete(`call:${callId}:ice:callee`);
  }
}
