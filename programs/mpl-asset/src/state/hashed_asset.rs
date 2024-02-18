use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;

use crate::state::{DataBlob, Key, SolanaAccount};

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount, PartialEq, Eq)]
pub struct HashedAsset {
    pub key: Key,       //1
    pub hash: [u8; 32], //32
}

impl HashedAsset {
    pub const LENGTH: usize = 1 + 32;

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
