use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::{error::MplCoreError, utils::load_key};

use super::{PluginType, PluginValidation, PluginValidationContext, ValidationResult};

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
                Err(MplCoreError::PluginNotAllowedOnAsset.into())
            } else {
                Ok(ValidationResult::Pass)
            }
        } else {
            Ok(ValidationResult::Pass)
        }
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
