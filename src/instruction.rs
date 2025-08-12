use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use crate::state::{ZerosolProof, BurnProof};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum ZerosolInstruction {
    /// Initialize the global state
    /// Accounts:
    /// 0. [signer] Authority
    /// 1. [writable] Global state account
    /// 2. [] Token mint
    /// 3. [] System program
    Initialize {
        epoch_length: u64,
        fee: u64,
    },

    /// Register a new account
    /// Accounts:
    /// 0. [signer] Payer
    /// 1. [writable] Zerosol account
    /// 2. [writable] Pending account
    /// 3. [] Global state
    /// 4. [] System program
    Register {
        public_key: [u8; 32],
        challenge: [u8; 32],
        response: [u8; 32],
    },

    /// Fund an account with tokens
    /// Accounts:
    /// 0. [signer] Funder
    /// 1. [writable] Zerosol account
    /// 2. [writable] Pending account
    /// 3. [writable] Funder token account
    /// 4. [writable] Program token account
    /// 5. [] Token program
    /// 6. [] Global state
    Fund {
        amount: u64,
    },

    /// Perform an anonymous transfer
    /// Accounts:
    /// 0. [signer] Relayer
    /// 1. [writable] Beneficiary account
    /// 2. [writable] Beneficiary pending account
    /// 3. [writable] Nonce account
    /// 4. [] Global state
    /// 5. [] System program
    /// 6..N. [writable] Participant accounts and pending accounts
    Transfer {
        commitments_c: Vec<[u8; 32]>,
        commitment_d: [u8; 32],
        public_keys: Vec<[u8; 32]>,
        nonce: [u8; 32],
        beneficiary: [u8; 32],
        proof: ZerosolProof,
    },

    /// Burn tokens (withdraw)
    /// Accounts:
    /// 0. [signer] Withdrawer
    /// 1. [writable] Zerosol account
    /// 2. [writable] Pending account
    /// 3. [writable] Withdrawer token account
    /// 4. [writable] Program token account
    /// 5. [writable] Nonce account
    /// 6. [] Token program
    /// 7. [] Global state
    /// 8. [] System program
    Burn {
        amount: u64,
        nonce: [u8; 32],
        proof: BurnProof,
    },

    /// Roll over accounts to new epoch
    /// Accounts:
    /// 0. [signer] Anyone
    /// 1. [writable] Zerosol account
    /// 2. [writable] Pending account
    /// 3. [writable] Global state
    RollOver,
}