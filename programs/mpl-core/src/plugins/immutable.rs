use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, program_error::ProgramError};

use crate::{
    error::MplCoreError,
    state::{Authority, DataBlob},
};

use super::{Plugin, PluginValidation, ValidationResult};

/// This plugin manages additional permissions to burn.
/// Any authorities approved are given permission to burn the asset on behalf of the owner.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct Immutable {}

impl Immutable {
    /// Initialize the Burn plugin.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Immutable {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBlob for Immutable {
    fn get_initial_size() -> usize {
        0
    }

    fn get_size(&self) -> usize {
        0
    }
}

impl PluginValidation for Immutable {
    fn validate_add_plugin(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
        new_plugin: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(plugin) = new_plugin {
            if plugin.manager() == Authority::Owner {
                Ok(ValidationResult::Approved)
            } else {
                Err(MplCoreError::ImmutableAsset.into())
            }
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }

    fn validate_remove_plugin(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
        plugin_to_remove: Option<&Plugin>,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(plugin) = plugin_to_remove {
            if plugin.manager() == Authority::Owner {
                Ok(ValidationResult::Approved)
            } else {
                Err(MplCoreError::ImmutableAsset.into())
            }
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }

    fn validate_update(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
    ) -> Result<ValidationResult, ProgramError> {
        Err(MplCoreError::ImmutableAsset.into())
    }

    fn validate_update_plugin(
        &self,
        _authority_info: &AccountInfo,
        _authority: &Authority,
        _resolved_authority: &[Authority],
        plugin_to_update: &Plugin,
    ) -> Result<ValidationResult, ProgramError> {
        if plugin_to_update.manager() == Authority::Owner {
            Ok(ValidationResult::Approved)
        } else {
            Err(MplCoreError::ImmutableAsset.into())
        }
    }
}
