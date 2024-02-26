mod burn;
mod collection;
mod freeze;
mod lifecycle;
mod plugin_header;
mod plugin_registry;
mod royalties;
mod transfer;
mod utils;

pub use burn::*;
pub use collection::*;
pub use freeze::*;
pub use lifecycle::*;
pub use plugin_header::*;
pub use plugin_registry::*;
pub use royalties::*;
pub use transfer::*;

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
    /// Collection plugin.
    Collection(Collection),
}

impl Plugin {
    /// Get the default authority for a plugin which defines who must allow the plugin to be created.
    pub fn default_authority(&self) -> Result<Authority, ProgramError> {
        match self {
            Plugin::Reserved => Err(MplCoreError::InvalidPlugin.into()),
            Plugin::Royalties(_) => Ok(Authority::UpdateAuthority),
            Plugin::Freeze(_) => Ok(Authority::Owner),
            Plugin::Burn(_) => Ok(Authority::Owner),
            Plugin::Transfer(_) => Ok(Authority::Owner),
            Plugin::Collection(_) => Ok(Authority::UpdateAuthority),
        }
    }
}

impl Compressible for Plugin {}

/// List of First Party Plugin types.
#[repr(u16)]
#[derive(Clone, Copy, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
pub enum PluginType {
    /// Reserved plugin.
    Reserved,
    /// Royalties plugin.
    Royalties,
    /// Freeze plugin.
    Freeze,
    /// Burn plugin.
    Burn,
    /// Transfer plugin.
    Transfer,
    /// Collection plugin.
    Collection,
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
            Plugin::Collection(_) => PluginType::Collection,
        }
    }
}

impl Plugin {
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
