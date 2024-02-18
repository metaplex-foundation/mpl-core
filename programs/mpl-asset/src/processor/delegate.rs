use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{instruction::accounts::DelegateAccounts, plugins::create_idempotent};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct DelegateArgs {}

pub(crate) fn delegate<'a>(accounts: &'a [AccountInfo<'a>], args: DelegateArgs) -> ProgramResult {
    let ctx = DelegateAccounts::context(accounts)?;

    create_idempotent(
        ctx.accounts.asset_address,
        ctx.accounts.owner,
        ctx.accounts.system_program,
    )?;

    Ok(())
}
