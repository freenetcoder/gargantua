# Gargantua Protocol API Documentation

## Overview

The Gargantua Protocol provides a comprehensive API for building privacy-preserving applications on Solana. This documentation covers all available instructions, account structures, and client SDK methods.

## Table of Contents

- [Program Instructions](#program-instructions)
- [Account Structures](#account-structures)
- [Client SDK](#client-sdk)
- [Error Codes](#error-codes)
- [Examples](#examples)

## Program Instructions

### Initialize

Initialize the Gargantua protocol with global parameters.

**Instruction**: `Initialize`

```rust
pub struct Initialize {
    pub epoch_length: u64,  // Duration of each epoch in seconds
    pub fee: u64,          // Fee per transaction in lamports
}
```

**Accounts**:
- `[signer]` Authority - Program authority
- `[writable]` Global State - Program configuration account
- `[]` Token Mint - SPL token mint
- `[]` System Program

**Example**:
```rust
let instruction = ZerosolInstruction::Initialize {
    epoch_length: 3600, // 1 hour epochs
    fee: 1000,          // 0.001 SOL fee
};
```

### Register

Register a new anonymous account using Schnorr signature authentication.

**Instruction**: `Register`

```rust
pub struct Register {
    pub public_key: [u8; 32],  // Anonymous public key
    pub challenge: [u8; 32],   // Schnorr challenge
    pub response: [u8; 32],    // Schnorr response
}
```

**Accounts**:
- `[signer]` Payer - Transaction fee payer
- `[writable]` Zerosol Account - Main anonymous account
- `[writable]` Pending Account - Temporary commitment storage
- `[]` Global State - Program configuration
- `[]` System Program

**Example**:
```rust
// Generate Schnorr signature
let (private_key, public_key) = generate_keypair();
let message = program_id.to_bytes();
let (challenge, response) = schnorr_sign(&private_key, &message);

let instruction = ZerosolInstruction::Register {
    public_key: public_key.to_bytes(),
    challenge: challenge.to_bytes(),
    response: response.to_bytes(),
};
```

### Fund

Deposit tokens into an anonymous account.

**Instruction**: `Fund`

```rust
pub struct Fund {
    pub amount: u64,  // Amount to deposit
}
```

**Accounts**:
- `[signer]` Funder - Token owner
- `[writable]` Zerosol Account - Target anonymous account
- `[writable]` Pending Account - Commitment updates
- `[writable]` Funder Token Account - Source token account
- `[writable]` Program Token Account - Program's token custody
- `[]` Token Program
- `[]` Global State

**Example**:
```rust
let instruction = ZerosolInstruction::Fund {
    amount: 1_000_000, // 1 token (assuming 6 decimals)
};
```

### Transfer

Perform an anonymous transfer between accounts.

**Instruction**: `Transfer`

```rust
pub struct Transfer {
    pub commitments_c: Vec<[u8; 32]>,    // Input commitments
    pub commitment_d: [u8; 32],          // Output commitment
    pub public_keys: Vec<[u8; 32]>,      // Participant public keys
    pub nonce: [u8; 32],                 // Unique transaction nonce
    pub beneficiary: [u8; 32],           // Fee recipient
    pub proof: ZerosolProof,             // Zero-knowledge proof
}
```

**Accounts**:
- `[signer]` Relayer - Transaction submitter
- `[writable]` Beneficiary Account - Fee recipient account
- `[writable]` Beneficiary Pending - Fee recipient pending
- `[writable]` Nonce Account - Replay protection
- `[]` Global State
- `[]` System Program
- `[writable]` Participant Accounts... - Input/output accounts

**Example**:
```rust
let proof = generate_transfer_proof(
    &input_commitments,
    &output_commitment,
    &amounts,
    &blindings,
)?;

let instruction = ZerosolInstruction::Transfer {
    commitments_c: input_commitments,
    commitment_d: output_commitment,
    public_keys: participant_keys,
    nonce: generate_nonce(),
    beneficiary: relayer_key,
    proof,
};
```

### Burn

Withdraw tokens from an anonymous account.

**Instruction**: `Burn`

```rust
pub struct Burn {
    pub amount: u64,        // Amount to withdraw
    pub nonce: [u8; 32],    // Unique nonce
    pub proof: BurnProof,   // Ownership proof
}
```

**Accounts**:
- `[signer]` Withdrawer - Token recipient
- `[writable]` Zerosol Account - Source anonymous account
- `[writable]` Pending Account - Commitment updates
- `[writable]` Withdrawer Token Account - Destination
- `[writable]` Program Token Account - Program custody
- `[writable]` Nonce Account - Replay protection
- `[]` Token Program
- `[]` Global State
- `[]` System Program

### RollOver

Update account commitments for the new epoch.

**Instruction**: `RollOver`

**Accounts**:
- `[signer]` Anyone - Can be called by anyone
- `[writable]` Zerosol Account - Account to update
- `[writable]` Pending Account - Pending commitments
- `[writable]` Global State

## Account Structures

### GlobalState

Program-wide configuration and state.

```rust
pub struct GlobalState {
    pub authority: Pubkey,           // Program authority
    pub token_mint: Pubkey,          // Supported token mint
    pub epoch_length: u64,           // Epoch duration in seconds
    pub fee: u64,                    // Transaction fee
    pub last_global_update: u64,     // Last update timestamp
    pub current_epoch: u64,          // Current epoch number
}
```

**Size**: 96 bytes

### ZerosolAccount

Main anonymous account storing commitments.

```rust
pub struct ZerosolAccount {
    pub commitment_left: [u8; 32],   // Left commitment (CLn)
    pub commitment_right: [u8; 32],  // Right commitment (CRn)
    pub public_key: [u8; 32],        // Anonymous public key
    pub last_rollover: u64,          // Last epoch update
    pub is_registered: bool,         // Registration status
}
```

**Size**: 105 bytes

### PendingAccount

Temporary storage for commitment updates within an epoch.

```rust
pub struct PendingAccount {
    pub commitment_left: [u8; 32],   // Pending left commitment
    pub commitment_right: [u8; 32],  // Pending right commitment
}
```

**Size**: 64 bytes

### NonceState

Prevents transaction replay attacks.

```rust
pub struct NonceState {
    pub nonce: [u8; 32],  // Unique nonce value
    pub epoch: u64,       // Epoch when used
    pub used: bool,       // Usage flag
}
```

**Size**: 41 bytes

## Client SDK

### GargantuaClient

Main client for interacting with the protocol.

```typescript
class GargantuaClient {
    constructor(connection: Connection, programId: PublicKey);
    
    // Account management
    async registerAccount(payer: Keypair): Promise<AnonymousAccount>;
    async getAccount(publicKey: PublicKey): Promise<ZerosolAccount>;
    
    // Transactions
    async deposit(account: AnonymousAccount, amount: number): Promise<string>;
    async transfer(from: AnonymousAccount, to: PublicKey, amount: number): Promise<string>;
    async withdraw(account: AnonymousAccount, amount: number): Promise<string>;
    
    // Utilities
    async getCurrentEpoch(): Promise<number>;
    async estimateFee(instruction: TransactionInstruction): Promise<number>;
}
```

### AnonymousAccount

Represents an anonymous account with cryptographic keys.

```typescript
class AnonymousAccount {
    constructor(privateKey: Uint8Array, publicKey: Uint8Array);
    
    // Key management
    getPrivateKey(): Uint8Array;
    getPublicKey(): Uint8Array;
    getAddress(): PublicKey;
    
    // Cryptographic operations
    sign(message: Uint8Array): Signature;
    generateCommitment(amount: number, randomness?: Uint8Array): Commitment;
    
    // Proof generation
    async generateTransferProof(inputs: Input[], outputs: Output[]): Promise<ZerosolProof>;
    async generateBurnProof(amount: number): Promise<BurnProof>;
}
```

### ProofGenerator

Utilities for generating zero-knowledge proofs.

```typescript
class ProofGenerator {
    // Range proofs
    static generateRangeProof(commitment: Commitment, value: number, randomness: Uint8Array): RangeProof;
    
    // Transfer proofs
    static generateTransferProof(
        inputs: CommitmentInput[],
        outputs: CommitmentOutput[],
        fee: number
    ): Promise<ZerosolProof>;
    
    // Burn proofs
    static generateBurnProof(
        account: ZerosolAccount,
        amount: number,
        privateKey: Uint8Array
    ): Promise<BurnProof>;
}
```

## Error Codes

| Code | Error | Description |
|------|-------|-------------|
| 0 | `InvalidInstruction` | Invalid instruction data |
| 1 | `AccountNotRegistered` | Account not registered |
| 2 | `AccountAlreadyRegistered` | Account already exists |
| 3 | `InvalidRegistrationSignature` | Invalid Schnorr signature |
| 4 | `TransferAmountOutOfRange` | Amount exceeds maximum |
| 5 | `NonceAlreadySeen` | Nonce already used |
| 6 | `TransferProofVerificationFailed` | Invalid transfer proof |
| 7 | `BurnProofVerificationFailed` | Invalid burn proof |
| 8 | `InnerProductProofVerificationFailed` | Invalid inner product proof |
| 9 | `SigmaProtocolChallengeFailed` | Sigma protocol error |
| 10 | `InvalidEpoch` | Epoch mismatch |
| 11 | `InsufficientFunds` | Insufficient balance |
| 12 | `InvalidAccountData` | Corrupted account data |
| 13 | `InvalidProofStructure` | Malformed proof |
| 14 | `RangeProofVerificationFailed` | Invalid range proof |
| 15 | `ConstraintSystemVerificationFailed` | Constraint verification failed |
| 16 | `BalanceConservationFailed` | Balance not conserved |
| 17 | `PolynomialEvaluationFailed` | Polynomial proof failed |
| 18 | `ArithmeticConstraintFailed` | Arithmetic constraint failed |
| 19 | `InvalidCommitment` | Invalid commitment |
| 20 | `EpochTransitionError` | Epoch transition failed |

## Examples

### Complete Transfer Example

```rust
use gargantua_sdk::*;

async fn anonymous_transfer() -> Result<()> {
    let client = GargantuaClient::new(
        Connection::new("https://api.devnet.solana.com"),
        PROGRAM_ID,
    )?;
    
    // Register accounts
    let alice = client.register_account(&payer).await?;
    let bob = client.register_account(&payer).await?;
    
    // Alice deposits 1000 tokens
    let deposit_tx = client.deposit(&alice, 1000).await?;
    println!("Deposit transaction: {}", deposit_tx);
    
    // Wait for confirmation
    client.confirm_transaction(&deposit_tx).await?;
    
    // Alice transfers 500 tokens to Bob anonymously
    let transfer_tx = client.transfer(&alice, &bob.get_address(), 500).await?;
    println!("Anonymous transfer: {}", transfer_tx);
    
    // Bob withdraws 300 tokens
    let withdraw_tx = client.withdraw(&bob, 300).await?;
    println!("Withdrawal transaction: {}", withdraw_tx);
    
    Ok(())
}
```

### Batch Operations

```rust
async fn batch_operations() -> Result<()> {
    let client = GargantuaClient::new(connection, program_id)?;
    
    // Create multiple accounts
    let accounts: Vec<AnonymousAccount> = (0..10)
        .map(|_| client.register_account(&payer))
        .collect::<Result<Vec<_>, _>>()
        .await?;
    
    // Batch deposits
    let deposit_txs = client.batch_deposit(&accounts, 1000).await?;
    
    // Wait for all confirmations
    for tx in deposit_txs {
        client.confirm_transaction(&tx).await?;
    }
    
    // Perform ring transfer (multiple inputs, multiple outputs)
    let ring_transfer = client.ring_transfer(
        &accounts[0..5],  // Input accounts
        &accounts[5..10], // Output accounts
        &[200, 200, 200, 200, 200], // Amounts
    ).await?;
    
    println!("Ring transfer completed: {}", ring_transfer);
    
    Ok(())
}
```

### Advanced Proof Generation

```rust
use gargantua_crypto::*;

fn generate_custom_proof() -> Result<ZerosolProof> {
    // Create commitments
    let value1 = 1000u64;
    let value2 = 500u64;
    let randomness1 = generate_randomness();
    let randomness2 = generate_randomness();
    
    let commitment1 = pedersen_commit(value1, randomness1);
    let commitment2 = pedersen_commit(value2, randomness2);
    
    // Generate range proofs
    let range_proof1 = generate_range_proof(commitment1, value1, randomness1, 32)?;
    let range_proof2 = generate_range_proof(commitment2, value2, randomness2, 32)?;
    
    // Create transfer proof
    let transfer_proof = TransferProofBuilder::new()
        .add_input(commitment1, value1, randomness1)
        .add_output(commitment2, value2, randomness2)
        .set_fee(1)
        .build()?;
    
    Ok(transfer_proof)
}
```

## Rate Limits and Quotas

| Operation | Limit | Window |
|-----------|-------|--------|
| Account Registration | 10 per minute | Per IP |
| Deposits | 100 per minute | Per account |
| Transfers | 1000 per minute | Per relayer |
| Withdrawals | 50 per minute | Per account |

## Best Practices

1. **Always verify proofs client-side** before submitting transactions
2. **Use fresh randomness** for each commitment
3. **Implement proper nonce management** to prevent replays
4. **Cache precomputed values** for better performance
5. **Monitor epoch transitions** for account rollovers
6. **Implement retry logic** for failed transactions
7. **Use batch operations** when possible for efficiency