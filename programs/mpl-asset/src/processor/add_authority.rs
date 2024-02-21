use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{instruction::accounts::AddAuthorityAccounts, utils::fetch_core_data};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct AddAuthorityArgs {}

pub(crate) fn add_authority<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddAuthorityArgs,
) -> ProgramResult {
    let ctx = AddAuthorityAccounts::context(accounts)?;
    fetch_core_data(ctx.accounts.asset_address)?;

    Ok(())
}
