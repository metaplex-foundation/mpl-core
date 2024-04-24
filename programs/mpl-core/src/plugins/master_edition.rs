use borsh::{BorshDeserialize, BorshSerialize};

use super::PluginValidation;

/// The edition plugin allows the creator to set an edition number on the asset
/// The default authority for this plugin is the creator.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Default, Debug, PartialEq, Eq)]
pub struct MasterEdition {
    /// The edition number.
    pub max_supply: u32,
    /// optional master edition name
    pub name: Option<String>,
    /// optional master edition uri
    pub uri: Option<String>,
}

impl PluginValidation for MasterEdition {}
