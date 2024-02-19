mod asset_signer;
mod collection;
mod delegate;
mod legacy_metadata;
mod royalties;
mod utils;

pub use asset_signer::*;
pub use collection::*;
pub use delegate::*;
pub use legacy_metadata::*;
pub use royalties::*;

use shank::ShankAccount;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
};
pub use utils::*;

use borsh::{BorshDeserialize, BorshSerialize};

use crate::{
    error::MplAssetError,
    state::{Authority, DataBlob, Key, SolanaAccount},
};

#[repr(u16)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum Plugin {
    Reserved,
    Royalties(Royalties),
    Delegate(Delegate),
    LegacyMetadata(LegacyMetadata),
    AssetSigner(AssetSigner),
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum PluginType {
    Reserved,
    Royalties,
    Delegate,
    LegacyMetadata,
    AssetSigner,
}

impl From<&Plugin> for PluginType {
    fn from(plugin: &Plugin) -> Self {
        match plugin {
            Plugin::Reserved => PluginType::Reserved,
            Plugin::Royalties(_) => PluginType::Royalties,
            Plugin::Delegate(_) => PluginType::Delegate,
            Plugin::LegacyMetadata(_) => PluginType::LegacyMetadata,
            Plugin::AssetSigner(_) => PluginType::AssetSigner,
        }
    }
}

impl Plugin {
    pub fn load(account: &AccountInfo, offset: usize) -> Result<Self, ProgramError> {
        let mut bytes: &[u8] = &(*account.data).borrow()[offset..];
        Self::deserialize(&mut bytes).map_err(|error| {
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

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
pub struct RegistryData {
    pub offset: usize,
    pub authorities: Vec<Authority>,
}

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
pub struct RegistryRecord {
    pub plugin_type: PluginType,
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
