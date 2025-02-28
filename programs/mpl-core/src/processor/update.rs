use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_memory::sol_memmove,
};
use std::collections::HashSet;

use crate::{
    error::MplCoreError,
    instruction::accounts::{
        Context, UpdateCollectionV1Accounts, UpdateV1Accounts, UpdateV2Accounts,
    },
    plugins::{
        fetch_plugin, list_plugins, ExternalPluginAdapter, HookableLifecycleEvent, Plugin,
        PluginHeaderV1, PluginRegistryV1, PluginType, UpdateDelegate, PERMANENT_DELEGATES,
    },
    state::{AssetV1, CollectionV1, DataBlob, Key, SolanaAccount, UpdateAuthority},
    utils::{
        assert_collection_authority, load_key, resize_or_reallocate_account, resolve_authority,
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

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct UpdateV2Args {
    pub new_name: Option<String>,
    pub new_uri: Option<String>,
    pub new_update_authority: Option<UpdateAuthority>,
}

impl From<UpdateV1Args> for UpdateV2Args {
    fn from(item: UpdateV1Args) -> Self {
        UpdateV2Args {
            new_name: item.new_name,
            new_uri: item.new_uri,
            new_update_authority: item.new_update_authority,
        }
    }
}

enum InstructionVersion {
    V1,
    V2,
}

pub(crate) fn update_v1<'a>(accounts: &'a [AccountInfo<'a>], args: UpdateV1Args) -> ProgramResult {
    let ctx = UpdateV1Accounts::context(accounts)?;
    let v2_accounts = UpdateV2Accounts {
        asset: ctx.accounts.asset,
        collection: ctx.accounts.collection,
        payer: ctx.accounts.payer,
        authority: ctx.accounts.authority,
        new_collection: None,
        system_program: ctx.accounts.system_program,
        log_wrapper: ctx.accounts.log_wrapper,
    };

    let v2_ctx = Context {
        accounts: v2_accounts,
        remaining_accounts: ctx.remaining_accounts,
    };

    update(
        v2_ctx,
        accounts,
        UpdateV2Args::from(args),
        InstructionVersion::V1,
    )
}

pub(crate) fn update_v2<'a>(accounts: &'a [AccountInfo<'a>], args: UpdateV2Args) -> ProgramResult {
    let ctx = UpdateV2Accounts::context(accounts)?;
    update(ctx, accounts, args, InstructionVersion::V2)
}

fn update<'a>(
    ctx: Context<UpdateV2Accounts<'a>>,
    accounts: &'a [AccountInfo<'a>],
    args: UpdateV2Args,
    ix_version: InstructionVersion,
) -> ProgramResult {
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
        args.new_update_authority.as_ref(),
        None,
        None,
        None,
        None,
        AssetV1::check_update,
        CollectionV1::check_update,
        PluginType::check_update,
        AssetV1::validate_update,
        CollectionV1::validate_update,
        Plugin::validate_update,
        Some(ExternalPluginAdapter::validate_update),
        Some(HookableLifecycleEvent::Update),
    )?;

    // Increment sequence number and save only if it is `Some(_)`.
    asset.increment_seq_and_save(ctx.accounts.asset)?;

    let asset_size = asset.len() as isize;

    let mut dirty = false;
    if let Some(new_update_authority) = args.new_update_authority {
        // If asset is currently in a collection, remove it from the collection.
        // This block is executed when we want to go from collection to no collection, or collection to
        // new collection.  The permission for this block is established by `validate_asset_permissions`.
        if let UpdateAuthority::Collection(_existing_collection_address) = asset.update_authority {
            match ix_version {
                InstructionVersion::V1 => {
                    // Removing from or changing collection requires collection size to be updated
                    // and is not supported by `UpdateV1`.
                    msg!("Error: Use UpdateV2 to remove asset from or change collection");
                    return Err(MplCoreError::NotAvailable.into());
                }
                InstructionVersion::V2 => {
                    let existing_collection_account = ctx
                        .accounts
                        .collection
                        .ok_or(MplCoreError::MissingCollection)?;

                    let mut existing_collection =
                        CollectionV1::load(existing_collection_account, 0)?;

                    existing_collection.decrement_size()?;
                    existing_collection.save(existing_collection_account, 0)?;
                }
            }
        }

        // If new update authority is a collection, add the asset to the new collection.
        // This block is executed when we want to go from no collection to collection, or collection
        // to new collection.  The permission for this block is established in the code block.  It is
        // similar to creating an asset, in that the new collection's authority or an update delegate
        // for the new collecton can approve adding the asset to the new collection.
        if let UpdateAuthority::Collection(new_collection_address) = new_update_authority {
            match ix_version {
                InstructionVersion::V1 => {
                    msg!("Error: Use UpdateV2 to add asset to a new collection");
                    return Err(MplCoreError::NotAvailable.into());
                }
                InstructionVersion::V2 => {
                    // Make sure the account was provided.
                    let new_collection_account = ctx
                        .accounts
                        .new_collection
                        .ok_or(MplCoreError::MissingCollection)?;

                    // Make sure account matches what was provided in the args.
                    if new_collection_account.key != &new_collection_address {
                        return Err(MplCoreError::InvalidCollection.into());
                    }

                    // Deserialize the collection.
                    let mut new_collection = CollectionV1::load(new_collection_account, 0)?;

                    // Get a set of all the plugins on the collection (if any).
                    let plugin_set: HashSet<_> =
                        if new_collection_account.data_len() > new_collection.len() {
                            let plugin_list = list_plugins::<CollectionV1>(new_collection_account)?;
                            plugin_list.into_iter().collect()
                        } else {
                            HashSet::new()
                        };

                    // Cannot move to a collection with permanent delegates.
                    if PERMANENT_DELEGATES.iter().any(|p| plugin_set.contains(p)) {
                        return Err(MplCoreError::PermanentDelegatesPreventMove.into());
                    }

                    // Create a default update delegate to be updated with the fetched plugin.
                    let mut plugin: UpdateDelegate = UpdateDelegate::default();

                    // Make sure the authority has authority to add the asset to the new collection.
                    if plugin_set.contains(&PluginType::UpdateDelegate) {
                        // Fetch the update delegate on the new collection.
                        // Do not ignore the return plugin as we need to check the additional delegates.
                        let (plugin_authority, fetched_plugin, _) =
                            fetch_plugin::<CollectionV1, UpdateDelegate>(
                                new_collection_account,
                                PluginType::UpdateDelegate,
                            )?;

                        plugin = fetched_plugin;

                        if (assert_collection_authority(
                            &new_collection,
                            authority,
                            &plugin_authority,
                        )
                        .is_err()
                            && authority.key != &new_collection.update_authority)
                            && !plugin.additional_delegates.contains(authority.key)
                        {
                            solana_program::msg!("UA: Rejected");
                            return Err(MplCoreError::InvalidAuthority.into());
                        }
                    } else if authority.key != &new_collection.update_authority
                        && !plugin.additional_delegates.contains(authority.key)
                    {
                        solana_program::msg!("UA: Rejected");
                        return Err(MplCoreError::InvalidAuthority.into());
                    }

                    new_collection.increment_size()?;
                    new_collection.save(new_collection_account, 0)?;
                }
            };
        }

        asset.update_authority = new_update_authority;
        dirty = true;
    }
    if let Some(new_name) = &args.new_name {
        asset.name.clone_from(new_name);
        dirty = true;
    }
    if let Some(new_uri) = &args.new_uri {
        asset.uri.clone_from(new_uri);
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
        ctx.accounts.new_update_authority.map(|a| a.key),
        None,
        None,
        None,
        None,
        CollectionV1::check_update,
        PluginType::check_update,
        CollectionV1::validate_update,
        Plugin::validate_update,
        Some(ExternalPluginAdapter::validate_update),
        Some(HookableLifecycleEvent::Update),
    )?;

    let collection_size = collection.len() as isize;

    let mut dirty = false;
    if let Some(new_update_authority) = ctx.accounts.new_update_authority {
        collection.update_authority = *new_update_authority.key;
        dirty = true;
    }
    if let Some(new_name) = &args.new_name {
        collection.name.clone_from(new_name);
        dirty = true;
    }
    if let Some(new_uri) = &args.new_uri {
        collection.uri.clone_from(new_uri);
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
    core_size: isize,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    if let (Some(mut plugin_header), Some(mut plugin_registry)) =
        (plugin_header.clone(), plugin_registry.clone())
    {
        // The new size of the asset and new offset of the plugin header.
        let new_core_size = core.len() as isize;

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
            .checked_add(plugin_header.len() as isize)
            .ok_or(MplCoreError::NumericalOverflow)?;

        let new_plugin_offset = plugin_offset
            .checked_add(size_diff)
            .ok_or(MplCoreError::NumericalOverflow)?;

        resize_or_reallocate_account(account, payer, system_program, new_size as usize)?;

        // SAFETY: `borrow_mut` will always return a valid pointer.
        // new_plugin_offset is derived from plugin_offset and size_diff using
        // checked arithmetic, so it will always be less than or equal to account.data_len().
        // This will fail and revert state if there is a memory violation.
        unsafe {
            let base = account.data.borrow_mut().as_mut_ptr();
            sol_memmove(
                base.add(new_plugin_offset as usize),
                base.add(plugin_offset as usize),
                registry_offset - plugin_offset as usize,
            );
        }

        plugin_header.save(account, new_core_size as usize)?;

        // Move offsets for existing registry records.
        plugin_registry.bump_offsets(core_size as usize, size_diff)?;

        plugin_registry.save(account, new_registry_offset as usize)?;
    } else {
        resize_or_reallocate_account(account, payer, system_program, core.len())?;
    }

    core.save(account, 0)?;

    Ok(())
}
