use borsh::BorshDeserialize;
use mpl_utils::resize_or_reallocate_account_raw;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};

use crate::{
    error::MplAssetError,
    state::{Asset, Authority, Key, PluginHeader},
    utils::DataBlob,
};

use super::{Plugin, PluginRegistry};

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
            version: 1,
            plugin_map_offset: asset.get_size() + PluginHeader::get_initial_size(),
        };
        let registry = PluginRegistry { registry: vec![] };

        resize_or_reallocate_account_raw(
            account,
            payer,
            system_program,
            header.plugin_map_offset + PluginRegistry::get_initial_size(),
        )?;

        header.save(account, asset.get_size())?;
        registry.save(account, header.plugin_map_offset)?;
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
    let PluginRegistry { registry } = PluginRegistry::load(account, header.plugin_map_offset)?;

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
    let PluginRegistry { registry } = PluginRegistry::load(account, header.plugin_map_offset)?;

    Ok(registry.iter().map(|(key, _)| *key).collect())
}
