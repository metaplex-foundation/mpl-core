use crate::error::MplCoreError;
use crate::state::{GroupV1, SolanaAccount};
use crate::utils::group::is_circular_reference;
use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_memory::sol_memcpy,
};

// Argument structs for child group operations.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddGroupsToGroupArgs {
    /// Indices for child group accounts being added.
    pub group_indices: Vec<u8>,
    /// Indices for authority signer accounts of each child group.
    pub authority_indices: Vec<u8>,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RemoveGroupsFromGroupArgs {
    /// Indices for child group accounts being removed.
    pub group_indices: Vec<u8>,
    /// Indices for authority signer accounts of each child group.
    pub authority_indices: Vec<u8>,
}

#[allow(clippy::too_many_lines)]
pub(crate) fn add_groups_to_group<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddGroupsToGroupArgs,
) -> ProgramResult {
    if args.group_indices.len() != args.authority_indices.len() {
        return Err(MplCoreError::InvalidGroupOperation.into());
    }

    let authority_info = accounts
        .get(0)
        .ok_or(MplCoreError::MissingUpdateAuthority)?;
    let parent_group_info = accounts.get(1).ok_or(MplCoreError::GroupNotFound)?;

    assert_signer(authority_info)?;

    let mut parent_group = GroupV1::load(parent_group_info, 0)?;
    parent_group.validate_authority(authority_info.key)?;

    for (i, child_idx) in args.group_indices.iter().enumerate() {
        let child_group_info = accounts
            .get(*child_idx as usize)
            .ok_or(MplCoreError::GroupNotFound)?;
        let child_authority_idx = args.authority_indices[i] as usize;
        let child_authority_info = accounts
            .get(child_authority_idx)
            .ok_or(MplCoreError::MissingUpdateAuthority)?;

        assert_signer(child_authority_info)?;
        let mut child_group = GroupV1::load(child_group_info, 0)?;
        child_group.validate_authority(child_authority_info.key)?;

        if is_circular_reference(parent_group_info.key, child_group_info.key, accounts)? {
            return Err(MplCoreError::CircularGroupReference.into());
        }

        if !parent_group.child_groups.contains(child_group_info.key) {
            parent_group.child_groups.push(*child_group_info.key);
        }
        if !child_group.parent_groups.contains(parent_group_info.key) {
            child_group.parent_groups.push(*parent_group_info.key);
        }

        let child_bytes = child_group.try_to_vec()?;
        sol_memcpy(
            &mut child_group_info.try_borrow_mut_data()?[0..child_bytes.len()],
            &child_bytes,
            child_bytes.len(),
        );
    }

    let parent_bytes = parent_group.try_to_vec()?;
    sol_memcpy(
        &mut parent_group_info.try_borrow_mut_data()?[0..parent_bytes.len()],
        &parent_bytes,
        parent_bytes.len(),
    );

    Ok(())
}

#[allow(clippy::too_many_lines)]
pub(crate) fn remove_groups_from_group<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RemoveGroupsFromGroupArgs,
) -> ProgramResult {
    if args.group_indices.len() != args.authority_indices.len() {
        return Err(MplCoreError::InvalidGroupOperation.into());
    }

    let authority_info = accounts
        .get(0)
        .ok_or(MplCoreError::MissingUpdateAuthority)?;
    let parent_group_info = accounts.get(1).ok_or(MplCoreError::GroupNotFound)?;

    assert_signer(authority_info)?;
    let mut parent_group = GroupV1::load(parent_group_info, 0)?;
    parent_group.validate_authority(authority_info.key)?;

    for (i, child_idx) in args.group_indices.iter().enumerate() {
        let child_group_info = accounts
            .get(*child_idx as usize)
            .ok_or(MplCoreError::GroupNotFound)?;
        let child_authority_idx = args.authority_indices[i] as usize;
        let child_authority_info = accounts
            .get(child_authority_idx)
            .ok_or(MplCoreError::MissingUpdateAuthority)?;

        assert_signer(child_authority_info)?;
        let mut child_group = GroupV1::load(child_group_info, 0)?;
        child_group.validate_authority(child_authority_info.key)?;

        if let Some(pos) = parent_group
            .child_groups
            .iter()
            .position(|k| k == child_group_info.key)
        {
            parent_group.child_groups.swap_remove(pos);
        }
        if let Some(pos) = child_group
            .parent_groups
            .iter()
            .position(|k| k == parent_group_info.key)
        {
            child_group.parent_groups.swap_remove(pos);
        }

        let child_bytes = child_group.try_to_vec()?;
        sol_memcpy(
            &mut child_group_info.try_borrow_mut_data()?[0..child_bytes.len()],
            &child_bytes,
            child_bytes.len(),
        );
    }

    let parent_bytes = parent_group.try_to_vec()?;
    sol_memcpy(
        &mut parent_group_info.try_borrow_mut_data()?[0..parent_bytes.len()],
        &parent_bytes,
        parent_bytes.len(),
    );

    Ok(())
}
