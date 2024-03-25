use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, program_error::ProgramError};

use crate::state::{Authority, DataBlob};

use super::{Plugin, PluginType, PluginValidation, ValidationResult};

/// The permanent transfer plugin allows any authority to transfer the asset.
/// The default authority for this plugin is the update authority.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, Default, PartialEq, Eq)]
pub struct PermanentTransferDelegate {}

impl DataBlob for PermanentTransferDelegate {
    fn get_initial_size() -> usize {
        0
    }

    fn get_size(&self) -> usize {
        0
    }
}

impl PluginValidation for PermanentTransferDelegate {
    fn validate_add_plugin(
        &self,
        _authority: &AccountInfo,
        _authorities: &Authority,
        new_plugin: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        // This plugin can only be added at creation time, so we
        // always reject it.
        if new_plugin.is_some()
            && PluginType::from(new_plugin.unwrap()) == PluginType::PermanentTransferDelegate
        {
            Ok(ValidationResult::Rejected)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    fn validate_revoke_plugin_authority(
        &self,
        _authority: &AccountInfo,
        _authorities: &Authority,
        _plugin_to_revoke: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        Ok(ValidationResult::Approved)
    }

    fn validate_transfer(
        &self,
        _authority_info: &AccountInfo,
        _new_owner: &AccountInfo,
        authority: &Authority,
        resolved_authorities: Option<&[Authority]>,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(resolved_authorities) = resolved_authorities {
            if resolved_authorities.contains(authority) {
                return Ok(ValidationResult::ForceApproved);
            }
        }

        Ok(ValidationResult::Pass)
    }
}
