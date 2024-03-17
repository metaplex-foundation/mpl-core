use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;

use crate::state::{DataBlob, Key, SolanaAccount};

/// The structure representing the hash of the asset.
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount, PartialEq, Eq)]
pub struct HashedAssetV1 {
    /// The account discriminator.
    pub key: Key, //1
    /// The hash of the asset content.
    pub hash: [u8; 32], //32
}

impl HashedAssetV1 {
    /// The length of the hashed asset account.
    pub const LENGTH: usize = 1 + 32;

    /// Create a new hashed asset.
    pub fn new(hash: [u8; 32]) -> Self {
        Self {
            key: Key::HashedAssetV1,
            hash,
        }
    }
}

impl DataBlob for HashedAssetV1 {
    fn get_initial_size() -> usize {
        HashedAssetV1::LENGTH
    }

    fn get_size(&self) -> usize {
        HashedAssetV1::LENGTH
    }
}

impl SolanaAccount for HashedAssetV1 {
    fn key() -> Key {
        Key::HashedAssetV1
    }
}
