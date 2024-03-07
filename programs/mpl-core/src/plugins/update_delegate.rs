use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, program_error::ProgramError};

use crate::{
    processor::{CompressArgs, CreateArgs, DecompressArgs},
    state::{Authority, DataBlob},
};

use super::{PluginValidation, ValidationResult};

/// This plugin manages additional permissions to burn.
/// Any authorities approved are given permission to burn the asset on behalf of the owner.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct UpdateDelegate {}

impl UpdateDelegate {
    /// Initialize the UpdateDelegate plugin.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for UpdateDelegate {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBlob for UpdateDelegate {
    fn get_initial_size() -> usize {
        0
    }

    fn get_size(&self) -> usize {
        0
    }
}

impl PluginValidation for UpdateDelegate {
    fn validate_create(
        &self,
        _authority_info: &AccountInfo,
        _args: &CreateArgs,
        _authority: &Authority,
    ) -> Result<super::ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_update(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
    ) -> Result<super::ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_burn(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
    ) -> Result<super::ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_transfer(
        &self,
        _authority_info: &AccountInfo,
        _new_owner: &AccountInfo,
        _authority: &Authority,
    ) -> Result<super::ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_compress(
        &self,
        _authority_info: &AccountInfo,
        _args: &CompressArgs,
        _authority: &Authority,
    ) -> Result<super::ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_decompress(
        &self,
        _authority_info: &AccountInfo,
        _args: &DecompressArgs,
        _authority: &Authority,
    ) -> Result<super::ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }
}
