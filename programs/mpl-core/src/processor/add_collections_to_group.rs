use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_memory::{sol_memcpy, sol_memmove},
    pubkey::Pubkey,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::{AddCollectionsToGroupV1Accounts, Context},
    plugins::{
        create_meta_idempotent, initialize_plugin, Groups, Plugin, PluginHeaderV1,
        PluginRegistryV1, PluginType,
    },
    state::{CollectionV1, DataBlob, GroupV1, SolanaAccount},
    utils::{
        is_valid_collection_authority, is_valid_group_authority, resize_or_reallocate_account,
        resolve_authority,
    },
};

/// Arguments for the `AddCollectionsToGroupV1` instruction.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddCollectionsToGroupV1Args {}

/// Processor for the `AddCollectionsToGroupV1` instruction.
#[allow(clippy::too_many_arguments)]
pub(crate) fn add_collections_to_group_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    _args: AddCollectionsToGroupV1Args,
) -> ProgramResult {
    // Use generated context to access fixed and remaining accounts.
    let ctx: Context<AddCollectionsToGroupV1Accounts> =
        AddCollectionsToGroupV1Accounts::context(accounts)?;

    let group_info = ctx.accounts.group;
    let payer_info = ctx.accounts.payer;
    let authority_info_opt = ctx.accounts.authority;
    let system_program_info = ctx.accounts.system_program;

    // Dynamic list of collection accounts passed after the fixed accounts.
    let remaining_accounts = ctx.remaining_accounts;

    // Basic guards.
    assert_signer(payer_info)?;
    let authority_info = resolve_authority(payer_info, authority_info_opt)?;

    if system_program_info.key != &solana_program::system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    // Deserialize group.
    let mut group = GroupV1::load(group_info, 0)?;

    // Authority check: must be group's update authority or delegate.
    if !is_valid_group_authority(group_info, authority_info)? {
        msg!("Error: Invalid authority for group account");
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // Process each collection.
    for collection_info in remaining_accounts.iter() {
        // Verify collection is writable.
        if !collection_info.is_writable {
            msg!("Error: Collection account must be writable");
            return Err(ProgramError::InvalidAccountData);
        }

        // Deserialize collection.
        let _collection_core = CollectionV1::load(collection_info, 0)?;

        // Authority must be update authority of the collection as well.
        if !is_valid_collection_authority(collection_info, authority_info)? {
            msg!("Error: Signer is not collection update authority/delegate");
            return Err(MplCoreError::InvalidAuthority.into());
        }

        // 1. Update Group account collections vector.
        if group.collections.contains(collection_info.key) {
            // Already present, skip to next.
            continue;
        }
        group.collections.push(*collection_info.key);

        // 2. Update or create Groups plugin on the collection.
        process_collection_groups_plugin_add(
            collection_info,
            *group_info.key,
            payer_info,
            system_program_info,
        )?;

        // The collection core itself does not change; no reserialization needed.
    }

    // Persist group changes.
    let serialized_group = group.try_to_vec()?;
    if serialized_group.len() != group_info.data_len() {
        resize_or_reallocate_account(
            group_info,
            payer_info,
            system_program_info,
            serialized_group.len(),
        )?;
    }

    sol_memcpy(
        &mut group_info.try_borrow_mut_data()?,
        &serialized_group,
        serialized_group.len(),
    );

    Ok(())
}

/// Add the parent group pubkey to the collection's Groups plugin, creating the plugin if necessary.
fn process_collection_groups_plugin_add<'a>(
    collection_info: &AccountInfo<'a>,
    parent_group: Pubkey,
    payer_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
) -> ProgramResult {
    // Ensure plugin metadata exists or create.
    let (_collection_core, header_offset, mut plugin_header, mut plugin_registry) =
        create_meta_idempotent::<CollectionV1>(collection_info, payer_info, system_program_info)?;

    // Attempt to fetch existing Groups plugin.
    let plugin_record_opt = plugin_registry
        .registry
        .iter()
        .find(|r| r.plugin_type == PluginType::Groups)
        .cloned();

    match plugin_record_opt {
        None => {
            // Plugin does not exist; create it.
            let groups_plugin = Groups {
                groups: vec![parent_group],
            };
            let plugin = Plugin::Groups(groups_plugin);
            initialize_plugin::<CollectionV1>(
                &plugin,
                &plugin.manager(),
                header_offset,
                &mut plugin_header,
                &mut plugin_registry,
                collection_info,
                payer_info,
                system_program_info,
            )?
        }
        Some(record) => {
            // Plugin exists, load, modify, and update.
            let mut plugin = Plugin::load(collection_info, record.offset)?;
            if let Plugin::Groups(inner) = &mut plugin {
                if inner.groups.contains(&parent_group) {
                    // Already present, nothing to do.
                    return Ok(());
                }
                inner.groups.push(parent_group);
            } else {
                // This should never happen.
                return Err(MplCoreError::InvalidPlugin.into());
            }

            // Serialize old and new plugin to compute size diff.
            let old_plugin_data =
                Plugin::deserialize(&mut &collection_info.data.borrow()[record.offset..])?
                    .try_to_vec()?;
            let new_plugin_data = plugin.try_to_vec()?;
            let size_diff = (new_plugin_data.len() as isize)
                .checked_sub(old_plugin_data.len() as isize)
                .ok_or(MplCoreError::NumericalOverflow)?;

            if size_diff != 0 {
                // Bump offsets for subsequent registry entries and header.
                plugin_registry.bump_offsets(record.offset, size_diff)?;
                let new_registry_offset = (plugin_header.plugin_registry_offset as isize)
                    .checked_add(size_diff)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                plugin_header.plugin_registry_offset = new_registry_offset as usize;

                // Resize account.
                let new_size = (collection_info.data_len() as isize)
                    .checked_add(size_diff)
                    .ok_or(MplCoreError::NumericalOverflow)?
                    as usize;
                resize_or_reallocate_account(
                    collection_info,
                    payer_info,
                    system_program_info,
                    new_size,
                )?;

                // Move trailing data to accommodate new plugin size.
                let next_plugin_offset = (record.offset + old_plugin_data.len()) as isize;
                let new_next_plugin_offset = next_plugin_offset + size_diff;

                let copy_len = plugin_header
                    .plugin_registry_offset
                    .saturating_sub(next_plugin_offset as usize);

                if copy_len > 0 {
                    unsafe {
                        let base_ptr = collection_info.data.borrow_mut().as_mut_ptr();
                        sol_memmove(
                            base_ptr.add(new_next_plugin_offset as usize),
                            base_ptr.add(next_plugin_offset as usize),
                            copy_len,
                        );
                    }
                }
            }

            // Save header, registry and new plugin.
            plugin_header.save(collection_info, header_offset)?;
            plugin_registry.save(collection_info, plugin_header.plugin_registry_offset)?;
            plugin.save(collection_info, record.offset)?;
        }
    }

    Ok(())
}
