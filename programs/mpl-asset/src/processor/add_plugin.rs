use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    instruction::accounts::AddPluginAccounts,
    plugins::{create_meta_idempotent, initialize_plugin, Plugin},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct AddPluginArgs {
    plugin: Plugin,
}

pub(crate) fn add_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddPluginArgs,
) -> ProgramResult {
    let ctx = AddPluginAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    let payer = match ctx.accounts.payer {
        Some(payer) => {
            assert_signer(payer)?;
            payer
        }
        None => ctx.accounts.authority,
    };

    create_meta_idempotent(
        ctx.accounts.asset_address,
        payer,
        ctx.accounts.system_program,
    )?;

    initialize_plugin(
        &args.plugin,
        &args.plugin.default_authority()?,
        ctx.accounts.asset_address,
        payer,
        ctx.accounts.system_program,
    )?;

    Ok(())
}
