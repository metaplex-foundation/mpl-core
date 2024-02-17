use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    error::MplAssetError,
    state::{Asset, Key, PluginHeader},
    utils::DataBlob,
};

use super::{Plugin, PluginRegistry};

//TODO:keith: Implement this function
/// Create plugin header and registry if it doesn't exist
pub fn create_idempotent(account: &AccountInfo) -> ProgramResult {
    let mut bytes: &[u8] = &(*account.data).borrow();
    let asset = Asset::deserialize(&mut bytes)?;

    // Check if the plugin header and registry exist.
    if asset.get_size() == account.data_len() {
        // They don't exist, so create them.
        let header = PluginHeader {
            version: 1,
            plugin_map_offset: asset.get_size() + PluginHeader::get_initial_size(),
        };
        let registry = PluginRegistry { registry: vec![] };

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

//TODO:keith: Implement this function
pub fn fetch_plugin(
    account: &AccountInfo,
    plugin: Key,
) -> Result<((Vec<Pubkey>, Plugin)), ProgramError> {
    // Fetch the plugin from the registry.
    let mut bytes: &[u8] = &(*account.data).borrow();
    let asset = Asset::deserialize(&mut bytes)?;

    let header = PluginHeader::load(account, asset.get_size())?;
    let PluginRegistry { registry } = PluginRegistry::load(account, header.plugin_map_offset)?;

    let plugin_data = registry
        .iter()
        .find(|(key, _)| *key == plugin)
        .map(|(_, data)| data)
        .ok_or(MplAssetError::PluginNotFound)?;

    let authorities = plugin_data
        .authorities
        .iter()
        .map(|authority| authority.key)
        .collect();
}

//TODO:keith: Implement this function
pub fn list_plugins() -> Vec<u8> {
    // Create plugin header and registry if it doesn't exist
    vec![]
}
