use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, program_error::ProgramError};

use crate::state::Authority;

use super::{Plugin, PluginType, PluginValidation, ValidationResult};

/// The edition plugin allows the creator to set an edition number on the asset
/// The default authority for this plugin is the creator.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct Edition {
    /// The edition number.
    pub number: u64,
}

impl Edition {
    /// Initialize the Edition plugin, 0 by default.
    pub fn new() -> Self {
        Self { number: 0 }
    }
}

impl PluginValidation for Edition {
    fn validate_add_plugin(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
        new_plugin: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        // This plugin can only be added at creation time, so we
        // always reject it.
        if new_plugin.is_some() && PluginType::from(new_plugin.unwrap()) == PluginType::Edition {
            Ok(ValidationResult::Rejected)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    fn validate_remove_plugin(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
        plugin: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        // This plugin cannot be removed
        // always reject it.
        if plugin.is_some() && PluginType::from(plugin.unwrap()) == PluginType::Edition {
            Ok(ValidationResult::Rejected)
        } else {
            Ok(ValidationResult::Pass)
        }
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
            && PluginType::from(plugin_to_revoke.unwrap()) == PluginType::Edition
        {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }
}
