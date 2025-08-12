export enum ErrorCode {
  // General errors
  INVALID_ARGUMENT = 'INVALID_ARGUMENT',
  NETWORK_ERROR = 'NETWORK_ERROR',
  TRANSACTION_FAILED = 'TRANSACTION_FAILED',
  
  // Account errors
  ACCOUNT_NOT_REGISTERED = 'ACCOUNT_NOT_REGISTERED',
  ACCOUNT_ALREADY_REGISTERED = 'ACCOUNT_ALREADY_REGISTERED',
  ACCOUNT_NOT_FOUND = 'ACCOUNT_NOT_FOUND',
  INVALID_ACCOUNT_DATA = 'INVALID_ACCOUNT_DATA',
  
  // Proof errors
  INVALID_PROOF_STRUCTURE = 'INVALID_PROOF_STRUCTURE',
  PROOF_VERIFICATION_FAILED = 'PROOF_VERIFICATION_FAILED',
  RANGE_PROOF_FAILED = 'RANGE_PROOF_FAILED',
  
  // Transfer errors
  INSUFFICIENT_FUNDS = 'INSUFFICIENT_FUNDS',
  INVALID_TRANSFER_AMOUNT = 'INVALID_TRANSFER_AMOUNT',
  BALANCE_MISMATCH = 'BALANCE_MISMATCH',
  
  // Signature errors
  INVALID_SIGNATURE = 'INVALID_SIGNATURE',
  SIGNATURE_VERIFICATION_FAILED = 'SIGNATURE_VERIFICATION_FAILED',
  
  // Epoch errors
  INVALID_EPOCH = 'INVALID_EPOCH',
  EPOCH_TRANSITION_ERROR = 'EPOCH_TRANSITION_ERROR',
  
  // Nonce errors
  NONCE_ALREADY_USED = 'NONCE_ALREADY_USED',
  INVALID_NONCE = 'INVALID_NONCE',
}

export class GargantuaError extends Error {
  public readonly code: ErrorCode;
  public readonly details?: any;

  constructor(message: string, code: ErrorCode, details?: any) {
    super(message);
    this.name = 'GargantuaError';
    this.code = code;
    this.details = details;

    // Maintains proper stack trace for where our error was thrown (only available on V8)
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, GargantuaError);
    }
  }

  /**
   * Create error from Solana program error
   */
  static fromProgramError(error: any): GargantuaError {
    if (error.code !== undefined) {
      const code = mapProgramErrorToCode(error.code);
      return new GargantuaError(error.message || 'Program error', code, error);
    }

    return new GargantuaError(
      error.message || 'Unknown program error',
      ErrorCode.TRANSACTION_FAILED,
      error
    );
  }

  /**
   * Check if error is a specific type
   */
  is(code: ErrorCode): boolean {
    return this.code === code;
  }

  /**
   * Convert to JSON for serialization
   */
  toJSON(): object {
    return {
      name: this.name,
      message: this.message,
      code: this.code,
      details: this.details,
      stack: this.stack,
    };
  }
}

/**
 * Map Solana program error codes to Gargantua error codes
 */
function mapProgramErrorToCode(programErrorCode: number): ErrorCode {
  switch (programErrorCode) {
    case 0:
      return ErrorCode.INVALID_ARGUMENT;
    case 1:
      return ErrorCode.ACCOUNT_NOT_REGISTERED;
    case 2:
      return ErrorCode.ACCOUNT_ALREADY_REGISTERED;
    case 3:
      return ErrorCode.SIGNATURE_VERIFICATION_FAILED;
    case 4:
      return ErrorCode.INVALID_TRANSFER_AMOUNT;
    case 5:
      return ErrorCode.NONCE_ALREADY_USED;
    case 6:
      return ErrorCode.PROOF_VERIFICATION_FAILED;
    case 7:
      return ErrorCode.PROOF_VERIFICATION_FAILED;
    case 8:
      return ErrorCode.PROOF_VERIFICATION_FAILED;
    case 9:
      return ErrorCode.PROOF_VERIFICATION_FAILED;
    case 10:
      return ErrorCode.INVALID_EPOCH;
    case 11:
      return ErrorCode.INSUFFICIENT_FUNDS;
    case 12:
      return ErrorCode.INVALID_ACCOUNT_DATA;
    case 13:
      return ErrorCode.INVALID_PROOF_STRUCTURE;
    case 14:
      return ErrorCode.RANGE_PROOF_FAILED;
    case 15:
      return ErrorCode.PROOF_VERIFICATION_FAILED;
    case 16:
      return ErrorCode.BALANCE_MISMATCH;
    case 17:
      return ErrorCode.PROOF_VERIFICATION_FAILED;
    case 18:
      return ErrorCode.PROOF_VERIFICATION_FAILED;
    case 19:
      return ErrorCode.INVALID_PROOF_STRUCTURE;
    case 20:
      return ErrorCode.EPOCH_TRANSITION_ERROR;
    default:
      return ErrorCode.TRANSACTION_FAILED;
  }
}

/**
 * Type guard to check if error is a GargantuaError
 */
export function isGargantuaError(error: any): error is GargantuaError {
  return error instanceof GargantuaError;
}