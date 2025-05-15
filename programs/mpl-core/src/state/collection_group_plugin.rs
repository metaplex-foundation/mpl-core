use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::pubkey::Pubkey;

use crate::state::{DataBlob, Key, SolanaAccount};

/// A plugin account stored on a Collection that tracks its membership in multiple Groups.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, PartialEq, Eq, ShankAccount)]
pub struct CollectionGroupPluginV1 {
    /// Account discriminator.
    pub key: Key, // 1
    /// The collection that owns this plugin.
    pub collection: Pubkey, // 32
    /// The groups this collection belongs to.
    pub groups: Vec<Pubkey>, // 4 + n * 32
}

impl CollectionGroupPluginV1 {
    /// Base length that does not depend on the number of groups.
    const BASE_LEN: usize = 1 // key
        + 32 // collection pubkey
        + 4; // groups length prefix

    /// Create a new plugin instance with no group memberships.
    pub fn new(collection: Pubkey) -> Self {
        Self {
            key: Key::CollectionGroupPluginV1,
            collection,
            groups: Vec::new(),
        }
    }

    /// Compute the required size for account allocation, given a maximum number of groups.
    pub fn size(max_groups: usize) -> usize {
        Self::BASE_LEN + max_groups * 32
    }
}

impl DataBlob for CollectionGroupPluginV1 {
    fn len(&self) -> usize {
        Self::BASE_LEN + self.groups.len() * 32
    }
}

impl SolanaAccount for CollectionGroupPluginV1 {
    fn key() -> Key {
        Key::CollectionGroupPluginV1
    }
}
