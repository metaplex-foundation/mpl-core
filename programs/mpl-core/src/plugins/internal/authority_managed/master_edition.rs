use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::{
    error::MplCoreError,
    plugins::{abstain, PluginType, PluginValidation, PluginValidationContext, ValidationResult},
    state::DataBlob,
};

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

impl PluginValidation for MasterEdition {
    fn validate_create(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // Target plugin doesn't need to be populated for create, so we check if it exists, otherwise we pass.
        if let Some(target_plugin) = ctx.target_plugin {
            // You can't create the master edition plugin on an asset.
            if PluginType::from(target_plugin) == PluginType::MasterEdition
                && ctx.asset_info.is_some()
            {
                return Err(MplCoreError::PluginNotAllowedOnAsset.into());
            }
        }

        abstain!()
    }

    fn validate_add_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // Target plugin must be populated for add_plugin.
        let target_plugin = ctx.target_plugin.ok_or(MplCoreError::InvalidPlugin)?;

        // You can't add the master edition plugin to an asset.
        if PluginType::from(target_plugin) == PluginType::MasterEdition && ctx.asset_info.is_some()
        {
            Err(MplCoreError::PluginNotAllowedOnAsset.into())
        } else {
            Ok(ValidationResult::Pass)
        }
    }
}

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
