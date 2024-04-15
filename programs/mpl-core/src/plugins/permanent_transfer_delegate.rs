use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::state::DataBlob;

use super::{PluginType, PluginValidation, PluginValidationContext, ValidationResult};

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
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // This plugin can only be added at creation time, so we
        // always reject it.
        if ctx.target_plugin.is_some()
            && PluginType::from(ctx.target_plugin.unwrap()) == PluginType::PermanentTransferDelegate
        {
            solana_program::msg!("PermanentTransferDelegate: Rejected");
            Ok(ValidationResult::Rejected)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    fn validate_revoke_plugin_authority(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        solana_program::msg!("PermanentTransferDelegate: Approved");
        Ok(ValidationResult::Approved)
    }

    fn validate_transfer(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(resolved_authorities) = ctx.resolved_authorities {
            if resolved_authorities.contains(ctx.self_authority) {
                solana_program::msg!("PermanentTransferDelegate: ForceApproved");
                return Ok(ValidationResult::ForceApproved);
            }
        }

        Ok(ValidationResult::Pass)
    }
}
