# Gargantua Protocol Integration Guide

## Overview

This guide provides comprehensive instructions for integrating the Gargantua Protocol into your application, whether you're building a wallet, DeFi protocol, or any other application that needs privacy-preserving payments.

## Table of Contents

1. [Quick Start](#quick-start)
2. [SDK Installation](#sdk-installation)
3. [Basic Integration](#basic-integration)
4. [Advanced Features](#advanced-features)
5. [DeFi Integration](#defi-integration)
6. [Wallet Integration](#wallet-integration)
7. [Enterprise Integration](#enterprise-integration)
8. [Testing](#testing)
9. [Production Deployment](#production-deployment)
10. [Troubleshooting](#troubleshooting)

## Quick Start

### Prerequisites

- Node.js 16+ or Rust 1.70+
- Solana CLI tools
- Basic understanding of Solana development
- Familiarity with cryptographic concepts

### 5-Minute Integration

```typescript
import { GargantuaClient, AnonymousAccount } from '@gargantua/sdk';
import { Connection, Keypair } from '@solana/web3.js';

// Initialize client
const connection = new Connection('https://api.devnet.solana.com');
const client = new GargantuaClient(connection);

// Create anonymous account
const payer = Keypair.generate();
const account = await client.registerAccount(payer);

// Deposit tokens
await client.deposit(account, 1000);

// Anonymous transfer
const recipient = await client.registerAccount(Keypair.generate());
await client.transfer(account, recipient.getAddress(), 500);

// Withdraw tokens
await client.withdraw(account, 300);
```

## SDK Installation

### JavaScript/TypeScript

```bash
npm install @gargantua/sdk
# or
yarn add @gargantua/sdk
```

### Rust

```toml
[dependencies]
gargantua-sdk = "0.1.0"
solana-program = "1.17"
solana-client = "1.17"
```

### Python

```bash
pip install gargantua-py
```

## Basic Integration

### 1. Client Initialization

```typescript
import { GargantuaClient } from '@gargantua/sdk';
import { Connection, PublicKey } from '@solana/web3.js';

const client = new GargantuaClient(
  new Connection('https://api.mainnet-beta.solana.com'),
  new PublicKey('GARGxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx') // Program ID
);
```

### 2. Account Management

#### Register Anonymous Account

```typescript
async function registerAccount(payer: Keypair): Promise<AnonymousAccount> {
  try {
    const account = await client.registerAccount(payer);
    console.log('Account registered:', account.getAddress().toString());
    return account;
  } catch (error) {
    console.error('Registration failed:', error);
    throw error;
  }
}
```

#### Load Existing Account

```typescript
async function loadAccount(privateKey: Uint8Array): Promise<AnonymousAccount> {
  const account = AnonymousAccount.fromPrivateKey(privateKey);
  
  // Verify account exists on-chain
  const accountData = await client.getAccount(account.getAddress());
  if (!accountData.isRegistered) {
    throw new Error('Account not registered');
  }
  
  return account;
}
```

### 3. Basic Operations

#### Deposit Tokens

```typescript
async function deposit(account: AnonymousAccount, amount: number): Promise<string> {
  try {
    const signature = await client.deposit(account, amount);
    await client.confirmTransaction(signature);
    console.log('Deposit successful:', signature);
    return signature;
  } catch (error) {
    console.error('Deposit failed:', error);
    throw error;
  }
}
```

#### Anonymous Transfer

```typescript
async function transfer(
  from: AnonymousAccount,
  to: PublicKey,
  amount: number
): Promise<string> {
  try {
    const signature = await client.transfer(from, to, amount);
    await client.confirmTransaction(signature);
    console.log('Transfer successful:', signature);
    return signature;
  } catch (error) {
    console.error('Transfer failed:', error);
    throw error;
  }
}
```

#### Withdraw Tokens

```typescript
async function withdraw(
  account: AnonymousAccount,
  amount: number,
  destination: PublicKey
): Promise<string> {
  try {
    const signature = await client.withdraw(account, amount, destination);
    await client.confirmTransaction(signature);
    console.log('Withdrawal successful:', signature);
    return signature;
  } catch (error) {
    console.error('Withdrawal failed:', error);
    throw error;
  }
}
```

## Advanced Features

### 1. Multi-Party Transfers

```typescript
async function multiPartyTransfer(
  inputs: Array<{ account: AnonymousAccount; amount: number }>,
  outputs: Array<{ address: PublicKey; amount: number }>,
  fee: number
): Promise<string> {
  const transferBuilder = client.createTransferBuilder();
  
  // Add inputs
  for (const input of inputs) {
    transferBuilder.addInput(input.account, input.amount);
  }
  
  // Add outputs
  for (const output of outputs) {
    transferBuilder.addOutput(output.address, output.amount);
  }
  
  // Set fee
  transferBuilder.setFee(fee);
  
  // Build and submit transaction
  const transaction = await transferBuilder.build();
  const signature = await client.submitTransaction(transaction);
  
  return signature;
}
```

### 2. Batch Operations

```typescript
async function batchDeposit(
  accounts: AnonymousAccount[],
  amounts: number[]
): Promise<string[]> {
  const batchBuilder = client.createBatchBuilder();
  
  for (let i = 0; i < accounts.length; i++) {
    batchBuilder.addDeposit(accounts[i], amounts[i]);
  }
  
  const transactions = await batchBuilder.build();
  const signatures = await client.submitBatch(transactions);
  
  return signatures;
}
```

### 3. Custom Proof Generation

```typescript
import { ProofGenerator } from '@gargantua/sdk';

async function generateCustomProof(
  inputs: CommitmentInput[],
  outputs: CommitmentOutput[]
): Promise<ZerosolProof> {
  const proofGenerator = new ProofGenerator();
  
  // Add range proofs for all amounts
  for (const input of inputs) {
    proofGenerator.addRangeProof(input.commitment, input.amount, input.randomness);
  }
  
  for (const output of outputs) {
    proofGenerator.addRangeProof(output.commitment, output.amount, output.randomness);
  }
  
  // Generate balance conservation proof
  proofGenerator.addBalanceConstraint(inputs, outputs);
  
  // Generate ownership proofs
  for (const input of inputs) {
    proofGenerator.addOwnershipProof(input.account, input.privateKey);
  }
  
  return await proofGenerator.build();
}
```

## DeFi Integration

### 1. DEX Integration

```typescript
class PrivateDEX {
  private client: GargantuaClient;
  
  constructor(client: GargantuaClient) {
    this.client = client;
  }
  
  async privateSwap(
    account: AnonymousAccount,
    inputToken: PublicKey,
    outputToken: PublicKey,
    inputAmount: number,
    minOutputAmount: number
  ): Promise<string> {
    // Create temporary anonymous account for swap
    const swapAccount = await this.client.registerAccount(Keypair.generate());
    
    // Deposit input tokens
    await this.client.deposit(account, inputAmount);
    
    // Perform swap through DEX
    const swapInstruction = await this.createSwapInstruction(
      swapAccount,
      inputToken,
      outputToken,
      inputAmount,
      minOutputAmount
    );
    
    // Execute swap
    const signature = await this.client.submitInstruction(swapInstruction);
    
    // Transfer output back to user
    await this.client.transfer(swapAccount, account.getAddress(), minOutputAmount);
    
    return signature;
  }
  
  private async createSwapInstruction(
    account: AnonymousAccount,
    inputToken: PublicKey,
    outputToken: PublicKey,
    inputAmount: number,
    minOutputAmount: number
  ): Promise<TransactionInstruction> {
    // Implementation depends on specific DEX
    // This is a placeholder for the actual swap logic
    throw new Error('Implement DEX-specific swap logic');
  }
}
```

### 2. Lending Protocol Integration

```typescript
class PrivateLending {
  private client: GargantuaClient;
  
  constructor(client: GargantuaClient) {
    this.client = client;
  }
  
  async privateLend(
    account: AnonymousAccount,
    amount: number,
    duration: number
  ): Promise<string> {
    // Generate lending proof
    const lendingProof = await this.generateLendingProof(account, amount, duration);
    
    // Create lending transaction
    const transaction = await this.createLendingTransaction(
      account,
      amount,
      duration,
      lendingProof
    );
    
    return await this.client.submitTransaction(transaction);
  }
  
  async privateBorrow(
    account: AnonymousAccount,
    collateralAmount: number,
    borrowAmount: number
  ): Promise<string> {
    // Generate borrowing proof
    const borrowingProof = await this.generateBorrowingProof(
      account,
      collateralAmount,
      borrowAmount
    );
    
    // Create borrowing transaction
    const transaction = await this.createBorrowingTransaction(
      account,
      collateralAmount,
      borrowAmount,
      borrowingProof
    );
    
    return await this.client.submitTransaction(transaction);
  }
}
```

## Wallet Integration

### 1. Wallet Adapter

```typescript
import { WalletAdapter } from '@solana/wallet-adapter-base';

export class GargantuaWalletAdapter extends WalletAdapter {
  private client: GargantuaClient;
  private accounts: Map<string, AnonymousAccount> = new Map();
  
  constructor() {
    super();
    this.client = new GargantuaClient(this.connection);
  }
  
  async connect(): Promise<void> {
    // Load existing anonymous accounts
    await this.loadAccounts();
    this.emit('connect', this.publicKey);
  }
  
  async createAnonymousAccount(): Promise<AnonymousAccount> {
    const payer = Keypair.generate();
    const account = await this.client.registerAccount(payer);
    
    // Store account securely
    await this.storeAccount(account);
    this.accounts.set(account.getAddress().toString(), account);
    
    return account;
  }
  
  async getAnonymousAccounts(): Promise<AnonymousAccount[]> {
    return Array.from(this.accounts.values());
  }
  
  async signTransaction(transaction: Transaction): Promise<Transaction> {
    // Add anonymous account signatures if needed
    return this.addAnonymousSignatures(transaction);
  }
  
  private async loadAccounts(): Promise<void> {
    // Load accounts from secure storage
    const storedAccounts = await this.getStoredAccounts();
    
    for (const accountData of storedAccounts) {
      const account = AnonymousAccount.fromPrivateKey(accountData.privateKey);
      this.accounts.set(account.getAddress().toString(), account);
    }
  }
  
  private async storeAccount(account: AnonymousAccount): Promise<void> {
    // Store account in secure storage (encrypted)
    const encryptedData = await this.encryptAccountData(account);
    await this.saveToSecureStorage(account.getAddress().toString(), encryptedData);
  }
}
```

### 2. UI Components

```typescript
import React, { useState, useEffect } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';

export const AnonymousAccountManager: React.FC = () => {
  const { wallet } = useWallet();
  const [accounts, setAccounts] = useState<AnonymousAccount[]>([]);
  const [selectedAccount, setSelectedAccount] = useState<AnonymousAccount | null>(null);
  
  useEffect(() => {
    loadAccounts();
  }, [wallet]);
  
  const loadAccounts = async () => {
    if (wallet instanceof GargantuaWalletAdapter) {
      const anonymousAccounts = await wallet.getAnonymousAccounts();
      setAccounts(anonymousAccounts);
    }
  };
  
  const createAccount = async () => {
    if (wallet instanceof GargantuaWalletAdapter) {
      const newAccount = await wallet.createAnonymousAccount();
      setAccounts([...accounts, newAccount]);
    }
  };
  
  const deposit = async (amount: number) => {
    if (selectedAccount && wallet instanceof GargantuaWalletAdapter) {
      await wallet.client.deposit(selectedAccount, amount);
      // Refresh account balance
      await loadAccounts();
    }
  };
  
  return (
    <div className="anonymous-account-manager">
      <h3>Anonymous Accounts</h3>
      
      <button onClick={createAccount}>
        Create New Anonymous Account
      </button>
      
      <div className="account-list">
        {accounts.map((account, index) => (
          <div
            key={account.getAddress().toString()}
            className={`account-item ${selectedAccount === account ? 'selected' : ''}`}
            onClick={() => setSelectedAccount(account)}
          >
            <div className="account-address">
              {account.getAddress().toString().slice(0, 8)}...
            </div>
            <div className="account-balance">
              Balance: Hidden
            </div>
          </div>
        ))}
      </div>
      
      {selectedAccount && (
        <div className="account-actions">
          <h4>Account Actions</h4>
          <DepositForm onDeposit={deposit} />
          <TransferForm account={selectedAccount} />
          <WithdrawForm account={selectedAccount} />
        </div>
      )}
    </div>
  );
};
```

## Enterprise Integration

### 1. Payroll System

```typescript
class PrivatePayroll {
  private client: GargantuaClient;
  private companyAccount: AnonymousAccount;
  
  constructor(client: GargantuaClient, companyAccount: AnonymousAccount) {
    this.client = client;
    this.companyAccount = companyAccount;
  }
  
  async processPayroll(
    employees: Array<{ address: PublicKey; salary: number }>,
    payrollDate: Date
  ): Promise<string[]> {
    const signatures: string[] = [];
    
    // Create batch transfer for all employees
    const transferBuilder = this.client.createTransferBuilder();
    
    // Add company account as input
    const totalPayroll = employees.reduce((sum, emp) => sum + emp.salary, 0);
    transferBuilder.addInput(this.companyAccount, totalPayroll);
    
    // Add each employee as output
    for (const employee of employees) {
      transferBuilder.addOutput(employee.address, employee.salary);
    }
    
    // Set fee
    transferBuilder.setFee(1000); // 0.001 SOL
    
    // Build and submit transaction
    const transaction = await transferBuilder.build();
    const signature = await this.client.submitTransaction(transaction);
    
    signatures.push(signature);
    
    // Log payroll for compliance (encrypted)
    await this.logPayroll(employees, payrollDate, signature);
    
    return signatures;
  }
  
  private async logPayroll(
    employees: Array<{ address: PublicKey; salary: number }>,
    payrollDate: Date,
    signature: string
  ): Promise<void> {
    const payrollRecord = {
      date: payrollDate,
      employeeCount: employees.length,
      totalAmount: employees.reduce((sum, emp) => sum + emp.salary, 0),
      transactionSignature: signature,
      timestamp: new Date(),
    };
    
    // Store encrypted payroll record
    await this.storeEncryptedRecord(payrollRecord);
  }
}
```

### 2. Supply Chain Payments

```typescript
class PrivateSupplyChain {
  private client: GargantuaClient;
  
  constructor(client: GargantuaClient) {
    this.client = client;
  }
  
  async processSupplierPayment(
    buyerAccount: AnonymousAccount,
    supplierAddress: PublicKey,
    amount: number,
    invoiceId: string
  ): Promise<string> {
    // Generate payment proof with invoice reference
    const paymentProof = await this.generatePaymentProof(
      buyerAccount,
      amount,
      invoiceId
    );
    
    // Create payment transaction
    const signature = await this.client.transfer(
      buyerAccount,
      supplierAddress,
      amount
    );
    
    // Record payment for audit trail
    await this.recordPayment(invoiceId, signature, amount);
    
    return signature;
  }
  
  private async generatePaymentProof(
    account: AnonymousAccount,
    amount: number,
    invoiceId: string
  ): Promise<any> {
    // Generate proof that includes invoice reference
    // This allows for selective disclosure to auditors
    return {
      invoiceId,
      amount,
      timestamp: Date.now(),
      accountProof: await account.generateOwnershipProof(),
    };
  }
}
```

## Testing

### 1. Unit Tests

```typescript
import { describe, it, expect, beforeEach } from '@jest/globals';
import { GargantuaClient, AnonymousAccount } from '@gargantua/sdk';

describe('Gargantua Integration', () => {
  let client: GargantuaClient;
  let account: AnonymousAccount;
  
  beforeEach(async () => {
    client = new GargantuaClient(testConnection);
    account = await client.registerAccount(testKeypair);
  });
  
  it('should deposit tokens successfully', async () => {
    const signature = await client.deposit(account, 1000);
    expect(signature).toBeDefined();
    
    // Verify deposit was processed
    const accountData = await client.getAccount(account.getAddress());
    expect(accountData.isRegistered).toBe(true);
  });
  
  it('should perform anonymous transfer', async () => {
    // Setup
    await client.deposit(account, 1000);
    const recipient = await client.registerAccount(Keypair.generate());
    
    // Transfer
    const signature = await client.transfer(account, recipient.getAddress(), 500);
    expect(signature).toBeDefined();
  });
  
  it('should withdraw tokens successfully', async () => {
    // Setup
    await client.deposit(account, 1000);
    
    // Withdraw
    const signature = await client.withdraw(account, 300);
    expect(signature).toBeDefined();
  });
});
```

### 2. Integration Tests

```typescript
describe('DeFi Integration', () => {
  let privateDEX: PrivateDEX;
  let account: AnonymousAccount;
  
  beforeEach(async () => {
    privateDEX = new PrivateDEX(client);
    account = await client.registerAccount(testKeypair);
    await client.deposit(account, 10000);
  });
  
  it('should perform private swap', async () => {
    const signature = await privateDEX.privateSwap(
      account,
      USDC_MINT,
      SOL_MINT,
      1000,
      900 // min output
    );
    
    expect(signature).toBeDefined();
  });
});
```

### 3. End-to-End Tests

```typescript
describe('Complete User Journey', () => {
  it('should complete full anonymous payment flow', async () => {
    // 1. Register accounts
    const alice = await client.registerAccount(Keypair.generate());
    const bob = await client.registerAccount(Keypair.generate());
    
    // 2. Alice deposits
    await client.deposit(alice, 5000);
    
    // 3. Alice transfers to Bob
    await client.transfer(alice, bob.getAddress(), 2000);
    
    // 4. Bob withdraws
    const signature = await client.withdraw(bob, 1500);
    
    expect(signature).toBeDefined();
  });
});
```

## Production Deployment

### 1. Environment Configuration

```typescript
// config/production.ts
export const productionConfig = {
  solana: {
    cluster: 'mainnet-beta',
    endpoint: 'https://api.mainnet-beta.solana.com',
    commitment: 'confirmed' as Commitment,
  },
  gargantua: {
    programId: new PublicKey('GARGxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx'),
    maxRetries: 3,
    confirmationTimeout: 30000,
  },
  security: {
    enableRateLimiting: true,
    maxTransactionsPerMinute: 100,
    requireTLS: true,
  },
};
```

### 2. Error Handling

```typescript
class GargantuaError extends Error {
  constructor(
    message: string,
    public code: string,
    public details?: any
  ) {
    super(message);
    this.name = 'GargantuaError';
  }
}

export class RobustGargantuaClient {
  private client: GargantuaClient;
  private retryConfig: RetryConfig;
  
  constructor(client: GargantuaClient, retryConfig: RetryConfig) {
    this.client = client;
    this.retryConfig = retryConfig;
  }
  
  async deposit(account: AnonymousAccount, amount: number): Promise<string> {
    return this.withRetry(async () => {
      try {
        return await this.client.deposit(account, amount);
      } catch (error) {
        throw this.handleError(error, 'DEPOSIT_FAILED');
      }
    });
  }
  
  private async withRetry<T>(operation: () => Promise<T>): Promise<T> {
    let lastError: Error;
    
    for (let attempt = 1; attempt <= this.retryConfig.maxRetries; attempt++) {
      try {
        return await operation();
      } catch (error) {
        lastError = error as Error;
        
        if (attempt === this.retryConfig.maxRetries || !this.isRetryable(error)) {
          throw error;
        }
        
        await this.delay(this.retryConfig.baseDelay * Math.pow(2, attempt - 1));
      }
    }
    
    throw lastError!;
  }
  
  private handleError(error: any, code: string): GargantuaError {
    if (error instanceof GargantuaError) {
      return error;
    }
    
    return new GargantuaError(
      error.message || 'Unknown error',
      code,
      error
    );
  }
}
```

### 3. Monitoring and Logging

```typescript
import { Logger } from 'winston';

export class MonitoredGargantuaClient {
  private client: GargantuaClient;
  private logger: Logger;
  private metrics: MetricsCollector;
  
  constructor(client: GargantuaClient, logger: Logger, metrics: MetricsCollector) {
    this.client = client;
    this.logger = logger;
    this.metrics = metrics;
  }
  
  async deposit(account: AnonymousAccount, amount: number): Promise<string> {
    const startTime = Date.now();
    const operationId = this.generateOperationId();
    
    this.logger.info('Starting deposit operation', {
      operationId,
      account: account.getAddress().toString(),
      amount,
    });
    
    try {
      const signature = await this.client.deposit(account, amount);
      
      const duration = Date.now() - startTime;
      this.metrics.recordSuccess('deposit', duration);
      
      this.logger.info('Deposit operation completed', {
        operationId,
        signature,
        duration,
      });
      
      return signature;
    } catch (error) {
      const duration = Date.now() - startTime;
      this.metrics.recordError('deposit', duration);
      
      this.logger.error('Deposit operation failed', {
        operationId,
        error: error.message,
        duration,
      });
      
      throw error;
    }
  }
}
```

## Troubleshooting

### Common Issues

#### 1. Account Registration Fails

**Problem**: Schnorr signature verification fails
**Solution**:
```typescript
// Ensure proper key generation
const keypair = Keypair.generate();
const privateKey = keypair.secretKey.slice(0, 32); // Use only first 32 bytes
const account = AnonymousAccount.fromPrivateKey(privateKey);
```

#### 2. Proof Generation Timeout

**Problem**: Client-side proof generation takes too long
**Solution**:
```typescript
// Use web workers for proof generation
const worker = new Worker('proof-worker.js');
const proof = await new Promise((resolve, reject) => {
  worker.postMessage({ inputs, outputs });
  worker.onmessage = (e) => resolve(e.data);
  worker.onerror = reject;
});
```

#### 3. Transaction Fails with Insufficient Funds

**Problem**: Account doesn't have enough balance
**Solution**:
```typescript
// Check balance before transaction
const balance = await client.getBalance(account);
if (balance < amount + fee) {
  throw new Error('Insufficient funds');
}
```

#### 4. Network Connectivity Issues

**Problem**: RPC endpoint is unreliable
**Solution**:
```typescript
// Use multiple RPC endpoints with fallback
const endpoints = [
  'https://api.mainnet-beta.solana.com',
  'https://solana-api.projectserum.com',
  'https://rpc.ankr.com/solana',
];

const client = new GargantuaClient(
  new Connection(endpoints[0], {
    commitment: 'confirmed',
    confirmTransactionInitialTimeout: 60000,
  })
);
```

### Debug Mode

```typescript
// Enable debug logging
const client = new GargantuaClient(connection, {
  debug: true,
  logLevel: 'debug',
});

// Monitor transaction status
client.on('transaction', (event) => {
  console.log('Transaction event:', event);
});

client.on('error', (error) => {
  console.error('Client error:', error);
});
```

### Performance Optimization

```typescript
// Batch multiple operations
const batchBuilder = client.createBatchBuilder();
batchBuilder.addDeposit(account1, 1000);
batchBuilder.addDeposit(account2, 2000);
batchBuilder.addTransfer(account1, account2.getAddress(), 500);

const signatures = await client.submitBatch(await batchBuilder.build());

// Use connection pooling
const connectionPool = new ConnectionPool([
  'https://api.mainnet-beta.solana.com',
  'https://solana-api.projectserum.com',
]);

const client = new GargantuaClient(connectionPool.getConnection());
```

## Support and Resources

### Documentation
- [API Reference](./API.md)
- [Protocol Specification](./PROTOCOL.md)
- [Security Analysis](./SECURITY.md)


---

*This integration guide is regularly updated. Please check for the latest version and join our community for real-time support and updates.*