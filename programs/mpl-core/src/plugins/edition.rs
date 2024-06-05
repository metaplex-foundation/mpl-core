use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::state::Authority;

use super::{Plugin, PluginType, PluginValidation, PluginValidationContext, ValidationResult};

/// The edition plugin allows the creator to set an edition number on the asset
/// The default authority for this plugin is the creator.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, Default, PartialEq, Eq)]
pub struct Edition {
    /// The edition number.
    pub number: u32,
}

impl PluginValidation for Edition {
    fn validate_add_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // This plugin can only be added at creation time, so we
        // always reject it.
        match ctx.target_plugin {
            Some(Plugin::Edition(_edition)) => {
                solana_program::msg!("Edition: Rejected");
                Ok(ValidationResult::Rejected)
            }
            _ => Ok(ValidationResult::Pass),
        }
    }

    fn validate_remove_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // This plugin cannot be removed
        // always reject it.
        match ctx.target_plugin {
            Some(Plugin::Edition(_edition)) => {
                solana_program::msg!("Edition: Rejected");
                Ok(ValidationResult::Rejected)
            }
            _ => Ok(ValidationResult::Pass),
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
            && PluginType::from(ctx.target_plugin.unwrap()) == PluginType::Edition
        {
            solana_program::msg!("Edition: Approved");
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }
}
