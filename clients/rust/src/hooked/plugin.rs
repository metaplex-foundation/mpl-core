#[cfg(feature = "anchor")]
use anchor_lang::prelude::AnchorDeserialize as CrateDeserialize;
#[cfg(not(feature = "anchor"))]
use borsh::BorshDeserialize as CrateDeserialize;
use num_traits::FromPrimitive;
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

use crate::{
    accounts::{BaseAssetV1, BaseCollectionV1, PluginHeaderV1},
    errors::MplCoreError,
    types::{
        ExternalPluginAdapter, ExternalPluginAdapterKey, ExternalPluginAdapterType, LinkedDataKey,
        Plugin, PluginAuthority, PluginType, RegistryRecord,
    },
    AddBlockerPlugin, AppDataWithData, AttributesPlugin, AutographPlugin, BaseAuthority,
    BasePlugin, BubblegumV2Plugin, BurnDelegatePlugin, DataBlob, DataSectionWithData,
    EditionPlugin, ExternalPluginAdaptersList, ExternalRegistryRecordSafe, FreezeDelegatePlugin,
    FreezeExecutePlugin, ImmutableMetadataPlugin, LifecycleHookWithData, MasterEditionPlugin,
    PermanentBurnDelegatePlugin, PermanentFreezeDelegatePlugin, PermanentTransferDelegatePlugin,
    PluginRegistryV1Safe, PluginsList, RegistryRecordSafe, RoyaltiesPlugin, SolanaAccount,
    TransferDelegatePlugin, UpdateDelegatePlugin, VerifiedCreatorsPlugin,
};

/// Fetch the plugin from the registry.
pub fn fetch_plugin<T: DataBlob + SolanaAccount, U: CrateDeserialize>(
    account: &AccountInfo,
    plugin_type: PluginType,
) -> Result<(PluginAuthority, U, usize), std::io::Error> {
    let asset = T::load(account, 0)?;

    if asset.len() == account.data_len() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            MplCoreError::PluginNotFound.to_string(),
        ));
    }

    let header = PluginHeaderV1::from_bytes(&(*account.data).borrow()[asset.len()..])?;
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

/// Fetch the plugin on an asset.
pub fn fetch_asset_plugin<U: CrateDeserialize>(
    account: &AccountInfo,
    plugin_type: PluginType,
) -> Result<(PluginAuthority, U, usize), std::io::Error> {
    fetch_plugin::<BaseAssetV1, U>(account, plugin_type)
}

/// Fetch the plugin on a collection.
pub fn fetch_collection_plugin<U: CrateDeserialize>(
    account: &AccountInfo,
    plugin_type: PluginType,
) -> Result<(PluginAuthority, U, usize), std::io::Error> {
    fetch_plugin::<BaseCollectionV1, U>(account, plugin_type)
}

/// Fetch the plugin registry, dropping any unknown plugins (i.e. `PluginType`s that are too new
///  for this client to know about).
pub fn fetch_plugins(account_data: &[u8]) -> Result<Vec<RegistryRecord>, std::io::Error> {
    let asset = BaseAssetV1::from_bytes(account_data)?;

    let header = PluginHeaderV1::from_bytes(&account_data[asset.len()..])?;
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

/// Fetch the external plugin adapter from the registry.
pub fn fetch_external_plugin_adapter<T: DataBlob + SolanaAccount, U: CrateDeserialize>(
    account: &AccountInfo,
    core: Option<&T>,
    plugin_key: &ExternalPluginAdapterKey,
) -> Result<(PluginAuthority, U, usize), std::io::Error> {
    let registry_record = fetch_external_registry_record(account, core, plugin_key)?;

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

/// Fetch the external plugin adapter from the registry.
pub fn fetch_wrapped_external_plugin_adapter<T: DataBlob + SolanaAccount>(
    account: &AccountInfo,
    core: Option<&T>,
    plugin_key: &ExternalPluginAdapterKey,
) -> Result<(ExternalRegistryRecordSafe, ExternalPluginAdapter), std::io::Error> {
    let registry_record = fetch_external_registry_record(account, core, plugin_key)?;

    // Deserialize the plugin.
    let plugin = ExternalPluginAdapter::deserialize(
        &mut &(*account.data).borrow()[registry_record.offset as usize..],
    )?;

    // Return the plugin and its authority.
    Ok((registry_record, plugin))
}

// Helper to unwrap optional data offset and data length.
fn unwrap_data_offset_and_data_len(
    data_offset: Option<u64>,
    data_len: Option<u64>,
) -> Result<(usize, usize), std::io::Error> {
    let data_offset = data_offset.ok_or(std::io::Error::new(
        std::io::ErrorKind::Other,
        MplCoreError::InvalidPlugin.to_string(),
    ))?;

    let data_len = data_len.ok_or(std::io::Error::new(
        std::io::ErrorKind::Other,
        MplCoreError::InvalidPlugin.to_string(),
    ))?;

    Ok((data_offset as usize, data_len as usize))
}

/// Fetch the external plugin adapter data offset and length.  These can be used to
/// directly slice the account data for use of the external plugin adapter data.
pub fn fetch_external_plugin_adapter_data_info<T: DataBlob + SolanaAccount>(
    account: &AccountInfo,
    core: Option<&T>,
    plugin_key: &ExternalPluginAdapterKey,
) -> Result<(usize, usize), std::io::Error> {
    let registry_record = fetch_external_registry_record(account, core, plugin_key)?;
    let (data_offset, data_len) =
        unwrap_data_offset_and_data_len(registry_record.data_offset, registry_record.data_len)?;

    // Return the data offset and length.
    Ok((data_offset, data_len))
}

// Internal helper to fetch just the external registry record for the external plugin key.
fn fetch_external_registry_record<T: DataBlob + SolanaAccount>(
    account: &AccountInfo,
    core: Option<&T>,
    plugin_key: &ExternalPluginAdapterKey,
) -> Result<ExternalRegistryRecordSafe, std::io::Error> {
    let size = match core {
        Some(core) => core.len(),
        None => {
            let asset = T::load(account, 0)?;

            if asset.len() == account.data_len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    MplCoreError::ExternalPluginAdapterNotFound.to_string(),
                ));
            }

            asset.len()
        }
    };

    let header = PluginHeaderV1::from_bytes(&(*account.data).borrow()[size..])?;
    let plugin_registry = PluginRegistryV1Safe::from_bytes(
        &(*account.data).borrow()[header.plugin_registry_offset as usize..],
    )?;

    // Find and return the registry record.
    let result = find_external_plugin_adapter(&plugin_registry, plugin_key, account)?;
    result.1.cloned().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            MplCoreError::ExternalPluginAdapterNotFound.to_string(),
        )
    })
}

/// List all plugins in an account, dropping any unknown plugins (i.e. `PluginType`s that are too
/// new for this client to know about). Note this also does not support external plugin adapters for now,
/// and will be updated when those are defined.
pub fn list_plugins(account_data: &[u8]) -> Result<Vec<PluginType>, std::io::Error> {
    let asset = BaseAssetV1::from_bytes(account_data)?;
    let header = PluginHeaderV1::from_bytes(&account_data[asset.len()..])?;
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
// plugins (i.e. `PluginType`s that are too new for this client to know about).
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
                    Plugin::MasterEdition(master_edition) => {
                        acc.master_edition = Some(MasterEditionPlugin {
                            base,
                            master_edition,
                        })
                    }
                    Plugin::AddBlocker(add_blocker) => {
                        acc.add_blocker = Some(AddBlockerPlugin { base, add_blocker })
                    }
                    Plugin::ImmutableMetadata(immutable_metadata) => {
                        acc.immutable_metadata = Some(ImmutableMetadataPlugin {
                            base,
                            immutable_metadata,
                        })
                    }
                    Plugin::VerifiedCreators(verified_creators) => {
                        acc.verified_creators = Some(VerifiedCreatorsPlugin {
                            base,
                            verified_creators,
                        })
                    }
                    Plugin::Autograph(autograph) => {
                        acc.autograph = Some(AutographPlugin { base, autograph })
                    }
                    Plugin::BubblegumV2(bubblegum_v2) => {
                        acc.bubblegum_v2 = Some(BubblegumV2Plugin { base, bubblegum_v2 })
                    }
                    Plugin::FreezeExecute(freeze_execute) => {
                        acc.freeze_execute = Some(FreezeExecutePlugin {
                            base,
                            freeze_execute,
                        })
                    }
                }
            }
            Ok(acc)
        });

    result
}

// Convert a slice of `AdapterRegistryRecordSafe` into the `ExternalPluginAdaptersList` type, dropping any unknown
// plugins (i.e. `ExternalPluginAdapterType`s that are too new for this client to know about).
pub(crate) fn registry_records_to_external_plugin_adapter_list(
    registry_records: &[ExternalRegistryRecordSafe],
    account_data: &[u8],
) -> Result<ExternalPluginAdaptersList, std::io::Error> {
    let result = registry_records.iter().try_fold(
        ExternalPluginAdaptersList::default(),
        |mut acc, record| {
            if ExternalPluginAdapterType::from_u8(record.plugin_type).is_some() {
                let plugin = ExternalPluginAdapter::deserialize(
                    &mut &account_data[record.offset as usize..],
                )?;

                match plugin {
                    ExternalPluginAdapter::LifecycleHook(lifecycle_hook) => {
                        let (data_offset, data_len) =
                            unwrap_data_offset_and_data_len(record.data_offset, record.data_len)?;

                        acc.lifecycle_hooks.push(LifecycleHookWithData {
                            base: lifecycle_hook,
                            data_offset,
                            data_len,
                        })
                    }
                    ExternalPluginAdapter::LinkedLifecycleHook(lifecycle_hook) => {
                        acc.linked_lifecycle_hooks.push(lifecycle_hook)
                    }
                    ExternalPluginAdapter::Oracle(oracle) => acc.oracles.push(oracle),
                    ExternalPluginAdapter::AppData(app_data) => {
                        let (data_offset, data_len) =
                            unwrap_data_offset_and_data_len(record.data_offset, record.data_len)?;

                        acc.app_data.push(AppDataWithData {
                            base: app_data,
                            data_offset,
                            data_len,
                        })
                    }
                    ExternalPluginAdapter::LinkedAppData(app_data) => {
                        acc.linked_app_data.push(app_data)
                    }
                    ExternalPluginAdapter::DataSection(data_section) => {
                        let (data_offset, data_len) =
                            unwrap_data_offset_and_data_len(record.data_offset, record.data_len)?;

                        acc.data_sections.push(DataSectionWithData {
                            base: data_section,
                            data_offset,
                            data_len,
                        })
                    }
                }
            }
            Ok(acc)
        },
    );

    result
}

pub(crate) fn find_external_plugin_adapter<'b>(
    plugin_registry: &'b PluginRegistryV1Safe,
    plugin_key: &ExternalPluginAdapterKey,
    account: &AccountInfo<'_>,
) -> Result<(Option<usize>, Option<&'b ExternalRegistryRecordSafe>), std::io::Error> {
    let mut result = (None, None);
    for (i, record) in plugin_registry.external_registry.iter().enumerate() {
        if record.plugin_type == ExternalPluginAdapterType::from(plugin_key) as u8
            && (match plugin_key {
                ExternalPluginAdapterKey::LifecycleHook(address)
                | ExternalPluginAdapterKey::Oracle(address)
                | ExternalPluginAdapterKey::LinkedLifecycleHook(address) => {
                    let pubkey_offset = record.offset.checked_add(1).ok_or(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        MplCoreError::NumericalOverflow,
                    ))?;
                    address
                        == &match Pubkey::deserialize(
                            &mut &account.data.borrow()[pubkey_offset as usize..],
                        ) {
                            Ok(address) => address,
                            Err(_) => {
                                return Err(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    Box::<dyn std::error::Error + Send + Sync>::from(
                                        MplCoreError::DeserializationError,
                                    ),
                                ))
                            }
                        }
                }
                ExternalPluginAdapterKey::AppData(authority) => {
                    let authority_offset =
                        record.offset.checked_add(1).ok_or(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            MplCoreError::NumericalOverflow,
                        ))?;
                    authority
                        == &match PluginAuthority::deserialize(
                            &mut &account.data.borrow()[authority_offset as usize..],
                        ) {
                            Ok(authority) => authority,
                            Err(_) => {
                                return Err(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    Box::<dyn std::error::Error + Send + Sync>::from(
                                        MplCoreError::DeserializationError,
                                    ),
                                ))
                            }
                        }
                }
                ExternalPluginAdapterKey::LinkedAppData(authority) => {
                    let authority_offset =
                        record.offset.checked_add(1).ok_or(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            MplCoreError::NumericalOverflow,
                        ))?;
                    authority
                        == &match PluginAuthority::deserialize(
                            &mut &account.data.borrow()[authority_offset as usize..],
                        ) {
                            Ok(authority) => authority,
                            Err(_) => {
                                return Err(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    Box::<dyn std::error::Error + Send + Sync>::from(
                                        MplCoreError::DeserializationError,
                                    ),
                                ))
                            }
                        }
                }
                ExternalPluginAdapterKey::DataSection(linked_data_key) => {
                    let linked_data_key_offset =
                        record.offset.checked_add(1).ok_or(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            MplCoreError::NumericalOverflow,
                        ))?;
                    linked_data_key
                        == &match LinkedDataKey::deserialize(
                            &mut &account.data.borrow()[linked_data_key_offset as usize..],
                        ) {
                            Ok(linked_data_key) => linked_data_key,
                            Err(_) => {
                                return Err(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    Box::<dyn std::error::Error + Send + Sync>::from(
                                        MplCoreError::DeserializationError,
                                    ),
                                ))
                            }
                        }
                }
            })
        {
            result = (Some(i), Some(record));
            break;
        }
    }

    Ok(result)
}
