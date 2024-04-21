use base64::prelude::*;
use borsh::BorshDeserialize;
use num_traits::FromPrimitive;
use solana_program::pubkey::Pubkey;
use std::{collections::HashMap, io::ErrorKind};

use crate::{
    accounts::{BaseAssetV1, BaseCollectionV1, PluginHeaderV1},
    types::{Key, Plugin, PluginAuthority, PluginType, UpdateAuthority},
    DataBlob, PluginRegistryV1Safe, RegistryRecordSafe,
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
        plugin_type: u8,
        plugin_slice: &mut &[u8],
        authority: PluginAuthority,
    ) -> Result<Self, std::io::Error> {
        let processed_plugin = if let Some(plugin_type) = PluginType::from_u8(plugin_type) {
            let plugin = Plugin::deserialize(plugin_slice)?;
            let indexable_plugin_schema = IndexablePluginSchemaV1 {
                index,
                offset,
                authority,
                data: plugin,
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
                    records[0].plugin_type,
                    &mut plugin_slice,
                    records[0].authority.clone(),
                )?;

                indexable_asset.add_processed_plugin(processed_plugin);
            }

            if let Some(record) = registry_records.last() {
                let mut plugin_slice =
                    &account[record.offset as usize..header.plugin_registry_offset as usize];

                let processed_plugin = ProcessedPlugin::from_data(
                    registry_records.len() as u64 - 1,
                    record.offset,
                    record.plugin_type,
                    &mut plugin_slice,
                    record.authority.clone(),
                )?;

                indexable_asset.add_processed_plugin(processed_plugin);
            }
        }
        Ok(indexable_asset)
    }
}
