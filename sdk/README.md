# Gargantua Protocol SDK

TypeScript SDK for the Gargantua Protocol - Zero-Knowledge Anonymous Payments on Solana.

## Installation

```bash
npm install @gargantua/sdk
```

## Quick Start

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
await client.transfer(account, recipient.getPublicKey(), 500);

// Withdraw tokens
await client.withdraw(account, 300);
```

## Features

- **Complete Anonymity**: Hide sender, receiver, and transaction amounts
- **Zero-Knowledge Proofs**: Bulletproofs for range proofs and balance verification
- **Solana Integration**: Native integration with Solana blockchain
- **TypeScript Support**: Full TypeScript support with type definitions
- **Easy to Use**: Simple API for complex cryptographic operations

## API Reference

### GargantuaClient

Main client for interacting with the Gargantua Protocol.

#### Constructor

```typescript
new GargantuaClient(connection: Connection, config?: GargantuaClientConfig)
```

#### Methods

- `registerAccount(payer: Keypair): Promise<AnonymousAccount>`
- `deposit(account: AnonymousAccount, amount: number): Promise<string>`
- `transfer(from: AnonymousAccount, to: PublicKey, amount: number): Promise<string>`
- `withdraw(account: AnonymousAccount, amount: number): Promise<string>`
- `getAccount(address: PublicKey): Promise<ZerosolAccount | null>`
- `getCurrentEpoch(): Promise<number>`

### AnonymousAccount

Represents an anonymous account with cryptographic capabilities.

#### Static Methods

- `AnonymousAccount.generate(): AnonymousAccount`
- `AnonymousAccount.fromPrivateKey(privateKey: Uint8Array): AnonymousAccount`
- `AnonymousAccount.fromSeed(seed: string): AnonymousAccount`

#### Methods

- `getPrivateKey(): Uint8Array`
- `getPublicKey(): PublicKey`
- `signMessage(message: Uint8Array): SchnorrSignature`
- `generateCommitment(value: number, randomness?: Uint8Array): Commitment`

### ProofGenerator

Utilities for generating zero-knowledge proofs.

#### Methods

- `generateTransferProof(inputs: TransferInput[], outputs: TransferOutput[], fee: number): Promise<TransferProof>`
- `generateBurnProof(account: AnonymousAccount, amount: number): Promise<BurnProof>`
- `generateRangeProof(value: number, bitLength?: number): Promise<RangeProof>`

## Configuration

### Client Configuration

```typescript
interface GargantuaClientConfig {
  programId?: PublicKey;
  commitment?: Commitment;
  confirmTransactionInitialTimeout?: number;
  maxRetries?: number;
}
```

### Network Endpoints

```typescript
import { DEVNET_RPC, TESTNET_RPC, MAINNET_RPC } from '@gargantua/sdk';

// Use predefined endpoints
const connection = new Connection(DEVNET_RPC);
```

## Error Handling

The SDK provides comprehensive error handling with specific error codes:

```typescript
import { GargantuaError, ErrorCode, isGargantuaError } from '@gargantua/sdk';

try {
  await client.transfer(account, recipient, amount);
} catch (error) {
  if (isGargantuaError(error)) {
    switch (error.code) {
      case ErrorCode.INSUFFICIENT_FUNDS:
        console.log('Not enough funds for transfer');
        break;
      case ErrorCode.ACCOUNT_NOT_REGISTERED:
        console.log('Account not registered');
        break;
      default:
        console.log('Unknown error:', error.message);
    }
  }
}
```

## Advanced Usage

### Multi-Party Transfers

```typescript
// Create transfer with multiple inputs and outputs
const transferBuilder = client.createTransferBuilder();

// Add inputs
transferBuilder.addInput(account1, 500);
transferBuilder.addInput(account2, 300);

// Add outputs
transferBuilder.addOutput(recipient1, 400);
transferBuilder.addOutput(recipient2, 350);

// Set fee
transferBuilder.setFee(50);

// Build and submit
const transaction = await transferBuilder.build();
const signature = await client.submitTransaction(transaction);
```

### Batch Operations

```typescript
// Batch multiple deposits
const batchBuilder = client.createBatchBuilder();

for (const account of accounts) {
  batchBuilder.addDeposit(account, 1000);
}

const transactions = await batchBuilder.build();
const signatures = await client.submitBatch(transactions);
```

### Custom Proof Generation

```typescript
import { ProofGenerator } from '@gargantua/sdk';

const proofGenerator = new ProofGenerator();

// Generate custom transfer proof
const proof = await proofGenerator.generateTransferProof(
  [{ account: sender, amount: 1000 }],
  [{ address: recipient, amount: 950 }],
  50 // fee
);
```

## Testing

The SDK includes comprehensive tests:

```bash
# Run all tests
npm test

# Run tests in watch mode
npm run test:watch

# Run tests with coverage
npm run test:coverage
```

## Building

```bash
# Build the SDK
npm run build

# Build in watch mode
npm run build:watch

# Type checking
npm run typecheck

# Linting
npm run lint
npm run lint:fix
```

## Examples

See the [examples](./examples) directory for complete usage examples:

- [Basic Usage](./examples/basic.ts)
- [DeFi Integration](./examples/defi.ts)
- [Wallet Integration](./examples/wallet.ts)
- [Enterprise Usage](./examples/enterprise.ts)

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](../LICENSE) file for details.

## Support

- [Documentation](https://docs.gargantua.protocol)
- [Discord](https://discord.gg/gargantua)
- [GitHub Issues](https://github.com/your-org/gargantua-protocol/issues)

## Security

For security concerns, please email security@gargantua.protocol instead of using the issue tracker.