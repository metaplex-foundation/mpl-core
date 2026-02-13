use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_memory::sol_memmove,
    pubkey::Pubkey,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::{Context, RemoveCollectionsFromGroupV1Accounts},
    plugins::{create_meta_idempotent, Plugin, PluginType},
    state::{CollectionV1, DataBlob, GroupV1, SolanaAccount},
    utils::{
        is_valid_collection_authority, is_valid_group_authority, resize_or_reallocate_account,
        resolve_authority, save_group_core_and_plugins,
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
    let ctx: Context<RemoveCollectionsFromGroupV1Accounts> =
        RemoveCollectionsFromGroupV1Accounts::context(accounts)?;
    let group_info = ctx.accounts.group;
    let payer_info = ctx.accounts.payer;
    let authority_info_opt = ctx.accounts.authority;
    let system_program_info = ctx.accounts.system_program;
    let collection_accounts = ctx.remaining_accounts;

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
    let old_group_core_len = group.len();

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
            msg!("Error: Collection is not a child of the provided group");
            return Err(MplCoreError::IncorrectAccount.into());
        }

        // Remove group from collection's Groups plugin.
        process_collection_groups_plugin_remove(
            collection_info,
            *group_info.key,
            payer_info,
            system_program_info,
        )?;
    }

    // Persist group changes while preserving any existing plugin metadata.
    save_group_core_and_plugins(
        group_info,
        &group,
        old_group_core_len,
        payer_info,
        system_program_info,
    )?;

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
            let old_registry_offset = plugin_header.plugin_registry_offset;
            let next_plugin_offset = record_offset
                .checked_add(old_plugin_data.len())
                .ok_or(MplCoreError::NumericalOverflow)?;
            let new_next_plugin_offset: usize = (next_plugin_offset as isize)
                .checked_add(size_diff)
                .ok_or(MplCoreError::NumericalOverflow)?
                .try_into()
                .map_err(|_| MplCoreError::NumericalOverflow)?;
            let copy_len = old_registry_offset
                .checked_sub(next_plugin_offset)
                .ok_or(MplCoreError::NumericalOverflow)?;

            plugin_registry.bump_offsets(record_offset, size_diff)?;
            let new_registry_offset: usize = (old_registry_offset as isize)
                .checked_add(size_diff)
                .ok_or(MplCoreError::NumericalOverflow)?
                .try_into()
                .map_err(|_| MplCoreError::NumericalOverflow)?;
            plugin_header.plugin_registry_offset = new_registry_offset;

            let new_size: usize = (collection_info.data_len() as isize)
                .checked_add(size_diff)
                .ok_or(MplCoreError::NumericalOverflow)?
                .try_into()
                .map_err(|_| MplCoreError::NumericalOverflow)?;

            // When shrinking, move trailing bytes before resizing so the source
            // region remains valid.
            if size_diff < 0 && copy_len > 0 {
                unsafe {
                    let base_ptr = collection_info.data.borrow_mut().as_mut_ptr();
                    sol_memmove(
                        base_ptr.add(new_next_plugin_offset),
                        base_ptr.add(next_plugin_offset),
                        copy_len,
                    );
                }
            }

            resize_or_reallocate_account(
                collection_info,
                payer_info,
                system_program_info,
                new_size,
            )?;

            // When growing, reallocate first so the destination region is valid.
            if size_diff > 0 && copy_len > 0 {
                unsafe {
                    let base_ptr = collection_info.data.borrow_mut().as_mut_ptr();
                    sol_memmove(
                        base_ptr.add(new_next_plugin_offset),
                        base_ptr.add(next_plugin_offset),
                        copy_len,
                    );
                }
            }
        }

        // Save changes.
        plugin_header.save(collection_info, header_offset)?;
        plugin_registry.save(collection_info, plugin_header.plugin_registry_offset)?;
        plugin.save(collection_info, record_offset)?;
    }

    Ok(())
}
