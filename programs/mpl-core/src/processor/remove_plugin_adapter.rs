use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::{
        RemoveCollectionPluginAdapterV1Accounts, RemovePluginAdapterV1Accounts,
    },
    plugins::{
        delete_plugin_adapter, fetch_wrapped_plugin_adapter, Plugin, PluginAdapterKey, PluginType,
    },
    state::{AssetV1, CollectionV1, DataBlob, Key},
    utils::{
        fetch_core_data, load_key, resolve_authority, validate_asset_permissions,
        validate_collection_permissions,
    },
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RemovePluginAdapterV1Args {
    /// Plugin adapter key.
    pub key: PluginAdapterKey,
}

pub(crate) fn remove_plugin_adapter<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RemovePluginAdapterV1Args,
) -> ProgramResult {
    let ctx = RemovePluginAdapterV1Accounts::context(accounts)?;

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

    let (asset, plugin_header, plugin_registry) = fetch_core_data::<AssetV1>(ctx.accounts.asset)?;

    // We don't have anything to delete if there's no plugin meta.
    if plugin_header.is_none() || plugin_registry.is_none() {
        return Err(MplCoreError::PluginNotFound.into());
    }

    let (_, plugin_to_remove) =
        fetch_wrapped_plugin_adapter::<AssetV1>(ctx.accounts.asset, Some(&asset), &args.key)?;

    // Validate asset permissions.
    let _ = validate_asset_permissions(
        accounts,
        authority,
        ctx.accounts.asset,
        ctx.accounts.collection,
        None,
        None,
        Some(&plugin_to_remove),
        AssetV1::check_remove_plugin_adapter,
        CollectionV1::check_remove_plugin_adapter,
        PluginType::check_remove_plugin_adapter,
        AssetV1::validate_remove_plugin_adapter,
        CollectionV1::validate_remove_plugin_adapter,
        Plugin::validate_remove_plugin_adapter,
        None,
        None,
    )?;

    process_remove_plugin_adapter(
        &args.key,
        &asset,
        ctx.accounts.asset,
        ctx.accounts.payer,
        ctx.accounts.system_program,
    )
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RemoveCollectionPluginAdapterV1Args {
    /// Plugin adapter key.
    pub key: PluginAdapterKey,
}

pub(crate) fn remove_collection_plugin_adapter<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RemoveCollectionPluginAdapterV1Args,
) -> ProgramResult {
    let ctx = RemoveCollectionPluginAdapterV1Accounts::context(accounts)?;

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

    let (_, plugin_to_remove) = fetch_wrapped_plugin_adapter::<CollectionV1>(
        ctx.accounts.collection,
        Some(&collection),
        &args.key,
    )?;

    // Validate asset permissions.
    let _ = validate_collection_permissions(
        accounts,
        authority,
        ctx.accounts.collection,
        None,
        Some(&plugin_to_remove),
        CollectionV1::check_remove_plugin_adapter,
        PluginType::check_remove_plugin_adapter,
        CollectionV1::validate_remove_plugin_adapter,
        Plugin::validate_remove_plugin_adapter,
        None,
        None,
    )?;

    process_remove_plugin_adapter(
        &args.key,
        &collection,
        ctx.accounts.collection,
        ctx.accounts.payer,
        ctx.accounts.system_program,
    )
}

fn process_remove_plugin_adapter<'a, T: DataBlob>(
    plugin_key: &PluginAdapterKey,
    core: &T,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    delete_plugin_adapter(plugin_key, core, account, payer, system_program)
}
