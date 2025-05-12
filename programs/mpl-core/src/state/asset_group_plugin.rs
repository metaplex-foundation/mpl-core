use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::pubkey::Pubkey;

use crate::state::{DataBlob, Key, SolanaAccount};

/// A plugin account stored on an Asset that tracks its membership in multiple Groups.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq, Eq, ShankAccount)]
pub struct AssetGroupPluginV1 {
    /// Account discriminator.
    pub key: Key, // 1
    /// The asset that owns this plugin.
    pub asset: Pubkey, // 32
    /// The groups this asset belongs to.
    pub groups: Vec<Pubkey>, // 4 + n * 32
}

impl AssetGroupPluginV1 {
    /// Base length that does not depend on the number of groups.
    const BASE_LEN: usize = 1 // key
        + 32 // asset pubkey
        + 4; // groups length prefix

    /// Create a new plugin instance with no group memberships.
    pub fn new(asset: Pubkey) -> Self {
        Self {
            key: Key::AssetGroupPluginV1,
            asset,
            groups: Vec::new(),
        }
    }

    /// Compute the required size for account allocation, given a maximum number of groups.
    pub fn size(max_groups: usize) -> usize {
        Self::BASE_LEN + max_groups * 32
    }
}

impl DataBlob for AssetGroupPluginV1 {
    fn len(&self) -> usize {
        Self::BASE_LEN + self.groups.len() * 32
    }
}

impl SolanaAccount for AssetGroupPluginV1 {
    fn key() -> Key {
        Key::AssetGroupPluginV1
    }
}
