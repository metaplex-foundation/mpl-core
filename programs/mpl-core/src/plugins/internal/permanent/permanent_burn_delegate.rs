use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::{
    plugins::{
        abstain, force_approve, reject, PluginType, PluginValidation, PluginValidationContext,
        ValidationResult,
    },
    state::DataBlob,
};

/// The permanent burn plugin allows any authority to burn the asset.
/// The default authority for this plugin is the update authority.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, Default, PartialEq, Eq)]
pub struct PermanentBurnDelegate {}

impl DataBlob for PermanentBurnDelegate {
    fn len(&self) -> usize {
        // Stateless data blob
        0
    }
}

impl PluginValidation for PermanentBurnDelegate {
    fn validate_add_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // This plugin can only be added at creation time, so we
        // always reject it.
        if ctx.target_plugin.is_some()
            && PluginType::from(ctx.target_plugin.unwrap()) == PluginType::PermanentBurnDelegate
        {
            reject!()
        } else {
            abstain!()
        }
    }

    fn validate_burn(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(resolved_authorities) = ctx.resolved_authorities {
            if resolved_authorities.contains(ctx.self_authority) {
                return force_approve!();
            }
        }

        abstain!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permanent_burn_delegate_len() {
        let permanent_burn_delegate = PermanentBurnDelegate::default();
        let serialized = permanent_burn_delegate.try_to_vec().unwrap();
        assert_eq!(serialized.len(), permanent_burn_delegate.len());
    }
}
