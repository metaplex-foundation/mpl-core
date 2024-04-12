use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, program_error::ProgramError};

use crate::{
    plugins::PluginType,
    state::{Authority, DataBlob},
};

use super::{Plugin, PluginValidation, ValidationResult};

/// This plugin manages the ability to transfer an asset and any authorities
/// approved are permitted to transfer the asset on behalf of the owner.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct TransferDelegate {}

impl TransferDelegate {
    /// Initialize the Transfer plugin.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for TransferDelegate {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBlob for TransferDelegate {
    fn get_initial_size() -> usize {
        0
    }

    fn get_size(&self) -> usize {
        0
    }
}

impl PluginValidation for TransferDelegate {
    fn validate_burn(
        &self,
        authority_info: &AccountInfo,
        authority: &Authority,
        _resolved_authorities: Option<&[Authority]>,
    ) -> Result<ValidationResult, ProgramError> {
        if authority
            == (&Authority::Address {
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
        authority_info: &AccountInfo,
        _new_owner: &AccountInfo,
        authority: &Authority,
        _resolved_authorities: Option<&[Authority]>,
    ) -> Result<ValidationResult, ProgramError> {
        if authority
            == (&Authority::Address {
                address: *authority_info.key,
            })
        {
            Ok(ValidationResult::Approved)
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
            && PluginType::from(plugin_to_revoke.unwrap()) == PluginType::TransferDelegate
        {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }
}
