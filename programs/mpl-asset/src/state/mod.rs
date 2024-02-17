mod asset;
pub use asset::*;

mod plugin_header;
pub use plugin_header::*;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

use crate::error::MplAssetError;
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
}

pub trait Compressible {
    fn hash(&self) -> Result<[u8; 32], ProgramError>;
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub enum Key {
    Uninitialized,
    Asset,
    HashedAsset,
    Collection,
    HashedCollection,
}
