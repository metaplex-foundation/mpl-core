mod non_fungible;
mod royalties;

use borsh::{BorshDeserialize, BorshSerialize};
pub use non_fungible::*;
pub use royalties::*;

use std::collections::HashMap;

use solana_program::pubkey::Pubkey;

macro_rules! interface_instruction {
    ($a:expr, $b:expr) => {
        (($a as u32) << 16u32) | ($b as u32)
    };
}

#[repr(C)]
#[derive(Clone, Copy, Debug, BorshSerialize, BorshDeserialize)]
pub enum Interface {
    Reserved,
    NonFungible,
    Royalties,
    MasterEdition,
    PrintEdition,
    Delegate,
    Inscription,
}

#[repr(u32)]
pub enum NonFungibleInstructions {
    Create = interface_instruction!(Interface::NonFungible, 0),
    Transfer = interface_instruction!(Interface::NonFungible, 1),
    Burn = interface_instruction!(Interface::NonFungible, 2),
}

pub struct RegistryData {
    pub offset: usize,
    pub authorities: Vec<Pubkey>,
}

pub struct InterfaceRegistry {
    pub registry: HashMap<Interface, RegistryData>,
}

pub trait DataStorage {
    fn get_required_length(&self);
    fn save(&self, data: &mut [u8]);
    fn load(&self, data: &[u8]);
    fn load_mut(&self, data: &mut [u8]);
}
