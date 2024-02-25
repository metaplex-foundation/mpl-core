use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::{
    instruction::accounts::{
        BurnAccounts, CompressAccounts, CreateAccounts, DecompressAccounts, TransferAccounts,
        UpdateAccounts,
    },
    processor::{BurnArgs, CompressArgs, CreateArgs, DecompressArgs, TransferArgs, UpdateArgs},
    state::Authority,
};

use super::{PluginValidation, ValidationResult};

#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub struct Collection {
    collection_address: Pubkey,
    managed: bool,
}

impl PluginValidation for Collection {
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
        _ctx: &BurnAccounts,
        _args: &BurnArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_transfer(
        &self,
        _ctx: &TransferAccounts,
        _args: &TransferArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        Ok(ValidationResult::Pass)
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
