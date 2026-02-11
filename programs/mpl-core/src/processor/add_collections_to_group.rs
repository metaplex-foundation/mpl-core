use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    program_memory::sol_memcpy,
};

use super::group_collection_plugin::process_collection_groups_plugin_add;
use crate::{
    error::MplCoreError,
    instruction::accounts::{AddCollectionsToGroupV1Accounts, Context},
    state::{CollectionV1, GroupV1, SolanaAccount},
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
