use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg};

use crate::{
    error::MplCoreError,
    instruction::accounts::{UpdateCollectionV1Accounts, UpdateV1Accounts},
    plugins::{
        ExternalPlugin, HookableLifecycleEvent, Plugin, PluginHeaderV1, PluginRegistryV1,
        PluginType,
    },
    state::{AssetV1, CollectionV1, DataBlob, Key, SolanaAccount, UpdateAuthority},
    utils::{
        load_key, move_plugins_and_registry, resize_or_reallocate_account, resolve_authority,
        validate_asset_permissions, validate_collection_permissions,
    },
};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct UpdateV1Args {
    pub new_name: Option<String>,
    pub new_uri: Option<String>,
    pub new_update_authority: Option<UpdateAuthority>,
}

pub(crate) fn update<'a>(accounts: &'a [AccountInfo<'a>], args: UpdateV1Args) -> ProgramResult {
    // Accounts.
    let ctx = UpdateV1Accounts::context(accounts)?;

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
        msg!("Error: Update for compressed is not available");
        return Err(MplCoreError::NotAvailable.into());
    }

    let (mut asset, plugin_header, plugin_registry) = validate_asset_permissions(
        accounts,
        authority,
        ctx.accounts.asset,
        ctx.accounts.collection,
        None,
        None,
        None,
        AssetV1::check_update,
        CollectionV1::check_update,
        PluginType::check_update,
        AssetV1::validate_update,
        CollectionV1::validate_update,
        Plugin::validate_update,
        Some(ExternalPlugin::validate_update),
        Some(HookableLifecycleEvent::Update),
    )?;

    // Increment sequence number and save only if it is `Some(_)`.
    asset.increment_seq_and_save(ctx.accounts.asset)?;

    let asset_size = asset.get_size();

    let mut dirty = false;
    if let Some(new_update_authority) = args.new_update_authority {
        if let UpdateAuthority::Collection(_collection_address) = new_update_authority {
            // Updating collections is not currently available.
            return Err(MplCoreError::NotAvailable.into());
        };

        if let UpdateAuthority::Collection(_collection_address) = asset.update_authority {
            // Removing from collection is not currently available.
            // will require the collection size to be updated
            return Err(MplCoreError::NotAvailable.into());
        }

        asset.update_authority = new_update_authority;
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
pub(crate) struct UpdateCollectionV1Args {
    pub new_name: Option<String>,
    pub new_uri: Option<String>,
}

pub(crate) fn update_collection<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: UpdateCollectionV1Args,
) -> ProgramResult {
    // Accounts.
    let ctx = UpdateCollectionV1Accounts::context(accounts)?;

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

    let (mut collection, plugin_header, plugin_registry) = validate_collection_permissions(
        accounts,
        authority,
        ctx.accounts.collection,
        None,
        None,
        CollectionV1::check_update,
        PluginType::check_update,
        CollectionV1::validate_update,
        Plugin::validate_update,
        Some(ExternalPlugin::validate_update),
        Some(HookableLifecycleEvent::Update),
    )?;

    let collection_size = collection.get_size();

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
    plugin_header: &Option<PluginHeaderV1>,
    plugin_registry: &Option<PluginRegistryV1>,
    core_size: usize,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    if let (Some(mut plugin_header), Some(mut plugin_registry)) =
        (plugin_header.clone(), plugin_registry.clone())
    {
        // The new size of the asset and new offset of the plugin header.
        let new_core_size = core.get_size();

        // The difference in size between the new and old asset which is used to calculate the new size of the account.
        let size_diff = (new_core_size as isize)
            .checked_sub(core_size as isize)
            .ok_or(MplCoreError::NumericalOverflow)?;

        // The offset of the first plugin is the core size plus the size of the plugin header.
        let plugin_offset = core_size
            .checked_add(plugin_header.get_size())
            .ok_or(MplCoreError::NumericalOverflow)?;

        move_plugins_and_registry(
            new_core_size,
            size_diff,
            plugin_offset,
            plugin_offset,
            &mut plugin_header,
            &mut plugin_registry,
            account,
            payer,
            system_program,
        )?;
    } else {
        resize_or_reallocate_account(account, payer, system_program, core.get_size())?;
    }

    core.save(account, 0)
}
