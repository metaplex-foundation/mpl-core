use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::{
    error::MplCoreError,
    plugins::{
        abstain, approve, reject, AssetValidationCommon, AssetValidationContext, Plugin,
        PluginValidation, PluginValidationContext, ValidationResult,
    },
    state::DataBlob,
};

/// The freeze delegate plugin allows any authority to lock the asset so it's no longer transferable.
/// The default authority for this plugin is the owner.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct FreezeDelegate {
    /// The current state of the asset and whether or not it's transferable.
    pub frozen: bool, // 1
}

impl FreezeDelegate {
    /// Initialize the Freeze plugin, unfrozen by default.
    pub fn new() -> Self {
        Self { frozen: false }
    }
}

impl Default for FreezeDelegate {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBlob for FreezeDelegate {
    fn get_initial_size() -> usize {
        1
    }

    fn get_size(&self) -> usize {
        1
    }
}

impl PluginValidation for FreezeDelegate {
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

    fn validate_approve_plugin_authority(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let AssetValidationContext::ApprovePluginAuthority { plugin } = asset_ctx {
            if let Plugin::FreezeDelegate(freeze) = plugin {
                if freeze.frozen {
                    return reject!();
                }
            }
            abstain!()
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }

    /// Validate the revoke plugin authority lifecycle action.
    fn validate_revoke_plugin_authority(
        &self,
        plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let AssetValidationContext::RevokePluginAuthority { plugin } = asset_ctx {
            if let Plugin::FreezeDelegate(freeze) = plugin {
                if freeze.frozen {
                    return reject!();
                } else if plugin_ctx.resolved_authorities.is_some()
                    && plugin_ctx
                        .resolved_authorities
                        .unwrap()
                        .contains(plugin_ctx.self_authority)
                {
                    return approve!();
                }
            }

            abstain!()
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }

    /// Validate the remove plugin lifecycle action.
    fn validate_remove_plugin(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let AssetValidationContext::RemovePlugin { .. } = asset_ctx {
            if self.frozen {
                reject!()
            } else {
                abstain!()
            }
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }
}
