use borsh::BorshDeserialize;

use crate::{
    accounts::{BaseAsset, PluginHeader, PluginRegistry},
    types::{CompressionProof, HashablePluginSchema, Plugin, RegistryRecord},
    DataBlob,
};

/// Fetch asset and the plugin registry.
pub fn fetch_all(account: &[u8]) -> Result<CompressionProof, std::io::Error> {
    let asset = BaseAsset::from_bytes(account)?;
    let asset_size = asset.get_size();
    let mut compression_proof = CompressionProof::new(asset.clone(), 0, vec![]);

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
    /// Create a new `CompressionProof`.  Note this uses a passed-in `seq` rather than
    /// the one contained in `asset` to avoid errors.
    pub fn new(asset: BaseAsset, seq: u64, plugins: Vec<HashablePluginSchema>) -> Self {
        Self {
            owner: asset.owner,
            update_authority: asset.update_authority,
            name: asset.name,
            uri: asset.uri,
            seq,
            plugins,
        }
    }
}
