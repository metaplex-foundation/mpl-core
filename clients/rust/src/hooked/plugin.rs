use borsh::BorshDeserialize;
use num_traits::FromPrimitive;
use solana_program::account_info::AccountInfo;

use crate::{
    accounts::{BaseAssetV1, PluginHeaderV1},
    errors::MplCoreError,
    types::{Plugin, PluginAuthority, PluginType, RegistryRecord},
    AttributesPlugin, BaseAuthority, BasePlugin, BurnDelegatePlugin, DataBlob, EditionPlugin,
    FreezeDelegatePlugin, PermanentBurnDelegatePlugin, PermanentFreezeDelegatePlugin,
    PermanentTransferDelegatePlugin, PluginRegistryV1Safe, PluginsList, RegistryRecordSafe,
    RoyaltiesPlugin, SolanaAccount, TransferDelegatePlugin, UpdateDelegatePlugin,
};

/// Fetch the plugin from the registry.
pub fn fetch_plugin<T: DataBlob + SolanaAccount, U: BorshDeserialize>(
    account: &AccountInfo,
    plugin_type: PluginType,
) -> Result<(PluginAuthority, U, usize), std::io::Error> {
    let asset = T::load(account, 0)?;

    if asset.get_size() == account.data_len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            MplCoreError::PluginNotFound.to_string(),
        ));
    }

    let header = PluginHeaderV1::from_bytes(&(*account.data).borrow()[asset.get_size()..])?;
    let plugin_registry = PluginRegistryV1Safe::from_bytes(
        &(*account.data).borrow()[header.plugin_registry_offset as usize..],
    )?;

    // Find the plugin in the registry.
    let registry_record = plugin_registry
        .registry
        .iter()
        .find(|record| {
            if let Some(plugin) = PluginType::from_u8(record.plugin_type) {
                plugin == plugin_type
            } else {
                false
            }
        })
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

    // Return the plugin and its authority.
    Ok((
        registry_record.authority.clone(),
        inner,
        registry_record.offset as usize,
    ))
}

/// Fetch the plugin registry, dropping any unknown plugins (i.e. `PluginType`s that are too new
///  for this client to know about).
pub fn fetch_plugins(account_data: &[u8]) -> Result<Vec<RegistryRecord>, std::io::Error> {
    let asset = BaseAssetV1::from_bytes(account_data)?;

    let header = PluginHeaderV1::from_bytes(&account_data[asset.get_size()..])?;
    let plugin_registry = PluginRegistryV1Safe::from_bytes(
        &account_data[(header.plugin_registry_offset as usize)..],
    )?;

    let filtered_plugin_registry = plugin_registry
        .registry
        .iter()
        .filter_map(|record| {
            PluginType::from_u8(record.plugin_type).map(|plugin_type| RegistryRecord {
                plugin_type,
                authority: record.authority.clone(),
                offset: record.offset,
            })
        })
        .collect();

    Ok(filtered_plugin_registry)
}

/// List all plugins in an account, dropping any unknown plugins (i.e. `PluginType`s that are too
/// new for this client to know about). Note this also does not support external plugins for now,
/// and will be updated when those are defined.
pub fn list_plugins(account_data: &[u8]) -> Result<Vec<PluginType>, std::io::Error> {
    let asset = BaseAssetV1::from_bytes(account_data)?;
    let header = PluginHeaderV1::from_bytes(&account_data[asset.get_size()..])?;
    let plugin_registry = PluginRegistryV1Safe::from_bytes(
        &account_data[(header.plugin_registry_offset as usize)..],
    )?;

    Ok(plugin_registry
        .registry
        .iter()
        .filter_map(|registry_record| PluginType::from_u8(registry_record.plugin_type))
        .collect())
}

// Convert a slice of `RegistryRecordSafe` into the `PluginsList` type, dropping any unknown
// plugins (i.e. `PluginType`s that are too new for this client to know about). Note this also does
// not support external plugins for now, and will be updated when those are defined.
pub(crate) fn registry_records_to_plugin_list(
    registry_records: &[RegistryRecordSafe],
    account_data: &[u8],
) -> Result<PluginsList, std::io::Error> {
    let result = registry_records
        .iter()
        .try_fold(PluginsList::default(), |mut acc, record| {
            if PluginType::from_u8(record.plugin_type).is_some() {
                let authority: BaseAuthority = record.authority.clone().into();
                let base = BasePlugin {
                    authority,
                    offset: Some(record.offset),
                };
                let plugin = Plugin::deserialize(&mut &account_data[record.offset as usize..])?;

                match plugin {
                    Plugin::Royalties(royalties) => {
                        acc.royalties = Some(RoyaltiesPlugin { base, royalties });
                    }
                    Plugin::FreezeDelegate(freeze_delegate) => {
                        acc.freeze_delegate = Some(FreezeDelegatePlugin {
                            base,
                            freeze_delegate,
                        });
                    }
                    Plugin::BurnDelegate(burn_delegate) => {
                        acc.burn_delegate = Some(BurnDelegatePlugin {
                            base,
                            burn_delegate,
                        });
                    }
                    Plugin::TransferDelegate(transfer_delegate) => {
                        acc.transfer_delegate = Some(TransferDelegatePlugin {
                            base,
                            transfer_delegate,
                        });
                    }
                    Plugin::UpdateDelegate(update_delegate) => {
                        acc.update_delegate = Some(UpdateDelegatePlugin {
                            base,
                            update_delegate,
                        });
                    }
                    Plugin::PermanentFreezeDelegate(permanent_freeze_delegate) => {
                        acc.permanent_freeze_delegate = Some(PermanentFreezeDelegatePlugin {
                            base,
                            permanent_freeze_delegate,
                        });
                    }
                    Plugin::Attributes(attributes) => {
                        acc.attributes = Some(AttributesPlugin { base, attributes });
                    }
                    Plugin::PermanentTransferDelegate(permanent_transfer_delegate) => {
                        acc.permanent_transfer_delegate = Some(PermanentTransferDelegatePlugin {
                            base,
                            permanent_transfer_delegate,
                        })
                    }
                    Plugin::PermanentBurnDelegate(permanent_burn_delegate) => {
                        acc.permanent_burn_delegate = Some(PermanentBurnDelegatePlugin {
                            base,
                            permanent_burn_delegate,
                        })
                    }
                    Plugin::Edition(edition) => acc.edition = Some(EditionPlugin { base, edition }),
                }
            }
            Ok(acc)
        });

    result
}
