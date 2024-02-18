mod collection;
mod royalties;
mod utils;

pub use collection::*;
pub use royalties::*;

use shank::ShankAccount;
pub use utils::*;

use borsh::{BorshDeserialize, BorshSerialize};

use crate::state::{Authority, DataBlob, Key, SolanaAccount};

#[repr(u16)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum Plugin {
    Reserved,
    Royalties,
    Delegate(Delegate),
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
pub struct RegistryData {
    pub offset: usize,
    pub authorities: Vec<Authority>,
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
pub struct RegistryRecord {
    pub key: Key,
    pub data: RegistryData,
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
pub struct ExternalPluginRecord {
    pub authority: Authority,
    pub data: RegistryData,
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct PluginRegistry {
    pub key: Key,                                    // 1
    pub registry: Vec<RegistryRecord>,               // 4
    pub external_plugins: Vec<ExternalPluginRecord>, // 4
}

impl DataBlob for PluginRegistry {
    fn get_initial_size() -> usize {
        9
    }

    fn get_size(&self) -> usize {
        9 //TODO: Fix this
    }
}

impl SolanaAccount for PluginRegistry {
    fn key() -> Key {
        Key::PluginRegistry
    }
}
