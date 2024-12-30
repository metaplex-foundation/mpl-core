use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::state::DataBlob;

use crate::plugins::{
    abstain, approve, PluginValidation, PluginValidationContext, ValidationResult,
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
    fn len(&self) -> usize {
        // Stateless data blob
        0
    }
}

impl PluginValidation for TransferDelegate {
    fn validate_transfer(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if ctx.resolved_authorities.is_some()
            && ctx
                .resolved_authorities
                .unwrap()
                .contains(ctx.self_authority)
        {
            return approve!();
        }

        abstain!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_delegate_len() {
        let transfer_delegate = TransferDelegate::default();
        let serialized = transfer_delegate.try_to_vec().unwrap();
        assert_eq!(serialized.len(), transfer_delegate.len());
    }
}
