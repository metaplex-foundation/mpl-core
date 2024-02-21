use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::instruction::accounts::RemoveAuthorityAccounts;

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct RemoveAuthorityArgs {}

pub(crate) fn remove_authority<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RemoveAuthorityArgs,
) -> ProgramResult {
    let ctx = RemoveAuthorityAccounts::context(accounts)?;
    Ok(())
}
