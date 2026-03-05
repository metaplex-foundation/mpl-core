use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::{
    plugins::{reject, Plugin, PluginType},
    state::DataBlob,
};

use crate::plugins::{abstain, PluginValidation, PluginValidationContext, ValidationResult};

/// The permanent freeze execute plugin allows any authority to lock the asset so its **Execute** lifecycle
/// event can be conditionally blocked. The default authority for this plugin is the update authority.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct PermanentFreezeExecute {
    /// Indicates whether the asset's Execute lifecycle event is currently frozen.
    pub frozen: bool, // 1
}

impl PermanentFreezeExecute {
    const BASE_LEN: usize = 1; // The frozen boolean

    /// Initialize the PermanentFreezeExecute plugin, unfrozen by default.
    pub fn new() -> Self {
        Self { frozen: false }
    }
}

impl Default for PermanentFreezeExecute {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBlob for PermanentFreezeExecute {
    fn len(&self) -> usize {
        Self::BASE_LEN
    }
}

impl PluginValidation for PermanentFreezeExecute {
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

    fn validate_add_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // This plugin can only be added at creation time, so we
        // always reject it.
        if ctx.target_plugin.is_some()
            && PluginType::from(ctx.target_plugin.unwrap()) == PluginType::PermanentFreezeExecute
        {
            reject!()
        } else {
            abstain!()
        }
    }

    /// Validate the remove plugin lifecycle action.
    fn validate_remove_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(Plugin::PermanentFreezeExecute(stored)) = ctx.target_plugin {
            // Only block removal if the stored plugin is frozen.
            if stored.frozen {
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
    fn test_permanent_freeze_execute_len() {
        let permanent_freeze_execute = PermanentFreezeExecute::default();
        let serialized = permanent_freeze_execute.try_to_vec().unwrap();
        assert_eq!(serialized.len(), permanent_freeze_execute.len());
    }
}
