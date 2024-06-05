use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::state::DataBlob;

use super::{reject, PluginValidation, PluginValidationContext, ValidationResult};

/// The immutable metadata plugin allows its authority to prevent plugin's meta from changing.
/// The default authority for this plugin is None.

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, Default)]
pub struct ImmutableMetadata {}

impl DataBlob for ImmutableMetadata {
    fn get_initial_size() -> usize {
        0
    }

    fn get_size(&self) -> usize {
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
