use borsh::BorshDeserialize;
use solana_program::account_info::AccountInfo;

use crate::{
    accounts::{BaseAsset, PluginHeader, PluginRegistry},
    errors::MplCoreError,
    types::{Authority, Plugin, PluginType, RegistryRecord},
    AttributesPlugin, BaseAuthority, BasePlugin, BurnPlugin, DataBlob, FreezePlugin,
    PermanentFreezePlugin, PluginsList, RoyaltiesPlugin, SolanaAccount, TransferPlugin,
    UpdateDelegatePlugin,
};

/// Fetch the plugin from the registry.
pub fn fetch_plugin<T: DataBlob + SolanaAccount, U: BorshDeserialize>(
    account: &AccountInfo,
    plugin_type: PluginType,
) -> Result<(Authority, U, usize), std::io::Error> {
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
        registry_record.authority.clone(),
        inner,
        registry_record.offset as usize,
    ))
}

/// Fetch the plugin registry.
pub fn fetch_plugins(account: &[u8]) -> Result<Vec<RegistryRecord>, std::io::Error> {
    let asset = BaseAsset::from_bytes(account)?;

    let header = PluginHeader::from_bytes(&account[asset.get_size()..])?;
    let PluginRegistry { registry, .. } =
        PluginRegistry::from_bytes(&account[(header.plugin_registry_offset as usize)..])?;

    Ok(registry)
}

/// Create plugin header and registry if it doesn't exist
pub fn list_plugins(account: &[u8]) -> Result<Vec<PluginType>, std::io::Error> {
    let asset = BaseAsset::from_bytes(account)?;

    let header = PluginHeader::from_bytes(&account[asset.get_size()..])?;
    let PluginRegistry { registry, .. } =
        PluginRegistry::from_bytes(&account[(header.plugin_registry_offset as usize)..])?;

    Ok(registry
        .iter()
        .map(|registry_record| registry_record.plugin_type.clone())
        .collect())
}

pub fn registry_records_to_plugin_list(
    registry_records: &[RegistryRecord],
    account_data: &[u8],
) -> Result<PluginsList, std::io::Error> {
    let result = registry_records
        .iter()
        .try_fold(PluginsList::default(), |mut acc, record| {
            let authority: BaseAuthority = record.authority.clone().into();
            let base = BasePlugin {
                authority,
                offset: Some(record.offset),
            };
            let plugin = Plugin::deserialize(&mut &account_data[record.offset as usize..])?;

            match plugin {
                Plugin::Reserved => (),
                Plugin::Royalties(royalties) => {
                    acc.royalties = Some(RoyaltiesPlugin { base, royalties });
                }
                Plugin::Freeze(freeze) => {
                    acc.freeze = Some(FreezePlugin { base, freeze });
                }
                Plugin::Burn(burn) => {
                    acc.burn = Some(BurnPlugin { base, burn });
                }
                Plugin::Transfer(transfer) => {
                    acc.transfer = Some(TransferPlugin { base, transfer });
                }
                Plugin::UpdateDelegate(update_delegate) => {
                    acc.update_delegate = Some(UpdateDelegatePlugin {
                        base,
                        update_delegate,
                    });
                }
                Plugin::PermanentFreeze(permanent_freeze) => {
                    acc.permanent_freeze = Some(PermanentFreezePlugin {
                        base,
                        permanent_freeze,
                    });
                }
                Plugin::Attributes(attributes) => {
                    acc.attributes = Some(AttributesPlugin { base, attributes });
                }
            };

            Ok(acc)
        });

    result
}
