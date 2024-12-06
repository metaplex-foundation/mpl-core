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
    const BASE_LEN: usize = 0;

    fn len(&self) -> usize {
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
