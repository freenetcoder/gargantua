import { PublicKey, Commitment } from '@solana/web3.js';

// Program ID (replace with actual deployed program ID)
export const PROGRAM_ID = new PublicKey('GARGxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx');

// Default configuration
export const DEFAULT_COMMITMENT: Commitment = 'confirmed';
export const DEFAULT_EPOCH_LENGTH = 3600; // 1 hour in seconds
export const DEFAULT_FEE = 1000; // 0.001 SOL in lamports
export const MAX_TRANSFER_AMOUNT = 4294967295; // 2^32 - 1

// Cryptographic constants
export const SCALAR_SIZE = 32;
export const POINT_SIZE = 32;
export const SIGNATURE_SIZE = 64;

// Range proof constants
export const DEFAULT_RANGE_BITS = 32;
export const MAX_RANGE_BITS = 64;

// Account sizes (in bytes)
export const GLOBAL_STATE_SIZE = 96;
export const ZEROSOL_ACCOUNT_SIZE = 105;
export const PENDING_ACCOUNT_SIZE = 64;
export const NONCE_STATE_SIZE = 41;

// Network constants
export const DEVNET_RPC = 'https://api.devnet.solana.com';
export const TESTNET_RPC = 'https://api.testnet.solana.com';
export const MAINNET_RPC = 'https://api.mainnet-beta.solana.com';

// Curve constants (Ristretto255)
export const CURVE_ORDER = new Uint8Array([
  0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x14, 0xde, 0xf9, 0xde, 0xa2, 0xf7, 0x9c, 0xd6,
  0x58, 0x12, 0x63, 0x1a, 0x5c, 0xf5, 0xd3, 0xed,
]);

export const GENERATOR_POINT = new Uint8Array([
  0x58, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
  0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
  0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
  0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
]);

export const H_GENERATOR_POINT = new Uint8Array([
  0x2b, 0xda, 0x7d, 0x3a, 0xe6, 0xa5, 0x57, 0xc7,
  0x16, 0x47, 0x7c, 0x10, 0x8b, 0xe0, 0xd0, 0xf9,
  0x4a, 0xbc, 0x6c, 0x4d, 0xc6, 0xb1, 0xbd, 0x93,
  0xca, 0xcc, 0xbc, 0xce, 0xaa, 0xa7, 0x1d, 0x6b,
]);

// Error messages
export const ERROR_MESSAGES = {
  INVALID_PRIVATE_KEY: 'Private key must be 32 bytes',
  INVALID_PUBLIC_KEY: 'Public key must be 32 bytes',
  INVALID_SIGNATURE: 'Invalid signature format',
  ACCOUNT_NOT_INITIALIZED: 'Account not initialized',
  INSUFFICIENT_BALANCE: 'Insufficient balance for transaction',
  INVALID_PROOF: 'Invalid zero-knowledge proof',
  NETWORK_TIMEOUT: 'Network request timed out',
  TRANSACTION_TIMEOUT: 'Transaction confirmation timed out',
} as const;

// Retry configuration
export const RETRY_CONFIG = {
  maxRetries: 3,
  baseDelay: 1000, // 1 second
  maxDelay: 10000, // 10 seconds
  backoffFactor: 2,
} as const;

// Rate limiting
export const RATE_LIMITS = {
  transactionsPerMinute: 100,
  proofsPerMinute: 50,
  queriesPerMinute: 1000,
} as const;