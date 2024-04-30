use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_memory::sol_memcpy,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::{UpdateCollectionPluginV1Accounts, UpdatePluginV1Accounts},
    plugins::{Plugin, PluginType, RegistryRecord},
    state::{AssetV1, CollectionV1, DataBlob, Key, SolanaAccount},
    utils::{
        load_key, resize_or_reallocate_account, resolve_authority, validate_asset_permissions,
        validate_collection_permissions,
    },
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct UpdatePluginV1Args {
    pub plugin: Plugin,
}

pub(crate) fn update_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: UpdatePluginV1Args,
) -> ProgramResult {
    // Accounts.
    let ctx = UpdatePluginV1Accounts::context(accounts)?;

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

    let (mut asset, plugin_header, plugin_registry) = validate_asset_permissions(
        accounts,
        authority,
        ctx.accounts.asset,
        ctx.accounts.collection,
        None,
        Some(&args.plugin),
        None,
        AssetV1::check_update_plugin,
        CollectionV1::check_update_plugin,
        PluginType::check_update_plugin,
        AssetV1::validate_update_plugin,
        CollectionV1::validate_update_plugin,
        Plugin::validate_update_plugin,
        None,
        None,
    )?;

    let mut plugin_registry = plugin_registry.ok_or(MplCoreError::PluginsNotInitialized)?;
    let mut plugin_header = plugin_header.ok_or(MplCoreError::PluginsNotInitialized)?;

    let plugin_registry_clone = plugin_registry.clone();
    let plugin_type: PluginType = (&args.plugin).into();
    let registry_record = plugin_registry_clone
        .registry
        .iter()
        .find(|record| record.plugin_type == plugin_type)
        .ok_or(MplCoreError::PluginNotFound)?;

    let plugin = Plugin::load(ctx.accounts.asset, registry_record.offset)?;
    let plugin_data = plugin.try_to_vec()?;
    let new_plugin_data = args.plugin.try_to_vec()?;

    // The difference in size between the new and old account which is used to calculate the new size of the account.
    let plugin_size = plugin_data.len() as isize;
    let size_diff = (new_plugin_data.len() as isize)
        .checked_sub(plugin_size)
        .ok_or(MplCoreError::NumericalOverflow)?;

    // The new size of the account.
    let new_size = (ctx.accounts.asset.data_len() as isize)
        .checked_add(size_diff)
        .ok_or(MplCoreError::NumericalOverflow)?;

    // The new offset of the plugin registry is the old offset plus the size difference.
    let registry_offset = plugin_header.plugin_registry_offset;
    let new_registry_offset = (registry_offset as isize)
        .checked_add(size_diff)
        .ok_or(MplCoreError::NumericalOverflow)?;
    plugin_header.plugin_registry_offset = new_registry_offset as usize;

    // The offset of the first plugin is the plugin offset plus the size of the plugin.
    let next_plugin_offset = (registry_record.offset as isize)
        .checked_add(plugin_size)
        .ok_or(MplCoreError::NumericalOverflow)?;

    let new_next_plugin_offset = next_plugin_offset
        .checked_add(size_diff)
        .ok_or(MplCoreError::NumericalOverflow)?;

    // //TODO: This is memory intensive, we should use memmove instead probably.
    let src =
        ctx.accounts.asset.data.borrow()[(next_plugin_offset as usize)..registry_offset].to_vec();

    resize_or_reallocate_account(
        ctx.accounts.asset,
        ctx.accounts.payer,
        ctx.accounts.system_program,
        new_size as usize,
    )?;

    sol_memcpy(
        &mut ctx.accounts.asset.data.borrow_mut()[(new_next_plugin_offset as usize)..],
        &src,
        src.len(),
    );

    plugin_header.save(ctx.accounts.asset, asset.get_size())?;
    plugin_registry.registry = plugin_registry
        .registry
        .clone()
        .iter_mut()
        .map(|record| {
            let new_offset = if record.offset > registry_record.offset {
                (record.offset as isize)
                    .checked_add(size_diff)
                    .ok_or(MplCoreError::NumericalOverflow)?
            } else {
                record.offset as isize
            };
            Ok(RegistryRecord {
                plugin_type: record.plugin_type,
                offset: new_offset as usize,
                authority: record.authority,
            })
        })
        .collect::<Result<Vec<_>, MplCoreError>>()?;
    plugin_registry.save(ctx.accounts.asset, new_registry_offset as usize)?;
    args.plugin
        .save(ctx.accounts.asset, registry_record.offset)?;

    // Increment sequence number and save only if it is `Some(_)`.
    asset.increment_seq_and_save(ctx.accounts.asset)?;

    process_update_plugin()
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct UpdateCollectionPluginV1Args {
    pub plugin: Plugin,
}

pub(crate) fn update_collection_plugin<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: UpdateCollectionPluginV1Args,
) -> ProgramResult {
    // Accounts.
    let ctx = UpdateCollectionPluginV1Accounts::context(accounts)?;

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

    // Validate collection permissions.
    let (collection, plugin_header, plugin_registry) = validate_collection_permissions(
        accounts,
        authority,
        ctx.accounts.collection,
        Some(&args.plugin),
        None,
        CollectionV1::check_update_plugin,
        PluginType::check_update_plugin,
        CollectionV1::validate_update_plugin,
        Plugin::validate_update_plugin,
        None,
        None,
    )?;

    // let (collection, plugin_header, plugin_registry) =
    //     fetch_core_data::<CollectionV1>(ctx.accounts.collection)?;
    let mut plugin_registry = plugin_registry.ok_or(MplCoreError::PluginsNotInitialized)?;
    let mut plugin_header = plugin_header.ok_or(MplCoreError::PluginsNotInitialized)?;

    let plugin_registry_clone = plugin_registry.clone();
    let plugin_type: PluginType = (&args.plugin).into();
    let registry_record = plugin_registry_clone
        .registry
        .iter()
        .find(|record| record.plugin_type == plugin_type)
        .ok_or(MplCoreError::PluginNotFound)?;

    let plugin = Plugin::load(ctx.accounts.collection, registry_record.offset)?;
    let plugin_data = plugin.try_to_vec()?;
    let new_plugin_data = args.plugin.try_to_vec()?;

    // The difference in size between the new and old account which is used to calculate the new size of the account.
    let plugin_size = plugin_data.len() as isize;
    let size_diff = (new_plugin_data.len() as isize)
        .checked_sub(plugin_size)
        .ok_or(MplCoreError::NumericalOverflow)?;

    // The new size of the account.
    let new_size = (ctx.accounts.collection.data_len() as isize)
        .checked_add(size_diff)
        .ok_or(MplCoreError::NumericalOverflow)?;

    // The new offset of the plugin registry is the old offset plus the size difference.
    let registry_offset = plugin_header.plugin_registry_offset;
    let new_registry_offset = (registry_offset as isize)
        .checked_add(size_diff)
        .ok_or(MplCoreError::NumericalOverflow)?;
    plugin_header.plugin_registry_offset = new_registry_offset as usize;

    // The offset of the first plugin is the plugin offset plus the size of the plugin.
    let next_plugin_offset = (registry_record.offset as isize)
        .checked_add(plugin_size)
        .ok_or(MplCoreError::NumericalOverflow)?;

    let new_next_plugin_offset = next_plugin_offset
        .checked_add(size_diff)
        .ok_or(MplCoreError::NumericalOverflow)?;

    // //TODO: This is memory intensive, we should use memmove instead probably.
    let src = ctx.accounts.collection.data.borrow()[(next_plugin_offset as usize)..registry_offset]
        .to_vec();

    resize_or_reallocate_account(
        ctx.accounts.collection,
        ctx.accounts.payer,
        ctx.accounts.system_program,
        new_size as usize,
    )?;

    sol_memcpy(
        &mut ctx.accounts.collection.data.borrow_mut()[(new_next_plugin_offset as usize)..],
        &src,
        src.len(),
    );

    plugin_header.save(ctx.accounts.collection, collection.get_size())?;
    plugin_registry.registry = plugin_registry
        .registry
        .clone()
        .iter_mut()
        .map(|record| {
            let new_offset = if record.offset > registry_record.offset {
                (record.offset as isize)
                    .checked_add(size_diff)
                    .ok_or(MplCoreError::NumericalOverflow)?
            } else {
                record.offset as isize
            };
            Ok(RegistryRecord {
                plugin_type: record.plugin_type,
                offset: new_offset as usize,
                authority: record.authority,
            })
        })
        .collect::<Result<Vec<_>, MplCoreError>>()?;
    plugin_registry.save(ctx.accounts.collection, new_registry_offset as usize)?;
    args.plugin
        .save(ctx.accounts.collection, registry_record.offset)?;

    process_update_plugin()
}

//TODO
fn process_update_plugin() -> ProgramResult {
    Ok(())
}
