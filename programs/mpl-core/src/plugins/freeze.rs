use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, program_error::ProgramError};

use crate::state::{Authority, CoreAsset, DataBlob};

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
        _authority_info: &AccountInfo,
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

    fn validate_update_plugin<T: CoreAsset>(
        &self,
        core_asset: &T,
        authority_info: &AccountInfo,
        authority: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        // The owner can't update the freeze status.
        if (authority_info.key != core_asset.owner()
            && (authority_info.key == &core_asset.update_authority().key()
                && authority == (&Authority::UpdateAuthority))
            || authority == (&Authority::Pubkey {
                address: *authority_info.key,
            }))
            // Unless the owner is the only authority.
            || (authority_info.key == core_asset.owner()
                && authority == (&Authority::Owner))
        {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    fn validate_burn(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
    ) -> Result<super::ValidationResult, ProgramError> {
        if self.frozen {
            Ok(ValidationResult::Rejected)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    fn validate_transfer(
        &self,
        _authority_info: &AccountInfo,
        _new_owner: &AccountInfo,
        _authority: &Authority,
    ) -> Result<super::ValidationResult, ProgramError> {
        if self.frozen {
            Ok(ValidationResult::Rejected)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    fn validate_compress(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
    ) -> Result<super::ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_decompress(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
    ) -> Result<super::ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_add_authority(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
    ) -> Result<super::ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }
}
