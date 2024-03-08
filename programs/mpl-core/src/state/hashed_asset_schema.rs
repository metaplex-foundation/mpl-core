use borsh::{BorshDeserialize, BorshSerialize};

use crate::state::Compressible;

/// The hashed asset schema is a schema that contains a hash of the asset and a vec of plugin hashes.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct HashedAssetSchema {
    /// The hash of the asset.
    pub asset_hash: [u8; 32],
    /// A vec of plugin hashes.
    pub plugin_hashes: Vec<[u8; 32]>,
}

impl Compressible for HashedAssetSchema {}
