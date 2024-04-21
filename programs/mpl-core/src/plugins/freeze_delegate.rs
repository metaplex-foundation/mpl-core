use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::state::{Authority, DataBlob};

use super::{Plugin, PluginValidation, PluginValidationContext, ValidationResult};

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
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if self.frozen {
            solana_program::msg!("FreezeDelegate: Rejected");
            Ok(ValidationResult::Rejected)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    fn validate_transfer(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if self.frozen {
            solana_program::msg!("FreezeDelegate: Rejected");
            Ok(ValidationResult::Rejected)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    fn validate_approve_plugin_authority(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(Plugin::FreezeDelegate(freeze)) = ctx.target_plugin {
            if freeze.frozen {
                solana_program::msg!("FreezeDelegate: Rejected");
                return Ok(ValidationResult::Rejected);
            }
        }
        Ok(ValidationResult::Pass)
    }

    /// Validate the revoke plugin authority lifecycle action.
    fn validate_revoke_plugin_authority(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(Plugin::FreezeDelegate(freeze)) = ctx.target_plugin {
            if freeze.frozen {
                solana_program::msg!("FreezeDelegate: Rejected");
                return Ok(ValidationResult::Rejected);
            } else if ctx.self_authority
                == &(Authority::Address {
                    address: *ctx.authority_info.key,
                })
            {
                solana_program::msg!("FreezeDelegate: Approved");
                return Ok(ValidationResult::Approved);
            }
        }

        Ok(ValidationResult::Pass)
    }

    /// Validate the remove plugin lifecycle action.
    fn validate_remove_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if ctx.target_plugin.is_some() && self.frozen {
            solana_program::msg!("FreezeDelegate: Rejected");
            Ok(ValidationResult::Rejected)
        } else {
            Ok(ValidationResult::Pass)
        }
    }
}
