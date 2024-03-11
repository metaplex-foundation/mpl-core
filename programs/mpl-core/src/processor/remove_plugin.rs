use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::{RemoveCollectionPluginAccounts, RemovePluginAccounts},
    plugins::{delete_plugin, fetch_wrapped_plugin, Plugin, PluginType},
    state::{Asset, Authority, Collection, DataBlob, Key},
    utils::{
        fetch_core_data, load_key, resolve_payer, resolve_to_authority, validate_asset_permissions,
        validate_collection_permissions,
    },
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

    if let Key::HashedAsset = load_key(ctx.accounts.asset, 0)? {
        msg!("Error: Remove plugin for compressed is not available");
        return Err(MplCoreError::NotAvailable.into());
    }

    let (mut asset, plugin_header, plugin_registry) = fetch_core_data::<Asset>(ctx.accounts.asset)?;

    // We don't have anything to delete if there's no plugin meta.
    if plugin_header.is_none() || plugin_registry.is_none() {
        return Err(MplCoreError::PluginNotFound.into());
    }

    //TODO: Make this better.
    let authority_type =
        resolve_to_authority(ctx.accounts.authority, ctx.accounts.collection, &asset)?;

    let (_, plugin_to_remove) =
        fetch_wrapped_plugin::<Asset>(ctx.accounts.asset, args.plugin_type)?;

    // Validate asset permissions.
    let _ = validate_asset_permissions(
        ctx.accounts.authority,
        ctx.accounts.asset,
        ctx.accounts.collection,
        None,
        Some(&plugin_to_remove),
        Asset::check_add_plugin,
        Collection::check_add_plugin,
        PluginType::check_add_plugin,
        Asset::validate_add_plugin,
        Collection::validate_add_plugin,
        Plugin::validate_add_plugin,
    )?;

    // Increment sequence number and save only if it is `Some(_)`.
    asset.increment_seq_and_save(ctx.accounts.asset)?;

    process_remove_plugin(
        &args.plugin_type,
        &asset,
        &authority_type,
        ctx.accounts.asset,
        payer,
        ctx.accounts.system_program,
    )
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

    let (_, plugin_to_remove) =
        fetch_wrapped_plugin::<Collection>(ctx.accounts.collection, args.plugin_type)?;

    // Validate collection permissions.
    let _ = validate_collection_permissions(
        ctx.accounts.authority,
        ctx.accounts.collection,
        Some(&plugin_to_remove),
        Collection::check_add_plugin,
        PluginType::check_add_plugin,
        Collection::validate_add_plugin,
        Plugin::validate_add_plugin,
    )?;

    process_remove_plugin(
        &args.plugin_type,
        &collection,
        &Authority::UpdateAuthority,
        ctx.accounts.collection,
        payer,
        ctx.accounts.system_program,
    )
}

//TODO
fn process_remove_plugin<'a, T: DataBlob>(
    plugin_type: &PluginType,
    core: &T,
    authority_type: &Authority,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    delete_plugin(
        plugin_type,
        core,
        authority_type,
        account,
        payer,
        system_program,
    )
}
