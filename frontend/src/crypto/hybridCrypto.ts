// @ts-nocheck
// Hybrid Encryption: ECDH + AES-GCM
// Equivalent to Rust: X25519 + ChaCha20-Poly1305

import { KeyPair, arrayBufferToBase64, base64ToArrayBuffer } from './keyManager';
import { CryptoError } from './types';

// ============================================================================
// Encrypted Message Format
// ============================================================================

export interface EncryptedMessage {
  ephemeral_public_key: string;  // Base64
  ciphertext: string;            // Base64
  nonce: string;                 // Base64
  signature: string;             // Base64 (Ed25519/ECDSA)
  timestamp: number;
  version: number;
}

// ============================================================================
// ECDH Key Derivation
// ============================================================================

/**
 * Derive shared secret using ECDH (P-256)
 * Equivalent to X25519 in Rust backend
 */
export async function deriveSharedSecret(
  myPrivateKey: string,
  peerPublicKey: string
): Promise<ArrayBuffer> {
  try {
    const privateKeyBuffer = base64ToArrayBuffer(myPrivateKey);
    const publicKeyBuffer = base64ToArrayBuffer(peerPublicKey);

    // Import private key
    const privateKey = await crypto.subtle.importKey(
      'pkcs8',
      privateKeyBuffer,
      { name: 'ECDH', namedCurve: 'P-256' },
      false,
      ['deriveBits']
    );

    // Import public key
    const publicKey = await crypto.subtle.importKey(
      'raw',
      publicKeyBuffer,
      { name: 'ECDH', namedCurve: 'P-256' },
      false,
      []
    );

    // Derive shared bits (256 bits for AES-256)
    const sharedBits = await crypto.subtle.deriveBits(
      { name: 'ECDH', public: publicKey },
      privateKey,
      256
    );

    return sharedBits;
  } catch (error) {
    throw new CryptoError('ECDH key derivation failed', error);
  }
}

// ============================================================================
// AES-GCM Encryption
// ============================================================================

/**
 * Encrypt message using AES-256-GCM
 * Equivalent to ChaCha20-Poly1305 in Rust backend
 */
export async function encryptMessage(
  plaintext: string | Uint8Array,
  sharedSecret: ArrayBuffer,
  signingKey: string
): Promise<EncryptedMessage> {
  try {
    // Generate ephemeral keypair for this message
    const ephemeralKeyPair = await crypto.subtle.generateKey(
      { name: 'ECDH', namedCurve: 'P-256' },
      true,
      ['deriveBits']
    );
    const ephemeralPublicKey = await crypto.subtle.exportKey('raw', ephemeralKeyPair.publicKey);

    // Derive AES key from shared secret
    const aesKey = await crypto.subtle.importKey(
      'raw',
      sharedSecret,
      'AES-GCM',
      false,
      ['encrypt']
    );

    // Generate random nonce (96 bits)
    const nonceBytes = new Uint8Array(12);
    crypto.getRandomValues(nonceBytes);

    // Encrypt
    const plaintextBuffer = typeof plaintext === 'string' 
      ? new TextEncoder().encode(plaintext) 
      : plaintext;

    const nonceArray = new Uint8Array(nonceBytes);
    
    const encrypted = await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv: nonceArray },
      aesKey,
      plaintextBuffer
    );

    // Sign the ciphertext (convert to proper ArrayBuffer)
    const encryptedArray = new Uint8Array(encrypted);
    const signature = await signData(encryptedArray.buffer as ArrayBuffer, signingKey);

    return {
      ephemeral_public_key: arrayBufferToBase64(ephemeralPublicKey),
      ciphertext: arrayBufferToBase64(encryptedArray.buffer),
      nonce: btoa(String.fromCharCode(...nonceBytes)),
      signature: arrayBufferToBase64(signature),
      timestamp: Date.now(),
      version: 1,
    };
  } catch (error) {
    throw new CryptoError('Encryption failed', error);
  }
}

/**
 * Decrypt message using AES-256-GCM
 */
export async function decryptMessage(
  encrypted: EncryptedMessage,
  myPrivateKey: string,
  verifyKey: string
): Promise<string> {
  try {
    const sharedSecret = await deriveSharedSecret(
      myPrivateKey,
      encrypted.ephemeral_public_key
    );

    // Import AES key
    const aesKey = await crypto.subtle.importKey(
      'raw',
      sharedSecret,
      'AES-GCM',
      false,
      ['decrypt']
    );

    const ciphertext = new Uint8Array(base64ToArrayBuffer(encrypted.ciphertext));
    const nonceBuffer = base64ToArrayBuffer(encrypted.nonce);

    // Verify signature
    const isValid = await verifySignature(
      ciphertext,
      encrypted.signature,
      verifyKey
    );

    if (!isValid) {
      throw new CryptoError('Signature verification failed');
    }

    // Decrypt
    const decrypted = await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv: nonceBuffer },
      aesKey,
      ciphertext.buffer
    );

    return new TextDecoder().decode(decrypted);
  } catch (error) {
    if (error instanceof CryptoError) throw error;
    throw new CryptoError('Decryption failed', error);
  }
}

// ============================================================================
// Digital Signatures (ECDSA P-256)
// ============================================================================

/**
 * Sign data using ECDSA
 */
export async function signData(
  data: Uint8Array,
  privateKey: string
): Promise<ArrayBuffer> {
  try {
    const keyBuffer = base64ToArrayBuffer(privateKey);
    const privateKeyObj = await crypto.subtle.importKey(
      'pkcs8',
      keyBuffer,
      { name: 'ECDSA', namedCurve: 'P-256' },
      false,
      ['sign']
    );

    return await crypto.subtle.sign(
      { name: 'ECDSA', hash: 'SHA-256' },
      privateKeyObj,
      data
    );
  } catch (error) {
    throw new CryptoError('Signing failed', error);
  }
}

/**
 * Verify ECDSA signature
 */
export async function verifySignature(
  data: Uint8Array,
  signature: string,
  publicKey: string
): Promise<boolean> {
  try {
    const keyBuffer = base64ToArrayBuffer(publicKey);
    const sigBuffer = base64ToArrayBuffer(signature);

    const publicKeyObj = await crypto.subtle.importKey(
      'raw',
      keyBuffer,
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
  } catch (error) {
    throw new CryptoError('Signature verification failed', error);
  }
}
