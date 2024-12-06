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
    /// Create a new hashed asset.
    pub fn new(hash: [u8; 32]) -> Self {
        Self {
            key: Key::HashedAssetV1,
            hash,
        }
    }
}

impl DataBlob for HashedAssetV1 {
    const BASE_LEN: usize = 1 // The Key
    + 32; // The hash

    fn len(&self) -> usize {
        Self::BASE_LEN
    }
}

impl SolanaAccount for HashedAssetV1 {
    fn key() -> Key {
        Key::HashedAssetV1
    }
}
