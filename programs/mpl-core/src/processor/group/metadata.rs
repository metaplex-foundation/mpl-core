use solana_program::pubkey::Pubkey;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_memory::sol_memcpy,
};

use mpl_utils::assert_signer;

use crate::error::MplCoreError;
use crate::state::{GroupV1, SolanaAccount};

use borsh::{BorshDeserialize, BorshSerialize};

// Argument struct specific to the `update_group_metadata` instruction.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct UpdateGroupMetadataArgs {
    /// Optional new name for the group.
    pub name: Option<String>,
    /// Optional new URI for the group.
    pub uri: Option<String>,
    /// Optional new update authority for the group.
    pub new_update_authority: Option<Pubkey>,
}

pub(crate) fn update_group_metadata<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: UpdateGroupMetadataArgs,
) -> ProgramResult {
    // 0 authority signer
    // 1 group account
    let authority_info = accounts
        .get(0)
        .ok_or(MplCoreError::MissingUpdateAuthority)?;
    let group_info = accounts.get(1).ok_or(MplCoreError::GroupNotFound)?;

    assert_signer(authority_info)?;

    // Load group
    let mut group = GroupV1::load(group_info, 0)?;

    // Check authority
    group.validate_authority(authority_info.key)?;

    // Apply updates
    if let Some(name) = args.name {
        group.name = name;
    }
    if let Some(uri) = args.uri {
        group.uri = uri;
    }
    if let Some(new_auth) = args.new_update_authority {
        group.update_authority = new_auth;
    }

    // Save
    let serialized = group.try_to_vec()?;
    sol_memcpy(
        &mut group_info.try_borrow_mut_data()?[0..serialized.len()],
        &serialized,
        serialized.len(),
    );

    Ok(())
}
