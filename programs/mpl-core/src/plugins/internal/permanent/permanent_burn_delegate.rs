use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::{
    error::MplCoreError,
    plugins::{
        abstain, force_approve, reject, AssetValidationCommon, AssetValidationContext, PluginType,
        PluginValidation, PluginValidationContext, ValidationResult,
    },
    state::DataBlob,
};

/// The permanent burn plugin allows any authority to burn the asset.
/// The default authority for this plugin is the update authority.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, Default, PartialEq, Eq)]
pub struct PermanentBurnDelegate {}

impl DataBlob for PermanentBurnDelegate {
    fn get_initial_size() -> usize {
        0
    }

    fn get_size(&self) -> usize {
        0
    }
}

impl PluginValidation for PermanentBurnDelegate {
    fn validate_add_plugin(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // This plugin can only be added at creation time, so we
        // always reject it.
        if let AssetValidationContext::AddPlugin { new_plugin } = asset_ctx {
            if PluginType::from(new_plugin) == PluginType::PermanentBurnDelegate {
                reject!()
            } else {
                abstain!()
            }
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }

    fn validate_burn(
        &self,
        plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        _asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(resolved_authorities) = plugin_ctx.resolved_authorities {
            if resolved_authorities.contains(plugin_ctx.self_authority) {
                return force_approve!();
            }
        }

        abstain!()
    }
}
