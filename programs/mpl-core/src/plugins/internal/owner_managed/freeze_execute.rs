use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::{
    plugins::{
        abstain, approve, reject, Plugin, PluginValidation, PluginValidationContext,
        ValidationResult,
    },
    state::DataBlob,
};

/// The FreezeExecute plugin allows any authority to lock the asset so its **Execute** lifecycle
/// event can be conditionally blocked. The default authority for this plugin is the asset owner.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct FreezeExecute {
    /// Indicates whether the asset's Execute lifecycle event is currently frozen.
    pub frozen: bool, // 1
}

impl FreezeExecute {
    const BASE_LEN: usize = 1; // The frozen boolean

    /// Initialize the plugin; assets are unfrozen by default.
    pub fn new() -> Self {
        Self { frozen: false }
    }
}

impl Default for FreezeExecute {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBlob for FreezeExecute {
    fn len(&self) -> usize {
        Self::BASE_LEN
    }
}

impl PluginValidation for FreezeExecute {
    /// Validate the execute lifecycle action.
    /// If the asset is frozen, reject the Execute operation; otherwise abstain.
    fn validate_execute(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if self.frozen {
            reject!()
        } else {
            abstain!()
        }
    }

    /// Validate the approve plugin authority lifecycle action.
    /// If the target FreezeExecute is frozen, reject authority changes.
    fn validate_approve_plugin_authority(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(Plugin::FreezeExecute(freeze)) = ctx.target_plugin {
            if freeze.frozen {
                return reject!();
            }
        }
        abstain!()
    }

    /// Validate the revoke plugin authority lifecycle action.
    /// If the target FreezeExecute is frozen, reject revocation.
    /// If unfrozen and the caller is a resolved authority, approve.
    fn validate_revoke_plugin_authority(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(Plugin::FreezeExecute(freeze)) = ctx.target_plugin {
            if freeze.frozen {
                return reject!();
            } else if ctx.resolved_authorities.is_some()
                && ctx
                    .resolved_authorities
                    .unwrap()
                    .contains(ctx.self_authority)
            {
                return approve!();
            }
        }
        abstain!()
    }

    /// Validate the remove plugin lifecycle action.
    /// If the target FreezeExecute is frozen, reject its removal.
    fn validate_remove_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(Plugin::FreezeExecute(freeze)) = ctx.target_plugin {
            if freeze.frozen {
                return reject!();
            }
        }
        abstain!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freeze_execute_len() {
        let freeze_execute = FreezeExecute::default();
        let serialized = borsh::to_vec(&freeze_execute).unwrap();
        assert_eq!(serialized.len(), freeze_execute.len());
    }
}
