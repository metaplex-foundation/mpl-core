mod attributes;
mod burn_delegate;
mod freeze_delegate;
mod lifecycle;
mod permanent_burn_delegate;
mod permanent_freeze_delegate;
mod permanent_transfer_delegate;
mod plugin_header;
mod plugin_registry;
mod royalties;
mod transfer;
mod update_delegate;
mod utils;

pub use attributes::*;
pub use burn_delegate::*;
pub use freeze_delegate::*;
pub use lifecycle::*;
use num_derive::ToPrimitive;
pub use permanent_burn_delegate::*;
pub use permanent_freeze_delegate::*;
pub use permanent_transfer_delegate::*;
pub use plugin_header::*;
pub use plugin_registry::*;
pub use royalties::*;
use shank::ShankType;
pub use transfer::*;
pub use update_delegate::*;
pub use utils::*;

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
};

use borsh::{BorshDeserialize, BorshSerialize};
use strum::EnumCount;

use crate::{
    error::MplCoreError,
    state::{Authority, Compressible, DataBlob},
};

/// Definition of the plugin variants, each containing a link to the plugin struct.
#[repr(u16)]
#[derive(Clone, Debug, Eq, PartialEq, ShankType)]
pub enum Plugin {
    /// Royalties plugin.
    Royalties(Royalties),
    /// Freeze Delegate plugin.
    FreezeDelegate(FreezeDelegate),
    /// Burn Delegate plugin.
    BurnDelegate(BurnDelegate),
    /// Transfer Delegate plugin.
    TransferDelegate(TransferDelegate),
    /// Update Delegate plugin.
    UpdateDelegate(UpdateDelegate),
    /// Permanent Freeze Delegate authority which allows the creator to freeze
    PermanentFreezeDelegate(PermanentFreezeDelegate),
    /// Attributes plugin for arbitrary Key-Value pairs.
    Attributes(Attributes),
    /// Permanent Transfer Delegate authority which allows the creator of an asset to become the person who can transfer an Asset
    PermanentTransferDelegate(PermanentTransferDelegate),
    /// Permanent Burn Delegate authority allows the creator of an asset to become the person who can burn an Asset
    PermanentBurnDelegate(PermanentBurnDelegate),
}

impl BorshSerialize for Plugin {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            Plugin::Royalties(royalties) => {
                PluginType::Royalties.serialize(writer)?;
                royalties.serialize(writer)
            }
            Plugin::FreezeDelegate(freeze_delegate) => {
                PluginType::FreezeDelegate.serialize(writer)?;
                freeze_delegate.serialize(writer)
            }
            Plugin::BurnDelegate(burn_delegate) => {
                PluginType::BurnDelegate.serialize(writer)?;
                burn_delegate.serialize(writer)
            }
            Plugin::TransferDelegate(transfer_delegate) => {
                PluginType::TransferDelegate.serialize(writer)?;
                transfer_delegate.serialize(writer)
            }
            Plugin::UpdateDelegate(update_delegate) => {
                PluginType::UpdateDelegate.serialize(writer)?;
                update_delegate.serialize(writer)
            }
            Plugin::PermanentFreezeDelegate(permanent_freeze_delegate) => {
                PluginType::PermanentFreezeDelegate.serialize(writer)?;
                permanent_freeze_delegate.serialize(writer)
            }
            Plugin::Attributes(attributes) => {
                PluginType::Attributes.serialize(writer)?;
                attributes.serialize(writer)
            }
            Plugin::PermanentTransferDelegate(permanent_transfer_delegate) => {
                PluginType::PermanentTransferDelegate.serialize(writer)?;
                permanent_transfer_delegate.serialize(writer)
            }
            Plugin::PermanentBurnDelegate(permanent_burn_delegate) => {
                PluginType::PermanentBurnDelegate.serialize(writer)?;
                permanent_burn_delegate.serialize(writer)
            }
        }
    }
}

impl BorshDeserialize for Plugin {
    fn deserialize_reader<R: std::io::prelude::Read>(reader: &mut R) -> std::io::Result<Self> {
        let plugin_type = PluginType::deserialize_reader(reader)?;
        match plugin_type {
            PluginType::Royalties => Royalties::deserialize_reader(reader).map(Plugin::Royalties),
            PluginType::FreezeDelegate => {
                FreezeDelegate::deserialize_reader(reader).map(Plugin::FreezeDelegate)
            }
            PluginType::BurnDelegate => {
                BurnDelegate::deserialize_reader(reader).map(Plugin::BurnDelegate)
            }
            PluginType::TransferDelegate => {
                TransferDelegate::deserialize_reader(reader).map(Plugin::TransferDelegate)
            }
            PluginType::UpdateDelegate => {
                UpdateDelegate::deserialize_reader(reader).map(Plugin::UpdateDelegate)
            }
            PluginType::PermanentFreezeDelegate => {
                PermanentFreezeDelegate::deserialize_reader(reader)
                    .map(Plugin::PermanentFreezeDelegate)
            }
            PluginType::Attributes => {
                Attributes::deserialize_reader(reader).map(Plugin::Attributes)
            }
            PluginType::PermanentTransferDelegate => {
                PermanentTransferDelegate::deserialize_reader(reader)
                    .map(Plugin::PermanentTransferDelegate)
            }
            PluginType::PermanentBurnDelegate => {
                PermanentBurnDelegate::deserialize_reader(reader).map(Plugin::PermanentBurnDelegate)
            }
        }
    }
}

impl Plugin {
    /// Get the default authority for a plugin which defines who must allow the plugin to be created.
    pub fn manager(&self) -> Authority {
        PluginType::from(self).manager()
    }

    /// Load and deserialize a plugin from an offset in the account.
    pub fn load(account: &AccountInfo, offset: usize) -> Result<Self, ProgramError> {
        let mut bytes: &[u8] = &(*account.data).borrow()[offset..];
        Self::deserialize(&mut bytes).map_err(|error| {
            msg!("Error: {}", error);
            MplCoreError::DeserializationError.into()
        })
    }

    /// Save and serialize a plugin to an offset in the account.
    pub fn save(&self, account: &AccountInfo, offset: usize) -> ProgramResult {
        borsh::to_writer(&mut account.data.borrow_mut()[offset..], self).map_err(|error| {
            msg!("Error: {}", error);
            MplCoreError::SerializationError.into()
        })
    }
}

impl Compressible for Plugin {}

/// List of First Party Plugin types.
#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, ToPrimitive, EnumCount, PartialOrd, Ord, ShankType)]
pub enum PluginType {
    /// Royalties plugin.
    Royalties,
    /// Freeze Delegate plugin.
    FreezeDelegate,
    /// Burn Delegate plugin.
    BurnDelegate,
    /// Transfer Delegate plugin.
    TransferDelegate,
    /// Update Delegate plugin.
    UpdateDelegate,
    /// The Permanent Freeze Delegate plugin.
    PermanentFreezeDelegate,
    /// The Attributes plugin.
    Attributes,
    /// The Permanent Transfer Delegate plugin.
    PermanentTransferDelegate,
    /// The Permanent Burn Delegate plugin.
    PermanentBurnDelegate,
}

impl BorshSerialize for PluginType {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        match self {
            PluginType::Royalties => 0u16.serialize(writer),
            PluginType::FreezeDelegate => 1u16.serialize(writer),
            PluginType::BurnDelegate => 2u16.serialize(writer),
            PluginType::TransferDelegate => 3u16.serialize(writer),
            PluginType::UpdateDelegate => 4u16.serialize(writer),
            PluginType::PermanentFreezeDelegate => 5u16.serialize(writer),
            PluginType::Attributes => 6u16.serialize(writer),
            PluginType::PermanentTransferDelegate => 7u16.serialize(writer),
            PluginType::PermanentBurnDelegate => 8u16.serialize(writer),
        }
    }
}

impl BorshDeserialize for PluginType {
    fn deserialize_reader<R: std::io::prelude::Read>(reader: &mut R) -> std::io::Result<Self> {
        let value = u16::deserialize_reader(reader)?;
        match value {
            0 => Ok(PluginType::Royalties),
            1 => Ok(PluginType::FreezeDelegate),
            2 => Ok(PluginType::BurnDelegate),
            3 => Ok(PluginType::TransferDelegate),
            4 => Ok(PluginType::UpdateDelegate),
            5 => Ok(PluginType::PermanentFreezeDelegate),
            6 => Ok(PluginType::Attributes),
            7 => Ok(PluginType::PermanentTransferDelegate),
            8 => Ok(PluginType::PermanentBurnDelegate),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid PluginType",
            )),
        }
    }
}

impl DataBlob for PluginType {
    fn get_initial_size() -> usize {
        2
    }

    fn get_size(&self) -> usize {
        2
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
        }
    }
}

impl PluginType {
    /// Get the default authority for a plugin which defines who must allow the plugin to be created.
    pub fn manager(&self) -> Authority {
        match self {
            PluginType::Royalties => Authority::UpdateAuthority,
            PluginType::FreezeDelegate => Authority::Owner,
            PluginType::BurnDelegate => Authority::Owner,
            PluginType::TransferDelegate => Authority::Owner,
            PluginType::UpdateDelegate => Authority::UpdateAuthority,
            PluginType::PermanentFreezeDelegate => Authority::UpdateAuthority,
            PluginType::Attributes => Authority::UpdateAuthority,
            PluginType::PermanentTransferDelegate => Authority::UpdateAuthority,
            PluginType::PermanentBurnDelegate => Authority::UpdateAuthority,
        }
    }
}

/// A pair of a plugin type and an optional authority.
#[repr(C)]
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Eq, Debug, Clone)]
pub(crate) struct PluginAuthorityPair {
    pub(crate) plugin: Plugin,
    pub(crate) authority: Option<Authority>,
}
