use borsh::{BorshDeserialize, BorshSerialize};

use super::PluginValidation;

/// The master edition plugin allows the creator to specify details on the master edition including max supply, name, and uri.
/// The default authority for this plugin is the creator.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Default, Debug, PartialEq, Eq)]
pub struct MasterEdition {
    /// The max supply of editions
    pub max_supply: Option<u32>,
    /// optional master edition name
    pub name: Option<String>,
    /// optional master edition uri
    pub uri: Option<String>,
}

impl PluginValidation for MasterEdition {}
