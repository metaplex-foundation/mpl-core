use borsh::BorshDeserialize;
use solana_program::account_info::AccountInfo;

use crate::{
    accounts::{Asset, PluginHeader, PluginRegistry},
    errors::MplCoreError,
    types::{Authority, Plugin, PluginType, RegistryRecord},
    DataBlob, SolanaAccount,
};

/// Fetch the plugin from the registry.
pub fn fetch_plugin<T: DataBlob + SolanaAccount, U: BorshDeserialize>(
    account: &AccountInfo,
    plugin_type: PluginType,
) -> Result<(Vec<Authority>, U, usize), std::io::Error> {
    let asset = T::load(account, 0)?;

    let header = PluginHeader::load(account, asset.get_size())?;
    let PluginRegistry { registry, .. } =
        PluginRegistry::load(account, header.plugin_registry_offset as usize)?;

    // Find the plugin in the registry.
    let registry_record = registry
        .iter()
        .find(|record| record.plugin_type == plugin_type)
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            MplCoreError::PluginNotFound.to_string(),
        ))?;

    // Deserialize the plugin.
    let plugin =
        Plugin::deserialize(&mut &(*account.data).borrow()[(registry_record.offset as usize)..])?;

    if PluginType::from(&plugin) != plugin_type {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            MplCoreError::PluginNotFound.to_string(),
        ));
    }

    let inner = U::deserialize(
        &mut &(*account.data).borrow()[registry_record.offset.checked_add(1).ok_or(
            std::io::Error::new(
                std::io::ErrorKind::Other,
                MplCoreError::NumericalOverflow.to_string(),
            ),
        )? as usize..],
    )?;

    // Return the plugin and its authorities.
    Ok((
        registry_record.authorities.clone(),
        inner,
        registry_record.offset as usize,
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
