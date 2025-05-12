use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke,
    program_memory::sol_memcpy, pubkey::Pubkey, rent::Rent, system_instruction, system_program,
    sysvar::Sysvar,
};

use mpl_utils::assert_signer;

use crate::error::MplCoreError;
use crate::state::{CollectionGroupPluginV1, CollectionV1, GroupV1, SolanaAccount};
use crate::utils::group::{find_collection_group_plugin_address, is_circular_reference};

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct CreateGroupArgs {
    /// Human-readable on-chain name for the group.
    pub name: String,
    /// URI pointing at off-chain JSON metadata describing the group.
    pub uri: String,
    /// Maximum number of collections that may be attached to the group.
    pub max_collections: u32,
    /// Maximum number of assets that may be attached to the group.
    pub max_assets: u32,
    /// Maximum number of direct child/parent groups.
    pub max_groups: u32,
    /// Maximum number of plugins allowed for the group.
    pub max_plugins: u32,
    /// Optional list of collections to attach during creation.
    /// The collection accounts must be passed in the remaining accounts array
    /// at the indices provided when the instruction is invoked.
    pub initial_collections: Vec<Pubkey>,
    /// Optional list of parent groups to attach this group to during creation.
    pub initial_parent_groups: Vec<Pubkey>,
    /// Optional list of assets to attach during creation (rarely used).
    pub initial_assets: Vec<Pubkey>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn create_group<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: CreateGroupArgs,
) -> ProgramResult {
    // TODO: these errors need to be fixed
    // Accounts
    let payer_info = accounts.get(0).ok_or(MplCoreError::MissingSystemProgram)?;
    let group_info = accounts.get(1).ok_or(MplCoreError::MissingSystemProgram)?;
    let update_authority_info = accounts.get(2).ok_or(MplCoreError::MissingSystemProgram)?;
    let system_program_info = accounts.get(3).ok_or(MplCoreError::MissingSystemProgram)?;

    // Guards
    assert_signer(payer_info)?;
    assert_signer(update_authority_info)?;
    assert_signer(group_info)?;

    if system_program_info.key != &system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    // Ensure account is uninitialized
    if group_info.data_len() > 0 && group_info.owner != &crate::ID {
        return Err(MplCoreError::IncorrectAccount.into());
    }

    // Construct the new group state
    let group_state = GroupV1::new(
        *update_authority_info.key,
        args.name.clone(),
        args.uri.clone(),
    );

    // Calculate required space using provided maxima.
    let space = GroupV1::size(
        &args.name,
        &args.uri,
        args.max_collections as usize,
        args.max_assets as usize,
        args.max_groups as usize,
        args.max_plugins as usize,
    );

    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space);

    // Create the account via CPI to system program (allocate to full `space` bytes)
    invoke(
        &system_instruction::create_account(
            payer_info.key,
            group_info.key,
            lamports,
            space as u64,
            &crate::ID,
        ),
        &[
            payer_info.clone(),
            group_info.clone(),
            system_program_info.clone(),
        ],
    )?;

    // Persist *initial* serialized data (may be smaller than space)
    let serialized_state = group_state.try_to_vec()?;
    sol_memcpy(
        &mut group_info.try_borrow_mut_data()?[0..serialized_state.len()],
        &serialized_state,
        serialized_state.len(),
    );

    // -------- Handle optional initial collections --------
    if !args.initial_collections.is_empty() {
        // Expect accounts layout after the first 4 accounts:
        // for each collection -> [collection_account, collection_update_authority_signer]
        let mut cursor = 4usize;
        let mut group = group_state; // we modify copy then save

        for expected_pubkey in &args.initial_collections {
            let collection_info = accounts
                .get(cursor)
                .ok_or(MplCoreError::MissingCollection)?;
            cursor += 1;
            let collection_authority_info = accounts
                .get(cursor)
                .ok_or(MplCoreError::MissingUpdateAuthority)?;
            cursor += 1;

            // Checks
            if collection_info.key != expected_pubkey {
                return Err(MplCoreError::InvalidCollection.into());
            }
            assert_signer(collection_authority_info)?;

            let collection = CollectionV1::load(collection_info, 0)?;
            if &collection.update_authority != collection_authority_info.key {
                return Err(MplCoreError::InvalidAuthority.into());
            }

            if !group.collections.contains(collection_info.key) {
                group.collections.push(*collection_info.key);
            }

            // ------------------------------------------------------------------------
            // Attempt to upsert the `CollectionGroupPluginV1` that tracks group
            // membership for this collection. The plugin PDA address is derived from
            // the collection pubkey and therefore deterministic. If the corresponding
            // account info is supplied in the `accounts` array we will either:
            //   • update the existing plugin to append this new group, or
            //   • initialize the account with a brand-new plugin instance.
            // If the account is not provided we silently skip – clients may choose to
            // manage plugin creation in a separate transaction.
            // ------------------------------------------------------------------------

            let (plugin_address, _bump) = find_collection_group_plugin_address(collection_info.key);

            if let Some(plugin_info) = accounts.iter().find(|a| a.key == &plugin_address) {
                // Ensure writable so we can mutate.
                if !plugin_info.is_writable {
                    return Err(MplCoreError::IncorrectAccount.into());
                }

                if plugin_info.data_len() > 0 {
                    // Existing plugin – load, validate, then append group reference.
                    let mut plugin = CollectionGroupPluginV1::load(plugin_info, 0)?;

                    // Sanity-check: plugin must belong to the right collection.
                    if &plugin.collection != collection_info.key {
                        return Err(MplCoreError::InvalidCollection.into());
                    }

                    if !plugin.groups.contains(group_info.key) {
                        plugin.groups.push(*group_info.key);
                        plugin.save(plugin_info, 0)?;
                    }
                } else {
                    // Account exists but is uninitialized – create fresh plugin record.
                    if plugin_info.owner != &crate::ID {
                        return Err(MplCoreError::IncorrectAccount.into());
                    }

                    let mut plugin = CollectionGroupPluginV1::new(*collection_info.key);
                    plugin.groups.push(*group_info.key);
                    plugin.save(plugin_info, 0)?;
                }
            }
        }

        // Save updated group data to account
        let updated_bytes = group.try_to_vec()?;
        sol_memcpy(
            &mut group_info.try_borrow_mut_data()?[0..updated_bytes.len()],
            &updated_bytes,
            updated_bytes.len(),
        );
    }

    // -------- Handle optional initial parent groups --------
    if !args.initial_parent_groups.is_empty() {
        // Expect for every parent -> [parent_group_account, parent_group_update_authority_signer]
        let mut cursor = 4 + args.initial_collections.len() * 2; // skip the first 4 + collections pairs
        let mut group = GroupV1::load(group_info, 0)?;

        for expected_pubkey in args.initial_parent_groups {
            let parent_group_info = accounts.get(cursor).ok_or(MplCoreError::GroupNotFound)?;
            cursor += 1;
            let parent_authority_info = accounts
                .get(cursor)
                .ok_or(MplCoreError::MissingUpdateAuthority)?;
            cursor += 1;

            // Validate expected pubkey matches.
            if parent_group_info.key != &expected_pubkey {
                return Err(MplCoreError::IncorrectAccount.into());
            }
            assert_signer(parent_authority_info)?;

            // Load parent group.
            let mut parent_group = GroupV1::load(parent_group_info, 0)?;

            // Validate authority.
            parent_group.validate_authority(parent_authority_info.key)?;

            // Detect circular references (deep) before linking.
            if is_circular_reference(parent_group_info.key, group_info.key, accounts)? {
                return Err(MplCoreError::CircularGroupReference.into());
            }

            // Link parent <-> child if not already.
            if !parent_group.child_groups.contains(group_info.key) {
                parent_group.child_groups.push(*group_info.key);
            }
            if !group.parent_groups.contains(parent_group_info.key) {
                group.parent_groups.push(*parent_group_info.key);
            }

            // Persist parent group changes.
            let parent_bytes = parent_group.try_to_vec()?;
            sol_memcpy(
                &mut parent_group_info.try_borrow_mut_data()?[0..parent_bytes.len()],
                &parent_bytes,
                parent_bytes.len(),
            );
        }

        // Persist updated child group with parent references.
        let updated_bytes = group.try_to_vec()?;
        sol_memcpy(
            &mut group_info.try_borrow_mut_data()?[0..updated_bytes.len()],
            &updated_bytes,
            updated_bytes.len(),
        );
    }

    Ok(())
}
