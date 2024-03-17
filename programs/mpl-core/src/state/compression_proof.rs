use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::state::{AssetV1, HashablePluginSchema, UpdateAuthority, Wrappable};

/// A simple struct to store the compression proof of an asset.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct CompressionProof {
    /// The owner of the asset.
    pub owner: Pubkey, //32
    /// The update authority of the asset.
    pub update_authority: UpdateAuthority, //33
    /// The name of the asset.
    pub name: String, //4
    /// The URI of the asset that points to the off-chain data.
    pub uri: String, //4
    /// The sequence number used for indexing with compression.
    pub seq: u64, //8
    /// The plugins for the asset.
    pub plugins: Vec<HashablePluginSchema>, //4
}

impl CompressionProof {
    /// Create a new `CompressionProof`.  Note this uses a passed-in `seq` rather than
    /// the one contained in `asset` to avoid errors.
    pub fn new(asset: AssetV1, seq: u64, plugins: Vec<HashablePluginSchema>) -> Self {
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

impl Wrappable for CompressionProof {}
