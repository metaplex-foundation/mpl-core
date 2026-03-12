use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
};

use super::groups_plugin_utils::process_asset_groups_plugin_add;
use crate::{
    error::MplCoreError,
    instruction::accounts::AddAssetsToGroupV1Accounts,
    instruction::accounts::Context,
    state::{GroupV1, SolanaAccount, MAX_GROUP_VECTOR_SIZE},
    utils::{
        is_valid_asset_authority, is_valid_group_authority, resolve_authority, save_flat_group,
    },
};

/// Arguments for the `AddAssetsToGroupV1` instruction.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddAssetsToGroupV1Args {}

/// Processor for the `AddAssetsToGroupV1` instruction.
#[allow(clippy::too_many_arguments)]
pub(crate) fn add_assets_to_group_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    _args: AddAssetsToGroupV1Args,
) -> ProgramResult {
    let ctx: Context<AddAssetsToGroupV1Accounts> = AddAssetsToGroupV1Accounts::context(accounts)?;

    let group_info = ctx.accounts.group;
    let payer_info = ctx.accounts.payer;
    let authority_info_opt = ctx.accounts.authority;
    let system_program_info = ctx.accounts.system_program;
    let remaining_accounts = ctx.remaining_accounts;

    assert_signer(payer_info)?;
    let authority_info = resolve_authority(payer_info, authority_info_opt)?;

    if system_program_info.key != &solana_program::system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    if !group_info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    let mut group = GroupV1::load(group_info, 0)?;

    if !is_valid_group_authority(group_info, authority_info)? {
        msg!("Error: Invalid authority for group");
        return Err(MplCoreError::InvalidAuthority.into());
    }

    for asset_info in remaining_accounts.iter() {
        if !asset_info.is_writable {
            msg!("Error: Asset account must be writable");
            return Err(ProgramError::InvalidAccountData);
        }

        if !is_valid_asset_authority(asset_info, authority_info)? {
            return Err(MplCoreError::InvalidAuthority.into());
        }

        if group.assets.contains(asset_info.key) {
            return Err(MplCoreError::DuplicateEntry.into());
        }

        if group.assets.len() >= MAX_GROUP_VECTOR_SIZE {
            return Err(MplCoreError::GroupVectorFull.into());
        }

        group.assets.push(*asset_info.key);

        process_asset_groups_plugin_add(
            asset_info,
            *group_info.key,
            payer_info,
            system_program_info,
        )?;
    }

    save_flat_group(group_info, &group, payer_info, system_program_info)?;

    Ok(())
}

