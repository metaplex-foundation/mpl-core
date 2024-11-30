use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::{
    plugins::{
        abstain, approve, AssetValidationCommon, AssetValidationContext, PluginValidation,
        PluginValidationContext, ValidationResult,
    },
    state::DataBlob,
};

/// This plugin manages additional permissions to burn.
/// Any authorities approved are given permission to burn the asset on behalf of the owner.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct BurnDelegate {}

impl BurnDelegate {
    /// Initialize the Burn plugin.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for BurnDelegate {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBlob for BurnDelegate {
    fn get_initial_size() -> usize {
        0
    }

    fn get_size(&self) -> usize {
        0
    }
}

impl PluginValidation for BurnDelegate {
    fn validate_burn(
        &self,
        plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        _asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if plugin_ctx.resolved_authorities.is_some()
            && plugin_ctx
                .resolved_authorities
                .unwrap()
                .contains(plugin_ctx.self_authority)
        {
            approve!()
        } else {
            abstain!()
        }
    }
}
