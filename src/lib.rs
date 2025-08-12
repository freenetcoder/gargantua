use solana_program::{
    account_info::AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod utils;
pub mod bulletproof;
pub mod curve_ops;
pub mod constraint_system;

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Initialize optimized curve operations on first use
    utils::init_optimized_curve_ops();
    
    processor::process_instruction(program_id, accounts, instruction_data)
}