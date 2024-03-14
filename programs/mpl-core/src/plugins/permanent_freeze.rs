use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, program_error::ProgramError};

use crate::{
    plugins::PluginType,
    state::{Authority, CoreAsset, DataBlob},
};

use super::{Plugin, PluginValidation, ValidationResult};

/// The permanent freeze plugin allows any authority to lock the asset so it's no longer transferable.
/// The default authority for this plugin is the update authority.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct PermanentFreeze {
    /// The current state of the asset and whether or not it's transferable.
    pub frozen: bool, // 1
}

impl PermanentFreeze {
    /// Initialize the PermanentFreeze plugin, unfrozen by default.
    pub fn new() -> Self {
        Self { frozen: false }
    }
}

impl Default for PermanentFreeze {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBlob for PermanentFreeze {
    fn get_initial_size() -> usize {
        1
    }

    fn get_size(&self) -> usize {
        1
    }
}

impl PluginValidation for PermanentFreeze {
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
        if (authority_info.key == &core_asset.update_authority().key()
                && authority == (&Authority::UpdateAuthority))
            || authority == (&Authority::Pubkey {
                address: *authority_info.key,
            })
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
        _resolved_authority: Option<&Authority>,
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
        _resolved_authority: Option<&Authority>,
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

    fn validate_add_plugin(
        &self,
        _authority: &AccountInfo,
        _authorities: &Authority,
        _new_plugin: Option<&super::Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        // This plugin can only be added at creation time, so we
        // always reject it.
        Ok(ValidationResult::Rejected)
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
        _new_plugin: Option<&super::Plugin>,
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
        if authorities
            == &(Authority::Pubkey {
                address: *authority.key,
            })
            && plugin_to_revoke.is_some()
            && PluginType::from(plugin_to_revoke.unwrap()) == PluginType::Freeze
        {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }
}
