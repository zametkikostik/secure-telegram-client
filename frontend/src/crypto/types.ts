// Crypto Error Types

export class CryptoError extends Error {
  public originalError?: unknown;

  constructor(message: string, originalError?: unknown) {
    super(message);
    this.name = 'CryptoError';
    this.originalError = originalError;
  }
}
