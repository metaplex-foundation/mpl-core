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

/// Arguments for the `RemoveCollectionsFromGroupV1` instruction.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RemoveCollectionsFromGroupV1Args {
    /// The list of collections to remove from the group.
    pub(crate) collections: Vec<Pubkey>,
}

/// Processor for the `RemoveCollectionsFromGroupV1` instruction.
#[allow(clippy::too_many_arguments)]
pub(crate) fn remove_collections_from_group_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RemoveCollectionsFromGroupV1Args,
) -> ProgramResult {
    // Expected account layout:
    //   0. [writable] Group account
    //   1. [writable, signer] Payer account (also default authority)
    //   2. [signer] Optional authority (update auth/delegate)
    //   3. [] System program
    //   4..N [writable] Collection accounts, one for each pubkey in args.collections

    if accounts.len() < 4 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let group_info = &accounts[0];
    let payer_info = &accounts[1];
    let authority_info_opt = accounts.get(2);
    let system_program_info = &accounts[3];

    let collection_accounts = &accounts[4..];

    // Basic guards
    assert_signer(payer_info)?;
    let authority_info = resolve_authority(payer_info, authority_info_opt)?;

    if system_program_info.key != &solana_program::system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    if collection_accounts.len() != args.collections.len() {
        msg!(
            "Error: Number of collection accounts ({}) does not match number of pubkeys in args ({}).",
            collection_accounts.len(),
            args.collections.len()
        );
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    // Deserialize group.
    let mut group = GroupV1::load(group_info, 0)?;

    // Authority check: must be the group's update authority or delegate.
    if !is_valid_group_authority(group_info, authority_info)? {
        msg!("Error: Invalid authority for group account");
        return Err(MplCoreError::InvalidAuthority.into());
    }

    for (i, collection_info) in collection_accounts.iter().enumerate() {
        if collection_info.key != &args.collections[i] {
            msg!(
                "Error: Collection account at position {} does not match provided pubkey list",
                i
            );
            return Err(MplCoreError::IncorrectAccount.into());
        }

        if !collection_info.is_writable {
            msg!("Error: Collection account must be writable");
            return Err(ProgramError::InvalidAccountData);
        }

        let _collection_core = CollectionV1::load(collection_info, 0)?;

        if !is_valid_collection_authority(collection_info, authority_info)? {
            msg!("Error: Signer is not collection update authority/delegate");
            return Err(MplCoreError::InvalidAuthority.into());
        }

        // Remove from group.collections if present.
        if let Some(pos) = group
            .collections
            .iter()
            .position(|pk| pk == collection_info.key)
        {
            group.collections.remove(pos);
        } else {
            // If collection not present, skip plugin update.
            continue;
        }

        // Remove group from collection's Groups plugin.
        process_collection_groups_plugin_remove(
            collection_info,
            *group_info.key,
            payer_info,
            system_program_info,
        )?;
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

fn process_collection_groups_plugin_remove<'a>(
    collection_info: &AccountInfo<'a>,
    parent_group: Pubkey,
    payer_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
) -> ProgramResult {
    let (_collection_core, header_offset, mut plugin_header, mut plugin_registry) =
        create_meta_idempotent::<CollectionV1>(collection_info, payer_info, system_program_info)?;

    let record_index_opt = plugin_registry
        .registry
        .iter()
        .position(|r| r.plugin_type == PluginType::Groups);

    if let Some(index) = record_index_opt {
        let record_offset = plugin_registry.registry[index].offset;

        let mut plugin = Plugin::load(collection_info, record_offset)?;
        if let Plugin::Groups(inner) = &mut plugin {
            if let Some(pos) = inner.groups.iter().position(|pk| pk == &parent_group) {
                inner.groups.remove(pos);
            } else {
                // Group not present, nothing to remove.
                return Ok(());
            }
        } else {
            return Err(MplCoreError::InvalidPlugin.into());
        }

        // Serialize sizes.
        let old_plugin_data =
            Plugin::deserialize(&mut &collection_info.data.borrow()[record_offset..])?
                .try_to_vec()?;
        let new_plugin_data = plugin.try_to_vec()?;
        let size_diff = (new_plugin_data.len() as isize)
            .checked_sub(old_plugin_data.len() as isize)
            .ok_or(MplCoreError::NumericalOverflow)?;

        if size_diff != 0 {
            plugin_registry.bump_offsets(record_offset, size_diff)?;
            let new_registry_offset = (plugin_header.plugin_registry_offset as isize)
                .checked_add(size_diff)
                .ok_or(MplCoreError::NumericalOverflow)?;
            plugin_header.plugin_registry_offset = new_registry_offset as usize;

            let new_size = (collection_info.data_len() as isize)
                .checked_add(size_diff)
                .ok_or(MplCoreError::NumericalOverflow)? as usize;
            resize_or_reallocate_account(
                collection_info,
                payer_info,
                system_program_info,
                new_size,
            )?;

            let next_plugin_offset = (record_offset + old_plugin_data.len()) as isize;
            let new_next_plugin_offset = next_plugin_offset + size_diff;

            unsafe {
                let base_ptr = collection_info.data.borrow_mut().as_mut_ptr();
                sol_memmove(
                    base_ptr.add(new_next_plugin_offset as usize),
                    base_ptr.add(next_plugin_offset as usize),
                    plugin_header.plugin_registry_offset - next_plugin_offset as usize,
                );
            }
        }

        // Save changes.
        plugin_header.save(collection_info, header_offset)?;
        plugin_registry.save(collection_info, plugin_header.plugin_registry_offset)?;
        plugin.save(collection_info, record_offset)?;
    }

    Ok(())
}
