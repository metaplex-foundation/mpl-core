use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::{
    instruction::accounts::{
        BurnAccounts, CompressAccounts, CreateAccounts, DecompressAccounts, TransferAccounts,
        UpdateAccounts,
    },
    processor::{BurnArgs, CompressArgs, CreateArgs, DecompressArgs, TransferArgs, UpdateArgs},
    state::{Authority, DataBlob},
};

use super::{PluginValidation, ValidationResult};

#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct Burn {}

impl Burn {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Burn {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBlob for Burn {
    fn get_initial_size() -> usize {
        0
    }

    fn get_size(&self) -> usize {
        0
    }
}

impl PluginValidation for Burn {
    fn validate_create(
        &self,
        _ctx: &CreateAccounts,
        _args: &CreateArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_update(
        &self,
        _ctx: &UpdateAccounts,
        _args: &UpdateArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_burn(
        &self,
        ctx: &BurnAccounts,
        _args: &BurnArgs,
        authorities: &[Authority],
    ) -> Result<super::ValidationResult, ProgramError> {
        if authorities.contains(&Authority::Pubkey {
            address: *ctx.authority.key,
        }) {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    fn validate_transfer(
        &self,
        _ctx: &TransferAccounts,
        _args: &TransferArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_compress(
        &self,
        _ctx: &CompressAccounts,
        _args: &CompressArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_decompress(
        &self,
        _ctx: &DecompressAccounts,
        _args: &DecompressArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }
}
