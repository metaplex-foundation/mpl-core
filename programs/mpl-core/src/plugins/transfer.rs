use borsh::{BorshDeserialize, BorshSerialize};

use crate::{
    instruction::accounts::{
        BurnAccounts, CompressAccounts, CreateAccounts, DecompressAccounts, TransferAccounts,
        UpdateAccounts,
    },
    processor::{BurnArgs, CompressArgs, CreateArgs, DecompressArgs, TransferArgs, UpdateArgs},
    state::{Authority, DataBlob},
};

use super::{PluginValidation, ValidationResult};

/// This plugin manages the ability to transfer an asset and any authorities
/// added are permitted to transfer the asset on behalf of the owner.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct Transfer {}

impl Transfer {
    /// Initialize the Transfer plugin.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Transfer {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBlob for Transfer {
    fn get_initial_size() -> usize {
        0
    }

    fn get_size(&self) -> usize {
        0
    }
}

impl PluginValidation for Transfer {
    fn validate_create(
        &self,
        _ctx: &CreateAccounts,
        _args: &CreateArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_update(
        &self,
        _ctx: &UpdateAccounts,
        _args: &UpdateArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_burn(
        &self,
        ctx: &BurnAccounts,
        _args: &BurnArgs,
        authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
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
        ctx: &TransferAccounts,
        _args: &TransferArgs,
        authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        if authorities.contains(&Authority::Pubkey {
            address: *ctx.authority.key,
        }) {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    fn validate_compress(
        &self,
        _ctx: &CompressAccounts,
        _args: &CompressArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_decompress(
        &self,
        _ctx: &DecompressAccounts,
        _args: &DecompressArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        Ok(ValidationResult::Pass)
    }
}
