use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_memory::sol_memcpy,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::UpdateGroupV1Accounts,
    state::{GroupV1, SolanaAccount},
    utils::{is_valid_group_authority, resize_or_reallocate_account, resolve_authority},
};

/// Arguments for the `UpdateGroupV1` instruction.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct UpdateGroupV1Args {
    /// New display name for the group (optional).
    pub(crate) new_name: Option<String>,
    /// New URI for the group (optional).
    pub(crate) new_uri: Option<String>,
}

/// Processor for the `UpdateGroupV1` instruction.
#[allow(clippy::too_many_arguments)]
pub(crate) fn update_group_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: UpdateGroupV1Args,
) -> ProgramResult {
    // Derive the typed account context from the raw slice.
    let ctx = UpdateGroupV1Accounts::context(accounts)?;

    // Basic guards.
    assert_signer(ctx.accounts.payer)?;
    let authority = resolve_authority(ctx.accounts.payer, ctx.accounts.authority)?;

    // Ensure the canonical system program is provided.
    if ctx.accounts.system_program.key != &solana_program::system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    // Deserialize the group account.
    let mut group = GroupV1::load(ctx.accounts.group, 0)?;

    // Ensure the signer is the update authority or update delegate of the group.
    if !is_valid_group_authority(ctx.accounts.group, authority)? {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // Track if any field is modified.
    let mut dirty = false;

    // Apply a new update authority if supplied as an account.
    if let Some(new_update_authority) = ctx.accounts.new_update_authority {
        group.update_authority = *new_update_authority.key;
        dirty = true;
    }

    // Apply inline argument changes.
    if let Some(new_name) = &args.new_name {
        group.name.clone_from(new_name);
        dirty = true;
    }

    if let Some(new_uri) = &args.new_uri {
        group.uri.clone_from(new_uri);
        dirty = true;
    }

    // Persist state changes if anything was updated.
    if dirty {
        let serialized = group.try_to_vec()?;

        // Resize the account if the serialized length differs.
        if serialized.len() != ctx.accounts.group.data_len() {
            resize_or_reallocate_account(
                ctx.accounts.group,
                ctx.accounts.payer,
                ctx.accounts.system_program,
                serialized.len(),
            )?;
        }

        // Write the updated data into the account.
        sol_memcpy(
            &mut ctx.accounts.group.try_borrow_mut_data()?,
            &serialized,
            serialized.len(),
        );
    }

    Ok(())
}
