mod asset;
pub use asset::*;

mod hashed_asset;
pub use hashed_asset::*;

mod hashed_asset_schema;
pub use hashed_asset_schema::*;

mod traits;
pub use traits::*;

mod collection;
pub use collection::*;

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
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
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
    /// A pubkey that is the permanent authority over a plugin.
    Permanent {
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
    Asset,
    /// An account holding a compressed asset.
    HashedAsset,
    /// A discriminator indicating the plugin header.
    PluginHeader,
    /// A discriminator indicating the plugin registry.
    PluginRegistry,
    /// A discriminator indicating the collection.
    Collection,
}

impl Key {
    /// Get the size of the Key.
    pub fn get_initial_size() -> usize {
        1
    }
}

//TODO: Implement this struct
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
    /// The plugins for the asset.
    pub plugins: Vec<HashablePluginSchema>, //4
}
