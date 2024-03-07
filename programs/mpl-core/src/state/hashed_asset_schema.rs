use borsh::{BorshDeserialize, BorshSerialize};
use std::cmp::Ordering;

use crate::{
    plugins::Plugin,
    state::{Authority, Compressible},
};

/// A type that stores a plugin's authorities and deserialized data into a
/// schema that will be later hashed into a hashed asset.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct HashablePluginSchema {
    /// This is the order the plugins are stored in the account, allowing us
    /// to keep track of their order in the hashing.
    pub index: usize,
    /// The authorities who have permission to utilize a plugin.
    pub authority: Authority,
    /// The deserialized plugin.
    pub plugin: Plugin,
}

impl HashablePluginSchema {
    /// Associated function for sorting `RegistryRecords` by offset.
    pub fn compare_indeces(a: &HashablePluginSchema, b: &HashablePluginSchema) -> Ordering {
        a.index.cmp(&b.index)
    }
}

impl Compressible for HashablePluginSchema {}

/// The hashed asset schema is a schema that contains a hash of the asset and a vec of plugin hashes.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct HashedAssetSchema {
    /// The hash of the asset.
    pub asset_hash: [u8; 32],
    /// A vec of plugin hashes.
    pub plugin_hashes: Vec<[u8; 32]>,
}

impl Compressible for HashedAssetSchema {}
