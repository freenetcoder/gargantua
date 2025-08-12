import {
  PublicKey,
  TransactionInstruction,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { serialize } from 'borsh';

import {
  GargantuaInstruction,
  InitializeData,
  RegisterData,
  FundData,
  TransferData,
  BurnData,
  TransferProof,
  BurnProof,
} from './types';

// Borsh schemas
const InitializeSchema = new Map([
  [InitializeData, {
    kind: 'struct',
    fields: [
      ['epochLength', 'u64'],
      ['fee', 'u64'],
    ],
  }],
]);

const RegisterSchema = new Map([
  [RegisterData, {
    kind: 'struct',
    fields: [
      ['publicKey', [32]],
      ['challenge', [32]],
      ['response', [32]],
    ],
  }],
]);

const FundSchema = new Map([
  [FundData, {
    kind: 'struct',
    fields: [
      ['amount', 'u64'],
    ],
  }],
]);

/**
 * Create initialize instruction
 */
export function createInitializeInstruction(
  programId: PublicKey,
  authority: PublicKey,
  globalState: PublicKey,
  tokenMint: PublicKey,
  epochLength: number,
  fee: number
): TransactionInstruction {
  const data = Buffer.concat([
    Buffer.from([GargantuaInstruction.Initialize]),
    serialize(InitializeSchema, new InitializeData({
      epochLength: BigInt(epochLength),
      fee: BigInt(fee),
    })),
  ]);

  return new TransactionInstruction({
    keys: [
      { pubkey: authority, isSigner: true, isWritable: false },
      { pubkey: globalState, isSigner: false, isWritable: true },
      { pubkey: tokenMint, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
}

/**
 * Create register instruction
 */
export function createRegisterInstruction(
  programId: PublicKey,
  payer: PublicKey,
  zerosolAccount: PublicKey,
  pendingAccount: PublicKey,
  globalState: PublicKey,
  publicKey: Uint8Array,
  challenge: Uint8Array,
  response: Uint8Array
): TransactionInstruction {
  const data = Buffer.concat([
    Buffer.from([GargantuaInstruction.Register]),
    serialize(RegisterSchema, new RegisterData({
      publicKey: Array.from(publicKey),
      challenge: Array.from(challenge),
      response: Array.from(response),
    })),
  ]);

  return new TransactionInstruction({
    keys: [
      { pubkey: payer, isSigner: true, isWritable: false },
      { pubkey: zerosolAccount, isSigner: false, isWritable: true },
      { pubkey: pendingAccount, isSigner: false, isWritable: true },
      { pubkey: globalState, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
}

/**
 * Create fund instruction
 */
export function createFundInstruction(
  programId: PublicKey,
  funder: PublicKey,
  zerosolAccount: PublicKey,
  pendingAccount: PublicKey,
  funderTokenAccount: PublicKey,
  programTokenAccount: PublicKey,
  globalState: PublicKey,
  amount: number
): TransactionInstruction {
  const data = Buffer.concat([
    Buffer.from([GargantuaInstruction.Fund]),
    serialize(FundSchema, new FundData({
      amount: BigInt(amount),
    })),
  ]);

  return new TransactionInstruction({
    keys: [
      { pubkey: funder, isSigner: true, isWritable: false },
      { pubkey: zerosolAccount, isSigner: false, isWritable: true },
      { pubkey: pendingAccount, isSigner: false, isWritable: true },
      { pubkey: funderTokenAccount, isSigner: false, isWritable: true },
      { pubkey: programTokenAccount, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: globalState, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
}

/**
 * Create transfer instruction
 */
export function createTransferInstruction(
  programId: PublicKey,
  relayer: PublicKey,
  beneficiaryAccount: PublicKey,
  beneficiaryPending: PublicKey,
  nonceAccount: PublicKey,
  globalState: PublicKey,
  commitments_c: number[][],
  commitment_d: number[],
  public_keys: number[][],
  nonce: number[],
  beneficiary: number[],
  proof: TransferProof
): TransactionInstruction {
  const transferData = new TransferData({
    commitments_c,
    commitment_d,
    public_keys,
    nonce,
    beneficiary,
    proof,
  });

  const data = Buffer.concat([
    Buffer.from([GargantuaInstruction.Transfer]),
    Buffer.from(JSON.stringify(transferData)), // Simplified serialization
  ]);

  return new TransactionInstruction({
    keys: [
      { pubkey: relayer, isSigner: true, isWritable: false },
      { pubkey: beneficiaryAccount, isSigner: false, isWritable: true },
      { pubkey: beneficiaryPending, isSigner: false, isWritable: true },
      { pubkey: nonceAccount, isSigner: false, isWritable: true },
      { pubkey: globalState, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
}

/**
 * Create burn instruction
 */
export function createBurnInstruction(
  programId: PublicKey,
  withdrawer: PublicKey,
  zerosolAccount: PublicKey,
  pendingAccount: PublicKey,
  withdrawerTokenAccount: PublicKey,
  programTokenAccount: PublicKey,
  nonceAccount: PublicKey,
  globalState: PublicKey,
  amount: number,
  nonce: number[],
  proof: BurnProof
): TransactionInstruction {
  const burnData = new BurnData({
    amount: BigInt(amount),
    nonce,
    proof,
  });

  const data = Buffer.concat([
    Buffer.from([GargantuaInstruction.Burn]),
    Buffer.from(JSON.stringify(burnData)), // Simplified serialization
  ]);

  return new TransactionInstruction({
    keys: [
      { pubkey: withdrawer, isSigner: true, isWritable: false },
      { pubkey: zerosolAccount, isSigner: false, isWritable: true },
      { pubkey: pendingAccount, isSigner: false, isWritable: true },
      { pubkey: withdrawerTokenAccount, isSigner: false, isWritable: true },
      { pubkey: programTokenAccount, isSigner: false, isWritable: true },
      { pubkey: nonceAccount, isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: globalState, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
}

/**
 * Create rollover instruction
 */
export function createRolloverInstruction(
  programId: PublicKey,
  signer: PublicKey,
  zerosolAccount: PublicKey,
  pendingAccount: PublicKey,
  globalState: PublicKey
): TransactionInstruction {
  const data = Buffer.from([GargantuaInstruction.RollOver]);

  return new TransactionInstruction({
    keys: [
      { pubkey: signer, isSigner: true, isWritable: false },
      { pubkey: zerosolAccount, isSigner: false, isWritable: true },
      { pubkey: pendingAccount, isSigner: false, isWritable: true },
      { pubkey: globalState, isSigner: false, isWritable: true },
    ],
    programId,
    data,
  });
}