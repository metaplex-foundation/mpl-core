mod add_blocker;
mod app_data;
mod attributes;
mod autograph;
mod burn_delegate;
mod data_section;
mod edition;
mod external_plugin_adapters;
mod freeze_delegate;
mod immutable_metadata;
mod lifecycle;
mod lifecycle_hook;
mod linked_app_data;
mod linked_lifecycle_hook;
mod master_edition;
mod oracle;
mod permanent_burn_delegate;
mod permanent_freeze_delegate;
mod permanent_transfer_delegate;
mod plugin_header;
mod plugin_registry;
mod royalties;
mod transfer;
mod update_delegate;
mod utils;
mod verified_creators;

pub use add_blocker::*;
pub use app_data::*;
pub use attributes::*;
pub use autograph::*;
pub use burn_delegate::*;
pub use data_section::*;
pub use edition::*;
pub use external_plugin_adapters::*;
pub use freeze_delegate::*;
pub use immutable_metadata::*;
pub use lifecycle::*;
pub use lifecycle_hook::*;
pub use linked_app_data::*;
pub use linked_lifecycle_hook::*;
pub use master_edition::*;
pub use oracle::*;
pub use permanent_burn_delegate::*;
pub use permanent_freeze_delegate::*;
pub use permanent_transfer_delegate::*;
pub use plugin_header::*;
pub use plugin_registry::*;
pub use royalties::*;
pub use transfer::*;
pub use update_delegate::*;
pub use utils::*;
pub use verified_creators::*;

use borsh::{BorshDeserialize, BorshSerialize};
use num_derive::ToPrimitive;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
};
use strum::EnumCount;

use crate::{
    error::MplCoreError,
    state::{Authority, Compressible, DataBlob},
};

/// Definition of the plugin variants, each containing a link to the plugin struct.
#[repr(C)]
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Eq, PartialEq)]
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
    /// Edition plugin allows creators to add an edition number to the asset
    Edition(Edition),
    /// Master Edition plugin allows creators to specify the max supply and master edition details
    MasterEdition(MasterEdition),
    /// AddBlocker plugin. Prevents plugins from being added.
    AddBlocker(AddBlocker),
    /// ImmutableMetadata plugin. Makes metadata of the asset immutable.
    ImmutableMetadata(ImmutableMetadata),
    /// VerifiedCreators plugin allows update auth to specify verified creators and additional creators to sign
    VerifiedCreators(VerifiedCreators),
    /// Autograph plugin allows anybody to add their signature to the asset with an optional message
    Autograph(Autograph),
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

/// List of first party plugin types.
#[repr(C)]
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
    PartialOrd,
    Ord,
)]
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
    /// The Edition plugin.
    Edition,
    /// The Master Edition plugin.
    MasterEdition,
    /// AddBlocker plugin.
    AddBlocker,
    /// ImmutableMetadata plugin.
    ImmutableMetadata,
    /// VerifiedCreators plugin.
    VerifiedCreators,
    /// Autograph plugin.
    Autograph,
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
            Plugin::AddBlocker(_) => PluginType::AddBlocker,
            Plugin::ImmutableMetadata(_) => PluginType::ImmutableMetadata,
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
            Plugin::VerifiedCreators(_) => PluginType::VerifiedCreators,
            Plugin::Autograph(_) => PluginType::Autograph,
        }
    }
}

impl PluginType {
    /// Get the default authority for a plugin which defines who must allow the plugin to be created.
    pub fn manager(&self) -> Authority {
        match self {
            PluginType::AddBlocker => Authority::UpdateAuthority,
            PluginType::ImmutableMetadata => Authority::UpdateAuthority,
            PluginType::Royalties => Authority::UpdateAuthority,
            PluginType::FreezeDelegate => Authority::Owner,
            PluginType::BurnDelegate => Authority::Owner,
            PluginType::TransferDelegate => Authority::Owner,
            PluginType::UpdateDelegate => Authority::UpdateAuthority,
            PluginType::PermanentFreezeDelegate => Authority::UpdateAuthority,
            PluginType::Attributes => Authority::UpdateAuthority,
            PluginType::PermanentTransferDelegate => Authority::UpdateAuthority,
            PluginType::PermanentBurnDelegate => Authority::UpdateAuthority,
            PluginType::Edition => Authority::UpdateAuthority,
            PluginType::MasterEdition => Authority::UpdateAuthority,
            PluginType::VerifiedCreators => Authority::UpdateAuthority,
            PluginType::Autograph => Authority::Owner,
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
