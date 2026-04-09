// High-level Message Encryption API
// Integrates with P2P Client for E2EE messaging

import { generateE2EEKeyPair, E2EEKeyPair, getPublicKeyBundle, saveKeyPair, loadKeyPair } from './keyManager';
import { encryptMessage, decryptMessage, EncryptedMessage } from './hybridCrypto';
import { p2p, P2PMessage } from '../services/p2pClient';

// ============================================================================
// E2EE Session Manager
// ============================================================================

export class E2EESession {
  private keyPair: E2EEKeyPair | null = null;
  private peerPublicKeys: Map<string, { x25519: string; ed25519: string }> = new Map();
  private messageQueue: Array<{ toDeviceId: string; content: string }> = [];

  constructor() {
    this.loadKeys();
  }

  // ============================================================================
  // Key Management
  // ============================================================================

  /**
   * Initialize E2EE - generate or load keys
   */
  async initialize(): Promise<{ deviceId: string; publicKeyBundle: any }> {
    // Try to load existing keys
    this.keyPair = await loadKeyPair();

    if (!this.keyPair) {
      // Generate new keys
      this.keyPair = await generateE2EEKeyPair();
      await saveKeyPair(this.keyPair);
    }

    const publicKeyBundle = getPublicKeyBundle(this.keyPair);

    return {
      deviceId: this.keyPair.deviceId,
      publicKeyBundle,
    };
  }

  /**
   * Get device ID
   */
  getDeviceId(): string | null {
    return this.keyPair?.deviceId || null;
  }

  /**
   * Get public key for sharing
   */
  getPublicKey(): string | null {
    return this.keyPair?.x25519.publicKey || null;
  }

  // ============================================================================
  // Peer Key Exchange
  // ============================================================================

  /**
   * Store peer's public key
   */
  storePeerKey(deviceId: string, x25519Key: string, ed25519Key: string): void {
    this.peerPublicKeys.set(deviceId, {
      x25519: x25519Key,
      ed25519: ed25519Key,
    });
  }

  /**
   * Get peer's public key
   */
  getPeerKey(deviceId: string): { x25519: string; ed25519: string } | null {
    return this.peerPublicKeys.get(deviceId) || null;
  }

  // ============================================================================
  // Key Signing & Verification (ECDSA P-256)
  // ============================================================================

  /**
   * Sign X25519 public key with ECDSA signing key
   * Prevents MITM attacks by verifying key ownership
   */
  async signPublicKey(): Promise<string> {
    if (!this.keyPair) {
      throw new Error('E2EE not initialized');
    }

    const data = new TextEncoder().encode(this.keyPair.x25519.publicKey);
    const privateKeyBuffer = Uint8Array.from(atob(this.keyPair.ed25519.privateKey), c => c.charCodeAt(0));

    // Import signing key
    const privateKey = await crypto.subtle.importKey(
      'pkcs8',
      privateKeyBuffer,
      { name: 'ECDSA', namedCurve: 'P-256' },
      false,
      ['sign']
    );

    const signature = await crypto.subtle.sign(
      { name: 'ECDSA', hash: 'SHA-256' },
      privateKey,
      data
    );

    return btoa(String.fromCharCode(...new Uint8Array(signature)));
  }

  /**
   * Verify X25519 public key signature
   * Returns true if key belongs to claimed owner
   */
  async verifyPublicKey(publicKey: string, signature: string, signerPublicKey: string): Promise<boolean> {
    try {
      const data = new TextEncoder().encode(publicKey);
      const sigBuffer = Uint8Array.from(atob(signature), c => c.charCodeAt(0));
      const pubKeyBuffer = Uint8Array.from(atob(signerPublicKey), c => c.charCodeAt(0));

      const publicKeyObj = await crypto.subtle.importKey(
        'raw',
        pubKeyBuffer,
        { name: 'ECDSA', namedCurve: 'P-256' },
        false,
        ['verify']
      );

      return await crypto.subtle.verify(
        { name: 'ECDSA', hash: 'SHA-256' },
        publicKeyObj,
        sigBuffer,
        data
      );
    } catch {
      return false;
    }
  }

  // ============================================================================
  // Message Signing
  // ============================================================================

  /**
   * Sign ciphertext before sending
   * Prevents message tampering
   */
  async signMessage(ciphertext: string): Promise<string> {
    if (!this.keyPair) {
      throw new Error('E2EE not initialized');
    }

    const data = new TextEncoder().encode(ciphertext);
    const privateKeyBuffer = Uint8Array.from(atob(this.keyPair.ed25519.privateKey), c => c.charCodeAt(0));

    const privateKey = await crypto.subtle.importKey(
      'pkcs8',
      privateKeyBuffer,
      { name: 'ECDSA', namedCurve: 'P-256' },
      false,
      ['sign']
    );

    const signature = await crypto.subtle.sign(
      { name: 'ECDSA', hash: 'SHA-256' },
      privateKey,
      data
    );

    return btoa(String.fromCharCode(...new Uint8Array(signature)));
  }

  /**
   * Verify message signature
   * Returns true if message wasn't tampered
   */
  async verifyMessageSignature(ciphertext: string, signature: string, signerPublicKey: string): Promise<boolean> {
    try {
      const data = new TextEncoder().encode(ciphertext);
      const sigBuffer = Uint8Array.from(atob(signature), c => c.charCodeAt(0));
      const pubKeyBuffer = Uint8Array.from(atob(signerPublicKey), c => c.charCodeAt(0));

      const publicKeyObj = await crypto.subtle.importKey(
        'raw',
        pubKeyBuffer,
        { name: 'ECDSA', namedCurve: 'P-256' },
        false,
        ['verify']
      );

      return await crypto.subtle.verify(
        { name: 'ECDSA', hash: 'SHA-256' },
        publicKeyObj,
        sigBuffer,
        data
      );
    } catch {
      return false;
    }
  }

  // ============================================================================
  // Encryption/Decryption
  // ============================================================================

  /**
   * Encrypt and send message via P2P with full ECDH key exchange + signatures + replay protection
   */
  async sendEncryptedMessage(
    toDeviceId: string,
    plaintext: string
  ): Promise<{ success: boolean; messageId?: string }> {
    if (!this.keyPair) {
      throw new Error('E2EE not initialized. Call initialize() first.');
    }

    const peerKey = this.getPeerKey(toDeviceId);
    if (!peerKey) {
      throw new Error(`No public key for device: ${toDeviceId}`);
    }

    try {
      // Generate ephemeral ECDH keypair for this message
      const ephemeralKeyPair = await crypto.subtle.generateKey(
        { name: 'ECDH', namedCurve: 'P-256' },
        true,
        ['deriveBits']
      );

      // Import peer's public key
      const peerKeyBuffer = Uint8Array.from(atob(peerKey.x25519), c => c.charCodeAt(0));
      const peerPublicKey = await crypto.subtle.importKey(
        'raw',
        peerKeyBuffer,
        { name: 'ECDH', namedCurve: 'P-256' },
        false,
        []
      );

      // Derive shared secret
      const sharedBits = await crypto.subtle.deriveBits(
        { name: 'ECDH', public: peerPublicKey },
        ephemeralKeyPair.privateKey,
        256
      );

      // Derive AES key from shared secret
      const aesKey = await crypto.subtle.importKey(
        'raw',
        sharedBits,
        'AES-GCM',
        false,
        ['encrypt']
      );

      // Encrypt message with timestamp (replay protection)
      const timestamp = Date.now();
      const nonce = crypto.getRandomValues(new Uint8Array(12));
      const plaintextWithTimestamp = JSON.stringify({ msg: plaintext, ts: timestamp });
      const plaintextBuffer = new TextEncoder().encode(plaintextWithTimestamp);

      const ciphertext = await crypto.subtle.encrypt(
        { name: 'AES-GCM', iv: nonce },
        aesKey,
        plaintextBuffer
      );

      // Sign the ciphertext
      const ciphertextB64 = btoa(String.fromCharCode(...new Uint8Array(ciphertext)));
      const signature = await this.signMessage(ciphertextB64);

      // Export ephemeral public key
      const ephemeralPubBuffer = await crypto.subtle.exportKey('raw', ephemeralKeyPair.publicKey);

      const encrypted: EncryptedMessage = {
        ephemeral_public_key: btoa(String.fromCharCode(...new Uint8Array(ephemeralPubBuffer))),
        ciphertext: ciphertextB64,
        nonce: btoa(String.fromCharCode(...nonce)),
        signature: signature, // ECDSA signature of ciphertext
        timestamp: timestamp, // Replay protection
        version: 2, // Version 2 = with signatures + timestamp
      };

      // Send via P2P
      const response = await p2p.sendMessage(
        toDeviceId,
        JSON.stringify(encrypted),
        'text'
      );

      return {
        success: response.success,
        messageId: response.messageId,
      };
    } catch (error) {
      console.error('E2EE encryption failed:', error);
      throw new Error(`Encryption failed: ${error}`);
    }
  }

  /**
   * Decrypt received message with ECDH + signature verification + replay protection
   */
  async decryptMessage(
    fromDeviceId: string,
    encryptedData: string
  ): Promise<{ text: string; timestamp: number; verified: boolean }> {
    if (!this.keyPair) {
      throw new Error('E2EE not initialized. Call initialize() first.');
    }

    try {
      const encrypted: EncryptedMessage = JSON.parse(encryptedData);

      // 1. Replay protection: check timestamp
      if (encrypted.timestamp) {
        const age = Date.now() - encrypted.timestamp;
        const maxAge = 24 * 60 * 60 * 1000; // 24 hours
        if (age > maxAge || age < 0) {
          console.warn('Message replay attack detected or clock skew');
          return { text: '[Message expired]', timestamp: encrypted.timestamp || 0, verified: false };
        }
      }

      // 2. Verify signature (if version 2+)
      let verified = false;
      if (encrypted.version >= 2 && encrypted.signature) {
        const peerKey = this.getPeerKey(fromDeviceId);
        if (peerKey) {
          verified = await this.verifyMessageSignature(
            encrypted.ciphertext,
            encrypted.signature,
            peerKey.ed25519
          );
          if (!verified) {
            console.warn('Message signature verification failed!');
            return { text: '[Signature invalid]', timestamp: encrypted.timestamp || 0, verified: false };
          }
        }
      }

      // 3. Import ephemeral public key
      const ephemeralPubBuffer = Uint8Array.from(atob(encrypted.ephemeral_public_key), c => c.charCodeAt(0));
      const ephemeralPublicKey = await crypto.subtle.importKey(
        'raw',
        ephemeralPubBuffer,
        { name: 'ECDH', namedCurve: 'P-256' },
        false,
        []
      );

      // 4. Import our private key
      const privateKeyBuffer = Uint8Array.from(atob(this.keyPair.x25519.privateKey), c => c.charCodeAt(0));
      const privateKey = await crypto.subtle.importKey(
        'pkcs8',
        privateKeyBuffer,
        { name: 'ECDH', namedCurve: 'P-256' },
        false,
        ['deriveBits']
      );

      // 5. Derive shared secret
      const sharedBits = await crypto.subtle.deriveBits(
        { name: 'ECDH', public: ephemeralPublicKey },
        privateKey,
        256
      );

      // 6. Derive AES key
      const aesKey = await crypto.subtle.importKey(
        'raw',
        sharedBits,
        'AES-GCM',
        false,
        ['decrypt']
      );

      // 7. Decrypt
      const ciphertext = Uint8Array.from(atob(encrypted.ciphertext), c => c.charCodeAt(0));
      const nonce = Uint8Array.from(atob(encrypted.nonce), c => c.charCodeAt(0));

      const decrypted = await crypto.subtle.decrypt(
        { name: 'AES-GCM', iv: nonce },
        aesKey,
        ciphertext
      );

      // 8. Parse timestamp from plaintext
      const text = new TextDecoder().decode(decrypted);
      const parsed = JSON.parse(text);

      return {
        text: parsed.msg || text,
        timestamp: parsed.ts || encrypted.timestamp || 0,
        verified: verified || encrypted.version < 2,
      };
    } catch (error) {
      console.error('Decryption failed:', error);
      return { text: '[Failed to decrypt]', timestamp: 0, verified: false };
    }
  }

  // ============================================================================
  // Message Queue (for offline)
  // ============================================================================

  /**
   * Queue message for later delivery
   */
  queueMessage(toDeviceId: string, content: string): void {
    this.messageQueue.push({ toDeviceId, content });
  }

  /**
   * Get queued messages
   */
  getQueuedMessages(): Array<{ toDeviceId: string; content: string }> {
    return [...this.messageQueue];
  }

  /**
   * Clear queued messages
   */
  clearQueue(): void {
    this.messageQueue = [];
  }

  // ============================================================================
  // Private Methods
  // ============================================================================

  private async loadKeys(): Promise<void> {
    try {
      this.keyPair = await loadKeyPair();
    } catch (error) {
      console.warn('Failed to load crypto keys:', error);
    }
  }
}

// ============================================================================
// Singleton Export
// ============================================================================

export const e2ee = new E2EESession();
