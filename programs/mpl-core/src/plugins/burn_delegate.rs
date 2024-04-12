use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::{
    plugins::PluginType,
    state::{Authority, DataBlob},
};

use super::{PluginValidation, PluginValidationContext, ValidationResult};

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
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if ctx.self_authority
            == (&Authority::Address {
                address: *ctx.authority_info.key,
            })
        {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    /// Validate the revoke plugin authority lifecycle action.
    fn validate_revoke_plugin_authority(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if ctx.self_authority
            == &(Authority::Address {
                address: *ctx.authority_info.key,
            })
            && ctx.target_plugin.is_some()
            && PluginType::from(ctx.target_plugin.unwrap()) == PluginType::BurnDelegate
        {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }
}
