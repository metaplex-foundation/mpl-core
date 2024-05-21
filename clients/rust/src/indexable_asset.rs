#[cfg(feature = "anchor")]
use anchor_lang::prelude::AnchorDeserialize;
use base64::prelude::*;
#[cfg(not(feature = "anchor"))]
use borsh::BorshDeserialize;
use num_traits::FromPrimitive;
use solana_program::pubkey::Pubkey;
use std::{collections::HashMap, io::ErrorKind};

use crate::{
    accounts::{BaseAssetV1, BaseCollectionV1, PluginHeaderV1},
    types::{
        ExternalCheckResult, ExternalPluginAdapter, ExternalPluginAdapterSchema,
        ExternalPluginAdapterType, HookableLifecycleEvent, Key, Plugin, PluginAuthority,
        PluginType, UpdateAuthority,
    },
    DataBlob, ExternalCheckResultBits, ExternalRegistryRecordSafe, PluginRegistryV1Safe,
    RegistryRecordSafe,
};

/// Schema used for indexing known plugin types.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexablePluginSchemaV1 {
    pub index: u64,
    pub offset: u64,
    pub authority: PluginAuthority,
    pub data: Plugin,
}

/// Schema used for indexing unknown plugin types, storing the plugin as raw data.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexableUnknownPluginSchemaV1 {
    pub index: u64,
    pub offset: u64,
    pub authority: PluginAuthority,
    pub r#type: u8,
    pub data: String,
}

/// Schema used for indexing known external plugin types.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexableExternalPluginSchemaV1 {
    pub index: u64,
    pub offset: u64,
    pub authority: PluginAuthority,
    pub lifecycle_checks: Option<LifecycleChecks>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub unknown_lifecycle_checks: Option<Vec<(u8, ExternalCheckResult)>>,
    pub external_plugin_adapter: ExternalPluginAdapter,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub data_offset: Option<u64>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub data_len: Option<u64>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub data: Option<String>,
}

/// Schema used for indexing unknown external plugin types, storing the plugin as raw data.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexableUnknownExternalPluginSchemaV1 {
    pub index: u64,
    pub offset: u64,
    pub authority: PluginAuthority,
    pub lifecycle_checks: Option<LifecycleChecks>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub unknown_lifecycle_checks: Option<Vec<(u8, ExternalCheckResult)>>,
    pub unknown_external_plugin_adapter_type: u8,
    pub unknown_external_plugin_adapter: String,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub data_offset: Option<u64>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub data_len: Option<u64>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub data: Option<String>,
}

// Type used to store a plugin that was processed and is either a known or unknown plugin type.
// Used when building an `IndexableAsset`.
#[derive(Clone, Debug, Eq, PartialEq)]
enum ProcessedPlugin {
    Known((PluginType, IndexablePluginSchemaV1)),
    Unknown(IndexableUnknownPluginSchemaV1),
}

impl ProcessedPlugin {
    fn from_data(
        index: u64,
        offset: u64,
        authority: PluginAuthority,
        plugin_type: u8,
        plugin_slice: &mut &[u8],
    ) -> Result<Self, std::io::Error> {
        let processed_plugin = if let Some(plugin_type) = PluginType::from_u8(plugin_type) {
            let data = Plugin::deserialize(plugin_slice)?;
            let indexable_plugin_schema = IndexablePluginSchemaV1 {
                index,
                offset,
                authority,
                data,
            };
            ProcessedPlugin::Known((plugin_type, indexable_plugin_schema))
        } else {
            let encoded: String = BASE64_STANDARD.encode(plugin_slice);
            ProcessedPlugin::Unknown(IndexableUnknownPluginSchemaV1 {
                index,
                offset,
                authority,
                r#type: plugin_type,
                data: encoded,
            })
        };

        Ok(processed_plugin)
    }
}

// Type used to store an external plugin that was processed and is either a known or unknown plugin
// type.  Used when building an `IndexableAsset`.
#[derive(Clone, Debug, Eq, PartialEq)]
enum ProcessedExternalPlugin {
    Known((ExternalPluginAdapterType, IndexableExternalPluginSchemaV1)),
    Unknown(IndexableUnknownExternalPluginSchemaV1),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct LifecycleChecks {
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    pub create: Vec<CheckResult>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    pub update: Vec<CheckResult>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    pub transfer: Vec<CheckResult>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    pub burn: Vec<CheckResult>,
}

impl LifecycleChecks {
    pub fn is_all_empty(&self) -> bool {
        self.create.is_empty()
            && self.update.is_empty()
            && self.transfer.is_empty()
            && self.burn.is_empty()
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckResult {
    CanListen,
    CanApprove,
    CanReject,
}

impl From<ExternalCheckResult> for Vec<CheckResult> {
    fn from(check_result: ExternalCheckResult) -> Self {
        let check_result_bits = ExternalCheckResultBits::from(check_result);
        let mut check_result_vec = vec![];
        if check_result_bits.can_listen() {
            check_result_vec.push(CheckResult::CanListen);
        }
        if check_result_bits.can_approve() {
            check_result_vec.push(CheckResult::CanApprove);
        }
        if check_result_bits.can_reject() {
            check_result_vec.push(CheckResult::CanReject);
        }
        check_result_vec
    }
}

struct ExternalPluginDataInfo<'a> {
    data_offset: u64,
    data_len: u64,
    data_slice: &'a [u8],
}

impl ProcessedExternalPlugin {
    fn from_data(
        index: u64,
        offset: u64,
        authority: PluginAuthority,
        lifecycle_checks: Option<Vec<(u8, ExternalCheckResult)>>,
        external_plugin_adapter_type: u8,
        plugin_slice: &mut &[u8],
        external_plugin_data_info: Option<ExternalPluginDataInfo>,
    ) -> Result<Self, std::io::Error> {
        // First process the lifecycle checks.
        let mut known_lifecycle_checks = LifecycleChecks::default();
        let mut unknown_lifecycle_checks = vec![];

        if let Some(checks) = lifecycle_checks {
            for (event, check) in checks {
                match HookableLifecycleEvent::from_u8(event) {
                    Some(val) => match val {
                        HookableLifecycleEvent::Create => {
                            known_lifecycle_checks.create = Vec::<CheckResult>::from(check)
                        }
                        HookableLifecycleEvent::Update => {
                            known_lifecycle_checks.update = Vec::<CheckResult>::from(check)
                        }
                        HookableLifecycleEvent::Transfer => {
                            known_lifecycle_checks.transfer = Vec::<CheckResult>::from(check)
                        }
                        HookableLifecycleEvent::Burn => {
                            known_lifecycle_checks.burn = Vec::<CheckResult>::from(check)
                        }
                    },
                    None => unknown_lifecycle_checks.push((event, check)),
                }
            }
        }

        // Save them as known and unknown.
        let known_lifecycle_checks =
            (!known_lifecycle_checks.is_all_empty()).then_some(known_lifecycle_checks);
        let unknown_lifecycle_checks =
            (!unknown_lifecycle_checks.is_empty()).then_some(unknown_lifecycle_checks);

        // Next, process the external plugin adapter and save them as known and unknown variants.
        let processed_plugin = if let Some(external_plugin_adapter_type) =
            ExternalPluginAdapterType::from_u8(external_plugin_adapter_type)
        {
            let external_plugin_adapter = ExternalPluginAdapter::deserialize(plugin_slice)?;
            let (data_offset, data_len, data) = match external_plugin_data_info {
                Some(data_info) => {
                    let schema = match &external_plugin_adapter {
                        ExternalPluginAdapter::LifecycleHook(lifecycle_hook) => {
                            &lifecycle_hook.schema
                        }
                        ExternalPluginAdapter::DataStore(data_store) => &data_store.schema,
                        _ => &ExternalPluginAdapterSchema::Binary, // is this possible
                    };

                    (
                        Some(data_info.data_offset),
                        Some(data_info.data_len),
                        Some(Self::convert_data_to_string(schema, data_info.data_slice)),
                    )
                }
                None => (None, None, None),
            };

            let indexable_plugin_schema = IndexableExternalPluginSchemaV1 {
                index,
                offset,
                authority,
                lifecycle_checks: known_lifecycle_checks,
                unknown_lifecycle_checks,
                external_plugin_adapter,
                data_offset,
                data_len,
                data,
            };
            ProcessedExternalPlugin::Known((external_plugin_adapter_type, indexable_plugin_schema))
        } else {
            let encoded: String = BASE64_STANDARD.encode(plugin_slice);

            let (data_offset, data_len, data) = match external_plugin_data_info {
                Some(data_info) => (
                    Some(data_info.data_offset),
                    Some(data_info.data_len),
                    // We don't know the schema so use base64.
                    Some(BASE64_STANDARD.encode(data_info.data_slice)),
                ),
                None => (None, None, None),
            };

            ProcessedExternalPlugin::Unknown(IndexableUnknownExternalPluginSchemaV1 {
                index,
                offset,
                authority,
                lifecycle_checks: known_lifecycle_checks,
                unknown_lifecycle_checks,
                unknown_external_plugin_adapter_type: external_plugin_adapter_type,
                unknown_external_plugin_adapter: encoded,
                data_offset,
                data_len,
                data,
            })
        };

        Ok(processed_plugin)
    }

    fn convert_data_to_string(schema: &ExternalPluginAdapterSchema, data_slice: &[u8]) -> String {
        match schema {
            ExternalPluginAdapterSchema::Binary => {
                // Encode the binary data as a base64 string.
                BASE64_STANDARD.encode(data_slice)
            }
            ExternalPluginAdapterSchema::Json => {
                // Convert the byte slice to a UTF-8 string, replacing invalid characterse.
                String::from_utf8_lossy(data_slice).to_string()
            }
            ExternalPluginAdapterSchema::MsgPack => {
                // Attempt to decode `MsgPack` to serde_json::Value and serialize to JSON string.
                match rmp_serde::decode::from_slice::<serde_json::Value>(data_slice) {
                    Ok(json_val) => serde_json::to_string(&json_val)
                        .unwrap_or_else(|_| BASE64_STANDARD.encode(data_slice)),
                    Err(_) => {
                        // Failed to decode `MsgPack`, fallback to base64.
                        BASE64_STANDARD.encode(data_slice)
                    }
                }
            }
        }
    }
}

/// A type used to store both Core Assets and Core Collections for indexing.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IndexableAsset {
    pub owner: Option<Pubkey>,
    pub update_authority: UpdateAuthority,
    pub name: String,
    pub uri: String,
    pub seq: u64,
    pub num_minted: Option<u32>,
    pub current_size: Option<u32>,
    pub plugins: HashMap<PluginType, IndexablePluginSchemaV1>,
    pub unknown_plugins: Vec<IndexableUnknownPluginSchemaV1>,
    pub external_plugins: HashMap<ExternalPluginAdapterType, IndexableExternalPluginSchemaV1>,
    pub unknown_external_plugins: Vec<IndexableUnknownExternalPluginSchemaV1>,
}

impl IndexableAsset {
    /// Create a new `IndexableAsset` from a `BaseAssetV1``.  Note this uses a passed-in `seq` rather than
    /// the one contained in `asset` to avoid errors.
    pub fn from_asset(asset: BaseAssetV1, seq: u64) -> Self {
        Self {
            owner: Some(asset.owner),
            update_authority: asset.update_authority,
            name: asset.name,
            uri: asset.uri,
            seq,
            num_minted: None,
            current_size: None,
            plugins: HashMap::new(),
            unknown_plugins: Vec::new(),
            external_plugins: HashMap::new(),
            unknown_external_plugins: Vec::new(),
        }
    }

    /// Create a new `IndexableAsset` from a `BaseCollectionV1`.
    pub fn from_collection(collection: BaseCollectionV1) -> Self {
        Self {
            owner: None,
            update_authority: UpdateAuthority::Address(collection.update_authority),
            name: collection.name,
            uri: collection.uri,
            seq: 0,
            num_minted: Some(collection.num_minted),
            current_size: Some(collection.current_size),
            plugins: HashMap::new(),
            unknown_plugins: Vec::new(),
            external_plugins: HashMap::new(),
            unknown_external_plugins: Vec::new(),
        }
    }

    // Add a processed plugin to the correct `IndexableAsset` struct member.
    fn add_processed_plugin(&mut self, plugin: ProcessedPlugin) {
        match plugin {
            ProcessedPlugin::Known((plugin_type, indexable_plugin_schema)) => {
                self.plugins.insert(plugin_type, indexable_plugin_schema);
            }
            ProcessedPlugin::Unknown(indexable_unknown_plugin_schema) => {
                self.unknown_plugins.push(indexable_unknown_plugin_schema)
            }
        }
    }

    // Add a processed external plugin to the correct `IndexableAsset` struct member.
    fn add_processed_external_plugin(&mut self, plugin: ProcessedExternalPlugin) {
        match plugin {
            ProcessedExternalPlugin::Known((plugin_type, indexable_plugin_schema)) => {
                self.external_plugins
                    .insert(plugin_type, indexable_plugin_schema);
            }
            ProcessedExternalPlugin::Unknown(indexable_unknown_plugin_schema) => self
                .unknown_external_plugins
                .push(indexable_unknown_plugin_schema),
        }
    }

    /// Fetch the base `Asset`` or `Collection`` and all the plugins and store in an `IndexableAsset`.
    pub fn fetch(key: Key, account: &[u8]) -> Result<Self, std::io::Error> {
        let (mut indexable_asset, base_size) = match key {
            Key::AssetV1 => {
                let asset = BaseAssetV1::from_bytes(account)?;
                let base_size = asset.get_size();
                let indexable_asset = Self::from_asset(asset, 0);
                (indexable_asset, base_size)
            }
            Key::CollectionV1 => {
                let collection = BaseCollectionV1::from_bytes(account)?;
                let base_size = collection.get_size();
                let indexable_asset = Self::from_collection(collection);
                (indexable_asset, base_size)
            }
            _ => return Err(ErrorKind::InvalidInput.into()),
        };

        if base_size != account.len() {
            let header = PluginHeaderV1::from_bytes(&account[base_size..])?;
            let plugin_registry = PluginRegistryV1Safe::from_bytes(
                &account[(header.plugin_registry_offset as usize)..],
            )?;

            let mut registry_records = plugin_registry.registry;
            registry_records.sort_by(RegistryRecordSafe::compare_offsets);

            for (i, records) in registry_records.windows(2).enumerate() {
                let mut plugin_slice =
                    &account[records[0].offset as usize..records[1].offset as usize];
                let processed_plugin = ProcessedPlugin::from_data(
                    i as u64,
                    records[0].offset,
                    records[0].authority.clone(),
                    records[0].plugin_type,
                    &mut plugin_slice,
                )?;

                indexable_asset.add_processed_plugin(processed_plugin);
            }

            if let Some(record) = registry_records.last() {
                let mut plugin_slice =
                    &account[record.offset as usize..header.plugin_registry_offset as usize];

                let processed_plugin = ProcessedPlugin::from_data(
                    registry_records.len() as u64 - 1,
                    record.offset,
                    record.authority.clone(),
                    record.plugin_type,
                    &mut plugin_slice,
                )?;

                indexable_asset.add_processed_plugin(processed_plugin);
            }

            let mut external_registry_records = plugin_registry.external_registry;
            external_registry_records.sort_by(ExternalRegistryRecordSafe::compare_offsets);

            for (i, records) in external_registry_records.windows(2).enumerate() {
                let mut plugin_slice =
                    &account[records[0].offset as usize..records[1].offset as usize];

                let external_plugin_data_info = Self::slice_external_plugin_data(
                    records[0].data_offset,
                    records[0].data_len,
                    account,
                )?;

                let processed_plugin = ProcessedExternalPlugin::from_data(
                    i as u64,
                    records[0].offset,
                    records[0].authority.clone(),
                    records[0].lifecycle_checks.clone(),
                    records[0].plugin_type,
                    &mut plugin_slice,
                    external_plugin_data_info,
                )?;

                indexable_asset.add_processed_external_plugin(processed_plugin);
            }

            if let Some(record) = external_registry_records.last() {
                let mut plugin_slice =
                    &account[record.offset as usize..header.plugin_registry_offset as usize];

                let external_plugin_data_info =
                    Self::slice_external_plugin_data(record.data_offset, record.data_len, account)?;

                let processed_plugin = ProcessedExternalPlugin::from_data(
                    external_registry_records.len() as u64 - 1,
                    record.offset,
                    record.authority.clone(),
                    record.lifecycle_checks.clone(),
                    record.plugin_type,
                    &mut plugin_slice,
                    external_plugin_data_info,
                )?;

                indexable_asset.add_processed_external_plugin(processed_plugin);
            }
        }
        Ok(indexable_asset)
    }

    fn slice_external_plugin_data(
        data_offset: Option<u64>,
        data_len: Option<u64>,
        account: &[u8],
    ) -> Result<Option<ExternalPluginDataInfo>, std::io::Error> {
        if data_offset.is_some() && data_len.is_some() {
            let data_offset = data_offset.unwrap() as usize;
            let data_len = data_len.unwrap() as usize;

            let end = data_offset
                .checked_add(data_len)
                .ok_or(ErrorKind::InvalidData)?;
            let data_slice = &account[data_offset..end];

            Ok(Some(ExternalPluginDataInfo {
                data_offset: data_offset as u64,
                data_len: data_len as u64,
                data_slice,
            }))
        } else {
            Ok(None)
        }
    }
}
