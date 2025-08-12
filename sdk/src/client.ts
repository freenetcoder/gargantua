import {
  Connection,
  PublicKey,
  Keypair,
  Transaction,
  TransactionInstruction,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
  sendAndConfirmTransaction,
  Commitment,
} from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { serialize } from 'borsh';

import { AnonymousAccount } from './account';
import { ProofGenerator } from './proof';
import {
  GargantuaInstruction,
  GlobalState,
  ZerosolAccount,
  PendingAccount,
  NonceState,
  TransferProof,
  BurnProof,
} from './types';
import { GargantuaError, ErrorCode } from './errors';
import { PROGRAM_ID, DEFAULT_COMMITMENT } from './constants';
import {
  createInitializeInstruction,
  createRegisterInstruction,
  createFundInstruction,
  createTransferInstruction,
  createBurnInstruction,
  createRolloverInstruction,
} from './instructions';

export interface GargantuaClientConfig {
  programId?: PublicKey;
  commitment?: Commitment;
  confirmTransactionInitialTimeout?: number;
  maxRetries?: number;
}

export class GargantuaClient {
  private connection: Connection;
  private programId: PublicKey;
  private commitment: Commitment;
  private confirmTransactionInitialTimeout: number;
  private maxRetries: number;

  constructor(
    connection: Connection,
    config: GargantuaClientConfig = {}
  ) {
    this.connection = connection;
    this.programId = config.programId || PROGRAM_ID;
    this.commitment = config.commitment || DEFAULT_COMMITMENT;
    this.confirmTransactionInitialTimeout = config.confirmTransactionInitialTimeout || 60000;
    this.maxRetries = config.maxRetries || 3;
  }

  /**
   * Initialize the Gargantua protocol
   */
  async initialize(
    authority: Keypair,
    tokenMint: PublicKey,
    epochLength: number = 3600,
    fee: number = 1000
  ): Promise<string> {
    const globalStateKeypair = Keypair.generate();
    
    const instruction = createInitializeInstruction(
      this.programId,
      authority.publicKey,
      globalStateKeypair.publicKey,
      tokenMint,
      epochLength,
      fee
    );

    const transaction = new Transaction().add(instruction);
    
    return await this.sendAndConfirmTransaction(
      transaction,
      [authority, globalStateKeypair]
    );
  }

  /**
   * Register a new anonymous account
   */
  async registerAccount(payer: Keypair): Promise<AnonymousAccount> {
    const account = AnonymousAccount.generate();
    const zerosolAccountKeypair = Keypair.generate();
    const pendingAccountKeypair = Keypair.generate();

    // Generate Schnorr signature for registration
    const message = this.programId.toBytes();
    const signature = account.signMessage(message);

    const instruction = createRegisterInstruction(
      this.programId,
      payer.publicKey,
      zerosolAccountKeypair.publicKey,
      pendingAccountKeypair.publicKey,
      await this.getGlobalStateAddress(),
      account.getPublicKeyBytes(),
      signature.challenge,
      signature.response
    );

    const transaction = new Transaction().add(instruction);
    
    const txSignature = await this.sendAndConfirmTransaction(
      transaction,
      [payer, zerosolAccountKeypair, pendingAccountKeypair]
    );

    // Store the account addresses
    account.setAccountAddress(zerosolAccountKeypair.publicKey);
    account.setPendingAddress(pendingAccountKeypair.publicKey);

    return account;
  }

  /**
   * Deposit tokens into an anonymous account
   */
  async deposit(
    account: AnonymousAccount,
    amount: number,
    payer?: Keypair
  ): Promise<string> {
    if (!account.getAccountAddress()) {
      throw new GargantuaError('Account not registered', ErrorCode.ACCOUNT_NOT_REGISTERED);
    }

    const funder = payer || account.getKeypair();
    if (!funder) {
      throw new GargantuaError('No payer provided and account has no keypair', ErrorCode.INVALID_ARGUMENT);
    }

    const funderTokenAccount = await this.getOrCreateTokenAccount(funder.publicKey);
    const programTokenAccount = await this.getProgramTokenAccount();

    const instruction = createFundInstruction(
      this.programId,
      funder.publicKey,
      account.getAccountAddress()!,
      account.getPendingAddress()!,
      funderTokenAccount,
      programTokenAccount,
      await this.getGlobalStateAddress(),
      amount
    );

    const transaction = new Transaction().add(instruction);
    
    return await this.sendAndConfirmTransaction(transaction, [funder]);
  }

  /**
   * Perform an anonymous transfer
   */
  async transfer(
    from: AnonymousAccount,
    to: PublicKey,
    amount: number,
    relayer?: Keypair
  ): Promise<string> {
    if (!from.getAccountAddress()) {
      throw new GargantuaError('Source account not registered', ErrorCode.ACCOUNT_NOT_REGISTERED);
    }

    const relayerKeypair = relayer || from.getKeypair();
    if (!relayerKeypair) {
      throw new GargantuaError('No relayer provided and account has no keypair', ErrorCode.INVALID_ARGUMENT);
    }

    // Generate proof for the transfer
    const proofGenerator = new ProofGenerator();
    const proof = await proofGenerator.generateTransferProof(
      [{ account: from, amount }],
      [{ address: to, amount }],
      1000 // fee
    );

    const nonceKeypair = Keypair.generate();
    const beneficiaryAccount = await this.getAccountAddress(to);
    const beneficiaryPending = await this.getPendingAddress(to);

    const instruction = createTransferInstruction(
      this.programId,
      relayerKeypair.publicKey,
      beneficiaryAccount,
      beneficiaryPending,
      nonceKeypair.publicKey,
      await this.getGlobalStateAddress(),
      proof.commitments_c,
      proof.commitment_d,
      proof.public_keys,
      proof.nonce,
      proof.beneficiary,
      proof
    );

    const transaction = new Transaction().add(instruction);
    
    return await this.sendAndConfirmTransaction(
      transaction,
      [relayerKeypair, nonceKeypair]
    );
  }

  /**
   * Withdraw tokens from an anonymous account
   */
  async withdraw(
    account: AnonymousAccount,
    amount: number,
    destination?: PublicKey
  ): Promise<string> {
    if (!account.getAccountAddress()) {
      throw new GargantuaError('Account not registered', ErrorCode.ACCOUNT_NOT_REGISTERED);
    }

    const withdrawer = account.getKeypair();
    if (!withdrawer) {
      throw new GargantuaError('Account has no keypair for withdrawal', ErrorCode.INVALID_ARGUMENT);
    }

    const destinationTokenAccount = destination 
      ? await this.getOrCreateTokenAccount(destination)
      : await this.getOrCreateTokenAccount(withdrawer.publicKey);

    const programTokenAccount = await this.getProgramTokenAccount();
    const nonceKeypair = Keypair.generate();

    // Generate burn proof
    const proofGenerator = new ProofGenerator();
    const proof = await proofGenerator.generateBurnProof(account, amount);

    const instruction = createBurnInstruction(
      this.programId,
      withdrawer.publicKey,
      account.getAccountAddress()!,
      account.getPendingAddress()!,
      destinationTokenAccount,
      programTokenAccount,
      nonceKeypair.publicKey,
      await this.getGlobalStateAddress(),
      amount,
      proof.nonce,
      proof
    );

    const transaction = new Transaction().add(instruction);
    
    return await this.sendAndConfirmTransaction(
      transaction,
      [withdrawer, nonceKeypair]
    );
  }

  /**
   * Get account information
   */
  async getAccount(address: PublicKey): Promise<ZerosolAccount | null> {
    try {
      const accountInfo = await this.connection.getAccountInfo(address, this.commitment);
      if (!accountInfo) return null;

      // Deserialize account data
      return this.deserializeZerosolAccount(accountInfo.data);
    } catch (error) {
      throw new GargantuaError(
        `Failed to get account: ${error}`,
        ErrorCode.NETWORK_ERROR
      );
    }
  }

  /**
   * Get global state
   */
  async getGlobalState(): Promise<GlobalState | null> {
    try {
      const globalStateAddress = await this.getGlobalStateAddress();
      const accountInfo = await this.connection.getAccountInfo(globalStateAddress, this.commitment);
      if (!accountInfo) return null;

      return this.deserializeGlobalState(accountInfo.data);
    } catch (error) {
      throw new GargantuaError(
        `Failed to get global state: ${error}`,
        ErrorCode.NETWORK_ERROR
      );
    }
  }

  /**
   * Get current epoch
   */
  async getCurrentEpoch(): Promise<number> {
    const globalState = await this.getGlobalState();
    if (!globalState) {
      throw new GargantuaError('Global state not found', ErrorCode.ACCOUNT_NOT_FOUND);
    }

    const now = Math.floor(Date.now() / 1000);
    return Math.floor(now / globalState.epochLength);
  }

  /**
   * Rollover account to new epoch
   */
  async rolloverAccount(account: AnonymousAccount, signer?: Keypair): Promise<string> {
    if (!account.getAccountAddress()) {
      throw new GargantuaError('Account not registered', ErrorCode.ACCOUNT_NOT_REGISTERED);
    }

    const signerKeypair = signer || account.getKeypair();
    if (!signerKeypair) {
      throw new GargantuaError('No signer provided', ErrorCode.INVALID_ARGUMENT);
    }

    const instruction = createRolloverInstruction(
      this.programId,
      signerKeypair.publicKey,
      account.getAccountAddress()!,
      account.getPendingAddress()!,
      await this.getGlobalStateAddress()
    );

    const transaction = new Transaction().add(instruction);
    
    return await this.sendAndConfirmTransaction(transaction, [signerKeypair]);
  }

  /**
   * Estimate transaction fee
   */
  async estimateFee(instruction: TransactionInstruction): Promise<number> {
    try {
      const transaction = new Transaction().add(instruction);
      const feeCalculator = await this.connection.getRecentBlockhash(this.commitment);
      return feeCalculator.feeCalculator.lamportsPerSignature * transaction.signatures.length;
    } catch (error) {
      throw new GargantuaError(
        `Failed to estimate fee: ${error}`,
        ErrorCode.NETWORK_ERROR
      );
    }
  }

  // Private helper methods

  private async sendAndConfirmTransaction(
    transaction: Transaction,
    signers: Keypair[]
  ): Promise<string> {
    try {
      return await sendAndConfirmTransaction(
        this.connection,
        transaction,
        signers,
        {
          commitment: this.commitment,
          confirmTransactionInitialTimeout: this.confirmTransactionInitialTimeout,
        }
      );
    } catch (error) {
      throw new GargantuaError(
        `Transaction failed: ${error}`,
        ErrorCode.TRANSACTION_FAILED
      );
    }
  }

  private async getGlobalStateAddress(): Promise<PublicKey> {
    const [address] = await PublicKey.findProgramAddress(
      [Buffer.from('global_state')],
      this.programId
    );
    return address;
  }

  private async getAccountAddress(publicKey: PublicKey): Promise<PublicKey> {
    const [address] = await PublicKey.findProgramAddress(
      [Buffer.from('zerosol_account'), publicKey.toBuffer()],
      this.programId
    );
    return address;
  }

  private async getPendingAddress(publicKey: PublicKey): Promise<PublicKey> {
    const [address] = await PublicKey.findProgramAddress(
      [Buffer.from('pending_account'), publicKey.toBuffer()],
      this.programId
    );
    return address;
  }

  private async getProgramTokenAccount(): Promise<PublicKey> {
    const [address] = await PublicKey.findProgramAddress(
      [Buffer.from('token_account')],
      this.programId
    );
    return address;
  }

  private async getOrCreateTokenAccount(owner: PublicKey): Promise<PublicKey> {
    // This would implement token account creation logic
    // For now, return a placeholder
    const [address] = await PublicKey.findProgramAddress(
      [Buffer.from('user_token'), owner.toBuffer()],
      this.programId
    );
    return address;
  }

  private deserializeZerosolAccount(data: Buffer): ZerosolAccount {
    // Implement Borsh deserialization for ZerosolAccount
    // This is a placeholder implementation
    return {
      commitmentLeft: Array.from(data.slice(0, 32)),
      commitmentRight: Array.from(data.slice(32, 64)),
      publicKey: Array.from(data.slice(64, 96)),
      lastRollover: data.readBigUInt64LE(96),
      isRegistered: data.readUInt8(104) === 1,
    };
  }

  private deserializeGlobalState(data: Buffer): GlobalState {
    // Implement Borsh deserialization for GlobalState
    // This is a placeholder implementation
    return {
      authority: new PublicKey(data.slice(0, 32)),
      tokenMint: new PublicKey(data.slice(32, 64)),
      epochLength: data.readBigUInt64LE(64),
      fee: data.readBigUInt64LE(72),
      lastGlobalUpdate: data.readBigUInt64LE(80),
      currentEpoch: data.readBigUInt64LE(88),
    };
  }
}