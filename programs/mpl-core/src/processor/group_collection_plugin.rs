use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_memory::sol_memmove,
    pubkey::Pubkey,
};

use crate::{
    error::MplCoreError,
    plugins::{create_meta_idempotent, initialize_plugin, Groups, Plugin, PluginType},
    state::{CollectionV1, SolanaAccount},
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
                    // Already present, nothing to do.
                    return Ok(());
                }
                inner.groups.push(parent_group);
            } else {
                // This should never happen.
                return Err(MplCoreError::InvalidPlugin.into());
            }

            // Serialize old and new plugin to compute size diff.
            let old_plugin_data =
                Plugin::deserialize(&mut &collection_info.data.borrow()[record.offset..])?
                    .try_to_vec()?;
            let new_plugin_data = plugin.try_to_vec()?;
            let size_diff = (new_plugin_data.len() as isize)
                .checked_sub(old_plugin_data.len() as isize)
                .ok_or(MplCoreError::NumericalOverflow)?;

            if size_diff != 0 {
                // Bump offsets for subsequent registry entries and header.
                plugin_registry.bump_offsets(record.offset, size_diff)?;
                let new_registry_offset = (plugin_header.plugin_registry_offset as isize)
                    .checked_add(size_diff)
                    .ok_or(MplCoreError::NumericalOverflow)?;
                plugin_header.plugin_registry_offset = new_registry_offset as usize;

                // Resize account.
                let new_size = (collection_info.data_len() as isize)
                    .checked_add(size_diff)
                    .ok_or(MplCoreError::NumericalOverflow)?
                    as usize;
                resize_or_reallocate_account(
                    collection_info,
                    payer_info,
                    system_program_info,
                    new_size,
                )?;

                // Move trailing data to accommodate new plugin size.
                let next_plugin_offset = (record.offset + old_plugin_data.len()) as isize;
                let new_next_plugin_offset = next_plugin_offset + size_diff;

                let copy_len = plugin_header
                    .plugin_registry_offset
                    .saturating_sub(next_plugin_offset as usize);

                if copy_len > 0 {
                    unsafe {
                        let base_ptr = collection_info.data.borrow_mut().as_mut_ptr();
                        sol_memmove(
                            base_ptr.add(new_next_plugin_offset as usize),
                            base_ptr.add(next_plugin_offset as usize),
                            copy_len,
                        );
                    }
                }
            }

            // Save header, registry and new plugin.
            plugin_header.save(collection_info, header_offset)?;
            plugin_registry.save(collection_info, plugin_header.plugin_registry_offset)?;
            plugin.save(collection_info, record.offset)?;
        }
    }

    Ok(())
}
