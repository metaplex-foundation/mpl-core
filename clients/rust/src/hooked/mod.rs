pub mod plugin;
pub use plugin::*;

pub mod advanced_types;
pub use advanced_types::*;

pub mod asset;
pub use asset::*;

pub mod collection;
pub use collection::*;

#[cfg(feature = "anchor")]
use anchor_lang::prelude::{
    AnchorDeserialize as CrateDeserialize, AnchorSerialize as CrateSerialize,
};
#[cfg(not(feature = "anchor"))]
use borsh::{BorshDeserialize as CrateDeserialize, BorshSerialize as CrateSerialize};
use modular_bitfield::{bitfield, specifiers::B29};
use num_traits::FromPrimitive;
use std::{cmp::Ordering, mem::size_of};

use crate::{
    accounts::{BaseAssetV1, BaseCollectionV1, PluginHeaderV1, PluginRegistryV1},
    errors::MplCoreError,
    types::{ExternalCheckResult, Key, Plugin, PluginType, RegistryRecord},
};
use solana_program::account_info::AccountInfo;

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
            Plugin::MasterEdition(_) => PluginType::MasterEdition,
            Plugin::AddBlocker(_) => PluginType::AddBlocker,
            Plugin::ImmutableMetadata(_) => PluginType::ImmutableMetadata,
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

/// Load the one byte key from the account data at the given offset.
pub fn load_key(account: &AccountInfo, offset: usize) -> Result<Key, std::io::Error> {
    let key = Key::from_u8((*account.data).borrow()[offset]).ok_or(std::io::Error::new(
        std::io::ErrorKind::Other,
        MplCoreError::DeserializationError.to_string(),
    ))?;

    Ok(key)
}

/// A trait for generic blobs of data that have size.
pub trait DataBlob: CrateSerialize + CrateDeserialize {
    /// Get the size of an empty instance of the data blob.
    fn get_initial_size() -> usize;
    /// Get the current size of the data blob.
    fn get_size(&self) -> usize;
}

/// A trait for Solana accounts.
pub trait SolanaAccount: CrateSerialize + CrateDeserialize {
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

/// Bitfield representation of lifecycle permissions for external plugin adapter, third party plugins.
#[bitfield(bits = 32)]
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct ExternalCheckResultBits {
    pub can_listen: bool,
    pub can_approve: bool,
    pub can_reject: bool,
    pub empty_bits: B29,
}

impl From<ExternalCheckResult> for ExternalCheckResultBits {
    fn from(check_result: ExternalCheckResult) -> Self {
        ExternalCheckResultBits::from_bytes(check_result.flags.to_le_bytes())
    }
}

impl From<ExternalCheckResultBits> for ExternalCheckResult {
    fn from(bits: ExternalCheckResultBits) -> Self {
        ExternalCheckResult {
            flags: u32::from_le_bytes(bits.into_bytes()),
        }
    }
}
