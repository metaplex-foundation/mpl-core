use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_memory::sol_memcpy,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::{
        WriteCollectionExternalPluginAdapterDataV1Accounts,
        WriteExternalPluginAdapterDataV1Accounts,
    },
    plugins::{
        fetch_wrapped_external_plugin_adapter, DataStore, ExternalPluginAdapter,
        ExternalPluginAdapterKey, LifecycleHook, PluginHeaderV1, PluginRegistryV1,
    },
    state::{AssetV1, Authority, CollectionV1, DataBlob, Key, SolanaAccount},
    utils::{
        fetch_core_data, load_key, resize_or_reallocate_account, resolve_authority,
        resolve_pubkey_to_authorities, resolve_pubkey_to_authorities_collection,
    },
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct WriteExternalPluginAdapterDataV1Args {
    /// External plugin adapter key.
    pub key: ExternalPluginAdapterKey,
    /// The data to write.
    pub data: Option<Vec<u8>>,
}

pub(crate) fn write_external_plugin_adapter_data<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: WriteExternalPluginAdapterDataV1Args,
) -> ProgramResult {
    let ctx = WriteExternalPluginAdapterDataV1Accounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.payer)?;
    let authority = resolve_authority(ctx.accounts.payer, ctx.accounts.authority)?;

    let (asset, header, registry) = fetch_core_data::<AssetV1>(ctx.accounts.asset)?;
    let mut header = header.ok_or(MplCoreError::PluginsNotInitialized)?;
    let mut registry = registry.ok_or(MplCoreError::PluginsNotInitialized)?;

    let authorities = resolve_pubkey_to_authorities(authority, ctx.accounts.collection, &asset)?;

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

    process_write_external_plugin_data::<AssetV1>(
        ctx.accounts.asset,
        ctx.accounts.payer,
        ctx.accounts.system_program,
        ctx.accounts.buffer,
        args.data.as_deref(),
        &asset,
        &mut header,
        &mut registry,
        &authorities,
        &args.key,
    )?;

    let (_asset, _header, registry) = fetch_core_data::<AssetV1>(ctx.accounts.asset)?;

    if let Some(registry) = registry {
        msg!("Registry: {:?}", registry);
    }
    Ok(())
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct WriteCollectionExternalPluginAdapterDataV1Args {
    /// External plugin adapter key.
    pub key: ExternalPluginAdapterKey,
    /// The data to write.
    pub data: Option<Vec<u8>>,
}

pub(crate) fn write_collection_external_plugin_adapter_data<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: WriteCollectionExternalPluginAdapterDataV1Args,
) -> ProgramResult {
    let ctx = WriteCollectionExternalPluginAdapterDataV1Accounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.payer)?;
    let authority = resolve_authority(ctx.accounts.payer, ctx.accounts.authority)?;

    let (collection, header, registry) = fetch_core_data::<CollectionV1>(ctx.accounts.collection)?;
    let mut header = header.ok_or(MplCoreError::PluginsNotInitialized)?;
    let mut registry = registry.ok_or(MplCoreError::PluginsNotInitialized)?;

    let authorities = resolve_pubkey_to_authorities_collection(authority, ctx.accounts.collection)?;

    if ctx.accounts.system_program.key != &solana_program::system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    if let Some(log_wrapper) = ctx.accounts.log_wrapper {
        if log_wrapper.key != &spl_noop::ID {
            return Err(MplCoreError::InvalidLogWrapperProgram.into());
        }
    }

    process_write_external_plugin_data::<CollectionV1>(
        ctx.accounts.collection,
        ctx.accounts.payer,
        ctx.accounts.system_program,
        ctx.accounts.buffer,
        args.data.as_deref(),
        &collection,
        &mut header,
        &mut registry,
        &authorities,
        &args.key,
    )
}

#[allow(clippy::too_many_arguments)]
fn process_write_external_plugin_data<'a, T: DataBlob + SolanaAccount>(
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    buffer: Option<&AccountInfo<'a>>,
    data: Option<&[u8]>,
    core: &T,
    header: &mut PluginHeaderV1,
    registry: &mut PluginRegistryV1,
    authorities: &[Authority],
    key: &ExternalPluginAdapterKey,
) -> ProgramResult {
    let (record, plugin) = fetch_wrapped_external_plugin_adapter::<T>(account, None, key)?;
    match plugin {
        ExternalPluginAdapter::LifecycleHook(LifecycleHook {
            data_authority: Some(data_authority),
            ..
        })
        | ExternalPluginAdapter::DataStore(DataStore { data_authority, .. }) => {
            if !authorities.contains(&data_authority) {
                return Err(MplCoreError::InvalidAuthority.into());
            }
        }
        ExternalPluginAdapter::Oracle(_) => todo!(),
        _ => return Err(MplCoreError::UnsupportedOperation.into()),
    }

    let data_offset = record
        .data_offset
        .ok_or(MplCoreError::InvalidPluginSetting)? as isize;
    let data_len = record.data_len.ok_or(MplCoreError::InvalidPluginSetting)? as isize;
    let data_to_move = data_offset
        .checked_add(data_len)
        .ok_or(MplCoreError::NumericalOverflow)?;

    let record_mut = registry
        .external_registry
        .iter_mut()
        .find(|r| **r == record)
        .ok_or(MplCoreError::PluginNotFound)?;

    match (buffer, data) {
        (Some(buffer), None) => {
            let size_diff = (buffer.data_len() as isize)
                .checked_sub(data_len as isize)
                .ok_or(MplCoreError::NumericalOverflow)?;

            record_mut.data_len = Some(buffer.data_len());

            move_plugins_and_registry(
                core.get_size(),
                size_diff,
                data_offset as usize,
                data_to_move as usize,
                header,
                registry,
                account,
                payer,
                system_program,
            )?;

            sol_memcpy(
                &mut account.data.borrow_mut()[(data_offset as usize)..],
                &buffer.data.borrow(),
                buffer.data_len(),
            );

            Ok(())
        }
        (None, Some(data)) => {
            let size_diff = (data.len() as isize)
                .checked_sub(data_len as isize)
                .ok_or(MplCoreError::NumericalOverflow)?;

            // solana_program::msg!("Registry: {:?}", registry.external_registry);

            record_mut.data_len = Some(data.len());

            move_plugins_and_registry(
                core.get_size(),
                size_diff,
                data_offset as usize,
                data_to_move as usize,
                header,
                registry,
                account,
                payer,
                system_program,
            )?;

            solana_program::msg!("Registry: {:?}", registry);

            msg!("Copying {:?} to {:?}", data, data_offset);
            sol_memcpy(
                &mut account.data.borrow_mut()[(data_offset as usize)..],
                data,
                data.len(),
            );

            Ok(())
        }
        _ => Err(MplCoreError::TwoDataSources.into()),
    }
}

// TODO: Copied until #113 is complete.
#[allow(clippy::too_many_arguments)]
fn move_plugins_and_registry<'a>(
    header_location: usize,
    size_diff: isize,
    first_plugin_location: usize,
    data_to_move_location: usize,
    plugin_header: &mut PluginHeaderV1,
    plugin_registry: &mut PluginRegistryV1,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    // The new size of the account.
    let new_size = (account.data_len() as isize)
        .checked_add(size_diff)
        .ok_or(MplCoreError::NumericalOverflow)?;

    // The new offset of the plugin registry is the old offset plus the size difference.
    let registry_offset = plugin_header.plugin_registry_offset;
    let new_registry_offset = (registry_offset as isize)
        .checked_add(size_diff)
        .ok_or(MplCoreError::NumericalOverflow)?;
    plugin_header.plugin_registry_offset = new_registry_offset as usize;

    let new_data_location = (data_to_move_location as isize)
        .checked_add(size_diff)
        .ok_or(MplCoreError::NumericalOverflow)?;

    // //TODO: This is memory intensive, we should use memmove instead probably.
    let src = account.data.borrow()[(data_to_move_location)..registry_offset].to_vec();

    resize_or_reallocate_account(account, payer, system_program, new_size as usize)?;

    sol_memcpy(
        &mut account.data.borrow_mut()[(new_data_location as usize)..],
        &src,
        src.len(),
    );

    plugin_header.save(account, header_location)?;

    // Move offsets for existing registry records.
    for record in &mut plugin_registry.external_registry {
        if first_plugin_location == data_to_move_location || first_plugin_location < record.offset {
            let new_offset = (record.offset as isize)
                .checked_add(size_diff)
                .ok_or(MplCoreError::NumericalOverflow)?;

            record.offset = new_offset as usize;
        }
    }

    for record in &mut plugin_registry.registry {
        if first_plugin_location == data_to_move_location || first_plugin_location < record.offset {
            let new_offset = (record.offset as isize)
                .checked_add(size_diff)
                .ok_or(MplCoreError::NumericalOverflow)?;

            record.offset = new_offset as usize;
        }
    }

    plugin_registry.save(account, new_registry_offset as usize)
}
