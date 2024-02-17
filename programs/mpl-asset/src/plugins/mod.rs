mod royalties;
pub use royalties::*;

use borsh::{BorshDeserialize, BorshSerialize};
use std::collections::HashMap;

use crate::state::Authority;

// macro_rules! plugin_instruction {
//     ($a:expr, $b:expr) => {
//         (($a as u32) << 16u32) | ($b as u32)
//     };
// }

#[repr(u16)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum Plugin {
    Reserved,
    Asset,
    HashedAsset,
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

pub struct RegistryData {
    pub offset: usize,
    pub authorities: Vec<Authority>,
}

pub struct PluginRegistry {
    pub registry: HashMap<Plugin, RegistryData>,
}

pub trait DataStorage {
    fn get_required_length(&self);
    fn save(&self, data: &mut [u8]);
    fn load(&self, data: &[u8]);
    fn load_mut(&self, data: &mut [u8]);
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
