use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, program_error::ProgramError};

use crate::{
    plugins::PluginType,
    state::{Authority, DataBlob},
};

use super::{Plugin, PluginValidation, ValidationResult};

/// This plugin manages additional permissions to burn.
/// Any authorities approved are given permission to burn the asset on behalf of the owner.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct Burn {}

impl Burn {
    /// Initialize the Burn plugin.
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

    fn validate_burn(
        &self,
        authority_info: &AccountInfo,
        authority: &Authority,
    ) -> Result<super::ValidationResult, ProgramError> {
        if authority
            == (&Authority::Pubkey {
                address: *authority_info.key,
            })
        {
            Ok(ValidationResult::Approved)
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
        Ok(ValidationResult::Pass)
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

    fn validate_add_plugin(
        &self,
        _authority: &AccountInfo,
        _authorities: &Authority,
        _new_plugin: Option<&super::Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_remove_plugin(
        &self,
        _authority: &AccountInfo,
        _authorities: &Authority,
        _plugin_to_remove: Option<&super::Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    fn validate_approve_plugin_authority(
        &self,
        _authority: &AccountInfo,
        _authorities: &Authority,
        _new_plugin: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Pass)
    }

    /// Validate the revoke plugin authority lifecycle action.
    fn validate_revoke_plugin_authority(
        &self,
        authority: &AccountInfo,
        authorities: &Authority,
        plugin_to_revoke: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        solana_program::msg!("Authority: {:?}", authority.key);
        solana_program::msg!("Authorities: {:?}", authorities);
        if authorities
            == &(Authority::Pubkey {
                address: *authority.key,
            })
            && plugin_to_revoke.is_some()
            && PluginType::from(plugin_to_revoke.unwrap()) == PluginType::Burn
        {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }
}
