use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum ZerosolError {
    #[error("Invalid instruction")]
    InvalidInstruction,
    #[error("Account not registered")]
    AccountNotRegistered,
    #[error("Account already registered")]
    AccountAlreadyRegistered,
    #[error("Invalid registration signature")]
    InvalidRegistrationSignature,
    #[error("Transfer amount out of range")]
    TransferAmountOutOfRange,
    #[error("Nonce already seen")]
    NonceAlreadySeen,
    #[error("Transfer proof verification failed")]
    TransferProofVerificationFailed,
    #[error("Burn proof verification failed")]
    BurnProofVerificationFailed,
    #[error("Inner product proof verification failed")]
    InnerProductProofVerificationFailed,
    #[error("Sigma protocol challenge equality failure")]
    SigmaProtocolChallengeFailed,
    #[error("Invalid epoch")]
    InvalidEpoch,
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Invalid account data")]
    InvalidAccountData,
    #[error("Invalid proof structure")]
    InvalidProofStructure,
    #[error("Range proof verification failed")]
    RangeProofVerificationFailed,
    #[error("Constraint system verification failed")]
    ConstraintSystemVerificationFailed,
    #[error("Balance conservation check failed")]
    BalanceConservationFailed,
    #[error("Polynomial evaluation verification failed")]
    PolynomialEvaluationFailed,
    #[error("Arithmetic constraint verification failed")]
    ArithmeticConstraintFailed,
    #[error("Invalid commitment")]
    InvalidCommitment,
    #[error("Epoch transition error")]
    EpochTransitionError,
}

impl From<ZerosolError> for ProgramError {
    fn from(e: ZerosolError) -> Self {
        ProgramError::Custom(e as u32)
    }
}