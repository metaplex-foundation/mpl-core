use borsh::BorshDeserialize;
use solana_program::account_info::AccountInfo;

use crate::{
    accounts::{BaseAssetV1, PluginHeaderV1, PluginRegistryV1},
    errors::MplCoreError,
    types::{Plugin, PluginAuthority, PluginType, RegistryRecord},
    AttributesPlugin, BaseAuthority, BasePlugin, BurnDelegatePlugin, DataBlob,
    FreezeDelegatePlugin, PermanentBurnDelegatePlugin, PermanentFreezeDelegatePlugin,
    PermanentTransferDelegatePlugin, PluginsList, RoyaltiesPlugin, SolanaAccount,
    TransferDelegatePlugin, UpdateDelegatePlugin,
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

    let header = PluginHeaderV1::load(account, asset.get_size())?;
    let PluginRegistryV1 { registry, .. } =
        PluginRegistryV1::load(account, header.plugin_registry_offset as usize)?;

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

    // Return the plugin and its authority.
    Ok((
        registry_record.authority.clone(),
        inner,
        registry_record.offset as usize,
    ))
}

/// Fetch the plugin registry.
pub fn fetch_plugins(account: &[u8]) -> Result<Vec<RegistryRecord>, std::io::Error> {
    let asset = BaseAssetV1::from_bytes(account)?;

    let header = PluginHeaderV1::from_bytes(&account[asset.get_size()..])?;
    let PluginRegistryV1 { registry, .. } =
        PluginRegistryV1::from_bytes(&account[(header.plugin_registry_offset as usize)..])?;

    Ok(registry)
}

/// Create plugin header and registry if it doesn't exist
pub fn list_plugins(account: &[u8]) -> Result<Vec<PluginType>, std::io::Error> {
    let asset = BaseAssetV1::from_bytes(account)?;

    let header = PluginHeaderV1::from_bytes(&account[asset.get_size()..])?;
    let PluginRegistryV1 { registry, .. } =
        PluginRegistryV1::from_bytes(&account[(header.plugin_registry_offset as usize)..])?;

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
            };

            Ok(acc)
        });

    result
}
