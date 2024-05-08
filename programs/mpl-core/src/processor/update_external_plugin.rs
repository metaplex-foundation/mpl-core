use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::{
        UpdateCollectionExternalPluginV1Accounts, UpdateExternalPluginV1Accounts,
    },
    plugins::{
        fetch_wrapped_external_plugin, find_external_plugin, ExternalPlugin, ExternalPluginKey,
        ExternalPluginUpdateInfo, Plugin, PluginHeaderV1, PluginRegistryV1, PluginType,
    },
    state::{AssetV1, CollectionV1, DataBlob, Key, PluginSolanaAccount, SolanaAccount},
    utils::{
        load_key, move_plugins_and_registry, resolve_authority, validate_asset_permissions,
        validate_collection_permissions,
    },
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct UpdateExternalPluginV1Args {
    /// External plugin key.
    pub key: ExternalPluginKey,
    /// Plugin info to update.
    pub update_info: ExternalPluginUpdateInfo,
}

pub(crate) fn update_external_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: UpdateExternalPluginV1Args,
) -> ProgramResult {
    // Accounts.
    let ctx = UpdateExternalPluginV1Accounts::context(accounts)?;

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
        msg!("Error: Update plugin for compressed is not available");
        return Err(MplCoreError::NotAvailable.into());
    }

    let (_, plugin) =
        fetch_wrapped_external_plugin::<AssetV1>(ctx.accounts.asset, None, &args.key)?;

    let (mut asset, plugin_header, plugin_registry) = validate_asset_permissions(
        accounts,
        authority,
        ctx.accounts.asset,
        ctx.accounts.collection,
        None,
        None,
        Some(&plugin),
        AssetV1::check_update_external_plugin,
        CollectionV1::check_update_external_plugin,
        PluginType::check_update_external_plugin,
        AssetV1::validate_update_external_plugin,
        CollectionV1::validate_update_external_plugin,
        Plugin::validate_update_external_plugin,
        None,
        None,
    )?;

    // Increment sequence number and save only if it is `Some(_)`.
    asset.increment_seq_and_save(ctx.accounts.asset)?;

    process_update_external_plugin(
        asset,
        plugin,
        args.key,
        args.update_info,
        plugin_header,
        plugin_registry,
        ctx.accounts.asset,
        ctx.accounts.payer,
        ctx.accounts.system_program,
    )
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct UpdateCollectionExternalPluginV1Args {
    /// External plugin key.
    pub key: ExternalPluginKey,
    /// Plugin info to update.
    pub update_info: ExternalPluginUpdateInfo,
}

pub(crate) fn update_collection_external_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: UpdateCollectionExternalPluginV1Args,
) -> ProgramResult {
    // Accounts.
    let ctx = UpdateCollectionExternalPluginV1Accounts::context(accounts)?;

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

    let (_, plugin) =
        fetch_wrapped_external_plugin::<CollectionV1>(ctx.accounts.collection, None, &args.key)?;

    // Validate collection permissions.
    let (collection, plugin_header, plugin_registry) = validate_collection_permissions(
        accounts,
        authority,
        ctx.accounts.collection,
        None,
        Some(&plugin),
        CollectionV1::check_update_plugin,
        PluginType::check_update_plugin,
        CollectionV1::validate_update_plugin,
        Plugin::validate_update_plugin,
        None,
        None,
    )?;

    process_update_external_plugin(
        collection,
        plugin,
        args.key,
        args.update_info,
        plugin_header,
        plugin_registry,
        ctx.accounts.collection,
        ctx.accounts.payer,
        ctx.accounts.system_program,
    )
}

#[allow(clippy::too_many_arguments)]
fn process_update_external_plugin<'a, T: DataBlob + SolanaAccount>(
    core: T,
    plugin: ExternalPlugin,
    key: ExternalPluginKey,
    update_info: ExternalPluginUpdateInfo,
    plugin_header: Option<PluginHeaderV1>,
    plugin_registry: Option<PluginRegistryV1>,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    let mut plugin_registry = plugin_registry.ok_or(MplCoreError::PluginsNotInitialized)?;
    let mut plugin_header = plugin_header.ok_or(MplCoreError::PluginsNotInitialized)?;

    let plugin_registry_clone = plugin_registry.clone();
    let (_, record) = find_external_plugin(&plugin_registry_clone, &key, account)?;
    let mut registry_record = record.ok_or(MplCoreError::PluginNotFound)?.clone();
    registry_record.update(&update_info)?;

    let mut new_plugin = plugin.clone();
    new_plugin.update(&update_info);

    let plugin_data = plugin.try_to_vec()?;
    let new_plugin_data = new_plugin.try_to_vec()?;

    // The difference in size between the new and old account which is used to calculate the new size of the account.
    let plugin_size = plugin_data.len();
    let size_diff = (new_plugin_data.len() as isize)
        .checked_sub(plugin_size as isize)
        .ok_or(MplCoreError::NumericalOverflow)?;

    let data_to_move_location = (registry_record.offset)
        .checked_add(plugin_size)
        .ok_or(MplCoreError::NumericalOverflow)?;

    move_plugins_and_registry(
        core.get_size(),
        size_diff,
        registry_record.offset,
        data_to_move_location,
        &mut plugin_header,
        &mut plugin_registry,
        account,
        payer,
        system_program,
    )?;

    new_plugin.save(account, registry_record.offset)
}
