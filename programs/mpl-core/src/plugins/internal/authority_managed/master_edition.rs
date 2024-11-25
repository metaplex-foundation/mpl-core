use borsh::{BorshDeserialize, BorshSerialize};

use crate::{plugins::PluginValidation, state::DataBlob};

/// The master edition plugin allows the creator to specify details on the master edition including max supply, name, and uri.
/// The default authority for this plugin is the creator.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Default, Debug, PartialEq, Eq)]
pub struct MasterEdition {
    /// The max supply of editions
    pub max_supply: Option<u32>, // 1 + optional 4
    /// optional master edition name
    pub name: Option<String>, // 1 + optional 4
    /// optional master edition uri
    pub uri: Option<String>, // 1 + optional 4
}

impl PluginValidation for MasterEdition {}

impl DataBlob for MasterEdition {
    fn get_initial_size() -> usize {
        1 + 1 + 1
    }

    fn get_size(&self) -> usize {
        1 + self.max_supply.map_or(0, |_| 4)
            + 1
            + self.name.as_ref().map_or(0, |name| 4 + name.len())
            + 1
            + self.uri.as_ref().map_or(0, |uri| 4 + uri.len())
    }
}
