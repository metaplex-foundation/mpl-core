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

impl MasterEdition {
    const BASE_LEN: usize = 1 // The max_supply option
    + 1 // The name option
    + 1; // The uri option
}

impl PluginValidation for MasterEdition {}

impl DataBlob for MasterEdition {
    fn len(&self) -> usize {
        Self::BASE_LEN
            + self.max_supply.map_or(0, |_| 4)
            + self.name.as_ref().map_or(0, |name| 4 + name.len())
            + self.uri.as_ref().map_or(0, |uri| 4 + uri.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_edition_default_len() {
        let master_edition = MasterEdition::default();
        let serialized = master_edition.try_to_vec().unwrap();
        assert_eq!(serialized.len(), master_edition.len());
    }

    #[test]
    fn test_master_edition_len() {
        let master_edition = MasterEdition {
            max_supply: Some(100),
            name: Some("test".to_string()),
            uri: Some("test".to_string()),
        };
        let serialized = master_edition.try_to_vec().unwrap();
        assert_eq!(serialized.len(), master_edition.len());
    }
}
