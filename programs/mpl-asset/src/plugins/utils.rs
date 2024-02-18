use borsh::{BorshDeserialize, BorshSerialize};
use mpl_utils::resize_or_reallocate_account_raw;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};

use crate::{
    error::MplAssetError,
    state::{Asset, Authority, Key, PluginHeader},
    utils::DataBlob,
};

use super::{Plugin, PluginRegistry, RegistryData};

/// Create plugin header and registry if it doesn't exist
pub fn create_idempotent<'a>(
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

pub fn assert_plugins_initialized(account: &AccountInfo) -> ProgramResult {
    let mut bytes: &[u8] = &(*account.data).borrow();
    let asset = Asset::deserialize(&mut bytes).unwrap();

    if asset.get_size() == account.data_len() {
        return Err(MplAssetError::PluginsNotInitialized.into());
    }

    Ok(())
}

/// Fetch the plugin from the registry.
pub fn fetch_plugin(
    account: &AccountInfo,
    key: Key,
) -> Result<(Vec<Authority>, Plugin), ProgramError> {
    let mut bytes: &[u8] = &(*account.data).borrow();
    let asset = Asset::deserialize(&mut bytes)?;

    let header = PluginHeader::load(account, asset.get_size())?;
    let PluginRegistry { registry, .. } =
        PluginRegistry::load(account, header.plugin_registry_offset)?;

    // Find the plugin in the registry.
    let plugin_data = registry
        .iter()
        .find(|(plugin_key, _)| *plugin_key == key)
        .map(|(_, data)| data)
        .ok_or(MplAssetError::PluginNotFound)?;

    // Deserialize the plugin.
    let plugin = Plugin::deserialize(&mut &(*account.data).borrow()[plugin_data.offset..])?;

    // Return the plugin and its authorities.
    Ok((plugin_data.authorities.clone(), plugin))
}

/// Create plugin header and registry if it doesn't exist
pub fn list_plugins(account: &AccountInfo) -> Result<Vec<Key>, ProgramError> {
    let mut bytes: &[u8] = &(*account.data).borrow();
    let asset = Asset::deserialize(&mut bytes)?;

    let header = PluginHeader::load(account, asset.get_size())?;
    let PluginRegistry { registry, .. } =
        PluginRegistry::load(account, header.plugin_registry_offset)?;

    Ok(registry.iter().map(|(key, _)| *key).collect())
}

/// Add a plugin into the registry
pub fn add_plugin<'a>(
    plugin: &Plugin,
    authority: Authority,
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

    let (plugin_size, key) = match plugin {
        Plugin::Reserved => todo!(),
        Plugin::Royalties => todo!(),
        Plugin::MasterEdition => todo!(),
        Plugin::PrintEdition => todo!(),
        Plugin::Delegate(delegate) => (delegate.get_size(), Key::Delegate),
        Plugin::Inscription => todo!(),
    };

    if let Some((_, registry_data)) = plugin_registry
        .registry
        .iter_mut()
        .find(|(search_key, _)| search_key == &key)
    {
        registry_data.authorities.push(authority.clone());

        let authority_bytes = authority.try_to_vec()?;

        let new_size = account
            .data_len()
            .checked_add(authority_bytes.len())
            .ok_or(MplAssetError::NumericalOverflow)?;
        resize_or_reallocate_account_raw(account, payer, system_program, new_size)?;

        plugin_registry.save(account, header.plugin_registry_offset)?;
    } else {
        let authority_bytes = authority.try_to_vec()?;

        let new_size = account
            .data_len()
            .checked_add(plugin_size)
            .ok_or(MplAssetError::NumericalOverflow)?
            .checked_add(authority_bytes.len())
            .ok_or(MplAssetError::NumericalOverflow)?;

        let old_registry_offset = header.plugin_registry_offset;
        let new_registry_offset = header
            .plugin_registry_offset
            .checked_add(plugin_size)
            .ok_or(MplAssetError::NumericalOverflow)?
            .checked_add(authority_bytes.len())
            .ok_or(MplAssetError::NumericalOverflow)?;

        header.plugin_registry_offset = new_registry_offset;
        plugin_registry.registry.push((
            key,
            RegistryData {
                offset: old_registry_offset,
                authorities: vec![authority],
            },
        ));

        resize_or_reallocate_account_raw(account, payer, system_program, new_size)?;

        header.save(account, asset.get_size())?;
        match plugin {
            Plugin::Reserved => todo!(),
            Plugin::Royalties => todo!(),
            Plugin::MasterEdition => todo!(),
            Plugin::PrintEdition => todo!(),
            Plugin::Delegate(delegate) => delegate.save(account, old_registry_offset)?,
            Plugin::Inscription => todo!(),
        };

        plugin_registry.save(account, new_registry_offset)?;
    }

    Ok(())
}
