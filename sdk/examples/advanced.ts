/**
 * Advanced usage example for Gargantua Protocol SDK
 * Demonstrates batch operations, custom proofs, and error handling
 */

import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { 
  GargantuaClient, 
  AnonymousAccount, 
  ProofGenerator,
  GargantuaError,
  ErrorCode,
  isGargantuaError 
} from '../src';

async function advancedExample() {
  console.log('🚀 Gargantua Protocol - Advanced Example');
  
  const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
  const client = new GargantuaClient(connection, {
    maxRetries: 5,
    confirmTransactionInitialTimeout: 60000,
  });
  
  try {
    // Create multiple accounts for demonstration
    console.log('👥 Creating multiple anonymous accounts...');
    const accounts: AnonymousAccount[] = [];
    const payers: Keypair[] = [];
    
    for (let i = 0; i < 3; i++) {
      const payer = Keypair.generate();
      payers.push(payer);
      
      const account = await client.registerAccount(payer);
      accounts.push(account);
      
      console.log(`✅ Account ${i + 1} created:`, account.getAccountAddress()?.toString());
    }
    
    // Batch deposit to all accounts
    console.log('\n💰 Performing batch deposits...');
    const depositPromises = accounts.map((account, index) => 
      client.deposit(account, 1000 * (index + 1), payers[index])
    );
    
    const depositSignatures = await Promise.all(depositPromises);
    console.log('✅ All deposits completed');
    depositSignatures.forEach((sig, index) => {
      console.log(`📄 Deposit ${index + 1} signature:`, sig);
    });
    
    // Multi-party transfer example
    console.log('\n🔄 Performing multi-party transfer...');
    
    // Create a complex transfer: accounts[0] and accounts[1] -> accounts[2]
    const proofGenerator = new ProofGenerator();
    
    const transferProof = await proofGenerator.generateTransferProof(
      [
        { account: accounts[0], amount: 500 },
        { account: accounts[1], amount: 300 },
      ],
      [
        { address: accounts[2].getPublicKey(), amount: 750 },
      ],
      50 // fee
    );
    
    console.log('✅ Transfer proof generated');
    console.log('🔍 Proof size:', JSON.stringify(transferProof).length, 'bytes');
    
    // Custom proof verification (client-side)
    console.log('\n🔐 Generating custom proofs...');
    
    // Generate range proof for a specific value
    const rangeProof = await proofGenerator.generateRangeProof(1000, 32);
    console.log('✅ Range proof generated for value 1000');
    
    // Generate ownership proof
    const ownershipProof = await accounts[0].generateOwnershipProof();
    console.log('✅ Ownership proof generated');
    console.log('🔑 Proof public key:', Buffer.from(ownershipProof.publicKey).toString('hex'));
    
    // Demonstrate error handling
    console.log('\n⚠️  Demonstrating error handling...');
    
    try {
      // Try to transfer more than available balance
      await client.transfer(accounts[0], accounts[1].getPublicKey(), 999999);
    } catch (error) {
      if (isGargantuaError(error)) {
        console.log('✅ Caught expected error:', error.code);
        
        switch (error.code) {
          case ErrorCode.INSUFFICIENT_FUNDS:
            console.log('💸 Insufficient funds for transfer');
            break;
          case ErrorCode.INVALID_TRANSFER_AMOUNT:
            console.log('❌ Invalid transfer amount');
            break;
          default:
            console.log('🤔 Other Gargantua error:', error.message);
        }
      } else {
        console.log('❌ Unexpected error:', error);
      }
    }
    
    // Account rollover example
    console.log('\n🔄 Performing account rollover...');
    
    try {
      const rolloverSignature = await client.rolloverAccount(accounts[0]);
      console.log('✅ Account rollover successful');
      console.log('📄 Rollover signature:', rolloverSignature);
    } catch (error) {
      console.log('ℹ️  Rollover not needed or failed:', error);
    }
    
    // Generate commitments with custom randomness
    console.log('\n🎲 Generating commitments with custom randomness...');
    
    const customRandomness = new Uint8Array(32);
    customRandomness.fill(42); // Use fixed randomness for demonstration
    
    const commitment1 = accounts[0].generateCommitment(1000, customRandomness);
    const commitment2 = accounts[0].generateCommitment(1000, customRandomness);
    
    console.log('✅ Generated identical commitments with same randomness');
    console.log('🔍 Commitment 1:', Buffer.from(commitment1.commitment).toString('hex').slice(0, 16) + '...');
    console.log('🔍 Commitment 2:', Buffer.from(commitment2.commitment).toString('hex').slice(0, 16) + '...');
    console.log('✅ Commitments match:', 
      Buffer.from(commitment1.commitment).equals(Buffer.from(commitment2.commitment))
    );
    
    // Performance measurement
    console.log('\n⚡ Performance measurements...');
    
    const startTime = Date.now();
    const performanceProof = await proofGenerator.generateRangeProof(12345, 32);
    const endTime = Date.now();
    
    console.log('✅ Range proof generation time:', endTime - startTime, 'ms');
    console.log('📊 Proof elements:', Object.keys(performanceProof).length);
    
    // Batch commitment generation
    console.log('\n📦 Batch commitment generation...');
    
    const batchStartTime = Date.now();
    const commitments = [];
    
    for (let i = 0; i < 10; i++) {
      const commitment = accounts[0].generateCommitment(i * 100);
      commitments.push(commitment);
    }
    
    const batchEndTime = Date.now();
    console.log('✅ Generated 10 commitments in', batchEndTime - batchStartTime, 'ms');
    console.log('📊 Average per commitment:', (batchEndTime - batchStartTime) / 10, 'ms');
    
    console.log('\n🎉 Advanced example completed successfully!');
    
  } catch (error) {
    console.error('❌ Fatal error in advanced example:', error);
    
    if (isGargantuaError(error)) {
      console.error('🔍 Error code:', error.code);
      console.error('📝 Error details:', error.details);
    }
    
    throw error;
  }
}

// Utility function to demonstrate SDK utilities
async function demonstrateUtilities() {
  console.log('\n🛠️  SDK Utilities Demonstration');
  
  // Account generation methods
  console.log('\n🔑 Account generation methods:');
  
  // Method 1: Random generation
  const randomAccount = AnonymousAccount.generate();
  console.log('✅ Random account generated');
  
  // Method 2: From seed
  const seedAccount = AnonymousAccount.fromSeed('my-secret-seed');
  console.log('✅ Seed-based account generated');
  
  // Method 3: From private key
  const privateKey = new Uint8Array(32);
  privateKey.fill(123);
  const keyAccount = AnonymousAccount.fromPrivateKey(privateKey);
  console.log('✅ Private key account generated');
  
  // Demonstrate deterministic generation
  const seed1 = AnonymousAccount.fromSeed('test-seed');
  const seed2 = AnonymousAccount.fromSeed('test-seed');
  
  console.log('🔍 Deterministic generation test:');
  console.log('   Same seed produces same keys:', 
    Buffer.from(seed1.getPrivateKey()).equals(Buffer.from(seed2.getPrivateKey()))
  );
  
  // Message signing
  console.log('\n✍️  Message signing:');
  const message = new TextEncoder().encode('Hello, Gargantua!');
  const signature = randomAccount.signMessage(message);
  
  console.log('✅ Message signed');
  console.log('🔍 Challenge length:', signature.challenge.length);
  console.log('🔍 Response length:', signature.response.length);
}

// Run the example
if (require.main === module) {
  Promise.resolve()
    .then(() => advancedExample())
    .then(() => demonstrateUtilities())
    .then(() => {
      console.log('\n🏁 All advanced examples completed!');
      process.exit(0);
    })
    .catch((error) => {
      console.error('💥 Fatal error:', error);
      process.exit(1);
    });
}

export { advancedExample, demonstrateUtilities };