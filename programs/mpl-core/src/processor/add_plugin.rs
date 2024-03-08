use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::{AddCollectionPluginAccounts, AddPluginAccounts},
    plugins::{create_meta_idempotent, initialize_plugin, Plugin},
    state::{Asset, Authority, Collection, DataBlob, Key, SolanaAccount},
    utils::{load_key, resolve_payer},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddPluginArgs {
    plugin: Plugin,
    init_authority: Option<Authority>,
}

pub(crate) fn add_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddPluginArgs,
) -> ProgramResult {
    let ctx = AddPluginAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    let payer = resolve_payer(ctx.accounts.authority, ctx.accounts.payer)?;

    if let Key::HashedAsset = load_key(ctx.accounts.asset, 0)? {
        msg!("Error: Adding plugin to compressed is not available");
        return Err(MplCoreError::NotAvailable.into());
    }

    process_add_plugin::<Asset>(
        ctx.accounts.asset,
        payer,
        ctx.accounts.system_program,
        &args.plugin,
        &args.init_authority.unwrap_or(args.plugin.manager()),
    )
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddCollectionPluginArgs {
    plugin: Plugin,
    init_authority: Option<Authority>,
}

pub(crate) fn add_collection_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddCollectionPluginArgs,
) -> ProgramResult {
    let ctx = AddCollectionPluginAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    let payer = resolve_payer(ctx.accounts.authority, ctx.accounts.payer)?;

    process_add_plugin::<Collection>(
        ctx.accounts.collection,
        payer,
        ctx.accounts.system_program,
        &args.plugin,
        &args.init_authority.unwrap_or(args.plugin.manager()),
    )
}

fn process_add_plugin<'a, T: DataBlob + SolanaAccount>(
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    plugin: &Plugin,
    authority: &Authority,
) -> ProgramResult {
    create_meta_idempotent::<T>(account, payer, system_program)?;
    initialize_plugin::<T>(plugin, authority, account, payer, system_program)
}
