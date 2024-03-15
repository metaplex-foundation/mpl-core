use borsh::BorshDeserialize;
use solana_program::pubkey::Pubkey;

use crate::{
    accounts::{BaseAsset, BaseCollection, PluginHeader, PluginRegistry},
    types::{CompressionProof, HashablePluginSchema, Plugin, RegistryRecord, UpdateAuthority},
    DataBlob,
};

/// Fetch asset and the plugin registry.
pub fn fetch_asset_compression_proof(account: &[u8]) -> Result<CompressionProof, std::io::Error> {
    let asset = BaseAsset::from_bytes(account)?;
    let asset_size = asset.get_size();
    let mut compression_proof = CompressionProof::from_asset(asset.clone(), 0, vec![]);

    if asset_size != account.len() {
        let header = PluginHeader::from_bytes(&account[asset.get_size()..])?;
        let plugin_registry =
            PluginRegistry::from_bytes(&account[(header.plugin_registry_offset as usize)..])?;

        let mut registry_records = plugin_registry.registry;

        // It should already be sorted but we just want to make sure.
        registry_records.sort_by(RegistryRecord::compare_offsets);

        for (i, record) in registry_records.into_iter().enumerate() {
            let plugin = Plugin::deserialize(&mut &account[record.offset as usize..])?;

            let hashable_plugin_schema = HashablePluginSchema {
                index: i as u64,
                authority: record.authority,
                plugin,
            };

            compression_proof.plugins.push(hashable_plugin_schema);
        }
    }
    Ok(compression_proof)
}

/// Fetch collection and the plugin registry.
pub fn fetch_collection_compression_proof(
    account: &[u8],
) -> Result<CompressionProof, std::io::Error> {
    let asset = BaseCollection::from_bytes(account)?;
    let asset_size = asset.get_size();
    let mut compression_proof = CompressionProof::from_collection(asset.clone(), vec![]);

    if asset_size != account.len() {
        let header = PluginHeader::from_bytes(&account[asset.get_size()..])?;
        let plugin_registry =
            PluginRegistry::from_bytes(&account[(header.plugin_registry_offset as usize)..])?;

        let mut registry_records = plugin_registry.registry;

        // It should already be sorted but we just want to make sure.
        registry_records.sort_by(RegistryRecord::compare_offsets);

        for (i, record) in registry_records.into_iter().enumerate() {
            let plugin = Plugin::deserialize(&mut &account[record.offset as usize..])?;

            let hashable_plugin_schema = HashablePluginSchema {
                index: i as u64,
                authority: record.authority,
                plugin,
            };

            compression_proof.plugins.push(hashable_plugin_schema);
        }
    }
    Ok(compression_proof)
}

impl CompressionProof {
    /// Create a new `CompressionProof` from a BaseAsset.  Note this uses a passed-in `seq` rather than
    /// the one contained in `asset` to avoid errors.
    pub fn from_asset(asset: BaseAsset, seq: u64, plugins: Vec<HashablePluginSchema>) -> Self {
        Self {
            owner: asset.owner,
            update_authority: asset.update_authority,
            name: asset.name,
            uri: asset.uri,
            seq,
            plugins,
        }
    }

    /// Create a new `CompressionProof` from a BaseCollecton.
    pub fn from_collection(asset: BaseCollection, plugins: Vec<HashablePluginSchema>) -> Self {
        Self {
            owner: Pubkey::default(),
            update_authority: UpdateAuthority::Address(asset.update_authority),
            name: asset.name,
            uri: asset.uri,
            seq: 0,
            plugins,
        }
    }
}
