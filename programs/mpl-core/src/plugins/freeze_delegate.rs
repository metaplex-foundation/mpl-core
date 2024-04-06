use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, program_error::ProgramError};

use crate::state::{Authority, DataBlob};

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
    fn validate_burn(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
        _resolved_authorities: Option<&[Authority]>,
    ) -> Result<ValidationResult, ProgramError> {
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
        _resolved_authorities: Option<&[Authority]>,
    ) -> Result<ValidationResult, ProgramError> {
        if self.frozen {
            Ok(ValidationResult::Rejected)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    fn validate_approve_plugin_authority(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
        plugin_to_approve: Option<&Plugin>,
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
                == &(Authority::Address {
                    address: *authority_info.key,
                })
            {
                return Ok(ValidationResult::Approved);
            }
        }

        Ok(ValidationResult::Pass)
    }

    /// Validate the remove plugin lifecycle action.
    fn validate_remove_plugin(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
        plugin_to_revoke: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        if plugin_to_revoke.is_some() && self.frozen {
            Ok(ValidationResult::Rejected)
        } else {
            Ok(ValidationResult::Pass)
        }
    }
}
