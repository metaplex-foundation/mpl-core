use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::{
    plugins::PluginType,
    state::{Authority, DataBlob},
};

use super::{approve, PluginValidation, PluginValidationContext, ValidationResult};

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
    fn validate_burn(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if ctx.self_authority
            == (&Authority::Address {
                address: *ctx.authority_info.key,
            })
        {
            approve!()
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    fn validate_transfer(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if ctx.self_authority
            == (&Authority::Address {
                address: *ctx.authority_info.key,
            })
        {
            approve!()
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
            && PluginType::from(ctx.target_plugin.unwrap()) == PluginType::TransferDelegate
        {
            approve!()
        } else {
            Ok(ValidationResult::Pass)
        }
    }
}
