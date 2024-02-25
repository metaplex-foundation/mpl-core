use crate::state::Compressible;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct PluginHash {
    pub plugin_authorities_hash: [u8; 32],
    pub plugin_hash: [u8; 32],
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct HashedAssetSchema {
    pub asset_hash: [u8; 32],
    pub plugin_hashes: Vec<PluginHash>,
}

impl Compressible for HashedAssetSchema {}
