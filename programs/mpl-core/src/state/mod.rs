mod asset;
pub use asset::*;

mod collect;
pub(crate) use collect::*;

mod collection;
pub use collection::*;

mod compression_proof;
pub use compression_proof::*;

mod hashable_plugin_schema;
pub use hashable_plugin_schema::*;

mod hashed_asset_schema;
pub use hashed_asset_schema::*;

mod hashed_asset;
pub use hashed_asset::*;

mod traits;
use strum::{EnumCount, EnumIter};
pub use traits::*;

mod update_authority;
pub use update_authority::*;

use borsh::{BorshDeserialize, BorshSerialize};
use num_derive::{FromPrimitive, ToPrimitive};
use solana_program::pubkey::Pubkey;

/// An enum representing the two types of data, compressed (stored in ledger) and uncompressed (stored in account state).
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub enum DataState {
    /// The data is stored in account state.
    AccountState,
    /// The data is stored in the ledger history (compressed).
    LedgerState,
}

/// Variants representing the different types of authority that can have permissions over plugins.
#[repr(u8)]
#[derive(
    Copy, Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq, PartialOrd, Ord, EnumCount,
)]
pub enum Authority {
    /// No authority, used for immutability.
    None,
    /// The owner of the core asset.
    Owner,
    /// The update authority of the core asset.
    UpdateAuthority,
    /// A pubkey that is the authority over a plugin.
    Address {
        /// The address of the authority.
        address: Pubkey,
    },
}

impl Authority {
    const BASE_LEN: usize = 1; // 1 byte for the discriminator
}

impl DataBlob for Authority {
    fn len(&self) -> usize {
        Self::BASE_LEN
            + if let Authority::Address { .. } = self {
                32
            } else {
                0
            }
    }
}

/// An enum representing account discriminators.
#[derive(
    Clone,
    Copy,
    BorshSerialize,
    BorshDeserialize,
    Debug,
    PartialEq,
    Eq,
    ToPrimitive,
    FromPrimitive,
    EnumIter,
)]
pub enum Key {
    /// Uninitialized or invalid account.
    Uninitialized,
    /// An account holding an uncompressed asset.
    AssetV1,
    /// An account holding a compressed asset.
    HashedAssetV1,
    /// A discriminator indicating the plugin header.
    PluginHeaderV1,
    /// A discriminator indicating the plugin registry.
    PluginRegistryV1,
    /// A discriminator indicating the collection.
    CollectionV1,
}

impl Key {
    const BASE_LEN: usize = 1; // 1 byte for the discriminator
}

impl DataBlob for Key {
    fn len(&self) -> usize {
        Self::BASE_LEN
    }
}

#[cfg(test)]
mod tests {
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn test_authority_len() {
        let authorities = vec![
            Authority::None,
            Authority::Owner,
            Authority::UpdateAuthority,
            Authority::Address {
                address: Pubkey::default(),
            },
        ];
        assert_eq!(
            authorities.len(),
            Authority::COUNT,
            "Must test all Authority variants"
        );
        for authority in authorities {
            let serialized = authority.try_to_vec().unwrap();
            assert_eq!(serialized.len(), authority.len());
        }
    }

    #[test]
    fn test_key_len() {
        for key in Key::iter() {
            let serialized = key.try_to_vec().unwrap();
            assert_eq!(serialized.len(), key.len());
        }
    }
}
