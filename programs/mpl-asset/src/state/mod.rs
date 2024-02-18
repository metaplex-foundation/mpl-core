mod asset;
pub use asset::*;

mod plugin_header;
use num_derive::FromPrimitive;
pub use plugin_header::*;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

use crate::plugins::Plugin;

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub enum DataState {
    AccountState,
    LedgerState,
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub enum Authority {
    Owner,
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

pub trait Compressible {
    fn hash(&self) -> Result<[u8; 32], ProgramError>;
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, FromPrimitive)]
pub enum Key {
    Uninitialized,
    Asset,
    HashedAsset,
    Collection,
    HashedCollection,
    PluginHeader,
}

#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub enum MigrationLevel {
    MigrateOnly,
    MigrateAndBurn,
}
