use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use spl_token::{instruction as token_instruction, state::Account as TokenAccount};
use curve25519_dalek::scalar::Scalar;

use crate::{
    error::ZerosolError,
    instruction::ZerosolInstruction,
    state::{GlobalState, ZerosolAccount, PendingAccount, NonceState},
    utils::{
        G1Point, MAX_TRANSFER_AMOUNT, hash_to_scalar, verify_schnorr_signature,
        pedersen_commit, scalar_from_bytes, map_to_curve_with_index,
    },
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = ZerosolInstruction::try_from_slice(instruction_data)
        .map_err(|_| ZerosolError::InvalidInstruction)?;

    match instruction {
        ZerosolInstruction::Initialize { epoch_length, fee } => {
            process_initialize(program_id, accounts, epoch_length, fee)
        }
        ZerosolInstruction::Register {
            public_key,
            challenge,
            response,
        } => process_register(program_id, accounts, public_key, challenge, response),
        ZerosolInstruction::Fund { amount } => process_fund(program_id, accounts, amount),
        ZerosolInstruction::Transfer {
            commitments_c,
            commitment_d,
            public_keys,
            nonce,
            beneficiary,
            proof,
        } => process_transfer(
            program_id,
            accounts,
            commitments_c,
            commitment_d,
            public_keys,
            nonce,
            beneficiary,
            proof,
        ),
        ZerosolInstruction::Burn {
            amount,
            nonce,
            proof,
        } => process_burn(program_id, accounts, amount, nonce, proof),
        ZerosolInstruction::RollOver => process_rollover(program_id, accounts),
    }
}

fn process_initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    epoch_length: u64,
    fee: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let authority_info = next_account_info(account_info_iter)?;
    let global_state_info = next_account_info(account_info_iter)?;
    let token_mint_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;

    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let rent = Rent::get()?;
    let space = GlobalState::LEN;
    let lamports = rent.minimum_balance(space);

    invoke(
        &system_instruction::create_account(
            authority_info.key,
            global_state_info.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[
            authority_info.clone(),
            global_state_info.clone(),
            system_program_info.clone(),
        ],
    )?;

    let global_state = GlobalState::new(
        *authority_info.key,
        *token_mint_info.key,
        epoch_length,
        fee,
    );

    global_state.serialize(&mut &mut global_state_info.data.borrow_mut()[..])?;

    msg!("Zerosol program initialized");
    Ok(())
}

fn process_register(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    public_key: [u8; 32],
    challenge: [u8; 32],
    response: [u8; 32],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let payer_info = next_account_info(account_info_iter)?;
    let zerosol_account_info = next_account_info(account_info_iter)?;
    let pending_account_info = next_account_info(account_info_iter)?;
    let global_state_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;

    if !payer_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Verify Schnorr signature
    let public_key_point = G1Point::from_bytes(&public_key)?;
    let challenge_scalar = scalar_from_bytes(&challenge);
    let response_scalar = scalar_from_bytes(&response);

    let message = program_id.to_bytes();
    if !verify_schnorr_signature(&public_key_point, &message, &challenge_scalar, &response_scalar) {
        return Err(ZerosolError::InvalidRegistrationSignature.into());
    }

    // Create zerosol account
    let rent = Rent::get()?;
    let space = ZerosolAccount::LEN;
    let lamports = rent.minimum_balance(space);

    invoke(
        &system_instruction::create_account(
            payer_info.key,
            zerosol_account_info.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[
            payer_info.clone(),
            zerosol_account_info.clone(),
            system_program_info.clone(),
        ],
    )?;

    // Create pending account
    let space = PendingAccount::LEN;
    let lamports = rent.minimum_balance(space);

    invoke(
        &system_instruction::create_account(
            payer_info.key,
            pending_account_info.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[
            payer_info.clone(),
            pending_account_info.clone(),
            system_program_info.clone(),
        ],
    )?;

    // Initialize accounts
    let mut zerosol_account = ZerosolAccount::new(public_key);
    zerosol_account.is_registered = true;
    zerosol_account.serialize(&mut &mut zerosol_account_info.data.borrow_mut()[..])?;

    let mut pending_account = PendingAccount::new();
    // Set initial pending commitment: y * 1 + g * 0 (public key with zero value)
    let g = G1Point::generator();
    pending_account.set_commitment_left(&public_key_point);
    pending_account.set_commitment_right(&g);
    pending_account.serialize(&mut &mut pending_account_info.data.borrow_mut()[..])?;

    msg!("Account registered successfully");
    Ok(())
}

fn process_fund(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let funder_info = next_account_info(account_info_iter)?;
    let zerosol_account_info = next_account_info(account_info_iter)?;
    let pending_account_info = next_account_info(account_info_iter)?;
    let funder_token_info = next_account_info(account_info_iter)?;
    let program_token_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let global_state_info = next_account_info(account_info_iter)?;

    if !funder_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if amount > MAX_TRANSFER_AMOUNT {
        return Err(ZerosolError::TransferAmountOutOfRange.into());
    }

    // Load accounts
    let mut zerosol_account = ZerosolAccount::try_from_slice(&zerosol_account_info.data.borrow())?;
    if !zerosol_account.is_registered {
        return Err(ZerosolError::AccountNotRegistered.into());
    }

    // Roll over if needed
    let global_state = GlobalState::try_from_slice(&global_state_info.data.borrow())?;
    let clock = Clock::get()?;
    let current_epoch = clock.unix_timestamp as u64 / global_state.epoch_length;
    
    if zerosol_account.last_rollover < current_epoch {
        rollover_account(&mut zerosol_account, pending_account_info, current_epoch)?;
    }

    // Transfer tokens
    invoke(
        &token_instruction::transfer(
            token_program_info.key,
            funder_token_info.key,
            program_token_info.key,
            funder_info.key,
            &[],
            amount,
        )?,
        &[
            funder_token_info.clone(),
            program_token_info.clone(),
            funder_info.clone(),
            token_program_info.clone(),
        ],
    )?;

    // Update pending commitment
    let mut pending_account = PendingAccount::try_from_slice(&pending_account_info.data.borrow())?;
    let current_left = pending_account.get_commitment_left()?;
    let g = G1Point::generator();
    let amount_scalar = Scalar::from(amount);
    let new_left = current_left.add(&g.mul(&amount_scalar));
    pending_account.set_commitment_left(&new_left);
    pending_account.serialize(&mut &mut pending_account_info.data.borrow_mut()[..])?;

    msg!("Account funded with {} tokens", amount);
    Ok(())
}

fn process_transfer(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    commitments_c: Vec<[u8; 32]>,
    commitment_d: [u8; 32],
    public_keys: Vec<[u8; 32]>,
    nonce: [u8; 32],
    beneficiary: [u8; 32],
    proof: crate::state::ZerosolProof,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let relayer_info = next_account_info(account_info_iter)?;
    let beneficiary_account_info = next_account_info(account_info_iter)?;
    let beneficiary_pending_info = next_account_info(account_info_iter)?;
    let nonce_account_info = next_account_info(account_info_iter)?;
    let global_state_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;

    if !relayer_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Check nonce hasn't been used
    if nonce_account_info.data_len() > 0 {
        let nonce_state = NonceState::try_from_slice(&nonce_account_info.data.borrow())?;
        if nonce_state.used {
            return Err(ZerosolError::NonceAlreadySeen.into());
        }
    } else {
        // Create nonce account
        let rent = Rent::get()?;
        let space = NonceState::LEN;
        let lamports = rent.minimum_balance(space);

        invoke(
            &system_instruction::create_account(
                relayer_info.key,
                nonce_account_info.key,
                lamports,
                space as u64,
                program_id,
            ),
            &[
                relayer_info.clone(),
                nonce_account_info.clone(),
                system_program_info.clone(),
            ],
        )?;
    }

    let global_state = GlobalState::try_from_slice(&global_state_info.data.borrow())?;
    let clock = Clock::get()?;
    let current_epoch = clock.unix_timestamp as u64 / global_state.epoch_length;

    // Verify proof (simplified - in practice would need full bulletproof verification)
    if !verify_transfer_proof(&proof, &commitments_c, &commitment_d, &public_keys, current_epoch) {
        return Err(ZerosolError::TransferProofVerificationFailed.into());
    }

    // Update beneficiary account with fee
    let mut beneficiary_account = ZerosolAccount::try_from_slice(&beneficiary_account_info.data.borrow())?;
    if !beneficiary_account.is_registered {
        return Err(ZerosolError::AccountNotRegistered.into());
    }

    if beneficiary_account.last_rollover < current_epoch {
        rollover_account(&mut beneficiary_account, beneficiary_pending_info, current_epoch)?;
    }

    let mut beneficiary_pending = PendingAccount::try_from_slice(&beneficiary_pending_info.data.borrow())?;
    let current_left = beneficiary_pending.get_commitment_left()?;
    let g = G1Point::generator();
    let fee_scalar = Scalar::from(global_state.fee);
    let new_left = current_left.add(&g.mul(&fee_scalar));
    beneficiary_pending.set_commitment_left(&new_left);
    beneficiary_pending.serialize(&mut &mut beneficiary_pending_info.data.borrow_mut()[..])?;

    // Process participant accounts
    let remaining_accounts = &accounts[6..];
    for (i, chunk) in remaining_accounts.chunks(2).enumerate() {
        if i >= public_keys.len() {
            break;
        }

        let account_info = &chunk[0];
        let pending_info = &chunk[1];

        let mut zerosol_account = ZerosolAccount::try_from_slice(&account_info.data.borrow())?;
        if !zerosol_account.is_registered {
            return Err(ZerosolError::AccountNotRegistered.into());
        }

        if zerosol_account.last_rollover < current_epoch {
            rollover_account(&mut zerosol_account, pending_info, current_epoch)?;
        }

        // Update pending commitments
        let mut pending_account = PendingAccount::try_from_slice(&pending_info.data.borrow())?;
        let current_left = pending_account.get_commitment_left()?;
        let current_right = pending_account.get_commitment_right()?;
        
        let c_point = G1Point::from_bytes(&commitments_c[i])?;
        let d_point = G1Point::from_bytes(&commitment_d)?;
        
        let new_left = current_left.add(&c_point);
        let new_right = current_right.add(&d_point);
        
        pending_account.set_commitment_left(&new_left);
        pending_account.set_commitment_right(&new_right);
        pending_account.serialize(&mut &mut pending_info.data.borrow_mut()[..])?;
    }

    // Mark nonce as used
    let nonce_state = NonceState::new(nonce, current_epoch);
    let mut used_nonce = nonce_state;
    used_nonce.used = true;
    used_nonce.serialize(&mut &mut nonce_account_info.data.borrow_mut()[..])?;

    msg!("Transfer completed successfully");
    Ok(())
}

fn process_burn(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    nonce: [u8; 32],
    proof: crate::state::BurnProof,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let withdrawer_info = next_account_info(account_info_iter)?;
    let zerosol_account_info = next_account_info(account_info_iter)?;
    let pending_account_info = next_account_info(account_info_iter)?;
    let withdrawer_token_info = next_account_info(account_info_iter)?;
    let program_token_info = next_account_info(account_info_iter)?;
    let nonce_account_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    let global_state_info = next_account_info(account_info_iter)?;
    let system_program_info = next_account_info(account_info_iter)?;

    if !withdrawer_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if amount > MAX_TRANSFER_AMOUNT {
        return Err(ZerosolError::TransferAmountOutOfRange.into());
    }

    // Check nonce
    if nonce_account_info.data_len() > 0 {
        let nonce_state = NonceState::try_from_slice(&nonce_account_info.data.borrow())?;
        if nonce_state.used {
            return Err(ZerosolError::NonceAlreadySeen.into());
        }
    } else {
        // Create nonce account
        let rent = Rent::get()?;
        let space = NonceState::LEN;
        let lamports = rent.minimum_balance(space);

        invoke(
            &system_instruction::create_account(
                withdrawer_info.key,
                nonce_account_info.key,
                lamports,
                space as u64,
                program_id,
            ),
            &[
                withdrawer_info.clone(),
                nonce_account_info.clone(),
                system_program_info.clone(),
            ],
        )?;
    }

    let global_state = GlobalState::try_from_slice(&global_state_info.data.borrow())?;
    let clock = Clock::get()?;
    let current_epoch = clock.unix_timestamp as u64 / global_state.epoch_length;

    // Load and rollover account
    let mut zerosol_account = ZerosolAccount::try_from_slice(&zerosol_account_info.data.borrow())?;
    if !zerosol_account.is_registered {
        return Err(ZerosolError::AccountNotRegistered.into());
    }

    if zerosol_account.last_rollover < current_epoch {
        rollover_account(&mut zerosol_account, pending_account_info, current_epoch)?;
    }

    // Verify burn proof (simplified)
    if !verify_burn_proof(&proof, &zerosol_account, amount, current_epoch) {
        return Err(ZerosolError::BurnProofVerificationFailed.into());
    }

    // Update pending commitment (subtract amount)
    let mut pending_account = PendingAccount::try_from_slice(&pending_account_info.data.borrow())?;
    let current_left = pending_account.get_commitment_left()?;
    let g = G1Point::generator();
    let amount_scalar = Scalar::from(amount);
    let new_left = current_left.add(&g.mul(&-amount_scalar));
    pending_account.set_commitment_left(&new_left);
    pending_account.serialize(&mut &mut pending_account_info.data.borrow_mut()[..])?;

    // Transfer tokens back to user
    let seeds = &[b"token_authority"];
    let (token_authority, bump) = Pubkey::find_program_address(seeds, program_id);
    let authority_seeds = &[&seeds[0][..], &[bump]];

    invoke_signed(
        &token_instruction::transfer(
            token_program_info.key,
            program_token_info.key,
            withdrawer_token_info.key,
            &token_authority,
            &[],
            amount,
        )?,
        &[
            program_token_info.clone(),
            withdrawer_token_info.clone(),
            token_program_info.clone(),
        ],
        &[authority_seeds],
    )?;

    // Mark nonce as used
    let nonce_state = NonceState::new(nonce, current_epoch);
    let mut used_nonce = nonce_state;
    used_nonce.used = true;
    used_nonce.serialize(&mut &mut nonce_account_info.data.borrow_mut()[..])?;

    msg!("Burn completed successfully, {} tokens withdrawn", amount);
    Ok(())
}

fn process_rollover(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let _signer_info = next_account_info(account_info_iter)?;
    let zerosol_account_info = next_account_info(account_info_iter)?;
    let pending_account_info = next_account_info(account_info_iter)?;
    let global_state_info = next_account_info(account_info_iter)?;

    let global_state = GlobalState::try_from_slice(&global_state_info.data.borrow())?;
    let clock = Clock::get()?;
    let current_epoch = clock.unix_timestamp as u64 / global_state.epoch_length;

    let mut zerosol_account = ZerosolAccount::try_from_slice(&zerosol_account_info.data.borrow())?;
    rollover_account(&mut zerosol_account, pending_account_info, current_epoch)?;

    msg!("Account rolled over to epoch {}", current_epoch);
    Ok(())
}

fn rollover_account(
    zerosol_account: &mut ZerosolAccount,
    pending_account_info: &AccountInfo,
    current_epoch: u64,
) -> ProgramResult {
    if zerosol_account.last_rollover >= current_epoch {
        return Ok(());
    }

    let pending_account = PendingAccount::try_from_slice(&pending_account_info.data.borrow())?;
    
    // Add pending commitments to main commitments
    let current_left = zerosol_account.get_commitment_left()?;
    let current_right = zerosol_account.get_commitment_right()?;
    let pending_left = pending_account.get_commitment_left()?;
    let pending_right = pending_account.get_commitment_right()?;
    
    let new_left = current_left.add(&pending_left);
    let new_right = current_right.add(&pending_right);
    
    zerosol_account.set_commitment_left(&new_left);
    zerosol_account.set_commitment_right(&new_right);
    zerosol_account.last_rollover = current_epoch;

    // Clear pending account
    let mut cleared_pending = PendingAccount::new();
    cleared_pending.serialize(&mut &mut pending_account_info.data.borrow_mut()[..])?;

    Ok(())
}

// Simplified proof verification functions
// In practice, these would implement full bulletproof verification
fn verify_transfer_proof(
    _proof: &crate::state::ZerosolProof,
    _commitments_c: &[[u8; 32]],
    _commitment_d: &[u8; 32],
    _public_keys: &[[u8; 32]],
    _epoch: u64,
) -> bool {
    // TODO: Implement full bulletproof verification
    // This is a placeholder that always returns true
    // In production, this would verify the zero-knowledge proof
    true
}

fn verify_burn_proof(
    _proof: &crate::state::BurnProof,
    _account: &ZerosolAccount,
    _amount: u64,
    _epoch: u64,
) -> bool {
    // TODO: Implement full bulletproof verification
    // This is a placeholder that always returns true
    // In production, this would verify the zero-knowledge proof
    true
}