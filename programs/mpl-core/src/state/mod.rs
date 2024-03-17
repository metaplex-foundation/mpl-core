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

//TODO: Consider making Permanent a subfield of Pubkey.
/// Variants representing the different types of authority that can have permissions over plugins.
#[repr(u8)]
#[derive(Copy, Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub enum Authority {
    /// No authority, used for immutability.
    None,
    /// The owner of the core asset.
    Owner,
    /// The update authority of the core asset.
    UpdateAuthority,
    /// A pubkey that is the authority over a plugin.
    Pubkey {
        /// The address of the authority.
        address: Pubkey,
    },
}

/// Different types of extra accounts that can be passed in for lifecycle hooks.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub enum ExtraAccounts {
    /// No extra accounts.
    None,
    /// Compatible with spl-token-2022 transfer hooks.
    SplHook {
        /// An account meta accounts derived from the account pubkey.
        extra_account_metas: Pubkey,
    },
    /// A simpler method of passing in extra accounts using deterministic PDAs.
    MplHook {
        /// The PDA derived from the account pubkey.
        mint_pda: Option<Pubkey>,
        /// The PDA derived from the collection pubkey.
        collection_pda: Option<Pubkey>,
        /// The PDA derived from the asset owner pubkey.
        owner_pda: Option<Pubkey>,
    },
}

/// An enum representing account discriminators.
#[derive(
    Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, ToPrimitive, FromPrimitive,
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
    /// Get the size of the Key.
    pub fn get_initial_size() -> usize {
        1
    }
}
