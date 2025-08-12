import { PublicKey } from '@solana/web3.js';

// Core types
export interface GlobalState {
  authority: PublicKey;
  tokenMint: PublicKey;
  epochLength: bigint;
  fee: bigint;
  lastGlobalUpdate: bigint;
  currentEpoch: bigint;
}

export interface ZerosolAccount {
  commitmentLeft: number[];
  commitmentRight: number[];
  publicKey: number[];
  lastRollover: bigint;
  isRegistered: boolean;
}

export interface PendingAccount {
  commitmentLeft: number[];
  commitmentRight: number[];
}

export interface NonceState {
  nonce: number[];
  epoch: bigint;
  used: boolean;
}

// Cryptographic types
export interface SchnorrSignature {
  challenge: Uint8Array;
  response: Uint8Array;
}

export interface Commitment {
  commitment: Uint8Array;
  value: number;
  randomness: Uint8Array;
}

export interface RangeProof {
  a: Uint8Array;
  s: Uint8Array;
  t1: Uint8Array;
  t2: Uint8Array;
  t_hat: Uint8Array;
  tau_x: Uint8Array;
  mu: Uint8Array;
}

export interface InnerProductProof {
  l_vec: number[][];
  r_vec: number[][];
  a: Uint8Array;
  b: Uint8Array;
}

// Proof types
export interface TransferProof {
  commitments_c: number[][];
  commitment_d: number[];
  public_keys: number[][];
  nonce: number[];
  beneficiary: number[];
  ba: number[];
  bs: number[];
  a: number[];
  b: number[];
  cln_g: number[][];
  crn_g: number[][];
  c_0g: number[][];
  dg: number[][];
  y_0g: number[][];
  gg: number[][];
  c_xg: number[][];
  y_xg: number[][];
  f: number[][];
  z_a: number[];
  t_1: number[];
  t_2: number[];
  t_hat: number[];
  mu: number[];
  c: number[];
  s_sk: number[];
  s_r: number[];
  s_b: number[];
  s_tau: number[];
  ip_proof: {
    l_points: number[][];
    r_points: number[][];
    a: number[];
    b: number[];
  };
}

export interface BurnProof {
  ba: number[];
  bs: number[];
  t_1: number[];
  t_2: number[];
  t_hat: number[];
  mu: number[];
  c: number[];
  s_sk: number[];
  s_b: number[];
  s_tau: number[];
  nonce: number[];
  ip_proof: {
    l_points: number[][];
    r_points: number[][];
    a: number[];
    b: number[];
  };
}

// Instruction types
export enum GargantuaInstruction {
  Initialize = 0,
  Register = 1,
  Fund = 2,
  Transfer = 3,
  Burn = 4,
  RollOver = 5,
}

export interface InitializeData {
  epochLength: bigint;
  fee: bigint;
}

export interface RegisterData {
  publicKey: number[];
  challenge: number[];
  response: number[];
}

export interface FundData {
  amount: bigint;
}

export interface TransferData {
  commitments_c: number[][];
  commitment_d: number[];
  public_keys: number[][];
  nonce: number[];
  beneficiary: number[];
  proof: TransferProof;
}

export interface BurnData {
  amount: bigint;
  nonce: number[];
  proof: BurnProof;
}

// Event types
export interface TransactionEvent {
  type: 'transaction';
  signature: string;
  status: 'pending' | 'confirmed' | 'failed';
  timestamp: number;
}

export interface ErrorEvent {
  type: 'error';
  error: Error;
  timestamp: number;
}

export type GargantuaEvent = TransactionEvent | ErrorEvent;