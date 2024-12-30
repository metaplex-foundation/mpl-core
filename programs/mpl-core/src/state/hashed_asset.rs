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
    const BASE_LEN: usize = 1 // The Key
    + 32; // The hash

    /// Create a new hashed asset.
    pub fn new(hash: [u8; 32]) -> Self {
        Self {
            key: Key::HashedAssetV1,
            hash,
        }
    }
}

impl DataBlob for HashedAssetV1 {
    fn len(&self) -> usize {
        Self::BASE_LEN
    }
}

impl SolanaAccount for HashedAssetV1 {
    fn key() -> Key {
        Key::HashedAssetV1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashed_asset_len() {
        let hashed_asset = HashedAssetV1::new([0; 32]);
        let serialized = hashed_asset.try_to_vec().unwrap();
        assert_eq!(serialized.len(), hashed_asset.len());
    }
}
