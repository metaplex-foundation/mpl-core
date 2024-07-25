use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::MplCoreError,
    state::{Authority, DataBlob},
};

use super::{
    abstain, approve, Plugin, PluginValidation, PluginValidationContext, ValidationResult,
};

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
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if !self.additional_delegates.is_empty() {
            return Err(MplCoreError::NotAvailable.into());
        }

        if let Some(resolved_authorities) = ctx.resolved_authorities {
            if resolved_authorities.contains(ctx.self_authority) {
                return approve!();
            }
        }

        abstain!()
    }

    fn validate_add_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(new_plugin) = ctx.target_plugin {
            if let Plugin::UpdateDelegate(update_delegate) = new_plugin {
                if !update_delegate.additional_delegates.is_empty() {
                    return Err(MplCoreError::NotAvailable.into());
                }
            }

            if ctx.self_authority
                == (&Authority::Address {
                    address: *ctx.authority_info.key,
                })
                && new_plugin.manager() == Authority::UpdateAuthority
            {
                approve!()
            } else {
                abstain!()
            }
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }

    fn validate_remove_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(plugin_to_remove) = ctx.target_plugin {
            if ctx.self_authority
                == (&Authority::Address {
                    address: *ctx.authority_info.key,
                })
                && plugin_to_remove.manager() == Authority::UpdateAuthority
            {
                approve!()
            } else {
                abstain!()
            }
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }

    fn validate_update(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if ctx.self_authority
            == (&Authority::Address {
                address: *ctx.authority_info.key,
            })
        {
            approve!()
        } else {
            abstain!()
        }
    }

    fn validate_update_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        let plugin_to_update = ctx.target_plugin.ok_or(MplCoreError::InvalidPlugin)?;
        if let Plugin::UpdateDelegate(update_delegate) = plugin_to_update {
            if !update_delegate.additional_delegates.is_empty() {
                return Err(MplCoreError::NotAvailable.into());
            }
        }

        abstain!()
    }
}
