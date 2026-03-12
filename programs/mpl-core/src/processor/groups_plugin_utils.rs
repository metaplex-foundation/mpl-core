use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_memory::sol_memmove,
    pubkey::Pubkey,
};

use crate::{
    error::MplCoreError,
    plugins::{
        create_meta_idempotent, initialize_plugin, Groups, Plugin, PluginHeaderV1, PluginType,
        PluginRegistryV1, RegistryRecord,
    },
    state::{AssetV1, CollectionV1, SolanaAccount},
    utils::resize_or_reallocate_account,
};

/// Add the parent group pubkey to the collection's Groups plugin, creating the plugin if necessary.
pub(crate) fn process_collection_groups_plugin_add<'a>(
    collection_info: &AccountInfo<'a>,
    parent_group: Pubkey,
    payer_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
) -> ProgramResult {
    // Ensure plugin metadata exists or create.
    let (_collection_core, header_offset, mut plugin_header, mut plugin_registry) =
        create_meta_idempotent::<CollectionV1>(collection_info, payer_info, system_program_info)?;

    // Attempt to fetch existing Groups plugin.
    let plugin_record_opt = plugin_registry
        .registry
        .iter()
        .find(|r| r.plugin_type == PluginType::Groups)
        .cloned();

    match plugin_record_opt {
        None => {
            // Plugin does not exist; create it.
            let groups_plugin = Groups {
                groups: vec![parent_group],
            };
            let plugin = Plugin::Groups(groups_plugin);
            initialize_plugin::<CollectionV1>(
                &plugin,
                &plugin.manager(),
                header_offset,
                &mut plugin_header,
                &mut plugin_registry,
                collection_info,
                payer_info,
                system_program_info,
            )?
        }
        Some(record) => {
            // Plugin exists, load, modify, and update.
            let mut plugin = Plugin::load(collection_info, record.offset)?;
            if let Plugin::Groups(inner) = &mut plugin {
                if inner.groups.contains(&parent_group) {
                    return Ok(());
                }
                inner.groups.push(parent_group);
            } else {
                return Err(MplCoreError::InvalidPlugin.into());
            }

            save_updated_groups_plugin(collection_info, payer_info, system_program_info, &plugin, &record, &mut plugin_header, &mut plugin_registry, header_offset)?;
        }
    }

    Ok(())
}

/// Add the parent group pubkey to the asset's Groups plugin, creating the plugin if necessary.
pub(crate) fn process_asset_groups_plugin_add<'a>(
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
                if inner.groups.contains(&parent_group) {
                    return Ok(());
                }
                inner.groups.push(parent_group);
            } else {
                return Err(MplCoreError::InvalidPlugin.into());
            }

            save_updated_groups_plugin(asset_info, payer_info, system_program_info, &plugin, &record, &mut plugin_header, &mut plugin_registry, header_offset)?;
        }
    }
    Ok(())
}

/// Shared helper that persists a modified Groups plugin back to an account,
/// handling the resize, memmove, and registry offset bump when the serialized
/// size changes. Uses shrink-before-move / grow-after-move ordering so that
/// source bytes are always valid during `sol_memmove`.
pub(crate) fn save_updated_groups_plugin<'a>(
    account_info: &AccountInfo<'a>,
    payer_info: &AccountInfo<'a>,
    system_program_info: &AccountInfo<'a>,
    plugin: &Plugin,
    record: &RegistryRecord,
    plugin_header: &mut PluginHeaderV1,
    plugin_registry: &mut PluginRegistryV1,
    header_offset: usize,
) -> ProgramResult {
    let old_plugin_data =
        Plugin::deserialize(&mut &account_info.data.borrow()[record.offset..])?
            .try_to_vec()?;
    let new_plugin_data = plugin.try_to_vec()?;
    let size_diff = (new_plugin_data.len() as isize)
        .checked_sub(old_plugin_data.len() as isize)
        .ok_or(MplCoreError::NumericalOverflow)?;

    if size_diff != 0 {
        let old_registry_offset = plugin_header.plugin_registry_offset;
        let next_plugin_offset = record
            .offset
            .checked_add(old_plugin_data.len())
            .ok_or(MplCoreError::NumericalOverflow)?;
        let new_next_plugin_offset: usize = (next_plugin_offset as isize)
            .checked_add(size_diff)
            .ok_or(MplCoreError::NumericalOverflow)?
            .try_into()
            .map_err(|_| MplCoreError::NumericalOverflow)?;

        plugin_registry.bump_offsets(record.offset, size_diff)?;
        plugin_header.plugin_registry_offset = (old_registry_offset as isize)
            .checked_add(size_diff)
            .ok_or(MplCoreError::NumericalOverflow)?
            .try_into()
            .map_err(|_| MplCoreError::NumericalOverflow)?;

        let new_size: usize = (account_info.data_len() as isize)
            .checked_add(size_diff)
            .ok_or(MplCoreError::NumericalOverflow)?
            .try_into()
            .map_err(|_| MplCoreError::NumericalOverflow)?;

        let copy_len = old_registry_offset
            .checked_sub(next_plugin_offset)
            .ok_or(MplCoreError::NumericalOverflow)?;

        // When shrinking, move data first while the full source region is still valid.
        if size_diff < 0 && copy_len > 0 {
            unsafe {
                let mut data = account_info.data.borrow_mut();
                let base_ptr = data.as_mut_ptr();
                sol_memmove(
                    base_ptr.add(new_next_plugin_offset),
                    base_ptr.add(next_plugin_offset),
                    copy_len,
                );
            }
        }

        resize_or_reallocate_account(
            account_info,
            payer_info,
            system_program_info,
            new_size,
        )?;

        // When growing, move data after reallocation so the destination region exists.
        if size_diff > 0 && copy_len > 0 {
            unsafe {
                let mut data = account_info.data.borrow_mut();
                let base_ptr = data.as_mut_ptr();
                sol_memmove(
                    base_ptr.add(new_next_plugin_offset),
                    base_ptr.add(next_plugin_offset),
                    copy_len,
                );
            }
        }
    }

    plugin_header.save(account_info, header_offset)?;
    plugin_registry.save(account_info, plugin_header.plugin_registry_offset)?;
    plugin.save(account_info, record.offset)?;

    Ok(())
}
