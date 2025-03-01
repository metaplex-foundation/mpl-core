use num_traits::ToPrimitive;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke, rent::Rent,
    system_instruction, sysvar::Sysvar,
};

use crate::{error::MplCoreError, state::Key};

pub(crate) fn close_program_account<'a>(
    account_to_close_info: &AccountInfo<'a>,
    funds_dest_account_info: &AccountInfo<'a>,
) -> ProgramResult {
    let rent = Rent::get()?;

    let account_size = account_to_close_info.data_len();
    let account_rent = rent.minimum_balance(account_size);
    let one_byte_rent = rent.minimum_balance(1);

    let amount_to_return = account_rent
        .checked_sub(one_byte_rent)
        .ok_or(MplCoreError::NumericalOverflowError)?;

    // Transfer lamports from the account to the destination account.
    let dest_starting_lamports = funds_dest_account_info.lamports();
    **funds_dest_account_info.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(amount_to_return)
        .ok_or(MplCoreError::NumericalOverflowError)?;
    **account_to_close_info.try_borrow_mut_lamports()? -= amount_to_return;

    account_to_close_info.realloc(1, false)?;
    account_to_close_info.data.borrow_mut()[0] = Key::Uninitialized.to_u8().unwrap();

    Ok(())
}

/// Resize an account using realloc and retain any lamport overages, modified from Solana Cookbook
pub(crate) fn resize_or_reallocate_account<'a>(
    target_account: &AccountInfo<'a>,
    funding_account: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    new_size: usize,
) -> ProgramResult {
    // If the account is already the correct size, return early
    let current_size = target_account.data_len();
    if new_size == current_size {
        return Ok(());
    }

    let rent = Rent::get()?;
    let new_minimum_balance = rent.minimum_balance(new_size);
    let current_minimum_balance = rent.minimum_balance(current_size);

    // Use match with comparison instead of if-chain
    match new_minimum_balance.cmp(&current_minimum_balance) {
        std::cmp::Ordering::Greater => {
            // New size requires more rent
            let lamports_diff = new_minimum_balance - current_minimum_balance;
            invoke(
                &system_instruction::transfer(
                    funding_account.key,
                    target_account.key,
                    lamports_diff,
                ),
                &[
                    funding_account.clone(),
                    target_account.clone(),
                    system_program.clone(),
                ],
            )?;
        }
        std::cmp::Ordering::Less => {
            // Return excess lamports to the funding account
            let lamports_diff = current_minimum_balance - new_minimum_balance;
            **funding_account.try_borrow_mut_lamports()? += lamports_diff;
            **target_account.try_borrow_mut_lamports()? -= lamports_diff;
        }
        std::cmp::Ordering::Equal => {
            // No lamport adjustment needed
        }
    }

    // Resize the account
    target_account.realloc(new_size, false)?;

    Ok(())
}
