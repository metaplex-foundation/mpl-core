use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;

use crate::{
    processor::{
        CompressArgs, CreateArgs, DecompressArgs, TransferArgs, UpdateArgs, UpdatePluginArgs,
    },
    state::{Asset, Authority, DataBlob},
};

use super::{PluginValidation, ValidationResult};

/// The freeze plugin allows any authority to lock the asset so it's no longer transferable.
/// The default authority for this plugin is the owner.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct Freeze {
    /// The current state of the asset and whether or not it's transferable.
    pub frozen: bool, // 1
}

impl Freeze {
    /// Initialize the Freeze plugin, unfrozen by default.
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
        _authority: &AccountInfo,
        _args: &CreateArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_update(
        &self,
        _authority: &AccountInfo,
        _args: &UpdateArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_update_plugin(
        &self,
        asset: &Asset,
        authority: &AccountInfo,
        _args: &UpdatePluginArgs,
        authorities: &[Authority],
    ) -> Result<ValidationResult, solana_program::program_error::ProgramError> {
        // The owner can't update the freeze status.
        if (authority.key != &asset.owner
            && (authority.key == &asset.update_authority.key()
                && authorities.contains(&Authority::UpdateAuthority))
            || authorities.contains(&Authority::Pubkey {
                address: *authority.key,
            }))
            // Unless the owner is the only authority.
            || (authority.key == &asset.owner
                && authorities.contains(&Authority::Owner)
                && authorities.len() == 1)
        {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    fn validate_burn(
        &self,
        _authority: &AccountInfo,
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
        _authority: &AccountInfo,
        _new_owner: &AccountInfo,
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
        _authority: &AccountInfo,
        _args: &CompressArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_decompress(
        &self,
        _authority: &AccountInfo,
        _args: &DecompressArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_add_authority(
        &self,
        _authority: &AccountInfo,
        _args: &crate::processor::AddPluginAuthorityArgs,
        _authorities: &[Authority],
    ) -> Result<super::ValidationResult, solana_program::program_error::ProgramError> {
        Ok(ValidationResult::Pass)
    }
}
