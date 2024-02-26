use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::resize_or_reallocate_account_raw;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    program_memory::sol_memcpy,
};

use crate::{
    error::MplCoreError,
    state::{Asset, Authority, DataBlob, Key, SolanaAccount},
    utils::{assert_authority, resolve_authority_to_default},
};

use super::{Plugin, PluginHeader, PluginRegistry, PluginType, RegistryData, RegistryRecord};

/// Create plugin header and registry if it doesn't exist
pub fn create_meta_idempotent<'a>(
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    let asset = {
        let mut bytes: &[u8] = &(*account.data).borrow();
        Asset::deserialize(&mut bytes)?
    };

    // Check if the plugin header and registry exist.
    if asset.get_size() == account.data_len() {
        // They don't exist, so create them.
        let header = PluginHeader {
            key: Key::PluginHeader,
            plugin_registry_offset: asset.get_size() + PluginHeader::get_initial_size(),
        };
        let registry = PluginRegistry {
            key: Key::PluginRegistry,
            registry: vec![],
            external_plugins: vec![],
        };

        resize_or_reallocate_account_raw(
            account,
            payer,
            system_program,
            header.plugin_registry_offset + PluginRegistry::get_initial_size(),
        )?;

        header.save(account, asset.get_size())?;
        registry.save(account, header.plugin_registry_offset)?;
    }

    Ok(())
}

/// Assert that the Plugin metadata has been initialized.
pub fn assert_plugins_initialized(account: &AccountInfo) -> ProgramResult {
    let mut bytes: &[u8] = &(*account.data).borrow();
    let asset = Asset::deserialize(&mut bytes).unwrap();

    if asset.get_size() == account.data_len() {
        return Err(MplCoreError::PluginsNotInitialized.into());
    }

    Ok(())
}

/// Fetch the plugin from the registry.
pub fn fetch_plugin(
    account: &AccountInfo,
    plugin_type: PluginType,
) -> Result<(Vec<Authority>, Plugin, usize), ProgramError> {
    let asset = Asset::load(account, 0)?;

    let header = PluginHeader::load(account, asset.get_size())?;
    let PluginRegistry { registry, .. } =
        PluginRegistry::load(account, header.plugin_registry_offset)?;

    // Find the plugin in the registry.
    let plugin_data = registry
        .iter()
        .find(
            |RegistryRecord {
                 plugin_type: plugin_type_iter,
                 data: _,
             }| *plugin_type_iter == plugin_type,
        )
        .map(
            |RegistryRecord {
                 plugin_type: _,
                 data,
             }| data,
        )
        .ok_or(MplCoreError::PluginNotFound)?;

    // Deserialize the plugin.
    let plugin = Plugin::deserialize(&mut &(*account.data).borrow()[plugin_data.offset..])?;

    // Return the plugin and its authorities.
    Ok((plugin_data.authorities.clone(), plugin, plugin_data.offset))
}

/// Fetch the plugin registry.
pub fn fetch_plugins(account: &AccountInfo) -> Result<Vec<RegistryRecord>, ProgramError> {
    let asset = Asset::load(account, 0)?;

    let header = PluginHeader::load(account, asset.get_size())?;
    let PluginRegistry { registry, .. } =
        PluginRegistry::load(account, header.plugin_registry_offset)?;

    Ok(registry)
}

/// Create plugin header and registry if it doesn't exist
pub fn list_plugins(account: &AccountInfo) -> Result<Vec<PluginType>, ProgramError> {
    let asset = Asset::load(account, 0)?;

    let header = PluginHeader::load(account, asset.get_size())?;
    let PluginRegistry { registry, .. } =
        PluginRegistry::load(account, header.plugin_registry_offset)?;

    Ok(registry
        .iter()
        .map(|registry_record| registry_record.plugin_type)
        .collect())
}

/// Add a plugin to the registry and initialize it.
pub fn initialize_plugin<'a>(
    plugin: &Plugin,
    authority: &Authority,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    let asset = {
        let mut bytes: &[u8] = &(*account.data).borrow();
        Asset::deserialize(&mut bytes)?
    };

    //TODO: Bytemuck this.
    let mut header = PluginHeader::load(account, asset.get_size())?;
    let mut plugin_registry = PluginRegistry::load(account, header.plugin_registry_offset)?;

    let plugin_type = plugin.into();
    let plugin_data = plugin.try_to_vec()?;
    let plugin_size = plugin_data.len();
    // let authority_bytes = authority.try_to_vec()?;

    if let Some(RegistryRecord {
        plugin_type: _,
        data: _,
    }) = plugin_registry.registry.iter_mut().find(
        |RegistryRecord {
             plugin_type: type_iter,
             data: _,
         }| type_iter == &plugin_type,
    ) {
        return Err(MplCoreError::PluginAlreadyExists.into());
    }

    let old_registry_offset = header.plugin_registry_offset;
    let registry_data = RegistryData {
        offset: old_registry_offset,
        authorities: vec![authority.clone()],
    };
    let size_increase = plugin_size
        .checked_add(Key::get_initial_size())
        .ok_or(MplCoreError::NumericalOverflow)?
        .checked_add(registry_data.clone().try_to_vec()?.len())
        .ok_or(MplCoreError::NumericalOverflow)?;
    let new_registry_offset = header
        .plugin_registry_offset
        .checked_add(plugin_size)
        .ok_or(MplCoreError::NumericalOverflow)?;
    header.plugin_registry_offset = new_registry_offset;
    plugin_registry.registry.push(RegistryRecord {
        plugin_type,
        data: registry_data.clone(),
    });
    let new_size = account
        .data_len()
        .checked_add(size_increase)
        .ok_or(MplCoreError::NumericalOverflow)?;
    resize_or_reallocate_account_raw(account, payer, system_program, new_size)?;
    header.save(account, asset.get_size())?;
    plugin.save(account, old_registry_offset)?;
    plugin_registry.save(account, new_registry_offset)?;

    Ok(())
}

/// Remove a plugin from the registry and delete it.
//TODO: Do the work to prevent deleting a plugin with other authorities.
pub fn delete_plugin<'a>(
    plugin_type: &PluginType,
    asset: &Asset,
    authority: &AccountInfo<'a>,
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    //TODO: Bytemuck this.
    let mut header = PluginHeader::load(account, asset.get_size())?;
    let mut plugin_registry = PluginRegistry::load(account, header.plugin_registry_offset)?;

    if let Some(index) = plugin_registry.registry.iter_mut().position(
        |RegistryRecord {
             plugin_type: type_iter,
             data: _,
         }| type_iter == plugin_type,
    ) {
        let registry_record = plugin_registry.registry.remove(index);
        let serialized_registry_record = registry_record.try_to_vec()?;

        // Only allow the default authority to delete the plugin.
        let authorities = registry_record.data.authorities;

        let resolved_authority = resolve_authority_to_default(asset, authority);
        if resolved_authority != authorities[0] {
            return Err(MplCoreError::InvalidAuthority.into());
        }

        let plugin_offset = registry_record.data.offset;
        let plugin = Plugin::load(account, plugin_offset)?;
        let serialized_plugin = plugin.try_to_vec()?;

        let next_plugin_offset = plugin_offset
            .checked_add(serialized_plugin.len())
            .ok_or(MplCoreError::NumericalOverflow)?;

        let new_size = account
            .data_len()
            .checked_sub(serialized_registry_record.len())
            .ok_or(MplCoreError::NumericalOverflow)?
            .checked_sub(serialized_plugin.len())
            .ok_or(MplCoreError::NumericalOverflow)?;

        let new_offset = header
            .plugin_registry_offset
            .checked_sub(serialized_plugin.len())
            .ok_or(MplCoreError::NumericalOverflow)?;

        let data_to_move = header
            .plugin_registry_offset
            .checked_sub(new_offset)
            .ok_or(MplCoreError::NumericalOverflow)?;

        //TODO: This is memory intensive, we should use memmove instead probably.
        let src = account.data.borrow()[next_plugin_offset..].to_vec();
        sol_memcpy(
            &mut account.data.borrow_mut()[plugin_offset..],
            &src,
            data_to_move,
        );

        header.plugin_registry_offset = new_offset;
        header.save(account, asset.get_size())?;

        plugin_registry.save(account, new_offset)?;

        resize_or_reallocate_account_raw(account, payer, system_program, new_size)?;
    } else {
        return Err(MplCoreError::PluginNotFound.into());
    }

    Ok(())
}

/// Add an authority to a plugin.
//TODO: Prevent duplicate authorities.
#[allow(clippy::too_many_arguments)]
pub fn add_authority_to_plugin<'a>(
    plugin_type: &PluginType,
    authority: &AccountInfo<'a>,
    new_authority: &Authority,
    asset: &Asset,
    account: &AccountInfo<'a>,
    plugin_header: &PluginHeader,
    plugin_registry: &mut PluginRegistry,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    let authorities = &plugin_registry
        .registry
        .iter_mut()
        .find(
            |RegistryRecord {
                 plugin_type: type_iter,
                 data: _,
             }| type_iter == plugin_type,
        )
        .ok_or(MplCoreError::PluginNotFound)?
        .data
        .authorities;

    assert_authority(asset, authority, authorities)?;

    if let Some(RegistryRecord {
        plugin_type: _,
        data: registry_data,
    }) = plugin_registry.registry.iter_mut().find(
        |RegistryRecord {
             plugin_type: type_iter,
             data: _,
         }| type_iter == plugin_type,
    ) {
        registry_data.authorities.push(new_authority.clone());

        let authority_bytes = new_authority.try_to_vec()?;

        let new_size = account
            .data_len()
            .checked_add(authority_bytes.len())
            .ok_or(MplCoreError::NumericalOverflow)?;
        resize_or_reallocate_account_raw(account, payer, system_program, new_size)?;

        plugin_registry.save(account, plugin_header.plugin_registry_offset)?;
    } else {
        return Err(MplCoreError::PluginNotFound.into());
    }

    Ok(())
}

/// Remove an authority from a plugin.
#[allow(clippy::too_many_arguments)]
pub fn remove_authority_from_plugin<'a>(
    plugin_type: &PluginType,
    authority: &AccountInfo<'a>,
    authority_to_remove: &Authority,
    asset: &Asset,
    account: &AccountInfo<'a>,
    plugin_header: &PluginHeader,
    plugin_registry: &mut PluginRegistry,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
) -> ProgramResult {
    let authorities = &plugin_registry
        .registry
        .iter_mut()
        .find(
            |RegistryRecord {
                 plugin_type: type_iter,
                 data: _,
             }| type_iter == plugin_type,
        )
        .ok_or(MplCoreError::PluginNotFound)?
        .data
        .authorities;

    let resolved_authority = resolve_authority_to_default(asset, authority);
    if resolved_authority != authorities[0] {
        return Err(MplCoreError::InvalidAuthority.into());
    }

    if let Some(RegistryRecord {
        plugin_type: _,
        data: registry_data,
    }) = plugin_registry.registry.iter_mut().find(
        |RegistryRecord {
             plugin_type: type_iter,
             data: _,
         }| type_iter == plugin_type,
    ) {
        let index = registry_data
            .authorities
            .iter()
            .position(|auth| auth == authority_to_remove)
            .ok_or(MplCoreError::InvalidAuthority)?;

        // Here we replace the default authority with None to indicate it's been removed.
        if index == 0 {
            registry_data.authorities[0] = Authority::None;
            plugin_registry.save(account, plugin_header.plugin_registry_offset)?;
        } else {
            registry_data.authorities.swap_remove(index);

            let authority_bytes = authority_to_remove.try_to_vec()?;

            let new_size = account
                .data_len()
                .checked_sub(authority_bytes.len())
                .ok_or(MplCoreError::NumericalOverflow)?;
            resize_or_reallocate_account_raw(account, payer, system_program, new_size)?;

            plugin_registry.save(account, plugin_header.plugin_registry_offset)?;
        }
    } else {
        return Err(MplCoreError::PluginNotFound.into());
    }

    Ok(())
}
