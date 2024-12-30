use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::{
    plugins::{abstain, approve, PluginValidation, PluginValidationContext, ValidationResult},
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
    fn len(&self) -> usize {
        // Stateless data blob
        0
    }
}

impl PluginValidation for BurnDelegate {
    fn validate_burn(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if ctx.resolved_authorities.is_some()
            && ctx
                .resolved_authorities
                .unwrap()
                .contains(ctx.self_authority)
        {
            approve!()
        } else {
            abstain!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_burn_delegate_len() {
        let burn_delegate = BurnDelegate::default();
        let serialized = burn_delegate.try_to_vec().unwrap();
        assert_eq!(serialized.len(), burn_delegate.len());
    }
}
