use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::state::DataBlob;

use crate::plugins::{
    abstain, approve, AssetValidationCommon, AssetValidationContext, PluginValidation,
    PluginValidationContext, ValidationResult,
};

/// This plugin manages the ability to transfer an asset and any authorities
/// approved are permitted to transfer the asset on behalf of the owner.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct TransferDelegate {}

impl TransferDelegate {
    /// Initialize the Transfer plugin.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for TransferDelegate {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBlob for TransferDelegate {
    fn get_initial_size() -> usize {
        0
    }

    fn get_size(&self) -> usize {
        0
    }
}

impl PluginValidation for TransferDelegate {
    fn validate_transfer(
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
            return approve!();
        }

        abstain!()
    }
}
