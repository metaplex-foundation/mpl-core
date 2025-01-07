use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::{RemoveCollectionPluginV1Accounts, RemovePluginV1Accounts},
    plugins::{delete_plugin, fetch_wrapped_plugin, Plugin, PluginType},
    state::{AssetV1, CollectionV1, DataBlob, Key},
    utils::{
        fetch_core_data, load_key, resolve_authority, validate_asset_permissions,
        validate_collection_permissions,
    },
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RemovePluginV1Args {
    plugin_type: PluginType,
}

pub(crate) fn remove_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RemovePluginV1Args,
) -> ProgramResult {
    let ctx = RemovePluginV1Accounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.payer)?;
    let authority = resolve_authority(ctx.accounts.payer, ctx.accounts.authority)?;

    if ctx.accounts.system_program.key != &solana_program::system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    if let Some(log_wrapper) = ctx.accounts.log_wrapper {
        if log_wrapper.key != &spl_noop::ID {
            return Err(MplCoreError::InvalidLogWrapperProgram.into());
        }
    }

    if let Key::HashedAssetV1 = load_key(ctx.accounts.asset, 0)? {
        msg!("Error: Remove plugin for compressed is not available");
        return Err(MplCoreError::NotAvailable.into());
    }

    let (mut asset, plugin_header, plugin_registry) =
        fetch_core_data::<AssetV1>(ctx.accounts.asset)?;

    // We don't have anything to delete if there's no plugin meta.
    if plugin_header.is_none() || plugin_registry.is_none() {
        return Err(MplCoreError::PluginNotFound.into());
    }

    let (plugin_authority, plugin_to_remove) =
        fetch_wrapped_plugin::<AssetV1>(ctx.accounts.asset, Some(&asset), args.plugin_type)?;

    // Validate asset permissions.
    let _ = validate_asset_permissions(
        accounts,
        authority,
        ctx.accounts.asset,
        ctx.accounts.collection,
        None,
        None,
        Some(&plugin_to_remove),
        Some(&plugin_authority),
        None,
        None,
        AssetV1::check_remove_plugin,
        CollectionV1::check_remove_plugin,
        PluginType::check_remove_plugin,
        AssetV1::validate_remove_plugin,
        CollectionV1::validate_remove_plugin,
        Plugin::validate_remove_plugin,
        None,
        None,
    )?;

    // Increment sequence number and save only if it is `Some(_)`.
    asset.increment_seq_and_save(ctx.accounts.asset)?;

    process_remove_plugin(
        &args.plugin_type,
        &asset,
        ctx.accounts.asset,
        ctx.accounts.payer,
        ctx.accounts.system_program,
    )
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RemoveCollectionPluginV1Args {
    plugin_type: PluginType,
}

pub(crate) fn remove_collection_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RemoveCollectionPluginV1Args,
) -> ProgramResult {
    let ctx = RemoveCollectionPluginV1Accounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.payer)?;
    let authority = resolve_authority(ctx.accounts.payer, ctx.accounts.authority)?;

    if ctx.accounts.system_program.key != &solana_program::system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    if let Some(log_wrapper) = ctx.accounts.log_wrapper {
        if log_wrapper.key != &spl_noop::ID {
            return Err(MplCoreError::InvalidLogWrapperProgram.into());
        }
    }

    let (collection, plugin_header, plugin_registry) =
        fetch_core_data::<CollectionV1>(ctx.accounts.collection)?;

    // We don't have anything to delete if there's no plugin meta.
    if plugin_header.is_none() || plugin_registry.is_none() {
        return Err(MplCoreError::PluginNotFound.into());
    }

    let (plugin_authority, plugin_to_remove) = fetch_wrapped_plugin::<CollectionV1>(
        ctx.accounts.collection,
        Some(&collection),
        args.plugin_type,
    )?;

    // Validate collection permissions.
    let _ = validate_collection_permissions(
        accounts,
        authority,
        ctx.accounts.collection,
        None,
        Some(&plugin_to_remove),
        Some(&plugin_authority),
        None,
        None,
        CollectionV1::check_remove_plugin,
        PluginType::check_remove_plugin,
        CollectionV1::validate_remove_plugin,
        Plugin::validate_remove_plugin,
        None,
        None,
    )?;

    process_remove_plugin(
        &args.plugin_type,
        &collection,
        ctx.accounts.collection,
        ctx.accounts.payer,
        ctx.accounts.system_program,
    )
}

//TODO
fn process_remove_plugin<'a, T: DataBlob>(
    plugin_type: &PluginType,
    core: &T,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    delete_plugin(plugin_type, core, account, payer, system_program)
}
