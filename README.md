# Gargantua Solana Program

A Solana implementation of the Zerosol anonymous payment system. This program provides privacy-preserving transactions using zero-knowledge proofs and bulletproofs.

## Features

- **Anonymous Transfers**: Transfer tokens without revealing sender, receiver, or amount
- **Account Registration**: Schnorr signature-based account registration
- **Epoch-based Rollover**: Periodic account state updates for efficiency
- **Bulletproof Integration**: Zero-knowledge range proofs for transaction validity
- **Token Integration**: Works with SPL tokens

## Architecture

The program consists of several key components:

### State Management
- `GlobalState`: Program configuration and epoch management
- `ZerosolAccount`: Main account commitments and metadata
- `PendingAccount`: Temporary commitments for current epoch
- `NonceState`: Prevents replay attacks

### Instructions
- `Initialize`: Set up the program with epoch length and fees
- `Register`: Register a new anonymous account with Schnorr signature
- `Fund`: Deposit tokens into an anonymous account
- `Transfer`: Perform anonymous transfers between accounts
- `Burn`: Withdraw tokens from an anonymous account
- `RollOver`: Update account state for new epoch

### Cryptographic Primitives
- Pedersen commitments using Ristretto points
- Schnorr signatures for account registration
- Hash-to-curve for deterministic point generation
- Bulletproof verification (placeholder implementation)

## Usage

### Building
```bash
cargo build-bpf
```

### Testing
```bash
cargo test
```

### Deployment
```bash
solana program deploy target/deploy/zerosol_solana.so
```

## Keys

1. **Curve Choice**: Uses Ristretto/Curve25519
2. **Account Model**: Account-based architecture
3. **Token Integration**: Uses SPL tokens
4. **Epoch Management**: Epoch handling using Solana's clock
5. **Proof Verification**: WIP

## Security Considerations

⚠️ **This is a proof-of-concept implementation**

- Bulletproof verification is not fully implemented
- Schnorr signature verification is simplified

## Implementation Notes

### Proof Verification
The current implementation includes placeholder functions for proof verification. A production version would need:
- Full bulletproof verification implementation
- Optimized elliptic curve operations
- Proper constraint system verification
- Gas/compute unit optimization

### Account Management
- Accounts are rolled over between epochs to maintain privacy
- Pending commitments are used to batch updates
- Nonce tracking prevents replay attacks

### Token Handling
- Integrates with SPL token program
- Uses program-derived addresses for token custody
- Supports deposit and withdrawal operations

## Future Improvements

1. **Complete Proof System**: Implement full bulletproof verification
2. **Optimization**: Reduce compute unit usage for complex operations
3. **Batching**: Support batch operations for efficiency
4. **Privacy Enhancements**: Additional anonymity features
5. **Audit**: Comprehensive security review

## License

Apache License 2.0 (matching original Solidity contracts)