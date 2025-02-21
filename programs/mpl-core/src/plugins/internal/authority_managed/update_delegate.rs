use std::collections::BTreeSet;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

use crate::{
    error::MplCoreError,
    plugins::{fetch_wrapped_plugin, reject, PluginType},
    state::{AssetV1, Authority, DataBlob, UpdateAuthority},
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
    const BASE_LEN: usize = 4; // The additional delegates length

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
    fn len(&self) -> usize {
        Self::BASE_LEN + self.additional_delegates.len() * 32
    }
}

impl PluginValidation for UpdateDelegate {
    fn validate_create(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(resolved_authorities) = ctx.resolved_authorities {
            if resolved_authorities.contains(ctx.self_authority) {
                return approve!();
            }
        }

        if self.additional_delegates.contains(ctx.authority_info.key) {
            return approve!();
        }

        abstain!()
    }

    fn validate_add_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(new_plugin) = ctx.target_plugin {
            if ((ctx.resolved_authorities.is_some()
                && ctx
                    .resolved_authorities
                    .unwrap()
                    .contains(ctx.self_authority))
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
            if ((ctx.resolved_authorities.is_some()
                && ctx
                    .resolved_authorities
                    .unwrap()
                    .contains(ctx.self_authority))
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
        if ((ctx.resolved_authorities.is_some()
        && ctx
            .resolved_authorities
            .unwrap()
            .contains(ctx.self_authority))
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
        if (ctx.resolved_authorities.is_some()
        && ctx
            .resolved_authorities
            .unwrap()
            .contains(ctx.self_authority))
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
        if (ctx.resolved_authorities.is_some()
            && ctx
                .resolved_authorities
                .unwrap()
                .contains(ctx.self_authority))
            || self.additional_delegates.contains(ctx.authority_info.key)
        {
            // The rules are:
            // - Collection UpdateDelegates should be able to add and remove assets from the collection
            // - Asset UpdateDelegates should not be able to add/remove assets from a collection
            // so:

            // If there is an asset
            if ctx.asset_info.is_some()
            // And it's part of a collection
                && ctx.collection_info.is_some()
                // And it's being removed from the collection.
                && ctx.new_asset_authority.is_some()
                && ctx.new_asset_authority.unwrap()
                    != &UpdateAuthority::Collection(*ctx.collection_info.unwrap().key)
                    // And the UpdateDelegate plugin is on the Asset, not the collection.
                && fetch_wrapped_plugin::<AssetV1>(
                    ctx.asset_info.unwrap(),
                    None,
                    PluginType::UpdateDelegate,
                )
                .is_ok()
            {
                // Then we reject.
                reject!()
            } else {
                // Otherwise, we approve.
                approve!()
            }
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
        if (ctx.resolved_authorities.is_some()
            && ctx
                .resolved_authorities
                .unwrap()
                .contains(ctx.self_authority))
            || self.additional_delegates.contains(ctx.authority_info.key)
        {
            if let Plugin::UpdateDelegate(update_delegate) = plugin {
                let existing: BTreeSet<_> = self.additional_delegates.iter().collect();
                let new: BTreeSet<_> = update_delegate.additional_delegates.iter().collect();

                if existing.difference(&new).collect::<Vec<_>>() == vec![&ctx.authority_info.key]
                    && new.difference(&existing).collect::<Vec<_>>().is_empty()
                {
                    return approve!();
                }
            }
            // UpdateDelegate has the same authority as UpdateAuthority, so if the target plugin authority is UpdateAuthority, we can approve.
            else if ctx.target_plugin_authority == Some(&Authority::UpdateAuthority) {
                return approve!();
            }
        }

        // Otherwise, abstain.
        abstain!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_delegate_default_len() {
        let update_delegate = UpdateDelegate::default();
        let serialized = update_delegate.try_to_vec().unwrap();
        assert_eq!(serialized.len(), update_delegate.len());
    }

    #[test]
    fn test_update_delegate_len() {
        let update_delegates = vec![
            UpdateDelegate {
                additional_delegates: vec![Pubkey::default()],
            },
            UpdateDelegate {
                additional_delegates: vec![Pubkey::default(), Pubkey::default()],
            },
        ];
        for update_delegate in update_delegates {
            let serialized = update_delegate.try_to_vec().unwrap();
            assert_eq!(serialized.len(), update_delegate.len());
        }
    }
}
