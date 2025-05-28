use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::assert_signer;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_memory::{sol_memcpy, sol_memmove},
    pubkey::Pubkey,
};

use crate::{
    error::MplCoreError,
    plugins::{
        create_meta_idempotent, initialize_plugin, Groups, Plugin, PluginHeaderV1,
        PluginRegistryV1, PluginType,
    },
    state::{AssetV1, DataBlob, GroupV1, SolanaAccount},
    utils::{is_valid_group_authority, resize_or_reallocate_account, resolve_authority},
};

/// Arguments for the `AddAssetsToGroupV1` instruction.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct AddAssetsToGroupV1Args {
    /// The list of assets to add to the group.
    pub(crate) assets: Vec<Pubkey>,
}

/// Returns true if the authority is allowed to mutate the asset (update authority or delegate).
fn is_valid_asset_authority(
    asset_info: &AccountInfo,
    authority_info: &AccountInfo,
) -> ProgramResult {
    use crate::plugins::fetch_wrapped_plugin;
    use crate::plugins::PluginType;
    use crate::state::UpdateAuthority;

    let asset_core = AssetV1::load(asset_info, 0)?;

    // Direct update authority address.
    if let UpdateAuthority::Address(addr) = asset_core.update_authority.clone() {
        if addr == *authority_info.key {
            return Ok(());
        }
    }

    // Check UpdateDelegate plugin.
    if let Ok((_pa, plugin)) =
        fetch_wrapped_plugin::<AssetV1>(asset_info, Some(&asset_core), PluginType::UpdateDelegate)
    {
        if let crate::plugins::Plugin::UpdateDelegate(update_delegate) = plugin {
            if update_delegate
                .additional_delegates
                .contains(authority_info.key)
            {
                return Ok(());
            }
        }
    }

    Err(MplCoreError::InvalidAuthority.into())
}

/// Processor for the `AddAssetsToGroupV1` instruction.
#[allow(clippy::too_many_arguments)]
pub(crate) fn add_assets_to_group_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    args: AddAssetsToGroupV1Args,
) -> ProgramResult {
    // Layout: 0 group (writable), 1 payer signer, 2 optional authority signer, 3 system program, 4..N asset accounts
    if accounts.len() < 4 {
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    let group_info = &accounts[0];
    let payer_info = &accounts[1];
    let authority_info_opt = accounts.get(2);
    let system_program_info = &accounts[3];
    let asset_accounts = &accounts[4..];

    assert_signer(payer_info)?;
    let authority_info = resolve_authority(payer_info, authority_info_opt)?;

    if system_program_info.key != &solana_program::system_program::ID {
        return Err(MplCoreError::InvalidSystemProgram.into());
    }

    if asset_accounts.len() != args.assets.len() {
        msg!("Error: Mismatch between asset accounts and provided pubkeys");
        return Err(ProgramError::NotEnoughAccountKeys);
    }

    // Load group
    let mut group = GroupV1::load(group_info, 0)?;

    // Ensure authority for group
    if !is_valid_group_authority(group_info, authority_info)? {
        msg!("Error: Invalid authority for group");
        return Err(MplCoreError::InvalidAuthority.into());
    }

    // Iterate assets
    for (i, asset_info) in asset_accounts.iter().enumerate() {
        if asset_info.key != &args.assets[i] {
            msg!("Error: Asset account order mismatch at index {}", i);
            return Err(MplCoreError::IncorrectAccount.into());
        }
        if !asset_info.is_writable {
            msg!("Error: Asset account must be writable");
            return Err(ProgramError::InvalidAccountData);
        }

        // authority must be valid for asset
        is_valid_asset_authority(asset_info, authority_info)?;

        // Update group.assets vector
        if !group.assets.contains(asset_info.key) {
            group.assets.push(*asset_info.key);
        } else {
            // already present, but still update plugin to ensure membership
        }

        // Ensure or update Groups plugin on asset
        process_asset_groups_plugin_add(
            asset_info,
            *group_info.key,
            payer_info,
            system_program_info,
        )?;
    }

    // Save group changes
    let serialized_group = group.try_to_vec()?;
    if serialized_group.len() != group_info.data_len() {
        resize_or_reallocate_account(
            group_info,
            payer_info,
            system_program_info,
            serialized_group.len(),
        )?;
    }
    sol_memcpy(
        &mut group_info.try_borrow_mut_data()?,
        &serialized_group,
        serialized_group.len(),
    );

    Ok(())
}

fn process_asset_groups_plugin_add<'a>(
    asset_info: &AccountInfo<'a>,
    parent_group: Pubkey,
    payer_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
) -> ProgramResult {
    let (_asset_core, header_offset, mut plugin_header, mut plugin_registry) =
        create_meta_idempotent::<AssetV1>(asset_info, payer_info, system_program_info)?;

    let plugin_record_opt = plugin_registry
        .registry
        .iter()
        .find(|r| r.plugin_type == PluginType::Groups)
        .cloned();

    match plugin_record_opt {
        None => {
            let plugin = Plugin::Groups(Groups {
                groups: vec![parent_group],
            });
            initialize_plugin::<AssetV1>(
                &plugin,
                &plugin.manager(),
                header_offset,
                &mut plugin_header,
                &mut plugin_registry,
                asset_info,
                payer_info,
                system_program_info,
            )?;
        }
        Some(record) => {
            let mut plugin = Plugin::load(asset_info, record.offset)?;
            if let Plugin::Groups(inner) = &mut plugin {
                if !inner.groups.contains(&parent_group) {
                    inner.groups.push(parent_group);
                } else {
                    return Ok(());
                }
            } else {
                return Err(MplCoreError::InvalidPlugin.into());
            }

            // serialize adjustments
            let old_data = Plugin::deserialize(&mut &asset_info.data.borrow()[record.offset..])?
                .try_to_vec()?;
            let new_data = plugin.try_to_vec()?;
            let size_diff = new_data.len() as isize - old_data.len() as isize;
            if size_diff != 0 {
                plugin_registry.bump_offsets(record.offset, size_diff)?;
                plugin_header.plugin_registry_offset =
                    (plugin_header.plugin_registry_offset as isize + size_diff) as usize;

                let new_size = (asset_info.data_len() as isize + size_diff) as usize;
                resize_or_reallocate_account(
                    asset_info,
                    payer_info,
                    system_program_info,
                    new_size,
                )?;

                let next_offset = (record.offset + old_data.len()) as isize;
                let new_next_offset = next_offset + size_diff;
                unsafe {
                    let base_ptr = asset_info.data.borrow_mut().as_mut_ptr();
                    sol_memmove(
                        base_ptr.add(new_next_offset as usize),
                        base_ptr.add(next_offset as usize),
                        plugin_header.plugin_registry_offset - next_offset as usize,
                    );
                }
            }
            plugin_header.save(asset_info, header_offset)?;
            plugin_registry.save(asset_info, plugin_header.plugin_registry_offset)?;
            plugin.save(asset_info, record.offset)?;
        }
    }
    Ok(())
}
