use std::collections::BTreeSet;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::MplCoreError,
    plugins::PluginType,
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
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        abstain!()
    }

    fn validate_add_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(new_plugin) = ctx.target_plugin {
            if (ctx.self_authority
                == (&Authority::Address {
                    address: *ctx.authority_info.key,
                })
                || self.additional_delegates.contains(ctx.authority_info.key))
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
            if (ctx.self_authority
                == (&Authority::Address {
                    address: *ctx.authority_info.key,
                })
                || self.additional_delegates.contains(ctx.authority_info.key))
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

    // Validate the approve plugin authority lifecycle action.
    fn validate_approve_plugin_authority(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        let plugin = ctx.target_plugin.ok_or(MplCoreError::InvalidPlugin)?;

        // If the plugin authority is the authority signing.
        if (ctx.self_authority
            == &(Authority::Address {
                address: *ctx.authority_info.key,
            })
            // Or the authority is one of the additional delegates.
            || self.additional_delegates.contains(ctx.authority_info.key))
            // And it's an authority-managed plugin.
            && plugin.manager() == Authority::UpdateAuthority
            // And the plugin is not an UpdateDelegate plugin, because we cannot change the authority of the UpdateDelegate plugin.
            && PluginType::from(plugin) != PluginType::UpdateDelegate
        {
            solana_program::msg!("UpdateDelegate: Approved");
            Ok(ValidationResult::Approved)
        } else {
            Ok(ValidationResult::Pass)
        }
    }

    /// Validate the revoke plugin authority lifecycle action.
    fn validate_revoke_plugin_authority(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        let plugin = ctx.target_plugin.ok_or(MplCoreError::InvalidPlugin)?;

        // If the plugin authority is the authority signing.
        if ctx.self_authority
            == &(Authority::Address {
                address: *ctx.authority_info.key,
            })
            // Or the authority is one of the additional delegates.
            || (self.additional_delegates.contains(ctx.authority_info.key) && PluginType::from(plugin) != PluginType::UpdateDelegate)
            // And it's an authority-managed plugin.
            && plugin.manager() == Authority::UpdateAuthority
        {
            approve!()
        } else {
            abstain!()
        }
    }

    fn validate_update(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if (ctx.self_authority
            == (&Authority::Address {
                address: *ctx.authority_info.key,
            })
            || self.additional_delegates.contains(ctx.authority_info.key))
            // We do not allow the collection authority to be changed by this delegate.
            && ctx.new_authority.is_none()
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
        let plugin = ctx.target_plugin.ok_or(MplCoreError::InvalidPlugin)?;

        // If the plugin itself is being updated.
        if ctx.self_authority
            == (&Authority::Address {
                address: *ctx.authority_info.key,
            })
            || self.additional_delegates.contains(ctx.authority_info.key)
        {
            if let Plugin::UpdateDelegate(update_delegate) = plugin {
                let existing: BTreeSet<_> = self.additional_delegates.iter().collect();
                let new: BTreeSet<_> = update_delegate.additional_delegates.iter().collect();

                if existing.difference(&new).collect::<Vec<_>>() == vec![&ctx.authority_info.key]
                    && new.difference(&existing).collect::<Vec<_>>().is_empty()
                {
                    solana_program::msg!("UpdateDelegate: Approved");
                    return Ok(ValidationResult::Approved);
                }
            } else {
                return Ok(ValidationResult::Approved);
            }
        }

        abstain!()
    }
}
