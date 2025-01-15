use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::{
    plugins::{reject, PluginType},
    state::DataBlob,
};

use crate::plugins::{abstain, PluginValidation, PluginValidationContext, ValidationResult};

/// The permanent freeze plugin allows any authority to lock the asset so it's no longer transferable.
/// The default authority for this plugin is the update authority.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct PermanentFreezeDelegate {
    /// The current state of the asset and whether or not it's transferable.
    pub frozen: bool, // 1
}

impl PermanentFreezeDelegate {
    const BASE_LEN: usize = 1; // The frozen boolean

    /// Initialize the PermanentFreezeDelegate plugin, unfrozen by default.
    pub fn new() -> Self {
        Self { frozen: false }
    }
}

impl Default for PermanentFreezeDelegate {
    fn default() -> Self {
        Self::new()
    }
}

impl DataBlob for PermanentFreezeDelegate {
    fn len(&self) -> usize {
        Self::BASE_LEN
    }
}

impl PluginValidation for PermanentFreezeDelegate {
    fn validate_burn(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if self.frozen {
            reject!()
        } else {
            abstain!()
        }
    }

    fn validate_transfer(
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
            && PluginType::from(ctx.target_plugin.unwrap()) == PluginType::PermanentFreezeDelegate
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
        if ctx.target_plugin.is_some() && self.frozen {
            reject!()
        } else {
            abstain!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permanent_freeze_delegate_len() {
        let permanent_freeze_delegate = PermanentFreezeDelegate::default();
        let serialized = permanent_freeze_delegate.try_to_vec().unwrap();
        assert_eq!(serialized.len(), permanent_freeze_delegate.len());
    }
}
