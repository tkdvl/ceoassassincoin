// SPDX-License-Identifier: MIT

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    msg,
};
use spl_token::state::{Account, Mint};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    // Fetch accounts
    let payer_account = next_account_info(accounts_iter)?;
    let token_account = next_account_info(accounts_iter)?;
    let charity_wallet_account = next_account_info(accounts_iter)?;
    let tracking_wallet_account = next_account_info(accounts_iter)?; // For tracking trades

    // Deserialize instruction data to get transaction amount
    let transaction_amount = u64::from_le_bytes(
        instruction_data.try_into().map_err(|_| ProgramError::InvalidInstructionData)?
    );

    // Define transaction fee percentages
    const CHARITY_FEE_PERCENT: u64 = 3; // 3% to Charity Wallet

    // Calculate charity amount
    let charity_amount = (transaction_amount * CHARITY_FEE_PERCENT) / 100;

    // Verify sufficient balance
    let token_account_data = Account::unpack(&token_account.data.borrow())?;
    if token_account_data.amount < transaction_amount {
        msg!("Insufficient funds for transaction");
        return Err(ProgramError::InsufficientFunds);
    }

    // Transfer to Charity Wallet
    msg!("Transferring {} tokens to Charity Wallet", charity_amount);
    let charity_transfer_result = spl_token::instruction::transfer(
        &spl_token::id(),
        token_account.key,
        charity_wallet_account.key,
        payer_account.key,
        &[],
        charity_amount,
    );
    if charity_transfer_result.is_err() {
        msg!("Charity transfer failed");
        return Err(ProgramError::Custom(0));
    }

    // Record transaction for reward tracking
    msg!("Recording transaction for rewards tracking");
    let tracking_result = spl_token::instruction::transfer(
        &spl_token::id(),
        token_account.key,
        tracking_wallet_account.key,
        payer_account.key,
        &[],
        transaction_amount - charity_amount, // Only record remaining amount
    );
    if tracking_result.is_err() {
        msg!("Reward tracking failed");
        return Err(ProgramError::Custom(1));
    }

    // Finalize the transaction
    msg!("Transaction completed successfully");
    Ok(())
}

// Airdrop distribution logic will need a separate function to calculate and distribute rewards
// based on tracked trades and milestones met for price appreciation.
