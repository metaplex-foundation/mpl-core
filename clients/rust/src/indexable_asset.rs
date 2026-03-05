#![warn(clippy::indexing_slicing)]
#[cfg(feature = "anchor")]
use anchor_lang::prelude::AnchorDeserialize;
use base64::prelude::*;
#[cfg(not(feature = "anchor"))]
use borsh::BorshDeserialize;
use num_traits::FromPrimitive;
use solana_program::pubkey::Pubkey;
use std::{cmp::Ordering, collections::HashMap, io::ErrorKind};

use crate::{
    accounts::{BaseAssetV1, BaseCollectionV1, PluginHeaderV1},
    convert_external_plugin_adapter_data_to_string,
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

#[cfg(feature = "serde")]
mod custom_serde {
    use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
    use serde_json::Value as JsonValue;

    pub fn serialize<S>(data: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match data {
            Some(s) => {
                if let Ok(json_value) = serde_json::from_str::<JsonValue>(s) {
                    json_value.serialize(serializer)
                } else {
                    serializer.serialize_str(s)
                }
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json_value: Option<JsonValue> = Option::deserialize(deserializer)?;
        match json_value {
            Some(JsonValue::String(s)) => Ok(Some(s)),
            Some(json_value) => Ok(Some(json_value.to_string())),
            None => Ok(None),
        }
    }
}

/// Schema used for indexing known external plugin types.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexableExternalPluginSchemaV1 {
    pub index: u64,
    pub offset: u64,
    pub authority: PluginAuthority,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub lifecycle_checks: Option<LifecycleChecks>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub unknown_lifecycle_checks: Option<Vec<(u8, Vec<IndexableCheckResult>)>>,
    pub r#type: ExternalPluginAdapterType,
    pub adapter_config: ExternalPluginAdapter,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub data_offset: Option<u64>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub data_len: Option<u64>,
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", with = "custom_serde")
    )]
    pub data: Option<String>,
}

/// Schema used for indexing unknown external plugin types, storing the plugin as raw data.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexableUnknownExternalPluginSchemaV1 {
    pub index: u64,
    pub offset: u64,
    pub authority: PluginAuthority,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub lifecycle_checks: Option<LifecycleChecks>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub unknown_lifecycle_checks: Option<Vec<(u8, Vec<IndexableCheckResult>)>>,
    pub r#type: u8,
    pub unknown_adapter_config: String,
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
        let processed_plugin = if let Some(known_plugin_type) = PluginType::from_u8(plugin_type) {
            let data = Plugin::deserialize(plugin_slice)?;
            let indexable_plugin_schema = IndexablePluginSchemaV1 {
                index,
                offset,
                authority,
                data,
            };
            ProcessedPlugin::Known((known_plugin_type, indexable_plugin_schema))
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
    Known(IndexableExternalPluginSchemaV1),
    Unknown(IndexableUnknownExternalPluginSchemaV1),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct LifecycleChecks {
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    pub create: Vec<IndexableCheckResult>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    pub update: Vec<IndexableCheckResult>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    pub transfer: Vec<IndexableCheckResult>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Vec::is_empty"))]
    pub burn: Vec<IndexableCheckResult>,
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
pub enum IndexableCheckResult {
    CanListen,
    CanApprove,
    CanReject,
}

impl From<ExternalCheckResult> for Vec<IndexableCheckResult> {
    fn from(check_result: ExternalCheckResult) -> Self {
        let check_result_bits = ExternalCheckResultBits::from(check_result);
        let mut check_result_vec = vec![];
        if check_result_bits.can_listen() {
            check_result_vec.push(IndexableCheckResult::CanListen);
        }
        if check_result_bits.can_approve() {
            check_result_vec.push(IndexableCheckResult::CanApprove);
        }
        if check_result_bits.can_reject() {
            check_result_vec.push(IndexableCheckResult::CanReject);
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
                let checks = Vec::<IndexableCheckResult>::from(check);
                match HookableLifecycleEvent::from_u8(event) {
                    Some(val) => match val {
                        HookableLifecycleEvent::Create => known_lifecycle_checks.create = checks,
                        HookableLifecycleEvent::Update => known_lifecycle_checks.update = checks,
                        HookableLifecycleEvent::Transfer => {
                            known_lifecycle_checks.transfer = checks
                        }
                        HookableLifecycleEvent::Burn => known_lifecycle_checks.burn = checks,
                    },
                    None => unknown_lifecycle_checks.push((event, checks)),
                }
            }
        }

        // Save them as known and unknown.
        let known_lifecycle_checks =
            (!known_lifecycle_checks.is_all_empty()).then_some(known_lifecycle_checks);
        let unknown_lifecycle_checks =
            (!unknown_lifecycle_checks.is_empty()).then_some(unknown_lifecycle_checks);

        // Next, process the external plugin adapter and save them as known and unknown variants.
        let processed_plugin = if let Some(r#type) =
            ExternalPluginAdapterType::from_u8(external_plugin_adapter_type)
        {
            let adapter_config = ExternalPluginAdapter::deserialize(plugin_slice)?;
            let (data_offset, data_len, data) = match external_plugin_data_info {
                Some(data_info) => {
                    let schema = match &adapter_config {
                        ExternalPluginAdapter::LifecycleHook(lc_hook) => &lc_hook.schema,
                        ExternalPluginAdapter::AppData(app_data) => &app_data.schema,
                        ExternalPluginAdapter::LinkedLifecycleHook(l_lc_hook) => &l_lc_hook.schema,
                        ExternalPluginAdapter::LinkedAppData(l_app_data) => &l_app_data.schema,
                        ExternalPluginAdapter::DataSection(data_section) => &data_section.schema,
                        // Assume binary for `Oracle`, but this should never happen.
                        ExternalPluginAdapter::Oracle(_) => &ExternalPluginAdapterSchema::Binary,
                    };

                    (
                        Some(data_info.data_offset),
                        Some(data_info.data_len),
                        Some(convert_external_plugin_adapter_data_to_string(
                            schema,
                            data_info.data_slice,
                        )),
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
                r#type,
                adapter_config,
                data_offset,
                data_len,
                data,
            };
            ProcessedExternalPlugin::Known(indexable_plugin_schema)
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
                r#type: external_plugin_adapter_type,
                unknown_adapter_config: encoded,
                data_offset,
                data_len,
                data,
            })
        };

        Ok(processed_plugin)
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
    pub external_plugins: Vec<IndexableExternalPluginSchemaV1>,
    pub unknown_external_plugins: Vec<IndexableUnknownExternalPluginSchemaV1>,
}

enum CombinedRecord<'a> {
    Internal(&'a RegistryRecordSafe),
    External(&'a ExternalRegistryRecordSafe),
}

struct CombinedRecordWithDataInfo<'a> {
    pub offset: u64,
    pub data_offset: Option<u64>,
    pub record: CombinedRecord<'a>,
}

impl<'a> CombinedRecordWithDataInfo<'a> {
    // Associated function for sorting `RegistryRecordIndexable` by offset.
    pub fn compare_offsets(
        a: &CombinedRecordWithDataInfo,
        b: &CombinedRecordWithDataInfo,
    ) -> Ordering {
        a.offset.cmp(&b.offset)
    }
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
            unknown_plugins: vec![],
            external_plugins: vec![],
            unknown_external_plugins: vec![],
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
            unknown_plugins: vec![],
            external_plugins: vec![],
            unknown_external_plugins: vec![],
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
            ProcessedExternalPlugin::Known(indexable_plugin_schema) => {
                self.external_plugins.push(indexable_plugin_schema);
            }
            ProcessedExternalPlugin::Unknown(indexable_unknown_plugin_schema) => self
                .unknown_external_plugins
                .push(indexable_unknown_plugin_schema),
        }
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
            let data_slice = account
                .get(data_offset..end)
                .ok_or(ErrorKind::InvalidData)?;

            Ok(Some(ExternalPluginDataInfo {
                data_offset: data_offset as u64,
                data_len: data_len as u64,
                data_slice,
            }))
        } else {
            Ok(None)
        }
    }

    fn process_combined_record(
        index: u64,
        combined_record: &CombinedRecord,
        plugin_slice: &mut &[u8],
        account: &[u8],
        indexable_asset: &mut IndexableAsset,
    ) -> Result<(), std::io::Error> {
        match combined_record {
            CombinedRecord::Internal(record) => {
                let processed_plugin = ProcessedPlugin::from_data(
                    index,
                    record.offset,
                    record.authority.clone(),
                    record.plugin_type,
                    plugin_slice,
                )?;

                indexable_asset.add_processed_plugin(processed_plugin);
            }
            CombinedRecord::External(record) => {
                let external_plugin_data_info =
                    Self::slice_external_plugin_data(record.data_offset, record.data_len, account)?;

                let processed_plugin = ProcessedExternalPlugin::from_data(
                    index,
                    record.offset,
                    record.authority.clone(),
                    record.lifecycle_checks.clone(),
                    record.plugin_type,
                    plugin_slice,
                    external_plugin_data_info,
                )?;

                indexable_asset.add_processed_external_plugin(processed_plugin);
            }
        }

        Ok(())
    }

    /// Fetch the base `Asset`` or `Collection`` and all the plugins and store in an
    /// `IndexableAsset`.
    pub fn fetch(key: Key, account: &[u8]) -> Result<Self, std::io::Error> {
        if Key::from_slice(account, 0)? != key {
            return Err(ErrorKind::InvalidInput.into());
        }

        let (mut indexable_asset, base_size) = match key {
            Key::AssetV1 => {
                let asset = BaseAssetV1::from_bytes(account)?;
                let base_size = asset.len();
                let indexable_asset = Self::from_asset(asset, 0);
                (indexable_asset, base_size)
            }
            Key::CollectionV1 => {
                let collection = BaseCollectionV1::from_bytes(account)?;
                let base_size = collection.len();
                let indexable_asset = Self::from_collection(collection);
                (indexable_asset, base_size)
            }
            _ => return Err(ErrorKind::InvalidInput.into()),
        };

        if base_size != account.len() {
            let header = PluginHeaderV1::from_bytes(
                account.get(base_size..).ok_or(ErrorKind::InvalidData)?,
            )?;
            let plugin_registry = PluginRegistryV1Safe::from_bytes(
                account
                    .get((header.plugin_registry_offset as usize)..)
                    .ok_or(ErrorKind::InvalidData)?,
            )?;

            // Combine internal and external plugin registry records.
            let mut combined_records = vec![];

            // Add internal registry records.
            for record in &plugin_registry.registry {
                combined_records.push(CombinedRecordWithDataInfo {
                    offset: record.offset,
                    data_offset: None,
                    record: CombinedRecord::Internal(record),
                });
            }

            // Add external registry records.
            for record in &plugin_registry.external_registry {
                combined_records.push(CombinedRecordWithDataInfo {
                    offset: record.offset,
                    data_offset: record.data_offset,
                    record: CombinedRecord::External(record),
                });
            }

            // Sort combined registry records by offset.
            combined_records.sort_by(CombinedRecordWithDataInfo::compare_offsets);

            // Process combined registry records using windows of 2 so that plugin slice length can
            // be calculated.
            for (i, records) in combined_records.windows(2).enumerate() {
                // For internal plugins, the end of the slice is the start of the next plugin.  For
                // external plugins, the end of the adapter is either the start of the data (if
                // present) or the start of the next plugin.
                let end = records
                    .first()
                    .ok_or(ErrorKind::InvalidData)?
                    .data_offset
                    .unwrap_or(records.get(1).ok_or(ErrorKind::InvalidData)?.offset);
                let mut plugin_slice = account
                    .get(
                        records.first().ok_or(ErrorKind::InvalidData)?.offset as usize
                            ..end as usize,
                    )
                    .ok_or(ErrorKind::InvalidData)?;

                Self::process_combined_record(
                    i as u64,
                    &records.first().ok_or(ErrorKind::InvalidData)?.record,
                    &mut plugin_slice,
                    account,
                    &mut indexable_asset,
                )?;
            }

            // Process the last combined registry record.
            if let Some(record) = combined_records.last() {
                // For the last plugin, if it is an internal plugin, the slice ends at the plugin
                // registry offset.  If it is an external plugin, the end of the adapter is either
                // the start of the data (if present) or the plugin registry offset.
                let end = record.data_offset.unwrap_or(header.plugin_registry_offset);
                let mut plugin_slice = account
                    .get(record.offset as usize..end as usize)
                    .ok_or(ErrorKind::InvalidData)?;

                Self::process_combined_record(
                    combined_records.len() as u64 - 1,
                    &record.record,
                    &mut plugin_slice,
                    account,
                    &mut indexable_asset,
                )?;
            }
        }
        Ok(indexable_asset)
    }
}
