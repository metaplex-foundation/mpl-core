use crate::{
    plugins::{abstain, reject, PluginValidation, PluginValidationContext, ValidationResult},
    state::DataBlob,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

/// Groups plugin for collections. Stores the immediate parent group accounts this collection
/// belongs to. Relationship updates are handled by specialized group instructions, and this
/// plugin overrides `validate_burn` to reject burns while the group set is non-empty.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, Default)]
pub struct Groups {
    /// The list of parent group accounts for this collection.
    pub groups: Vec<Pubkey>, // 4 + len * 32
}

impl Groups {
    const BASE_LEN: usize = 4; // length of the groups vector
}

impl DataBlob for Groups {
    fn len(&self) -> usize {
        Self::BASE_LEN + self.groups.len() * 32
    }
}

impl PluginValidation for Groups {
    fn validate_burn(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if !self.groups.is_empty() {
            reject!()
        } else {
            abstain!()
        }
    }
}
