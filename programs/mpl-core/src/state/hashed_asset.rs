use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;

use crate::state::{DataBlob, Key, SolanaAccount};

/// The structure representing the hash of the asset.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount, PartialEq, Eq)]
pub struct HashedAsset {
    /// The account discriminator.
    pub key: Key, //1
    /// The hash of the asset content.
    pub hash: [u8; 32], //32
}

impl HashedAsset {
    /// The length of the hashed asset account.
    pub const LENGTH: usize = 1 + 32;

    /// Create a new hashed asset.
    pub fn new(hash: [u8; 32]) -> Self {
        Self {
            key: Key::HashedAsset,
            hash,
        }
    }
}

impl DataBlob for HashedAsset {
    fn get_initial_size() -> usize {
        HashedAsset::LENGTH
    }

    fn get_size(&self) -> usize {
        HashedAsset::LENGTH
    }
}

impl SolanaAccount for HashedAsset {
    fn key() -> Key {
        Key::HashedAsset
    }
}
