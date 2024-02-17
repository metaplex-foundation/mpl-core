mod collection;
mod royalties;
mod utils;

pub use collection::*;
pub use royalties::*;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
};
pub use utils::*;

use borsh::{BorshDeserialize, BorshSerialize};

use crate::{
    error::MplAssetError,
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
    Delegate,
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
pub struct PluginRegistry {
    pub registry: Vec<(Key, RegistryData)>, // 4
}

impl PluginRegistry {
    pub fn load(account: &AccountInfo, offset: usize) -> Result<Self, ProgramError> {
        let mut bytes: &[u8] = &(*account.data).borrow()[offset..];
        PluginRegistry::deserialize(&mut bytes).map_err(|error| {
            msg!("Error: {}", error);
            MplAssetError::DeserializationError.into()
        })
    }

    pub fn save(&self, account: &AccountInfo, offset: usize) -> ProgramResult {
        borsh::to_writer(&mut account.data.borrow_mut()[offset..], self).map_err(|error| {
            msg!("Error: {}", error);
            MplAssetError::SerializationError.into()
        })
    }
}

impl DataBlob for PluginRegistry {
    fn get_initial_size() -> usize {
        4
    }

    fn get_size(&self) -> usize {
        4 //TODO: Fix this
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
