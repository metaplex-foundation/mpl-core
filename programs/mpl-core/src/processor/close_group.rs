use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};

use crate::{
    error::MplCoreError,
    instruction::accounts::CloseGroupV1Accounts,
    state::{GroupV1, SolanaAccount},
    utils::{close_program_account, is_valid_group_authority, resolve_authority},
};

/// Arguments for the `CloseGroupV1` instruction.
///
/// Currently, no arguments are required but the struct is kept for
/// forward-compatibility and to mirror the pattern used by other
/// processors in the codebase.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone, Default)]
pub(crate) struct CloseGroupV1Args {}

/// Processor for the `CloseGroupV1` instruction.
pub(crate) fn close_group_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    _args: CloseGroupV1Args,
) -> ProgramResult {
    // Derive the typed account context from the raw slice.
    let ctx = CloseGroupV1Accounts::context(accounts)?;

    // Basic guards.
    assert_signer(ctx.accounts.payer)?;
    let authority = resolve_authority(ctx.accounts.payer, ctx.accounts.authority)?;

    // Deserialize the group account.
    let group = GroupV1::load(ctx.accounts.group, 0)?;

    // Ensure the signer is the update authority or delegate of the group.
    if !is_valid_group_authority(ctx.accounts.group, authority)? {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // Ensure the group has no children, parents, or assets.
    if !(group.collections.is_empty()
        && group.groups.is_empty()
        && group.parent_groups.is_empty()
        && group.assets.is_empty())
    {
        return Err(MplCoreError::GroupMustBeEmpty.into());
    }

    // Close the group account, transferring rent-exempt lamports back to the payer.
    close_program_account(ctx.accounts.group, ctx.accounts.payer)
}
