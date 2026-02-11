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

/// Arguments for the `AddGroupsToGroupV1` instruction.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddGroupsToGroupV1Args {
    /// The list of child groups to add to the parent group.
    pub(crate) groups: Vec<Pubkey>,
}

/// Processor for the `AddGroupsToGroupV1` instruction.
#[allow(clippy::too_many_arguments)]
pub(crate) fn add_groups_to_group_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddGroupsToGroupV1Args,
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

    // Remaining accounts are the child group accounts.
    let child_group_accounts = &accounts[4..];

    // Basic guards.
    assert_signer(payer_info)?;
    let authority_info = resolve_authority(payer_info, authority_info_opt)?;

    if system_program_info.key != &solana_program::system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    // Validate arg count matches remaining accounts.
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

    // Authority check: must be update authority or delegate of the parent group.
    if !is_valid_group_authority(parent_group_info, authority_info)? {
        msg!("Error: Invalid authority for parent group account");
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // Process each child group account.
    for (i, child_info) in child_group_accounts.iter().enumerate() {
        // Ensure account key matches expected pubkey.
        if child_info.key != &args.groups[i] {
            msg!(
                "Error: Child group account at position {} does not match provided pubkey list",
                i
            );
            return Err(MplCoreError::IncorrectAccount.into());
        }

        // Ensure child group account is writable.
        if !child_info.is_writable {
            msg!("Error: Child group account must be writable");
            return Err(ProgramError::InvalidAccountData);
        }

        if child_info.key == parent_group_info.key {
            msg!("Error: Parent group cannot be added as its own child group");
            return Err(MplCoreError::IncorrectAccount.into());
        }

        // Deserialize child group.
        let mut child_group = GroupV1::load(child_info, 0)?;

        // Authority must also be update authority or delegate for the child group.
        if !is_valid_group_authority(child_info, authority_info)? {
            msg!("Error: Signer is not child group update authority/delegate");
            return Err(MplCoreError::InvalidAuthority.into());
        }

        // 1. Update parent group's child list if not already present.
        if !parent_group.groups.contains(child_info.key) {
            parent_group.groups.push(*child_info.key);
        }

        // 2. Update child's parent_groups list if not already present.
        if !child_group.parent_groups.contains(parent_group_info.key) {
            child_group.parent_groups.push(*parent_group_info.key);

            // Persist child group changes.
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
