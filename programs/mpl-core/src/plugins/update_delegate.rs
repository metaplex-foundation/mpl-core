use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::MplCoreError,
    plugins::PluginType,
    state::{Authority, DataBlob},
};

use super::{Plugin, PluginValidation, ValidationResult};

/// This plugin manages additional permissions to burn.
/// Any authorities approved are given permission to burn the asset on behalf of the owner.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct UpdateDelegate {
    /// Additional update delegates.  Not currently available to be used.
    pub additional_delegates: Vec<Pubkey>, // 4
}

impl UpdateDelegate {
    /// Initialize the UpdateDelegate plugin.
    pub fn new() -> Self {
        Self {
            additional_delegates: vec![],
        }
    }
}

impl Default for UpdateDelegate {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBlob for UpdateDelegate {
    fn get_initial_size() -> usize {
        0
    }

    fn get_size(&self) -> usize {
        0
    }
}

impl PluginValidation for UpdateDelegate {
    fn validate_create(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        if !self.additional_delegates.is_empty() {
            return Err(MplCoreError::NotAvailable.into());
        }
        Ok(ValidationResult::Pass)
    }

    fn validate_add_plugin(
        &self,
        authority_info: &AccountInfo,
        authority: &Authority,
        new_plugin: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(new_plugin) = new_plugin {
            if let Plugin::UpdateDelegate(update_delegate) = new_plugin {
                if !update_delegate.additional_delegates.is_empty() {
                    return Err(MplCoreError::NotAvailable.into());
                }
            }

            if authority
                == (&Authority::Address {
                    address: *authority_info.key,
                })
                && new_plugin.manager() == Authority::UpdateAuthority
            {
                Ok(ValidationResult::Approved)
            } else {
                Ok(ValidationResult::Pass)
            }
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }

    fn validate_remove_plugin(
        &self,
        authority_info: &AccountInfo,
        authority: &Authority,
        plugin_to_remove: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(plugin_to_remove) = plugin_to_remove {
            if authority
                == (&Authority::Address {
                    address: *authority_info.key,
                })
                && plugin_to_remove.manager() == Authority::UpdateAuthority
            {
                Ok(ValidationResult::Approved)
            } else {
                Ok(ValidationResult::Pass)
            }
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }

    /// Validate the revoke plugin authority lifecycle action.
    fn validate_revoke_plugin_authority(
        &self,
        authority_info: &AccountInfo,
        authority: &Authority,
        plugin_to_revoke: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        solana_program::msg!("authority_info: {:?}", authority_info.key);
        solana_program::msg!("authority: {:?}", authority);
        if authority
            == &(Authority::Address {
                address: *authority_info.key,
            })
            && plugin_to_revoke.is_some()
            && PluginType::from(plugin_to_revoke.unwrap()) == PluginType::UpdateDelegate
        {
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    fn validate_update(
        &self,
        authority_info: &AccountInfo,
        authority: &Authority,
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

    fn validate_update_plugin(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
        _resolved_authorities: &[Authority],
        plugin_to_update: &Plugin,
    ) -> Result<ValidationResult, ProgramError> {
        if let Plugin::UpdateDelegate(update_delegate) = plugin_to_update {
            if !update_delegate.additional_delegates.is_empty() {
                return Err(MplCoreError::NotAvailable.into());
            }
        }

        Ok(ValidationResult::Pass)
    }
}
