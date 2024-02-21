mod asset;
pub use asset::*;

mod hashed_asset;
pub use hashed_asset::*;

mod traits;
pub use traits::*;

use crate::plugins::Plugin;
use borsh::{BorshDeserialize, BorshSerialize};
use num_derive::{FromPrimitive, ToPrimitive};
use solana_program::pubkey::Pubkey;

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub enum DataState {
    AccountState,
    LedgerState,
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub enum Authority {
    None,
    Owner,
    UpdateAuthority,
    Pubkey { address: Pubkey },
    Permanent { address: Pubkey },
    SameAs { plugin: Plugin },
    Collection,
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub enum ExtraAccounts {
    None,
    SplHook {
        extra_account_metas: Pubkey,
    },
    MplHook {
        mint_pda: Option<Pubkey>,
        collection_pda: Option<Pubkey>,
        owner_pda: Option<Pubkey>,
    },
}
#[derive(
    Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, ToPrimitive, FromPrimitive,
)]
pub enum Key {
    Uninitialized,
    Asset,
    HashedAsset,
    PluginHeader,
    PluginRegistry,
}

impl Key {
    pub fn get_initial_size() -> usize {
        1
    }
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum MigrationLevel {
    MigrateOnly,
    MigrateAndBurn,
}

//TODO: Implement this struct
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub struct CompressionProof {
    pub key: Key,                 //1
    pub update_authority: Pubkey, //32
    pub owner: Pubkey,            //32
    pub name: String,             //4
    pub uri: String,              //4
}
