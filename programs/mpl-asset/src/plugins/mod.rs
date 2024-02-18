mod collection;
mod delegate;
mod royalties;
mod utils;

pub use collection::*;
pub use delegate::*;
pub use royalties::*;

use shank::ShankAccount;
pub use utils::*;

use borsh::{BorshDeserialize, BorshSerialize};

use crate::{
    state::{Authority, Key},
    utils::DataBlob,
};

// macro_rules! plugin_instruction {
//     ($a:expr, $b:expr) => {
//         (($a as u32) << 16u32) | ($b as u32)
//     };
// }

#[repr(u16)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum Plugin {
    Reserved,
    Royalties,
    MasterEdition,
    PrintEdition,
    Delegate(Delegate),
    Inscription,
}

// #[repr(u32)]
// pub enum NonFungibleInstructions {
//     Create = plugin_instruction!(Plugin::Metadata, 0),
//     Transfer = plugin_instruction!(Plugin::Metadata, 1),
//     Burn = plugin_instruction!(Plugin::Metadata, 2),
//}

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

// impl PluginRegistry {
//     pub fn load(account: &AccountInfo, offset: usize) -> Result<Self, ProgramError> {
//         let mut bytes: &[u8] = &(*account.data).borrow()[offset..];
//         PluginRegistry::deserialize(&mut bytes).map_err(|error| {
//             msg!("Error: {}", error);
//             MplAssetError::DeserializationError.into()
//         })
//     }

//     pub fn save(&self, account: &AccountInfo, offset: usize) -> ProgramResult {
//         borsh::to_writer(&mut account.data.borrow_mut()[offset..], self).map_err(|error| {
//             msg!("Error: {}", error);
//             MplAssetError::SerializationError.into()
//         })
//     }
// }

impl DataBlob for PluginRegistry {
    fn key() -> Key {
        Key::PluginRegistry
    }

    fn get_initial_size() -> usize {
        9
    }

    fn get_size(&self) -> usize {
        9 //TODO: Fix this
    }
}

// pub trait PluginTrait
// where
//     Self: BorshSerialize + BorshDeserialize + Clone + std::fmt::Debug + Sized,
// {
//     fn get_plugin() -> Plugin;
//     fn get_authority(&self) -> Option<Pubkey> {
//         None
//     }
// }
