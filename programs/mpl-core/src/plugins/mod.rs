mod burn;
mod freeze;
mod lifecycle;
mod plugin_header;
mod plugin_registry;
mod royalties;
mod transfer;
mod update_delegate;
mod utils;

pub use burn::*;
pub use freeze::*;
pub use lifecycle::*;
use num_derive::ToPrimitive;
pub use plugin_header::*;
pub use plugin_registry::*;
pub use royalties::*;
use strum::EnumCount;
pub use transfer::*;
pub use update_delegate::*;

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
};
pub use utils::*;

use borsh::{BorshDeserialize, BorshSerialize};

use crate::{
    error::MplCoreError,
    state::{Authority, Compressible, DataBlob},
};

/// Definition of the plugin variants, each containing a link to the plugin struct.
#[repr(u16)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum Plugin {
    /// Reserved for uninitialized or invalid plugins.
    Reserved,
    /// Royalties plugin.
    Royalties(Royalties),
    /// Freeze plugin.
    Freeze(Freeze),
    /// Burn plugin.
    Burn(Burn),
    /// Transfer plugin.
    Transfer(Transfer),
    /// Update Delegate plugin.
    UpdateDelegate(UpdateDelegate),
}

impl Plugin {
    /// Get the default authority for a plugin which defines who must allow the plugin to be created.
    pub fn default_authority(&self) -> Authority {
        // match self {
        //     Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
        //     Plugin::Royalties(_) => Ok(Authority::UpdateAuthority),
        //     Plugin::Freeze(_) => Ok(Authority::Owner),
        //     Plugin::Burn(_) => Ok(Authority::Owner),
        //     Plugin::Transfer(_) => Ok(Authority::Owner),
        //     Plugin::UpdateDelegate(_) => Ok(Authority::UpdateAuthority),
        // }

        PluginType::from(self).default_authority()
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
#[derive(
    Clone,
    Copy,
    Debug,
    BorshSerialize,
    BorshDeserialize,
    Eq,
    PartialEq,
    ToPrimitive,
    EnumCount,
    Default,
    PartialOrd,
    Ord,
)]
pub enum PluginType {
    /// Reserved plugin.
    #[default]
    Reserved,
    /// Royalties plugin.
    Royalties,
    /// Freeze plugin.
    Freeze,
    /// Burn plugin.
    Burn,
    /// Transfer plugin.
    Transfer,
    /// Update Delegate plugin.
    UpdateDelegate,
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
            Plugin::Reserved => PluginType::Reserved,
            Plugin::Royalties(_) => PluginType::Royalties,
            Plugin::Freeze(_) => PluginType::Freeze,
            Plugin::Burn(_) => PluginType::Burn,
            Plugin::Transfer(_) => PluginType::Transfer,
            Plugin::UpdateDelegate(_) => PluginType::UpdateDelegate,
        }
    }
}

impl PluginType {
    /// Get the default authority for a plugin which defines who must allow the plugin to be created.
    pub fn default_authority(&self) -> Authority {
        match self {
            PluginType::Reserved => Authority::None,
            PluginType::Royalties => Authority::UpdateAuthority,
            PluginType::Freeze => Authority::Owner,
            PluginType::Burn => Authority::Owner,
            PluginType::Transfer => Authority::Owner,
            PluginType::UpdateDelegate => Authority::UpdateAuthority,
        }
    }
}
