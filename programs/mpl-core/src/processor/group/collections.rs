use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_memory::sol_memcpy,
    pubkey::Pubkey,
};

use mpl_utils::assert_signer;

use crate::error::MplCoreError;
use crate::state::{CollectionGroupPluginV1, CollectionV1, GroupV1, SolanaAccount};

use borsh::{BorshDeserialize, BorshSerialize};

// Argument structs specific to collection/group membership instructions.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddCollectionsToGroupArgs {
    /// Indices in the instruction account list that point at the collection
    /// accounts being added.
    pub collection_indices: Vec<u8>,
    /// Indices in the instruction account list that point at the corresponding
    /// collection authority signer accounts (update authority *or* delegate).
    pub authority_indices: Vec<u8>,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RemoveCollectionsFromGroupArgs {
    /// Indices for collection accounts to remove.
    pub collection_indices: Vec<u8>,
    /// Indices for collection authority signer accounts.
    pub authority_indices: Vec<u8>,
}

use crate::utils::group::find_collection_group_plugin_address;

#[allow(clippy::too_many_lines)]
pub(crate) fn add_collections_to_group<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddCollectionsToGroupArgs,
) -> ProgramResult {
    if args.collection_indices.len() != args.authority_indices.len() {
        return Err(MplCoreError::InvalidGroupOperation.into());
    }

    let authority_info = accounts
        .get(0)
        .ok_or(MplCoreError::MissingUpdateAuthority)?;
    let group_info = accounts.get(1).ok_or(MplCoreError::GroupNotFound)?;

    // Authority must have signed.
    assert_signer(authority_info)?;

    // Deserialize the group state.
    let mut group = GroupV1::load(group_info, 0)?;

    // Validate authority (update authority **or** delegated authority).
    group.validate_authority(authority_info.key)?;

    // Iterate through the provided collection indices, performing all
    // necessary validations and gathering the collections that will be
    // appended so we can update the group in a single write.
    let mut collections_to_add: Vec<Pubkey> = Vec::new();

    for (i, collection_idx) in args.collection_indices.iter().enumerate() {
        // Safe-guard against out-of-bounds indices coming from malicious input.
        let collection_info = accounts
            .get(*collection_idx as usize)
            .ok_or(MplCoreError::CollectionNotFound)?;

        let authority_idx = args.authority_indices[i] as usize;
        let collection_authority_info = accounts
            .get(authority_idx)
            .ok_or(MplCoreError::MissingUpdateAuthority)?;

        // Collection authority must sign.
        assert_signer(collection_authority_info)?;

        // Deserialize the collection account.
        let collection = CollectionV1::load(collection_info, 0)?;

        // Verify authority matches the collection's update authority.
        if &collection.update_authority != collection_authority_info.key {
            return Err(MplCoreError::InvalidAuthority.into());
        }

        // Skip if the collection is already part of the group.
        if group.collections.contains(collection_info.key) {
            continue;
        }

        // Record for later insertion.
        collections_to_add.push(*collection_info.key);
    }

    // Mutate group state if there are any new collections to add.
    if !collections_to_add.is_empty() {
        group.collections.extend(collections_to_add.iter());

        // Persist the updated group data back into the account – we assume the
        // account was created with sufficient space to hold the maximum number
        // of collections so no re-allocation is required.
        let serialized = group.try_to_vec()?;
        sol_memcpy(
            &mut group_info.try_borrow_mut_data()?[0..serialized.len()],
            &serialized,
            serialized.len(),
        );

        // Attempt to update the `CollectionGroupPluginV1` for each collection
        // that has just been added to this group.
        for collection_pubkey in &collections_to_add {
            let (plugin_address, _bump) = find_collection_group_plugin_address(collection_pubkey);

            if let Some(plugin_info) = accounts.iter().find(|a| a.key == &plugin_address) {
                // Ensure the account is writable so we can mutate it.
                if !plugin_info.is_writable {
                    return Err(MplCoreError::IncorrectAccount.into());
                }

                if plugin_info.data_len() > 0 {
                    let mut plugin = CollectionGroupPluginV1::load(plugin_info, 0)?;

                    if &plugin.collection != collection_pubkey {
                        return Err(MplCoreError::InvalidCollection.into());
                    }

                    if !plugin.groups.contains(group_info.key) {
                        plugin.groups.push(*group_info.key);
                        plugin.save(plugin_info, 0)?;
                    }
                } else {
                    if plugin_info.owner != &crate::ID {
                        return Err(MplCoreError::IncorrectAccount.into());
                    }
                    let mut plugin = CollectionGroupPluginV1::new(*collection_pubkey);
                    plugin.groups.push(*group_info.key);
                    plugin.save(plugin_info, 0)?;
                }
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
pub(crate) fn remove_collections_from_group<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RemoveCollectionsFromGroupArgs,
) -> ProgramResult {
    if args.collection_indices.len() != args.authority_indices.len() {
        return Err(MplCoreError::InvalidGroupOperation.into());
    }

    let authority_info = accounts
        .get(0)
        .ok_or(MplCoreError::MissingUpdateAuthority)?;
    let group_info = accounts.get(1).ok_or(MplCoreError::GroupNotFound)?;

    assert_signer(authority_info)?;

    let mut group = GroupV1::load(group_info, 0)?;
    group.validate_authority(authority_info.key)?;

    for (i, collection_idx) in args.collection_indices.iter().enumerate() {
        let collection_info = accounts
            .get(*collection_idx as usize)
            .ok_or(MplCoreError::CollectionNotFound)?;
        let authority_idx = args.authority_indices[i] as usize;
        let collection_authority_info = accounts
            .get(authority_idx)
            .ok_or(MplCoreError::MissingUpdateAuthority)?;

        assert_signer(collection_authority_info)?;
        let collection = CollectionV1::load(collection_info, 0)?;
        if &collection.update_authority != collection_authority_info.key {
            return Err(MplCoreError::InvalidAuthority.into());
        }

        if let Some(pos) = group
            .collections
            .iter()
            .position(|k| k == collection_info.key)
        {
            group.collections.swap_remove(pos);
        }

        let (plugin_address, _bump) = find_collection_group_plugin_address(collection_info.key);

        if let Some(plugin_info) = accounts.iter().find(|a| a.key == &plugin_address) {
            if plugin_info.data_len() > 0 {
                let mut plugin = CollectionGroupPluginV1::load(plugin_info, 0)?;

                if &plugin.collection != collection_info.key {
                    return Err(MplCoreError::InvalidCollection.into());
                }

                if let Some(pos) = plugin.groups.iter().position(|k| k == group_info.key) {
                    plugin.groups.swap_remove(pos);
                    plugin.save(plugin_info, 0)?;
                }
            }
        }
    }

    let serialized = group.try_to_vec()?;
    sol_memcpy(
        &mut group_info.try_borrow_mut_data()?[0..serialized.len()],
        &serialized,
        serialized.len(),
    );

    Ok(())
}
