use solana_program::{rent::Rent, system_program, sysvar::Sysvar};

use super::*;
use crate::state::{COLLECT_RECIPIENT1, COLLECT_RECIPIENT2};

use crate::{
    error::MplCoreError, instruction::accounts::CollectAccounts, state::Key, utils::load_key, ID,
};

pub(crate) fn collect<'a>(accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    // Accounts.
    let ctx = CollectAccounts::context(accounts)?;

    if *ctx.accounts.recipient1.key != COLLECT_RECIPIENT1 {
        return Err(MplCoreError::IncorrectAccount.into());
    }

    if *ctx.accounts.recipient2.key != COLLECT_RECIPIENT2 {
        return Err(MplCoreError::IncorrectAccount.into());
    }

    let recipient1_info = ctx.accounts.recipient1;
    let recipient2_info = ctx.accounts.recipient2;

    for account_info in ctx.remaining_accounts {
        if account_info.owner != &ID {
            return Err(MplCoreError::IncorrectAccount.into());
        }

        collect_from_account(account_info, recipient1_info, recipient2_info)?;
    }

    Ok(())
}

fn collect_from_account(
    account_info: &AccountInfo,
    dest1_info: &AccountInfo,
    dest2_info: &AccountInfo,
) -> ProgramResult {
    let rent = Rent::get()?;

    let (fee_amount, rent_amount) = match load_key(account_info, 0)? {
        Key::Uninitialized => {
            account_info.assign(&system_program::ID);

            let uninitialized_rent = rent.minimum_balance(1);
            let fee_amount = account_info
                .lamports()
                .checked_sub(uninitialized_rent)
                .ok_or(MplCoreError::NumericalOverflowError)?;
            (fee_amount, uninitialized_rent)
        }
        Key::AssetV1 | Key::HashedAssetV1 => {
            let asset_rent = rent.minimum_balance(account_info.data_len());
            let fee_amount = account_info
                .lamports()
                .checked_sub(asset_rent)
                .ok_or(MplCoreError::NumericalOverflowError)?;

            (fee_amount, asset_rent)
        }
        _ => return Err(MplCoreError::IncorrectAccount.into()),
    };

    let split_fee_amount = fee_amount
        .checked_div(2)
        .ok_or(MplCoreError::NumericalOverflowError)?;

    **dest1_info.lamports.borrow_mut() = dest1_info
        .lamports()
        .checked_add(split_fee_amount)
        .ok_or(MplCoreError::NumericalOverflowError)?;
    **dest2_info.lamports.borrow_mut() = dest2_info
        .lamports()
        .checked_add(
            fee_amount
                .checked_sub(split_fee_amount)
                .ok_or(MplCoreError::NumericalOverflowError)?,
        )
        .ok_or(MplCoreError::NumericalOverflowError)?;

    **account_info.lamports.borrow_mut() = rent_amount;

    Ok(())
}
