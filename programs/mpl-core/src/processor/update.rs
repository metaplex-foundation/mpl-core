use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_memory::sol_memcpy,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::{UpdateAccounts, UpdateCollectionAccounts},
    plugins::{Plugin, PluginHeader, PluginRegistry, PluginType, RegistryRecord},
    state::{Asset, Collection, DataBlob, Key, SolanaAccount, UpdateAuthority},
    utils::{
        load_key, resize_or_reallocate_account, resolve_authority, validate_asset_permissions,
        validate_collection_permissions,
    },
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct UpdateArgs {
    pub new_name: Option<String>,
    pub new_uri: Option<String>,
}

pub(crate) fn update<'a>(accounts: &'a [AccountInfo<'a>], args: UpdateArgs) -> ProgramResult {
    // Accounts.
    let ctx = UpdateAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.payer)?;
    let authority = resolve_authority(ctx.accounts.payer, ctx.accounts.authority)?;

    if let Key::HashedAsset = load_key(ctx.accounts.asset, 0)? {
        msg!("Error: Update for compressed is not available");
        return Err(MplCoreError::NotAvailable.into());
    }

    let (mut asset, plugin_header, plugin_registry) = validate_asset_permissions(
        authority,
        ctx.accounts.asset,
        ctx.accounts.collection,
        None,
        None,
        Asset::check_update,
        Collection::check_update,
        PluginType::check_update,
        Asset::validate_update,
        Collection::validate_update,
        Plugin::validate_update,
    )?;

    // Increment sequence number and save only if it is `Some(_)`.
    asset.increment_seq_and_save(ctx.accounts.asset)?;

    let asset_size = asset.get_size() as isize;

    let mut dirty = false;
    if let Some(new_update_authority) = ctx.accounts.new_update_authority {
        asset.update_authority = UpdateAuthority::Address(*new_update_authority.key);
        dirty = true;
    }
    if let Some(new_name) = &args.new_name {
        asset.name = new_name.clone();
        dirty = true;
    }
    if let Some(new_uri) = &args.new_uri {
        asset.uri = new_uri.clone();
        dirty = true;
    }
    if dirty {
        process_update(
            asset,
            &plugin_header,
            &plugin_registry,
            asset_size,
            ctx.accounts.asset,
            ctx.accounts.payer,
            ctx.accounts.system_program,
        )?;
    }

    Ok(())
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct UpdateCollectionArgs {
    pub new_name: Option<String>,
    pub new_uri: Option<String>,
}

pub(crate) fn update_collection<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: UpdateCollectionArgs,
) -> ProgramResult {
    // Accounts.
    let ctx = UpdateCollectionAccounts::context(accounts)?;

    // Guards.
    assert_signer(ctx.accounts.payer)?;
    let authority = resolve_authority(ctx.accounts.payer, ctx.accounts.authority)?;

    let (mut collection, plugin_header, plugin_registry) = validate_collection_permissions(
        authority,
        ctx.accounts.collection,
        None,
        Collection::check_update,
        PluginType::check_update,
        Collection::validate_update,
        Plugin::validate_update,
    )?;

    let collection_size = collection.get_size() as isize;

    let mut dirty = false;
    if let Some(new_update_authority) = ctx.accounts.new_update_authority {
        collection.update_authority = *new_update_authority.key;
        dirty = true;
    }
    if let Some(new_name) = &args.new_name {
        collection.name = new_name.clone();
        dirty = true;
    }
    if let Some(new_uri) = &args.new_uri {
        collection.uri = new_uri.clone();
        dirty = true;
    }
    if dirty {
        process_update(
            collection,
            &plugin_header,
            &plugin_registry,
            collection_size,
            ctx.accounts.collection,
            ctx.accounts.payer,
            ctx.accounts.system_program,
        )?;
    }

    Ok(())
}

fn process_update<'a, T: DataBlob + SolanaAccount>(
    core: T,
    plugin_header: &Option<PluginHeader>,
    plugin_registry: &Option<PluginRegistry>,
    core_size: isize,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    if let (Some(mut plugin_header), Some(mut plugin_registry)) =
        (plugin_header.clone(), plugin_registry.clone())
    {
        // The new size of the asset and new offset of the plugin header.
        let new_core_size = core.get_size() as isize;

        // The difference in size between the new and old asset which is used to calculate the new size of the account.
        let size_diff = new_core_size
            .checked_sub(core_size)
            .ok_or(MplCoreError::NumericalOverflow)?;

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

        // The offset of the first plugin is the core size plus the size of the plugin header.
        let plugin_offset = core_size
            .checked_add(plugin_header.get_size() as isize)
            .ok_or(MplCoreError::NumericalOverflow)?;

        let new_plugin_offset = plugin_offset
            .checked_add(size_diff)
            .ok_or(MplCoreError::NumericalOverflow)?;

        // //TODO: This is memory intensive, we should use memmove instead probably.
        let src = account.data.borrow()[(plugin_offset as usize)..registry_offset].to_vec();

        resize_or_reallocate_account(account, payer, system_program, new_size as usize)?;

        sol_memcpy(
            &mut account.data.borrow_mut()[(new_plugin_offset as usize)..],
            &src,
            src.len(),
        );

        plugin_header.save(account, new_core_size as usize)?;
        plugin_registry.registry = plugin_registry
            .registry
            .iter_mut()
            .map(|record| {
                let new_offset = (record.offset as isize)
                    .checked_add(size_diff)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                Ok(RegistryRecord {
                    plugin_type: record.plugin_type,
                    offset: new_offset as usize,
                    authority: record.authority,
                })
            })
            .collect::<Result<Vec<_>, MplCoreError>>()?;
        plugin_registry.save(account, new_registry_offset as usize)?;
    } else {
        resize_or_reallocate_account(account, payer, system_program, core.get_size())?;
    }

    core.save(account, 0)?;

    Ok(())
}
