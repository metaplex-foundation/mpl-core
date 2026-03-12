use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use super::groups_plugin_utils::save_updated_groups_plugin;
use crate::{
    error::MplCoreError,
    instruction::accounts::{Context, RemoveAssetsFromGroupV1Accounts},
    plugins::{create_meta_idempotent, Plugin, PluginType},
    state::{AssetV1, GroupV1, SolanaAccount},
    utils::{is_valid_asset_authority, is_valid_group_authority, resolve_authority, save_flat_group},
};

/// Args for RemoveAssetsFromGroupV1
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct RemoveAssetsFromGroupV1Args {
    pub(crate) assets: Vec<Pubkey>,
}

pub(crate) fn remove_assets_from_group_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: RemoveAssetsFromGroupV1Args,
) -> ProgramResult {
    let ctx: Context<RemoveAssetsFromGroupV1Accounts> =
        RemoveAssetsFromGroupV1Accounts::context(accounts)?;
    let group_info = ctx.accounts.group;
    let payer_info = ctx.accounts.payer;
    let authority_info_opt = ctx.accounts.authority;
    let system_program_info = ctx.accounts.system_program;
    let asset_accounts = ctx.remaining_accounts;

    assert_signer(payer_info)?;
    let authority_info = resolve_authority(payer_info, authority_info_opt)?;

    if system_program_info.key != &solana_program::system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    if !group_info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    if asset_accounts.len() != args.assets.len() {
        msg!("Error: account/pubkey mismatch");
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let mut group = GroupV1::load(group_info, 0)?;
    if !is_valid_group_authority(group_info, authority_info)? {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    for (i, asset_info) in asset_accounts.iter().enumerate() {
        if asset_info.key != &args.assets[i] {
            return Err(MplCoreError::IncorrectAccount.into());
        }
        if !asset_info.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }

        if !is_valid_asset_authority(asset_info, authority_info)? {
            return Err(MplCoreError::InvalidAuthority.into());
        }

        // remove asset from group list
        if let Some(pos) = group.assets.iter().position(|pk| pk == asset_info.key) {
            group.assets.remove(pos);
        } else {
            msg!("Error: Asset is not a child of the provided group");
            return Err(MplCoreError::IncorrectAccount.into());
        }

        process_asset_groups_plugin_remove(
            asset_info,
            *group_info.key,
            payer_info,
            system_program_info,
        )?;
    }

    save_flat_group(group_info, &group, payer_info, system_program_info)?;
    Ok(())
}

fn process_asset_groups_plugin_remove<'a>(
    asset_info: &AccountInfo<'a>,
    parent_group: Pubkey,
    payer_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
) -> ProgramResult {
    let (_asset_core, header_offset, mut plugin_header, mut plugin_registry) =
        create_meta_idempotent::<AssetV1>(asset_info, payer_info, system_program_info)?;

    let record_index_opt = plugin_registry
        .registry
        .iter()
        .position(|r| r.plugin_type == PluginType::Groups);

    if let Some(index) = record_index_opt {
        let record = plugin_registry.registry[index].clone();
        let mut plugin = Plugin::load(asset_info, record.offset)?;
        if let Plugin::Groups(inner) = &mut plugin {
            if let Some(pos) = inner.groups.iter().position(|pk| pk == &parent_group) {
                inner.groups.remove(pos);
            } else {
                return Ok(());
            }
        } else {
            return Err(MplCoreError::InvalidPlugin.into());
        }

        save_updated_groups_plugin(
            asset_info,
            payer_info,
            system_program_info,
            &plugin,
            &record,
            &mut plugin_header,
            &mut plugin_registry,
            header_offset,
        )?;
    }
    Ok(())
}
