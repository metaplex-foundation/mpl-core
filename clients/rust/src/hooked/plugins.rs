use std::{io::Error, io::ErrorKind};

use borsh::BorshDeserialize;

use crate::{
    accounts::{Asset, CollectionData, PluginHeader, PluginRegistry},
    errors::MplCoreError,
    types::{Authority, Plugin, PluginType, RegistryRecord},
};

/// Fetch the plugin from the registry.
pub fn fetch_plugin(
    account: &[u8],
    plugin_type: PluginType,
) -> Result<(Vec<Authority>, Plugin, u64), std::io::Error> {
    let asset = Asset::from_bytes(account)?;

    let header = PluginHeader::from_bytes(&account[asset.get_size()..])?;
    let PluginRegistry { registry, .. } =
        PluginRegistry::from_bytes(&account[(header.plugin_registry_offset as usize)..])?;

    // Find the plugin in the registry.
    let registry_record = registry
        .iter()
        .find(|record| record.plugin_type == plugin_type)
        .ok_or(Error::new(
            ErrorKind::Other,
            MplCoreError::PluginNotFound.to_string(),
        ))?;

    // Deserialize the plugin.
    let plugin = Plugin::deserialize(&mut &account[(registry_record.offset as usize)..])?;

    // Return the plugin and its authorities.
    Ok((
        registry_record.authorities.clone(),
        plugin,
        registry_record.offset,
    ))
}

/// Fetch the collection plugin from the registry.
pub fn fetch_collection_plugin(
    account: &[u8],
    plugin_type: PluginType,
) -> Result<(Vec<Authority>, Plugin, u64), std::io::Error> {
    let collection = CollectionData::from_bytes(account)?;

    let header = PluginHeader::from_bytes(&account[collection.get_size()..])?;
    let PluginRegistry { registry, .. } =
        PluginRegistry::from_bytes(&account[(header.plugin_registry_offset as usize)..])?;

    // Find the plugin in the registry.
    let registry_record = registry
        .iter()
        .find(|record| record.plugin_type == plugin_type)
        .ok_or(Error::new(
            ErrorKind::Other,
            MplCoreError::PluginNotFound.to_string(),
        ))?;

    // Deserialize the plugin.
    let plugin = Plugin::deserialize(&mut &account[(registry_record.offset as usize)..])?;

    // Return the plugin and its authorities.
    Ok((
        registry_record.authorities.clone(),
        plugin,
        registry_record.offset,
    ))
}

/// Fetch the plugin registry.
pub fn fetch_plugins(account: &[u8]) -> Result<Vec<RegistryRecord>, std::io::Error> {
    let asset = Asset::from_bytes(account)?;

    let header = PluginHeader::from_bytes(&account[asset.get_size()..])?;
    let PluginRegistry { registry, .. } =
        PluginRegistry::from_bytes(&account[(header.plugin_registry_offset as usize)..])?;

    Ok(registry)
}

/// Create plugin header and registry if it doesn't exist
pub fn list_plugins(account: &[u8]) -> Result<Vec<PluginType>, std::io::Error> {
    let asset = Asset::from_bytes(account)?;

    let header = PluginHeader::from_bytes(&account[asset.get_size()..])?;
    let PluginRegistry { registry, .. } =
        PluginRegistry::from_bytes(&account[(header.plugin_registry_offset as usize)..])?;

    Ok(registry
        .iter()
        .map(|registry_record| registry_record.plugin_type.clone())
        .collect())
}
