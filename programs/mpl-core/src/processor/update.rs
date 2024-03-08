use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_memory::sol_memcpy,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::{UpdateAccounts, UpdateCollectionAccounts},
    plugins::{Plugin, PluginType, RegistryRecord},
    state::{Asset, Collection, DataBlob, SolanaAccount, UpdateAuthority},
    utils::{
        resize_or_reallocate_account, resolve_payer, validate_asset_permissions,
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
    assert_signer(ctx.accounts.authority)?;
    let payer = resolve_payer(ctx.accounts.authority, ctx.accounts.payer)?;

    let (mut asset, plugin_header, plugin_registry) = validate_asset_permissions(
        ctx.accounts.authority,
        ctx.accounts.asset,
        ctx.accounts.collection,
        None,
        Asset::check_update,
        Collection::check_update,
        PluginType::check_update,
        Asset::validate_update,
        Collection::validate_update,
        Plugin::validate_update,
    )?;

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
        if let (Some(mut plugin_header), Some(mut plugin_registry)) =
            (plugin_header, plugin_registry)
        {
            let new_asset_size = asset.get_size() as isize;
            let size_diff = new_asset_size
                .checked_sub(asset_size)
                .ok_or(MplCoreError::NumericalOverflow)?;
            let new_size = (ctx.accounts.asset.data_len() as isize)
                .checked_add(size_diff)
                .ok_or(MplCoreError::NumericalOverflow)?;
            let new_registry_offset = (plugin_header.plugin_registry_offset as isize)
                .checked_add(size_diff)
                .ok_or(MplCoreError::NumericalOverflow)?;
            let registry_offset = plugin_header.plugin_registry_offset;
            plugin_header.plugin_registry_offset = new_registry_offset as usize;

            let plugin_offset = asset_size
                .checked_add(size_diff)
                .ok_or(MplCoreError::NumericalOverflow)?;
            let new_plugin_offset = new_asset_size
                .checked_add(size_diff)
                .ok_or(MplCoreError::NumericalOverflow)?;

            // //TODO: This is memory intensive, we should use memmove instead probably.
            let src = ctx.accounts.asset.data.borrow()[(plugin_offset as usize)..registry_offset]
                .to_vec();

            resize_or_reallocate_account(
                ctx.accounts.asset,
                payer,
                ctx.accounts.system_program,
                new_size as usize,
            )?;

            sol_memcpy(
                &mut ctx.accounts.asset.data.borrow_mut()[(new_plugin_offset as usize)..],
                &src,
                src.len(),
            );

            plugin_header.save(ctx.accounts.asset, new_asset_size as usize)?;
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
                        authority: record.authority.clone(),
                    })
                })
                .collect::<Result<Vec<_>, MplCoreError>>()?;
            plugin_registry.save(ctx.accounts.asset, new_registry_offset as usize)?;
        } else {
            resize_or_reallocate_account(
                ctx.accounts.asset,
                payer,
                ctx.accounts.system_program,
                asset.get_size(),
            )?;
        }

        asset.save(ctx.accounts.asset, 0)?;
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
    assert_signer(ctx.accounts.authority)?;
    let payer = resolve_payer(ctx.accounts.authority, ctx.accounts.payer)?;

    let (mut asset, plugin_header, plugin_registry) = validate_collection_permissions(
        ctx.accounts.authority,
        ctx.accounts.collection,
        Collection::check_update,
        PluginType::check_update,
        Collection::validate_update,
        Plugin::validate_update,
    )?;

    let asset_size = asset.get_size() as isize;

    let mut dirty = false;
    if let Some(new_update_authority) = ctx.accounts.new_update_authority {
        asset.update_authority = *new_update_authority.key;
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
        if let (Some(mut plugin_header), Some(mut plugin_registry)) =
            (plugin_header, plugin_registry)
        {
            let new_asset_size = asset.get_size() as isize;
            let size_diff = new_asset_size
                .checked_sub(asset_size)
                .ok_or(MplCoreError::NumericalOverflow)?;
            let new_size = (ctx.accounts.collection.data_len() as isize)
                .checked_add(size_diff)
                .ok_or(MplCoreError::NumericalOverflow)?;
            let new_registry_offset = (plugin_header.plugin_registry_offset as isize)
                .checked_add(size_diff)
                .ok_or(MplCoreError::NumericalOverflow)?;
            let registry_offset = plugin_header.plugin_registry_offset;
            plugin_header.plugin_registry_offset = new_registry_offset as usize;

            let plugin_offset = asset_size
                .checked_add(size_diff)
                .ok_or(MplCoreError::NumericalOverflow)?;
            let new_plugin_offset = new_asset_size
                .checked_add(size_diff)
                .ok_or(MplCoreError::NumericalOverflow)?;

            // //TODO: This is memory intensive, we should use memmove instead probably.
            let src = ctx.accounts.collection.data.borrow()
                [(plugin_offset as usize)..registry_offset]
                .to_vec();

            resize_or_reallocate_account(
                ctx.accounts.collection,
                payer,
                ctx.accounts.system_program,
                new_size as usize,
            )?;

            sol_memcpy(
                &mut ctx.accounts.collection.data.borrow_mut()[(new_plugin_offset as usize)..],
                &src,
                src.len(),
            );

            plugin_header.save(ctx.accounts.collection, new_asset_size as usize)?;
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
                        authority: record.authority.clone(),
                    })
                })
                .collect::<Result<Vec<_>, MplCoreError>>()?;
            plugin_registry.save(ctx.accounts.collection, new_registry_offset as usize)?;
        } else {
            resize_or_reallocate_account(
                ctx.accounts.collection,
                payer,
                ctx.accounts.system_program,
                asset.get_size(),
            )?;
        }

        asset.save(ctx.accounts.collection, 0)?;
    }

    Ok(())
}
