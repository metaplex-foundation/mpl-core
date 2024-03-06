use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    instruction::accounts::{AddCollectionPluginAccounts, AddPluginAccounts},
    plugins::{create_meta_idempotent, initialize_plugin, Plugin},
    state::{Asset, Collection},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddPluginArgs {
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

    let _default_auth = args.plugin.default_authority()?;

    create_meta_idempotent(ctx.accounts.asset, payer, ctx.accounts.system_program)?;

    initialize_plugin::<Asset>(
        &args.plugin,
        &[args.plugin.default_authority()?],
        ctx.accounts.asset,
        payer,
        ctx.accounts.system_program,
    )?;

    process_add_plugin()
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddCollectionPluginArgs {
    plugin: Plugin,
}

pub(crate) fn add_collection_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddCollectionPluginArgs,
) -> ProgramResult {
    let ctx = AddCollectionPluginAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    let payer = match ctx.accounts.payer {
        Some(payer) => {
            assert_signer(payer)?;
            payer
        }
        None => ctx.accounts.authority,
    };

    let _default_auth = args.plugin.default_authority()?;

    create_meta_idempotent(ctx.accounts.collection, payer, ctx.accounts.system_program)?;

    initialize_plugin::<Collection>(
        &args.plugin,
        &[args.plugin.default_authority()?],
        ctx.accounts.collection,
        payer,
        ctx.accounts.system_program,
    )?;

    process_add_plugin()
}

//TODO
fn process_add_plugin() -> ProgramResult {
    Ok(())
}
