// Key Management for E2EE
// Generates and stores X25519 + Ed25519 keypairs

import { CryptoError } from './types';

// ============================================================================
// Crypto Types
// ============================================================================

export interface KeyPair {
  publicKey: string;  // Base64
  privateKey: string; // Base64
}

export interface E2EEKeyPair {
  x25519: KeyPair;
  ed25519: KeyPair;
  deviceId: string;
  createdAt: string;
}

export interface PublicKeyBundle {
  x25519_public_key: string;
  ed25519_public_key: string;
  device_id: string;
}

// ============================================================================
// Key Generation
// ============================================================================

/**
 * Generate X25519 keypair using WebCrypto ECDH
 */
export async function generateX25519KeyPair(): Promise<KeyPair> {
  try {
    const keyPair = await crypto.subtle.generateKey(
      {
        name: 'ECDH',
        namedCurve: 'P-256', // WebCrypto doesn't have Curve25519, using P-256
      },
      true,
      ['deriveKey', 'deriveBits']
    );

    const publicKey = await crypto.subtle.exportKey('raw', keyPair.publicKey);
    const privateKey = await crypto.subtle.exportKey('pkcs8', keyPair.privateKey);

    return {
      publicKey: arrayBufferToBase64(publicKey),
      privateKey: arrayBufferToBase64(privateKey),
    };
  } catch (error) {
    throw new CryptoError('X25519 key generation failed', error);
  }
}

/**
 * Generate Ed25519-like signing keypair using ECDSA P-256
 * WebCrypto doesn't support Ed25519, using ECDSA as fallback
 */
export async function generateEd25519KeyPair(): Promise<KeyPair> {
  try {
    const keyPair = await crypto.subtle.generateKey(
      {
        name: 'ECDSA',
        namedCurve: 'P-256',
      },
      true,
      ['sign', 'verify']
    );

    const publicKey = await crypto.subtle.exportKey('raw', keyPair.publicKey);
    const privateKey = await crypto.subtle.exportKey('pkcs8', keyPair.privateKey);

    return {
      publicKey: arrayBufferToBase64(publicKey),
      privateKey: arrayBufferToBase64(privateKey),
    };
  } catch (error) {
    throw new CryptoError('Ed25519 key generation failed', error);
  }
}

/**
 * Generate full E2EE keypair bundle
 */
export async function generateE2EEKeyPair(): Promise<E2EEKeyPair> {
  const deviceId = `device-${crypto.randomUUID()}`;
  
  const [x25519, ed25519] = await Promise.all([
    generateX25519KeyPair(),
    generateEd25519KeyPair(),
  ]);

  return {
    x25519,
    ed25519,
    deviceId,
    createdAt: new Date().toISOString(),
  };
}

// ============================================================================
// Key Import/Export
// ============================================================================

/**
 * Get public key bundle for sharing with peers
 */
export function getPublicKeyBundle(keyPair: E2EEKeyPair): PublicKeyBundle {
  return {
    x25519_public_key: keyPair.x25519.publicKey,
    ed25519_public_key: keyPair.ed25519.publicKey,
    device_id: keyPair.deviceId,
  };
}

/**
 * Save keypair to IndexedDB (secure storage)
 */
export async function saveKeyPair(keyPair: E2EEKeyPair): Promise<void> {
  try {
    const db = await openCryptoDB();
    const tx = db.transaction('keypairs', 'readwrite');
    const store = tx.objectStore('keypairs');
    
    store.put({
      deviceId: keyPair.deviceId,
      x25519: keyPair.x25519,
      ed25519: keyPair.ed25519,
      createdAt: keyPair.createdAt,
    });

    return new Promise((resolve, reject) => {
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
    });
  } catch (error) {
    throw new CryptoError('Failed to save keypair', error);
  }
}

/**
 * Load keypair from IndexedDB
 */
export async function loadKeyPair(deviceId?: string): Promise<E2EEKeyPair | null> {
  try {
    const db = await openCryptoDB();
    const tx = db.transaction('keypairs', 'readonly');
    const store = tx.objectStore('keypairs');

    let request: IDBRequest;
    if (deviceId) {
      request = store.get(deviceId);
    } else {
      // Get first available keypair
      request = store.openCursor();
    }

    const result = await new Promise<any>((resolve, reject) => {
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });

    if (!result) return null;

    const data = result.value || result;
    return {
      x25519: data.x25519,
      ed25519: data.ed25519,
      deviceId: data.deviceId,
      createdAt: data.createdAt,
    };
  } catch (error) {
    throw new CryptoError('Failed to load keypair', error);
  }
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Convert ArrayBuffer to Base64 string
 */
export function arrayBufferToBase64(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (let i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}

/**
 * Convert Base64 string to ArrayBuffer
 */
export function base64ToArrayBuffer(base64: string): ArrayBuffer {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes.buffer;
}

// ============================================================================
// IndexedDB Setup
// ============================================================================

const DB_NAME = 'secure-messenger-crypto';
const DB_VERSION = 1;
const STORE_NAME = 'keypairs';

function openCryptoDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);

    request.onerror = () => reject(request.error);
    request.onsuccess = () => resolve(request.result);

    request.onupgradeneeded = (event) => {
      const db = (event.target as IDBOpenDBRequest).result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME, { keyPath: 'deviceId' });
      }
    };
  });
}
