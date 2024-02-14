mod asset;
mod royalties;

pub use asset::*;
use borsh::{BorshDeserialize, BorshSerialize};
pub use royalties::*;

use std::collections::HashMap;

use solana_program::{program_error::ProgramError, pubkey::Pubkey};

macro_rules! interface_instruction {
    ($a:expr, $b:expr) => {
        (($a as u32) << 16u32) | ($b as u32)
    };
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, BorshSerialize, BorshDeserialize)]
pub enum Interface {
    Reserved,
    Asset,
    HashedAsset,
    Royalties,
    MasterEdition,
    PrintEdition,
    Delegate,
    Inscription,
}

#[repr(u32)]
pub enum NonFungibleInstructions {
    Create = interface_instruction!(Interface::Asset, 0),
    Transfer = interface_instruction!(Interface::Asset, 1),
    Burn = interface_instruction!(Interface::Asset, 2),
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

pub trait Compressible {
    fn hash(&self) -> Result<[u8; 32], ProgramError>;
}
