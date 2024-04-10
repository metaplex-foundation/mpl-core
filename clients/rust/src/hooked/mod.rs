pub mod plugin;
pub use plugin::*;

pub mod advanced_types;
pub use advanced_types::*;

pub mod asset;
pub use asset::*;

pub mod collection;
pub use collection::*;

use borsh::{BorshDeserialize, BorshSerialize};
use std::{cmp::Ordering, mem::size_of};

use crate::{
    accounts::{BaseAssetV1, BaseCollectionV1, PluginHeaderV1, PluginRegistryV1},
    errors::MplCoreError,
    types::{Key, Plugin, PluginType, RegistryRecord},
};
use solana_program::account_info::AccountInfo;

impl PluginType {
    // Needed to determine if a plugin is a known or unknown type.
    pub fn from_u8(n: u8) -> Option<PluginType> {
        match n {
            0 => Some(PluginType::Royalties),
            1 => Some(PluginType::FreezeDelegate),
            2 => Some(PluginType::BurnDelegate),
            3 => Some(PluginType::TransferDelegate),
            4 => Some(PluginType::UpdateDelegate),
            5 => Some(PluginType::PermanentFreezeDelegate),
            6 => Some(PluginType::Attributes),
            7 => Some(PluginType::PermanentTransferDelegate),
            8 => Some(PluginType::PermanentBurnDelegate),
            9 => Some(PluginType::Edition),
            _ => None,
        }
    }
}

impl From<&Plugin> for PluginType {
    fn from(plugin: &Plugin) -> Self {
        match plugin {
            Plugin::Royalties(_) => PluginType::Royalties,
            Plugin::FreezeDelegate(_) => PluginType::FreezeDelegate,
            Plugin::BurnDelegate(_) => PluginType::BurnDelegate,
            Plugin::TransferDelegate(_) => PluginType::TransferDelegate,
            Plugin::UpdateDelegate(_) => PluginType::UpdateDelegate,
            Plugin::PermanentFreezeDelegate(_) => PluginType::PermanentFreezeDelegate,
            Plugin::Attributes(_) => PluginType::Attributes,
            Plugin::PermanentTransferDelegate(_) => PluginType::PermanentTransferDelegate,
            Plugin::PermanentBurnDelegate(_) => PluginType::PermanentBurnDelegate,
            Plugin::Edition(_) => PluginType::Edition,
        }
    }
}

impl BaseAssetV1 {
    /// The base length of the asset account with an empty name and uri and no seq.
    pub const BASE_LENGTH: usize = 1 + 32 + 33 + 4 + 4 + 1;
}

impl BaseCollectionV1 {
    /// The base length of the collection account with an empty name and uri.
    pub const BASE_LENGTH: usize = 1 + 32 + 4 + 4 + 4 + 4;
}

impl DataBlob for BaseAssetV1 {
    fn get_initial_size() -> usize {
        BaseAssetV1::BASE_LENGTH
    }

    fn get_size(&self) -> usize {
        let mut size = BaseAssetV1::BASE_LENGTH + self.name.len() + self.uri.len();
        if self.seq.is_some() {
            size += size_of::<u64>();
        }
        size
    }
}

impl SolanaAccount for BaseAssetV1 {
    fn key() -> Key {
        Key::AssetV1
    }
}

impl DataBlob for BaseCollectionV1 {
    fn get_initial_size() -> usize {
        Self::BASE_LENGTH
    }

    fn get_size(&self) -> usize {
        Self::BASE_LENGTH + self.name.len() + self.uri.len()
    }
}

impl SolanaAccount for BaseCollectionV1 {
    fn key() -> Key {
        Key::CollectionV1
    }
}

impl SolanaAccount for PluginRegistryV1 {
    fn key() -> Key {
        Key::PluginRegistryV1
    }
}

impl SolanaAccount for PluginHeaderV1 {
    fn key() -> Key {
        Key::PluginHeaderV1
    }
}

impl Key {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Key::Uninitialized),
            1 => Some(Key::AssetV1),
            2 => Some(Key::HashedAssetV1),
            3 => Some(Key::PluginHeaderV1),
            4 => Some(Key::PluginRegistryV1),
            5 => Some(Key::CollectionV1),
            _ => None,
        }
    }
}

/// Load the one byte key from the account data at the given offset.
pub fn load_key(account: &AccountInfo, offset: usize) -> Result<Key, std::io::Error> {
    let key = Key::from_u8((*account.data).borrow()[offset]).ok_or(std::io::Error::new(
        std::io::ErrorKind::Other,
        MplCoreError::DeserializationError.to_string(),
    ))?;

    Ok(key)
}

/// A trait for generic blobs of data that have size.
pub trait DataBlob: BorshSerialize + BorshDeserialize {
    /// Get the size of an empty instance of the data blob.
    fn get_initial_size() -> usize;
    /// Get the current size of the data blob.
    fn get_size(&self) -> usize;
}

/// A trait for Solana accounts.
pub trait SolanaAccount: BorshSerialize + BorshDeserialize {
    /// Get the discriminator key for the account.
    fn key() -> Key;

    /// Load the account from the given account info starting at the offset.
    fn load(account: &AccountInfo, offset: usize) -> Result<Self, std::io::Error> {
        let key = load_key(account, offset)?;

        if key != Self::key() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                MplCoreError::DeserializationError.to_string(),
            ));
        }

        let mut bytes: &[u8] = &(*account.data).borrow()[offset..];
        Self::deserialize(&mut bytes)
    }

    /// Save the account to the given account info starting at the offset.
    fn save(&self, account: &AccountInfo, offset: usize) -> Result<(), std::io::Error> {
        borsh::to_writer(&mut account.data.borrow_mut()[offset..], self)
    }
}

impl RegistryRecord {
    /// Associated function for sorting `RegistryRecords` by offset.
    pub fn compare_offsets(a: &RegistryRecord, b: &RegistryRecord) -> Ordering {
        a.offset.cmp(&b.offset)
    }
}
