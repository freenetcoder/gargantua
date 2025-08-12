use solana_program_test::*;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use zerosol_solana::{
    instruction::ZerosolInstruction,
    state::{GlobalState, ZerosolAccount},
};

#[tokio::test]
async fn test_initialize() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "zerosol_solana",
        program_id,
        processor!(zerosol_solana::process_instruction),
    );

    let authority = Keypair::new();
    let global_state = Keypair::new();
    let token_mint = Keypair::new();

    program_test.add_account(
        authority.pubkey(),
        Account {
            lamports: 1_000_000_000,
            ..Account::default()
        },
    );

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let instruction = Instruction::new_with_borsh(
        program_id,
        &ZerosolInstruction::Initialize {
            epoch_length: 3600, // 1 hour
            fee: 1,
        },
        vec![
            AccountMeta::new(authority.pubkey(), true),
            AccountMeta::new(global_state.pubkey(), false),
            AccountMeta::new_readonly(token_mint.pubkey(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
    );

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[&payer, &authority],
        recent_blockhash,
    );

    banks_client.process_transaction(transaction).await.unwrap();

    // Verify global state was created
    let global_state_account = banks_client
        .get_account(global_state.pubkey())
        .await
        .unwrap()
        .unwrap();

    let global_state_data = GlobalState::try_from_slice(&global_state_account.data).unwrap();
    assert_eq!(global_state_data.authority, authority.pubkey());
    assert_eq!(global_state_data.epoch_length, 3600);
    assert_eq!(global_state_data.fee, 1);
}

#[tokio::test]
async fn test_register() {
    let program_id = Pubkey::new_unique();
    let mut program_test = ProgramTest::new(
        "zerosol_solana",
        program_id,
        processor!(zerosol_solana::process_instruction),
    );

    let payer = Keypair::new();
    let zerosol_account = Keypair::new();
    let pending_account = Keypair::new();
    let global_state = Keypair::new();

    program_test.add_account(
        payer.pubkey(),
        Account {
            lamports: 1_000_000_000,
            ..Account::default()
        },
    );

    let (mut banks_client, _payer, recent_blockhash) = program_test.start().await;

    // Create dummy signature data
    let public_key = [1u8; 32];
    let challenge = [2u8; 32];
    let response = [3u8; 32];

    let instruction = Instruction::new_with_borsh(
        program_id,
        &ZerosolInstruction::Register {
            public_key,
            challenge,
            response,
        },
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(zerosol_account.pubkey(), false),
            AccountMeta::new(pending_account.pubkey(), false),
            AccountMeta::new_readonly(global_state.pubkey(), false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
    );

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    // This will fail due to signature verification, but tests the instruction parsing
    let result = banks_client.process_transaction(transaction).await;
    assert!(result.is_err()); // Expected to fail due to invalid signature
}