use std::collections::BTreeSet;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::MplCoreError,
    plugins::{AssetValidationCommon, AssetValidationContext, PluginType},
    state::{Authority, DataBlob},
};

use crate::plugins::{
    abstain, approve, Plugin, PluginValidation, PluginValidationContext, ValidationResult,
};

/// This plugin manages additional permissions to burn.
/// Any authorities approved are given permission to burn the asset on behalf of the owner.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct UpdateDelegate {
    /// Additional update delegates.  Not currently available to be used.
    pub additional_delegates: Vec<Pubkey>, // 4 + len * 32
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
        4
    }

    fn get_size(&self) -> usize {
        4 + self.additional_delegates.len() * 32
    }
}

impl PluginValidation for UpdateDelegate {
    fn validate_create(
        &self,
        plugin_ctx: &PluginValidationContext,
        common: &AssetValidationCommon,
        _asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(resolved_authorities) = plugin_ctx.resolved_authorities {
            if resolved_authorities.contains(plugin_ctx.self_authority) {
                return approve!();
            }
        }

        if self
            .additional_delegates
            .contains(common.authority_info.key)
        {
            return approve!();
        }

        abstain!()
    }

    fn validate_add_plugin(
        &self,
        plugin_ctx: &PluginValidationContext,
        common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let AssetValidationContext::AddPlugin { new_plugin } = asset_ctx {
            if ((plugin_ctx.resolved_authorities.is_some()
                && plugin_ctx
                    .resolved_authorities
                    .unwrap()
                    .contains(plugin_ctx.self_authority))
                || self
                    .additional_delegates
                    .contains(common.authority_info.key))
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
        plugin_ctx: &PluginValidationContext,
        common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let AssetValidationContext::RemovePlugin { plugin_to_remove } = asset_ctx {
            if ((plugin_ctx.resolved_authorities.is_some()
                && plugin_ctx
                    .resolved_authorities
                    .unwrap()
                    .contains(plugin_ctx.self_authority))
                || self
                    .additional_delegates
                    .contains(common.authority_info.key))
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
        plugin_ctx: &PluginValidationContext,
        common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let AssetValidationContext::ApprovePluginAuthority { plugin } = asset_ctx {
            // If the plugin authority is the authority signing.
            if ((plugin_ctx.resolved_authorities.is_some()
                && plugin_ctx
                    .resolved_authorities
                    .unwrap()
                    .contains(plugin_ctx.self_authority))
                // Or the authority is one of the additional delegates.
                || self.additional_delegates.contains(common.authority_info.key))
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
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }

    /// Validate the revoke plugin authority lifecycle action.
    fn validate_revoke_plugin_authority(
        &self,
        plugin_ctx: &PluginValidationContext,
        common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let AssetValidationContext::RevokePluginAuthority { plugin } = asset_ctx {
            // If the plugin authority is the authority signing.
            if (plugin_ctx.resolved_authorities.is_some()
                && plugin_ctx
                    .resolved_authorities
                    .unwrap()
                    .contains(plugin_ctx.self_authority))
                // Or the authority is one of the additional delegates.
                || (self.additional_delegates.contains(common.authority_info.key) && PluginType::from(plugin) != PluginType::UpdateDelegate)
                // And it's an authority-managed plugin.
                && plugin.manager() == Authority::UpdateAuthority
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
        plugin_ctx: &PluginValidationContext,
        common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let AssetValidationContext::Update {
            new_update_authority,
            ..
        } = asset_ctx
        {
            if ((plugin_ctx.resolved_authorities.is_some()
            && plugin_ctx
                .resolved_authorities
            .unwrap()
                .contains(plugin_ctx.self_authority))
                // Or the authority is one of the additional delegates.
                || self.additional_delegates.contains(common.authority_info.key))
                // We do not allow the root authority (either Collection or Address) to be changed by this delegate.
                && new_update_authority.is_none()
            {
                approve!()
            } else {
                abstain!()
            }
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }

    fn validate_update_plugin(
        &self,
        plugin_ctx: &PluginValidationContext,
        common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let AssetValidationContext::UpdatePlugin { new_plugin } = asset_ctx {
            // If the plugin itself is being updated.
            if (plugin_ctx.resolved_authorities.is_some()
                && plugin_ctx
                    .resolved_authorities
                    .unwrap()
                    .contains(plugin_ctx.self_authority))
                // Or the authority is one of the additional delegates.
                || self.additional_delegates.contains(common.authority_info.key)
            {
                if let Plugin::UpdateDelegate(update_delegate) = new_plugin {
                    let existing: BTreeSet<_> = self.additional_delegates.iter().collect();
                    let new: BTreeSet<_> = update_delegate.additional_delegates.iter().collect();

                    if existing.difference(&new).collect::<Vec<_>>()
                        == vec![&common.authority_info.key]
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
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }
}
