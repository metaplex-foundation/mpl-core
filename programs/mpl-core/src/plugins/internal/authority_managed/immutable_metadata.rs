use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::{
    plugins::{reject, PluginValidation, PluginValidationContext, ValidationResult},
    state::DataBlob,
};

/// The immutable metadata plugin allows its authority to prevent plugin's meta from changing.
/// The default authority for this plugin is None.

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, Default)]
pub struct ImmutableMetadata {}

impl DataBlob for ImmutableMetadata {
    fn len(&self) -> usize {
        // Stateless data blob
        0
    }
}

impl PluginValidation for ImmutableMetadata {
    /// Validate the update lifecycle action.
    fn validate_update(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        reject!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_immutable_metadata_len() {
        let immutable_metadata = ImmutableMetadata {};
        let serialized = immutable_metadata.try_to_vec().unwrap();
        assert_eq!(serialized.len(), immutable_metadata.len());
    }
}
