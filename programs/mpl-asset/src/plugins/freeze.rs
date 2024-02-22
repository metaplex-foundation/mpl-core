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

#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct Freeze {
    pub frozen: bool, // 1
}

impl Freeze {
    pub fn new() -> Self {
        Self { frozen: false }
    }
}

impl Default for Freeze {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBlob for Freeze {
    fn get_initial_size() -> usize {
        1
    }

    fn get_size(&self) -> usize {
        1
    }
}

impl PluginValidation for Freeze {
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

    fn validate_update_plugin(
        &self,
        asset: &crate::state::Asset,
        ctx: &crate::instruction::accounts::UpdatePluginAccounts,
        _args: &crate::processor::UpdatePluginArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, solana_program::program_error::ProgramError> {
        if !self.frozen
            && ((ctx.authority.key == &asset.owner && authorities.contains(&Authority::Owner))
                || (ctx.authority.key == &asset.update_authority
                    && authorities.contains(&Authority::UpdateAuthority))
                || authorities.contains(&Authority::Pubkey {
                    address: *ctx.authority.key,
                }))
        {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    fn validate_burn(
        &self,
        _ctx: &BurnAccounts,
        _args: &BurnArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        if self.frozen {
            Ok(ValidationResult::Rejected)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    fn validate_transfer(
        &self,
        _ctx: &TransferAccounts,
        _args: &TransferArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        if self.frozen {
            Ok(ValidationResult::Rejected)
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
