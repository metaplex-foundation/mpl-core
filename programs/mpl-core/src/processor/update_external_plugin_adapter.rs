use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_memory::sol_memcpy,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::{
        UpdateCollectionExternalPluginAdapterV1Accounts, UpdateExternalPluginAdapterV1Accounts,
    },
    plugins::{
        fetch_wrapped_external_plugin_adapter, find_external_plugin_adapter_mut,
        ExternalPluginAdapter, ExternalPluginAdapterKey, ExternalPluginAdapterUpdateInfo,
        PluginHeaderV1, PluginRegistryV1, PluginValidationContext, ValidationResult,
    },
    state::{AssetV1, CollectionV1, DataBlob, Key, SolanaAccount},
    utils::{
        fetch_core_data, load_key, resize_or_reallocate_account, resolve_authority,
        resolve_pubkey_to_authorities, resolve_pubkey_to_authorities_collection,
    },
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct UpdateExternalPluginAdapterV1Args {
    /// External plugin adapter key.
    pub key: ExternalPluginAdapterKey,
    /// Plugin info to update.
    pub update_info: ExternalPluginAdapterUpdateInfo,
}

pub(crate) fn update_external_plugin_adapter<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: UpdateExternalPluginAdapterV1Args,
) -> ProgramResult {
    // Accounts.
    let ctx = UpdateExternalPluginAdapterV1Accounts::context(accounts)?;

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

    let (mut asset, plugin_header, plugin_registry) =
        fetch_core_data::<AssetV1>(ctx.accounts.asset)?;
    let resolved_authorities =
        resolve_pubkey_to_authorities(authority, ctx.accounts.collection, &asset)?;
    let (external_registry_record, external_plugin_adapter) =
        fetch_wrapped_external_plugin_adapter::<AssetV1>(ctx.accounts.asset, None, &args.key)?;

    let validation_ctx = PluginValidationContext {
        accounts,
        asset_info: Some(ctx.accounts.asset),
        collection_info: ctx.accounts.collection,
        self_authority: &external_registry_record.authority,
        authority_info: authority,
        resolved_authorities: Some(&resolved_authorities),
        new_owner: None,
        new_asset_authority: None,
        new_collection_authority: None,
        target_plugin: None,
    };

    if ExternalPluginAdapter::validate_update_external_plugin_adapter(
        &external_plugin_adapter,
        &validation_ctx,
    )? != ValidationResult::Approved
    {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // Increment sequence number and save only if it is `Some(_)`.
    asset.increment_seq_and_save(ctx.accounts.asset)?;

    process_update_external_plugin_adapter(
        asset,
        external_plugin_adapter,
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
pub(crate) struct UpdateCollectionExternalPluginAdapterV1Args {
    /// External plugin adapter key.
    pub key: ExternalPluginAdapterKey,
    /// Plugin info to update.
    pub update_info: ExternalPluginAdapterUpdateInfo,
}

pub(crate) fn update_collection_external_plugin_adapter<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: UpdateCollectionExternalPluginAdapterV1Args,
) -> ProgramResult {
    // Accounts.
    let ctx = UpdateCollectionExternalPluginAdapterV1Accounts::context(accounts)?;

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
    let resolved_authorities =
        resolve_pubkey_to_authorities_collection(authority, ctx.accounts.collection)?;
    let (external_registry_record, external_plugin_adapter) =
        fetch_wrapped_external_plugin_adapter::<CollectionV1>(
            ctx.accounts.collection,
            None,
            &args.key,
        )?;

    let validation_ctx = PluginValidationContext {
        accounts,
        asset_info: None,
        collection_info: Some(ctx.accounts.collection),
        self_authority: &external_registry_record.authority,
        authority_info: authority,
        resolved_authorities: Some(&resolved_authorities),
        new_owner: None,
        new_asset_authority: None,
        new_collection_authority: None,
        target_plugin: None,
    };

    if ExternalPluginAdapter::validate_update_external_plugin_adapter(
        &external_plugin_adapter,
        &validation_ctx,
    )? != ValidationResult::Approved
    {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    process_update_external_plugin_adapter(
        collection,
        external_plugin_adapter,
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
fn process_update_external_plugin_adapter<'a, T: DataBlob + SolanaAccount>(
    core: T,
    plugin: ExternalPluginAdapter,
    key: ExternalPluginAdapterKey,
    update_info: ExternalPluginAdapterUpdateInfo,
    plugin_header: Option<PluginHeaderV1>,
    plugin_registry: Option<PluginRegistryV1>,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    let mut plugin_registry = plugin_registry.ok_or(MplCoreError::PluginsNotInitialized)?;
    let mut plugin_header = plugin_header.ok_or(MplCoreError::PluginsNotInitialized)?;

    // Update the registry record using a mutable reference that ties back to `plugin_registry`.
    let (_, record) = find_external_plugin_adapter_mut(&mut plugin_registry, &key, account)?;
    let registry_record = record.ok_or(MplCoreError::PluginNotFound)?;
    let old_registry_record_size = registry_record.try_to_vec()?.len() as isize;

    registry_record.update(&update_info)?;
    let new_registry_record_size = registry_record.try_to_vec()?.len() as isize;
    let registry_record_size_diff = new_registry_record_size
        .checked_sub(old_registry_record_size)
        .ok_or(MplCoreError::NumericalOverflow)?;

    // Clone into a new copy, dropping the mutable reference so that `plugin_registry` can be mutably borrowed again later.
    let registry_record = registry_record.clone();

    let mut new_plugin = plugin.clone();
    new_plugin.update(&update_info);

    let plugin_data = plugin.try_to_vec()?;
    let new_plugin_data = new_plugin.try_to_vec()?;

    // The difference in size between the new and old account which is used to calculate the new size of the account.
    let plugin_size = plugin_data.len() as isize;
    let plugin_size_diff = (new_plugin_data.len() as isize)
        .checked_sub(plugin_size)
        .ok_or(MplCoreError::NumericalOverflow)?;

    // The new size of the account.
    let new_size = (account.data_len() as isize)
        .checked_add(plugin_size_diff)
        .ok_or(MplCoreError::NumericalOverflow)?
        .checked_add(registry_record_size_diff)
        .ok_or(MplCoreError::NumericalOverflow)?;

    // The new offset of the plugin registry is the old offset plus the size difference.
    let registry_offset = plugin_header.plugin_registry_offset;
    let new_registry_offset = (registry_offset as isize)
        .checked_add(plugin_size_diff)
        .ok_or(MplCoreError::NumericalOverflow)?;
    plugin_header.plugin_registry_offset = new_registry_offset as usize;

    // The offset of the first plugin is the plugin offset plus the size of the plugin.
    let next_plugin_offset = (registry_record.offset as isize)
        .checked_add(plugin_size)
        .ok_or(MplCoreError::NumericalOverflow)?;

    let new_next_plugin_offset = next_plugin_offset
        .checked_add(plugin_size_diff)
        .ok_or(MplCoreError::NumericalOverflow)?;

    // //TODO: This is memory intensive, we should use memmove instead probably.
    let src = account.data.borrow()[(next_plugin_offset as usize)..registry_offset].to_vec();

    resize_or_reallocate_account(account, payer, system_program, new_size as usize)?;

    sol_memcpy(
        &mut account.data.borrow_mut()[(new_next_plugin_offset as usize)..],
        &src,
        src.len(),
    );

    plugin_header.save(account, core.get_size())?;

    // Move offsets for existing registry records.
    plugin_registry.bump_offsets(registry_record.offset, plugin_size_diff)?;

    plugin_registry.save(account, new_registry_offset as usize)?;
    new_plugin.save(account, registry_record.offset)?;

    Ok(())
}
