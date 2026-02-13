use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::{Context, RemoveGroupsFromGroupV1Accounts},
    state::{DataBlob, GroupV1, SolanaAccount},
    utils::{is_valid_group_authority, resolve_authority, save_group_core_and_plugins},
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
    let ctx: Context<RemoveGroupsFromGroupV1Accounts> =
        RemoveGroupsFromGroupV1Accounts::context(accounts)?;
    let parent_group_info = ctx.accounts.parent_group;
    let payer_info = ctx.accounts.payer;
    let authority_info_opt = ctx.accounts.authority;
    let system_program_info = ctx.accounts.system_program;
    let child_group_accounts = ctx.remaining_accounts;

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
    let old_parent_core_len = parent_group.len();

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
        let old_child_core_len = child_group.len();

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
            msg!("Error: Child group is not linked to the parent group");
            return Err(MplCoreError::IncorrectAccount.into());
        }

        // Remove parent from child's parent_groups if present.
        if let Some(pos) = child_group
            .parent_groups
            .iter()
            .position(|pk| pk == parent_group_info.key)
        {
            child_group.parent_groups.remove(pos);

            // Persist child changes while preserving plugin metadata.
            save_group_core_and_plugins(
                child_info,
                &child_group,
                old_child_core_len,
                payer_info,
                system_program_info,
            )?;
        }
    }

    // Persist parent group changes while preserving plugin metadata.
    save_group_core_and_plugins(
        parent_group_info,
        &parent_group,
        old_parent_core_len,
        payer_info,
        system_program_info,
    )?;

    Ok(())
}
