use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_memory::sol_memmove,
    pubkey::Pubkey,
};

use crate::{
    error::MplCoreError,
    instruction::accounts::{Context, RemoveAssetsFromGroupV1Accounts},
    plugins::{create_meta_idempotent, Plugin, PluginType},
    state::{AssetV1, DataBlob, GroupV1, SolanaAccount},
    utils::{
        is_valid_asset_authority, is_valid_group_authority, resize_or_reallocate_account,
        resolve_authority, save_group_core_and_plugins,
    },
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
    if asset_accounts.len() != args.assets.len() {
        msg!("Error: account/pubkey mismatch");
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let mut group = GroupV1::load(group_info, 0)?;
    let old_group_core_len = group.len();
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

    save_group_core_and_plugins(
        group_info,
        &group,
        old_group_core_len,
        payer_info,
        system_program_info,
    )?;
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
        let record_offset = plugin_registry.registry[index].offset;
        let mut plugin = Plugin::load(asset_info, record_offset)?;
        if let Plugin::Groups(inner) = &mut plugin {
            if let Some(pos) = inner.groups.iter().position(|pk| pk == &parent_group) {
                inner.groups.remove(pos);
            } else {
                return Ok(());
            }
        } else {
            return Err(MplCoreError::InvalidPlugin.into());
        }

        let old_data =
            Plugin::deserialize(&mut &asset_info.data.borrow()[record_offset..])?.try_to_vec()?;
        let new_data = plugin.try_to_vec()?;
        let size_diff = (new_data.len() as isize)
            .checked_sub(old_data.len() as isize)
            .ok_or(MplCoreError::NumericalOverflow)?;
        if size_diff != 0 {
            let old_registry_offset = plugin_header.plugin_registry_offset;
            let next_offset = record_offset
                .checked_add(old_data.len())
                .ok_or(MplCoreError::NumericalOverflow)?;
            let new_next_offset: usize = (next_offset as isize)
                .checked_add(size_diff)
                .ok_or(MplCoreError::NumericalOverflow)?
                .try_into()
                .map_err(|_| MplCoreError::NumericalOverflow)?;

            plugin_registry.bump_offsets(record_offset, size_diff)?;
            plugin_header.plugin_registry_offset = (old_registry_offset as isize)
                .checked_add(size_diff)
                .ok_or(MplCoreError::NumericalOverflow)?
                .try_into()
                .map_err(|_| MplCoreError::NumericalOverflow)?;

            let new_size: usize = (asset_info.data_len() as isize)
                .checked_add(size_diff)
                .ok_or(MplCoreError::NumericalOverflow)?
                .try_into()
                .map_err(|_| MplCoreError::NumericalOverflow)?;

            // Copy the bytes that sit between the end of the plugin we just
            // updated/removed and the start of the plugin registry. In some
            // edge-cases (e.g. when the plugin we touched was the **last**
            // plugin in the account), there is nothing left to move which
            // would previously lead to arithmetic overflow when calculating
            // the length of the range to copy. Using checked math and gating
            // `sol_memmove` behind a size-check keeps the behavior identical
            // for the common case while rejecting invalid layouts safely.

            let copy_len = old_registry_offset
                .checked_sub(next_offset)
                .ok_or(MplCoreError::NumericalOverflow)?;

            // When shrinking, shift trailing bytes before reallocation so the
            // source region remains fully addressable.
            if size_diff < 0 && copy_len > 0 {
                unsafe {
                    let base_ptr = asset_info.data.borrow_mut().as_mut_ptr();
                    sol_memmove(
                        base_ptr.add(new_next_offset),
                        base_ptr.add(next_offset),
                        copy_len,
                    );
                }
            }

            resize_or_reallocate_account(asset_info, payer_info, system_program_info, new_size)?;

            // When growing, reallocate first so the destination region is
            // available for the move.
            if size_diff > 0 && copy_len > 0 {
                unsafe {
                    let base_ptr = asset_info.data.borrow_mut().as_mut_ptr();
                    sol_memmove(
                        base_ptr.add(new_next_offset),
                        base_ptr.add(next_offset),
                        copy_len,
                    );
                }
            }
        }
        plugin_header.save(asset_info, header_offset)?;
        plugin_registry.save(asset_info, plugin_header.plugin_registry_offset)?;
        plugin.save(asset_info, record_offset)?;
    }
    Ok(())
}
