use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::{
    error::MplCoreError,
    plugins::{reject, AssetValidationCommon, AssetValidationContext, PluginType},
    state::DataBlob,
};

use crate::plugins::{abstain, PluginValidation, PluginValidationContext, ValidationResult};

/// The permanent freeze plugin allows any authority to lock the asset so it's no longer transferable.
/// The default authority for this plugin is the update authority.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct PermanentFreezeDelegate {
    /// The current state of the asset and whether or not it's transferable.
    pub frozen: bool, // 1
}

impl PermanentFreezeDelegate {
    /// Initialize the PermanentFreezeDelegate plugin, unfrozen by default.
    pub fn new() -> Self {
        Self { frozen: false }
    }
}

impl Default for PermanentFreezeDelegate {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBlob for PermanentFreezeDelegate {
    fn get_initial_size() -> usize {
        1
    }

    fn get_size(&self) -> usize {
        1
    }
}

impl PluginValidation for PermanentFreezeDelegate {
    fn validate_burn(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        _asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if self.frozen {
            reject!()
        } else {
            abstain!()
        }
    }

    fn validate_transfer(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        _asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if self.frozen {
            reject!()
        } else {
            abstain!()
        }
    }

    fn validate_add_plugin(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // This plugin can only be added at creation time, so we
        // always reject it.
        if let AssetValidationContext::AddPlugin { new_plugin } = asset_ctx {
            if PluginType::from(new_plugin) == PluginType::PermanentFreezeDelegate {
                reject!()
            } else {
                abstain!()
            }
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }

    /// Validate the remove plugin lifecycle action.
    fn validate_remove_plugin(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        _asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if self.frozen {
            reject!()
        } else {
            abstain!()
        }
    }
}
