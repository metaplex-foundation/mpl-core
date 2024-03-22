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
pub struct PermanentFreezeDelegate {
    /// The current state of the asset and whether or not it's transferable.
    pub frozen: bool, // 1
}

impl PermanentFreezeDelegate {
    /// Initialize the PermanentFreezeDelegate plugin, unfrozen by default.
    pub fn new() -> Self {
        Self { frozen: false }
    }
}

impl Default for PermanentFreezeDelegate {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBlob for PermanentFreezeDelegate {
    fn get_initial_size() -> usize {
        1
    }

    fn get_size(&self) -> usize {
        1
    }
}

impl PluginValidation for PermanentFreezeDelegate {
    fn validate_update_plugin<T: CoreAsset>(
        &self,
        core_asset: &T,
        authority_info: &AccountInfo,
        authority: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        // The owner can't update the freeze status.
        if (authority_info.key == &core_asset.update_authority().key()
                && authority == (&Authority::UpdateAuthority))
            || authority == (&Authority::Address {
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
        _resolved_authority: Option<&Authority>,
    ) -> Result<ValidationResult, ProgramError> {
        if self.frozen {
            Ok(ValidationResult::Rejected)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    fn validate_add_plugin(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
        _new_plugin: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        // This plugin can only be added at creation time, so we
        // always reject it.
        Ok(ValidationResult::Rejected)
    }

    /// Validate the revoke plugin authority lifecycle action.
    fn validate_revoke_plugin_authority(
        &self,
        authority_info: &AccountInfo,
        authority: &Authority,
        plugin_to_revoke: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        if authority
            == &(Authority::Address {
                address: *authority_info.key,
            })
            && plugin_to_revoke.is_some()
            && PluginType::from(plugin_to_revoke.unwrap()) == PluginType::PermanentFreezeDelegate
        {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    /// Validate the remove plugin lifecycle action.
    fn validate_remove_plugin(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
        plugin_to_revoke: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        if plugin_to_revoke.is_some()
            && PluginType::from(plugin_to_revoke.unwrap()) == PluginType::PermanentFreezeDelegate
            && self.frozen
        {
            Ok(ValidationResult::Rejected)
        } else {
            Ok(ValidationResult::Pass)
        }
    }
}
