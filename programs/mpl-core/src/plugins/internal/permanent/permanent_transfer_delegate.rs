use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::error::MplCoreError;
use crate::state::DataBlob;

use crate::plugins::{
    abstain, force_approve, reject, AssetValidationCommon, AssetValidationContext, PluginType,
    PluginValidation, PluginValidationContext, ValidationResult,
};

/// The permanent transfer plugin allows any authority to transfer the asset.
/// The default authority for this plugin is the update authority.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, Default, PartialEq, Eq)]
pub struct PermanentTransferDelegate {}

impl DataBlob for PermanentTransferDelegate {
    fn get_initial_size() -> usize {
        0
    }

    fn get_size(&self) -> usize {
        0
    }
}

impl PluginValidation for PermanentTransferDelegate {
    fn validate_add_plugin(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // This plugin can only be added at creation time, so we
        // always reject it.
        if let AssetValidationContext::AddPlugin { new_plugin } = asset_ctx {
            if PluginType::from(new_plugin) == PluginType::PermanentTransferDelegate {
                reject!()
            } else {
                abstain!()
            }
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }

    fn validate_transfer(
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
