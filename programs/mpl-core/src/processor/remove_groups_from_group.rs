use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    program_memory::sol_memcpy, pubkey::Pubkey,
};

use crate::{
    error::MplCoreError,
    state::{DataBlob, GroupV1, SolanaAccount},
    utils::{is_valid_group_authority, resize_or_reallocate_account, resolve_authority},
};

/// Arguments for the `RemoveGroupsFromGroupV1` instruction.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RemoveGroupsFromGroupV1Args {
    /// The list of child groups to remove from the parent group.
    pub(crate) groups: Vec<Pubkey>,
}

/// Processor for the `RemoveGroupsFromGroupV1` instruction.
#[allow(clippy::too_many_arguments)]
pub(crate) fn remove_groups_from_group_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RemoveGroupsFromGroupV1Args,
) -> ProgramResult {
    // Expected account layout:
    //   0. [writable] Parent group account
    //   1. [writable, signer] Payer account (also default authority)
    //   2. [signer] Optional authority (update auth/delegate)
    //   3. [] System program
    //   4..N [writable] Child group accounts, one for each pubkey in args.groups

    if accounts.len() < 4 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let parent_group_info = &accounts[0];
    let payer_info = &accounts[1];
    let authority_info_opt = accounts.get(2);
    let system_program_info = &accounts[3];

    // Remaining accounts are child group accounts.
    let child_group_accounts = &accounts[4..];

    // Basic guards.
    assert_signer(payer_info)?;
    let authority_info = resolve_authority(payer_info, authority_info_opt)?;

    if system_program_info.key != &solana_program::system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    // Validate arg count.
    if child_group_accounts.len() != args.groups.len() {
        msg!(
            "Error: Number of group accounts ({}) does not match number of pubkeys in args ({}).",
            child_group_accounts.len(),
            args.groups.len()
        );
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    // Deserialize parent group.
    let mut parent_group = GroupV1::load(parent_group_info, 0)?;

    // Authority check: must be parent group's update authority or delegate.
    if !is_valid_group_authority(parent_group_info, authority_info)? {
        msg!("Error: Invalid authority for parent group account");
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // Iterate child groups.
    for (i, child_info) in child_group_accounts.iter().enumerate() {
        if child_info.key != &args.groups[i] {
            msg!(
                "Error: Child group account at position {} does not match provided pubkey list",
                i
            );
            return Err(MplCoreError::IncorrectAccount.into());
        }

        if !child_info.is_writable {
            msg!("Error: Child group account must be writable");
            return Err(ProgramError::InvalidAccountData);
        }

        let mut child_group = GroupV1::load(child_info, 0)?;

        // Authority must also be update authority or delegate for the child group.
        if !is_valid_group_authority(child_info, authority_info)? {
            msg!("Error: Signer is not child group update authority/delegate");
            return Err(MplCoreError::InvalidAuthority.into());
        }

        // Remove child from parent list if present.
        if let Some(pos) = parent_group
            .groups
            .iter()
            .position(|pk| pk == child_info.key)
        {
            parent_group.groups.remove(pos);
        } else {
            // Not present, skip modification of child.
            continue;
        }

        // Remove parent from child's parent_groups if present.
        if let Some(pos) = child_group
            .parent_groups
            .iter()
            .position(|pk| pk == parent_group_info.key)
        {
            child_group.parent_groups.remove(pos);

            // Persist child changes.
            let serialized_child = child_group.try_to_vec()?;
            if serialized_child.len() != child_info.data_len() {
                resize_or_reallocate_account(
                    child_info,
                    payer_info,
                    system_program_info,
                    serialized_child.len(),
                )?;
            }
            sol_memcpy(
                &mut child_info.try_borrow_mut_data()?,
                &serialized_child,
                serialized_child.len(),
            );
        }
    }

    // Persist parent group changes.
    let serialized_parent = parent_group.try_to_vec()?;
    if serialized_parent.len() != parent_group_info.data_len() {
        resize_or_reallocate_account(
            parent_group_info,
            payer_info,
            system_program_info,
            serialized_parent.len(),
        )?;
    }
    sol_memcpy(
        &mut parent_group_info.try_borrow_mut_data()?,
        &serialized_parent,
        serialized_parent.len(),
    );

    Ok(())
}
