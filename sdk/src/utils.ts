import { PublicKey } from '@solana/web3.js';
import { sha256 } from 'js-sha256';
import { GargantuaError, ErrorCode } from './errors';

/**
 * Convert a number to a little-endian byte array
 */
export function numberToBytes(value: number, size: number = 8): Uint8Array {
  const bytes = new Uint8Array(size);
  const view = new DataView(bytes.buffer);
  
  if (size === 8) {
    view.setBigUint64(0, BigInt(value), true);
  } else if (size === 4) {
    view.setUint32(0, value, true);
  } else if (size === 2) {
    view.setUint16(0, value, true);
  } else if (size === 1) {
    view.setUint8(0, value);
  } else {
    throw new GargantuaError(`Unsupported byte size: ${size}`, ErrorCode.INVALID_ARGUMENT);
  }
  
  return bytes;
}

/**
 * Convert a little-endian byte array to a number
 */
export function bytesToNumber(bytes: Uint8Array): number {
  const view = new DataView(bytes.buffer);
  
  if (bytes.length === 8) {
    return Number(view.getBigUint64(0, true));
  } else if (bytes.length === 4) {
    return view.getUint32(0, true);
  } else if (bytes.length === 2) {
    return view.getUint16(0, true);
  } else if (bytes.length === 1) {
    return view.getUint8(0);
  } else {
    throw new GargantuaError(`Unsupported byte array length: ${bytes.length}`, ErrorCode.INVALID_ARGUMENT);
  }
}

/**
 * Generate cryptographically secure random bytes
 */
export function generateRandomBytes(size: number): Uint8Array {
  const bytes = new Uint8Array(size);
  crypto.getRandomValues(bytes);
  return bytes;
}

/**
 * Hash data using SHA-256
 */
export function hashData(data: Uint8Array): Uint8Array {
  const hash = sha256.create();
  hash.update(data);
  return new Uint8Array(hash.arrayBuffer());
}

/**
 * Combine multiple byte arrays
 */
export function combineBytes(...arrays: Uint8Array[]): Uint8Array {
  const totalLength = arrays.reduce((sum, arr) => sum + arr.length, 0);
  const result = new Uint8Array(totalLength);
  
  let offset = 0;
  for (const array of arrays) {
    result.set(array, offset);
    offset += array.length;
  }
  
  return result;
}

/**
 * Convert hex string to bytes
 */
export function hexToBytes(hex: string): Uint8Array {
  if (hex.length % 2 !== 0) {
    throw new GargantuaError('Hex string must have even length', ErrorCode.INVALID_ARGUMENT);
  }
  
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substr(i, 2), 16);
  }
  
  return bytes;
}

/**
 * Convert bytes to hex string
 */
export function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map(byte => byte.toString(16).padStart(2, '0'))
    .join('');
}

/**
 * Validate public key format
 */
export function validatePublicKey(publicKey: any): PublicKey {
  if (publicKey instanceof PublicKey) {
    return publicKey;
  }
  
  if (typeof publicKey === 'string') {
    try {
      return new PublicKey(publicKey);
    } catch (error) {
      throw new GargantuaError('Invalid public key string', ErrorCode.INVALID_ARGUMENT);
    }
  }
  
  if (publicKey instanceof Uint8Array && publicKey.length === 32) {
    try {
      return new PublicKey(publicKey);
    } catch (error) {
      throw new GargantuaError('Invalid public key bytes', ErrorCode.INVALID_ARGUMENT);
    }
  }
  
  throw new GargantuaError('Invalid public key format', ErrorCode.INVALID_ARGUMENT);
}

/**
 * Sleep for a specified number of milliseconds
 */
export function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

/**
 * Retry a function with exponential backoff
 */
export async function retry<T>(
  fn: () => Promise<T>,
  maxRetries: number = 3,
  baseDelay: number = 1000
): Promise<T> {
  let lastError: Error;
  
  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error as Error;
      
      if (attempt === maxRetries) {
        break;
      }
      
      const delay = baseDelay * Math.pow(2, attempt);
      await sleep(delay);
    }
  }
  
  throw lastError!;
}

/**
 * Format amount for display
 */
export function formatAmount(amount: number, decimals: number = 6): string {
  const divisor = Math.pow(10, decimals);
  const formatted = (amount / divisor).toFixed(decimals);
  
  // Remove trailing zeros
  return formatted.replace(/\.?0+$/, '');
}

/**
 * Parse amount from string
 */
export function parseAmount(amount: string, decimals: number = 6): number {
  const multiplier = Math.pow(10, decimals);
  const parsed = parseFloat(amount);
  
  if (isNaN(parsed) || parsed < 0) {
    throw new GargantuaError('Invalid amount format', ErrorCode.INVALID_ARGUMENT);
  }
  
  return Math.floor(parsed * multiplier);
}

/**
 * Validate amount is within valid range
 */
export function validateAmount(amount: number, maxAmount: number = 4294967295): void {
  if (!Number.isInteger(amount) || amount < 0 || amount > maxAmount) {
    throw new GargantuaError(
      `Amount must be a non-negative integer <= ${maxAmount}`,
      ErrorCode.INVALID_TRANSFER_AMOUNT
    );
  }
}

/**
 * Generate a unique nonce
 */
export function generateNonce(): Uint8Array {
  const timestamp = Date.now();
  const random = generateRandomBytes(24);
  const timestampBytes = numberToBytes(timestamp, 8);
  
  return combineBytes(timestampBytes, random);
}

/**
 * Check if two byte arrays are equal
 */
export function bytesEqual(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) {
    return false;
  }
  
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) {
      return false;
    }
  }
  
  return true;
}

/**
 * Derive program address
 */
export async function deriveProgramAddress(
  seeds: (Uint8Array | string)[],
  programId: PublicKey
): Promise<PublicKey> {
  const seedBytes = seeds.map(seed => 
    typeof seed === 'string' ? new TextEncoder().encode(seed) : seed
  );
  
  const [address] = await PublicKey.findProgramAddress(seedBytes, programId);
  return address;
}

/**
 * Validate signature format
 */
export function validateSignature(signature: any): void {
  if (!signature || typeof signature !== 'object') {
    throw new GargantuaError('Invalid signature format', ErrorCode.INVALID_SIGNATURE);
  }
  
  if (!signature.challenge || !signature.response) {
    throw new GargantuaError('Signature missing challenge or response', ErrorCode.INVALID_SIGNATURE);
  }
  
  if (!(signature.challenge instanceof Uint8Array) || signature.challenge.length !== 32) {
    throw new GargantuaError('Invalid signature challenge', ErrorCode.INVALID_SIGNATURE);
  }
  
  if (!(signature.response instanceof Uint8Array) || signature.response.length !== 32) {
    throw new GargantuaError('Invalid signature response', ErrorCode.INVALID_SIGNATURE);
  }
}

/**
 * Create a timeout promise
 */
export function withTimeout<T>(promise: Promise<T>, timeoutMs: number): Promise<T> {
  const timeoutPromise = new Promise<never>((_, reject) => {
    setTimeout(() => {
      reject(new GargantuaError('Operation timed out', ErrorCode.NETWORK_ERROR));
    }, timeoutMs);
  });
  
  return Promise.race([promise, timeoutPromise]);
}