use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use crate::utils::G1Point;
use curve25519_dalek::scalar::Scalar;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ZerosolAccount {
    pub commitment_left: [u8; 32],   // CLn commitment
    pub commitment_right: [u8; 32],  // CRn commitment
    pub public_key: [u8; 32],        // y public key
    pub last_rollover: u64,          // last epoch when account was rolled over
    pub is_registered: bool,
}

impl ZerosolAccount {
    pub const LEN: usize = 32 + 32 + 32 + 8 + 1;

    pub fn new(public_key: [u8; 32]) -> Self {
        Self {
            commitment_left: [0; 32],
            commitment_right: [0; 32],
            public_key,
            last_rollover: 0,
            is_registered: false,
        }
    }

    pub fn get_commitment_left(&self) -> Result<G1Point, solana_program::program_error::ProgramError> {
        G1Point::from_bytes(&self.commitment_left)
    }

    pub fn get_commitment_right(&self) -> Result<G1Point, solana_program::program_error::ProgramError> {
        G1Point::from_bytes(&self.commitment_right)
    }

    pub fn get_public_key(&self) -> Result<G1Point, solana_program::program_error::ProgramError> {
        G1Point::from_bytes(&self.public_key)
    }

    pub fn set_commitment_left(&mut self, point: &G1Point) {
        self.commitment_left = point.to_bytes();
    }

    pub fn set_commitment_right(&mut self, point: &G1Point) {
        self.commitment_right = point.to_bytes();
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PendingAccount {
    pub commitment_left: [u8; 32],   // pending CLn commitment
    pub commitment_right: [u8; 32],  // pending CRn commitment
}

impl PendingAccount {
    pub const LEN: usize = 32 + 32;

    pub fn new() -> Self {
        Self {
            commitment_left: [0; 32],
            commitment_right: [0; 32],
        }
    }

    pub fn get_commitment_left(&self) -> Result<G1Point, solana_program::program_error::ProgramError> {
        G1Point::from_bytes(&self.commitment_left)
    }

    pub fn get_commitment_right(&self) -> Result<G1Point, solana_program::program_error::ProgramError> {
        G1Point::from_bytes(&self.commitment_right)
    }

    pub fn set_commitment_left(&mut self, point: &G1Point) {
        self.commitment_left = point.to_bytes();
    }

    pub fn set_commitment_right(&mut self, point: &G1Point) {
        self.commitment_right = point.to_bytes();
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct GlobalState {
    pub authority: Pubkey,
    pub token_mint: Pubkey,
    pub epoch_length: u64,
    pub fee: u64,
    pub last_global_update: u64,
    pub current_epoch: u64,
}

impl GlobalState {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 8 + 8;

    pub fn new(authority: Pubkey, token_mint: Pubkey, epoch_length: u64, fee: u64) -> Self {
        Self {
            authority,
            token_mint,
            epoch_length,
            fee,
            last_global_update: 0,
            current_epoch: 0,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct NonceState {
    pub nonce: [u8; 32],
    pub epoch: u64,
    pub used: bool,
}

impl NonceState {
    pub const LEN: usize = 32 + 8 + 1;

    pub fn new(nonce: [u8; 32], epoch: u64) -> Self {
        Self {
            nonce,
            epoch,
            used: false,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct InnerProductProof {
    pub l_points: Vec<[u8; 32]>,
    pub r_points: Vec<[u8; 32]>,
    pub a: [u8; 32],
    pub b: [u8; 32],
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ZerosolProof {
    pub ba: [u8; 32],
    pub bs: [u8; 32],
    pub a: [u8; 32],
    pub b: [u8; 32],
    pub cln_g: Vec<[u8; 32]>,
    pub crn_g: Vec<[u8; 32]>,
    pub c_0g: Vec<[u8; 32]>,
    pub dg: Vec<[u8; 32]>,
    pub y_0g: Vec<[u8; 32]>,
    pub gg: Vec<[u8; 32]>,
    pub c_xg: Vec<[u8; 32]>,
    pub y_xg: Vec<[u8; 32]>,
    pub f: Vec<[u8; 32]>,
    pub z_a: [u8; 32],
    pub t_1: [u8; 32],
    pub t_2: [u8; 32],
    pub t_hat: [u8; 32],
    pub mu: [u8; 32],
    pub c: [u8; 32],
    pub s_sk: [u8; 32],
    pub s_r: [u8; 32],
    pub s_b: [u8; 32],
    pub s_tau: [u8; 32],
    pub ip_proof: InnerProductProof,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct BurnProof {
    pub ba: [u8; 32],
    pub bs: [u8; 32],
    pub t_1: [u8; 32],
    pub t_2: [u8; 32],
    pub t_hat: [u8; 32],
    pub mu: [u8; 32],
    pub c: [u8; 32],
    pub s_sk: [u8; 32],
    pub s_b: [u8; 32],
    pub s_tau: [u8; 32],
    pub ip_proof: InnerProductProof,
}