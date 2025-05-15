use crate::error::MplCoreError;
use crate::state::{GroupV1, SolanaAccount};
use crate::utils::close_program_account;
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

pub(crate) fn close_group<'a>(accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    let update_authority_info = accounts
        .get(0)
        .ok_or(MplCoreError::MissingUpdateAuthority)?;
    let group_info = accounts.get(1).ok_or(MplCoreError::GroupNotFound)?;

    // Guards
    assert_signer(update_authority_info)?;

    // Deserialize group
    let group_state = GroupV1::load(group_info, 0)?;

    // Validate authority (update or delegated)
    group_state.validate_authority(update_authority_info.key)?;

    // Ensure group empty
    if !group_state.collections.is_empty()
        || !group_state.child_groups.is_empty()
        || !group_state.parent_groups.is_empty()
        || !group_state.assets.is_empty()
        || !group_state.allowed_plugins.is_empty()
    {
        return Err(MplCoreError::GroupNotEmpty.into());
    }

    // Close the account, returning lamports
    close_program_account(group_info, update_authority_info)
}
