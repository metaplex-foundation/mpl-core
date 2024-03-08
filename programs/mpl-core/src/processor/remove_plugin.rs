use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplCoreError,
    instruction::accounts::{RemoveCollectionPluginAccounts, RemovePluginAccounts},
    plugins::{delete_plugin, PluginType},
    state::{Asset, Authority, Collection},
    utils::{fetch_core_data, resolve_payer, resolve_to_authority},
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RemovePluginArgs {
    plugin_type: PluginType,
}

pub(crate) fn remove_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RemovePluginArgs,
) -> ProgramResult {
    let ctx = RemovePluginAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    let payer = resolve_payer(ctx.accounts.authority, ctx.accounts.payer)?;

    let (asset, plugin_header, plugin_registry) = fetch_core_data::<Asset>(ctx.accounts.asset)?;

    // We don't have anything to delete if there's no plugin meta.
    if plugin_header.is_none() || plugin_registry.is_none() {
        return Err(MplCoreError::PluginNotFound.into());
    }

    //TODO: Make this better.
    let authority_type =
        resolve_to_authority(ctx.accounts.authority, ctx.accounts.collection, &asset)?;

    delete_plugin(
        &args.plugin_type,
        &asset,
        &authority_type,
        ctx.accounts.asset,
        payer,
        ctx.accounts.system_program,
    )?;

    process_remove_plugin()
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RemoveCollectionPluginArgs {
    plugin_type: PluginType,
}

pub(crate) fn remove_collection_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RemoveCollectionPluginArgs,
) -> ProgramResult {
    let ctx = RemoveCollectionPluginAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.authority)?;
    let payer = resolve_payer(ctx.accounts.authority, ctx.accounts.payer)?;

    let (collection, plugin_header, plugin_registry) =
        fetch_core_data::<Collection>(ctx.accounts.collection)?;

    // We don't have anything to delete if there's no plugin meta.
    if plugin_header.is_none() || plugin_registry.is_none() {
        return Err(MplCoreError::PluginNotFound.into());
    }

    if ctx.accounts.authority.key != &collection.update_authority {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    delete_plugin(
        &args.plugin_type,
        &collection,
        &Authority::UpdateAuthority,
        ctx.accounts.collection,
        payer,
        ctx.accounts.system_program,
    )?;

    process_remove_plugin()
}

//TODO
fn process_remove_plugin() -> ProgramResult {
    Ok(())
}
