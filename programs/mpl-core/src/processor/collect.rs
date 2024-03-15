use solana_program::{rent::Rent, system_program, sysvar::Sysvar};

use super::*;
use crate::state::{DataBlob, COLLECT_RECIPIENT};

use crate::{
    error::MplCoreError,
    instruction::accounts::CollectAccounts,
    state::{Asset, HashedAsset, Key},
    utils::{fetch_core_data, load_key},
    ID,
};

pub(crate) fn collect<'a>(accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    // Accounts.
    let ctx = CollectAccounts::context(accounts)?;

    if *ctx.accounts.recipient.key != COLLECT_RECIPIENT {
        return Err(MplCoreError::IncorrectAccount.into());
    }

    let recipient_info = ctx.accounts.recipient;

    for account_info in ctx.remaining_accounts {
        if account_info.owner != &ID {
            return Err(MplCoreError::IncorrectAccount.into());
        }

        collect_from_account(account_info, recipient_info)?;
    }

    Ok(())
}

fn collect_from_account(account_info: &AccountInfo, dest_info: &AccountInfo) -> ProgramResult {
    let rent = Rent::get()?;

    let (fee_amount, rent_amount) = match load_key(account_info, 0)? {
        Key::Uninitialized => {
            account_info.assign(&system_program::ID);

            (account_info.lamports(), 0)
        }
        Key::Asset => {
            let (asset, header, registry) = fetch_core_data::<Asset>(account_info)?;
            let header_size = match header {
                Some(header) => header.get_size(),
                None => 0,
            };

            let registry_size = match registry {
                Some(registry) => registry.get_size(),
                None => 0,
            };

            // TODO overflow?
            let asset_rent = rent.minimum_balance(asset.get_size() + header_size + registry_size);
            let fee_amount = account_info
                .lamports()
                .checked_sub(asset_rent)
                .ok_or(MplCoreError::NumericalOverflowError)?;

            (fee_amount, asset_rent)
        }
        Key::HashedAsset => {
            // TODO use DataBlob trait instead?
            let hashed_rent = rent.minimum_balance(HashedAsset::LENGTH);
            let fee_amount = account_info
                .lamports()
                .checked_sub(hashed_rent)
                .ok_or(MplCoreError::NumericalOverflowError)?;

            (fee_amount, hashed_rent)
        }
        _ => return Err(MplCoreError::IncorrectAccount.into()),
    };

    let dest_starting_lamports = dest_info.lamports();
    **dest_info.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(fee_amount)
        .ok_or(MplCoreError::NumericalOverflowError)?;
    **account_info.lamports.borrow_mut() = rent_amount;

    Ok(())
}
