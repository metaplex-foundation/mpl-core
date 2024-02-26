use crate::state::Compressible;
use borsh::{BorshDeserialize, BorshSerialize};

/// The plugin hash is a hash of the plugin authority and the plugin itself.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct PluginHash {
    /// The hash of the plugin authorities.
    pub plugin_authorities_hash: [u8; 32],
    /// The hash of the plugin.
    pub plugin_hash: [u8; 32],
}

/// The hashed asset schema is a schema that contains a hash of the asset and a vec of plugin hashes.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct HashedAssetSchema {
    /// The hash of the asset.
    pub asset_hash: [u8; 32],
    /// A vec of plugin hashes.
    pub plugin_hashes: Vec<PluginHash>,
}

impl Compressible for HashedAssetSchema {}
