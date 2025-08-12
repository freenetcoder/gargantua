/**
 * Basic usage example for Gargantua Protocol SDK
 */

import { Connection, Keypair, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { GargantuaClient, AnonymousAccount } from '../src';

async function basicExample() {
  console.log('🚀 Gargantua Protocol - Basic Example');
  
  // Initialize connection to Solana devnet
  const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
  
  // Initialize Gargantua client
  const client = new GargantuaClient(connection);
  
  try {
    // Step 1: Create payer account with some SOL
    const payer = Keypair.generate();
    console.log('💰 Payer address:', payer.publicKey.toString());
    
    // In a real scenario, you would fund this account
    // For demo purposes, we'll assume it has funds
    
    // Step 2: Register anonymous account
    console.log('\n📝 Registering anonymous account...');
    const anonymousAccount = await client.registerAccount(payer);
    console.log('✅ Anonymous account registered');
    console.log('🔑 Account address:', anonymousAccount.getAccountAddress()?.toString());
    
    // Step 3: Deposit tokens
    console.log('\n💸 Depositing 1000 tokens...');
    const depositAmount = 1000;
    const depositSignature = await client.deposit(anonymousAccount, depositAmount, payer);
    console.log('✅ Deposit successful');
    console.log('📄 Transaction signature:', depositSignature);
    
    // Step 4: Create another anonymous account for transfer
    console.log('\n👤 Creating recipient account...');
    const recipientPayer = Keypair.generate();
    const recipientAccount = await client.registerAccount(recipientPayer);
    console.log('✅ Recipient account created');
    
    // Step 5: Perform anonymous transfer
    console.log('\n🔄 Performing anonymous transfer of 500 tokens...');
    const transferAmount = 500;
    const transferSignature = await client.transfer(
      anonymousAccount,
      recipientAccount.getPublicKey(),
      transferAmount
    );
    console.log('✅ Anonymous transfer successful');
    console.log('📄 Transaction signature:', transferSignature);
    
    // Step 6: Withdraw tokens
    console.log('\n💳 Withdrawing 300 tokens...');
    const withdrawAmount = 300;
    const withdrawSignature = await client.withdraw(anonymousAccount, withdrawAmount);
    console.log('✅ Withdrawal successful');
    console.log('📄 Transaction signature:', withdrawSignature);
    
    // Step 7: Check current epoch
    console.log('\n⏰ Checking current epoch...');
    const currentEpoch = await client.getCurrentEpoch();
    console.log('📅 Current epoch:', currentEpoch);
    
    console.log('\n🎉 Basic example completed successfully!');
    
  } catch (error) {
    console.error('❌ Error:', error);
    
    if (error instanceof Error) {
      console.error('Message:', error.message);
      console.error('Stack:', error.stack);
    }
  }
}

// Run the example
if (require.main === module) {
  basicExample()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error('Fatal error:', error);
      process.exit(1);
    });
}

export { basicExample };