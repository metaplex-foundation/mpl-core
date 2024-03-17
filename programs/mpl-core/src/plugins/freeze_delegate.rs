use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, program_error::ProgramError};

use crate::state::{Authority, CoreAsset, DataBlob};

use super::{Plugin, PluginValidation, ValidationResult};

/// The freeze delegate plugin allows any authority to lock the asset so it's no longer transferable.
/// The default authority for this plugin is the owner.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct FreezeDelegate {
    /// The current state of the asset and whether or not it's transferable.
    pub frozen: bool, // 1
}

impl FreezeDelegate {
    /// Initialize the Freeze plugin, unfrozen by default.
    pub fn new() -> Self {
        Self { frozen: false }
    }
}

impl Default for FreezeDelegate {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBlob for FreezeDelegate {
    fn get_initial_size() -> usize {
        1
    }

    fn get_size(&self) -> usize {
        1
    }
}

impl PluginValidation for FreezeDelegate {
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

    fn validate_approve_plugin_authority(
        &self,
        _authority: &AccountInfo,
        _authorities: &Authority,
        plugin_to_approve: Option<&super::Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(Plugin::FreezeDelegate(freeze)) = plugin_to_approve {
            if freeze.frozen {
                return Ok(ValidationResult::Rejected);
            }
        }
        Ok(ValidationResult::Pass)
    }

    /// Validate the revoke plugin authority lifecycle action.
    fn validate_revoke_plugin_authority(
        &self,
        authority_info: &AccountInfo,
        authority: &Authority,
        plugin_to_revoke: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(Plugin::FreezeDelegate(freeze)) = plugin_to_revoke {
            if freeze.frozen {
                return Ok(ValidationResult::Rejected);
            } else if authority
                == &(Authority::Pubkey {
                    address: *authority_info.key,
                })
            {
                return Ok(ValidationResult::Approved);
            }
        }

        Ok(ValidationResult::Pass)
    }
}
