use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::state::{Authority, DataBlob};

use super::{PluginType, PluginValidation, PluginValidationContext, ValidationResult};

/// The allowlist plugin allows its authority to add multiple plugins to allowlist.
/// Plugins that are not whitelisted are prevented from being added/revoked.
/// The whitelist is immutable when the authority is None.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, Default)]
pub struct Allowlist {
    must_be_empty: bool,
    plugins: Vec<PluginType>,
}

impl DataBlob for Allowlist {
    fn get_initial_size() -> usize {
        1
    }

    fn get_size(&self) -> usize {
        1 + self.plugins.len() * 4
    }
}

impl PluginValidation for Allowlist {
    /// Validate the add plugin lifecycle action.
    fn validate_add_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        let (target_plugin, target_plugin_type) = match ctx.target_plugin {
            Some(plugin) => (plugin, PluginType::from(plugin)),
            None => return Ok(ValidationResult::Pass),
        };

        if let Authority::Owner = target_plugin.manager() {
            return Ok(ValidationResult::Pass);
        }

        if self.must_be_empty || !self.plugins.contains(&target_plugin_type) {
            return Ok(ValidationResult::Rejected);
        }

        Ok(ValidationResult::Pass)
    }

    /// Validate the remove plugin lifecycle action.
    fn validate_remove_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        let (target_plugin, target_plugin_type) = match ctx.target_plugin {
            Some(plugin) => (plugin, PluginType::from(plugin)),
            None => return Ok(ValidationResult::Pass),
        };

        if let Authority::Owner = target_plugin.manager() {
            return Ok(ValidationResult::Pass);
        }

        if !self.plugins.contains(&target_plugin_type) {
            return Ok(ValidationResult::Rejected);
        }

        Ok(ValidationResult::Pass)
    }

    /// Validate the update_plugin lifecycle action.
    fn validate_update_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        let (_target_plugin, target_plugin_type) = match ctx.target_plugin {
            Some(plugin) => (plugin, PluginType::from(plugin)),
            None => return Ok(ValidationResult::Pass),
        };

        if ctx.self_authority != &Authority::None && target_plugin_type == PluginType::Allowlist {
            return Ok(ValidationResult::Pass);
        }

        if !self.plugins.contains(&target_plugin_type) || self.must_be_empty {
            return Ok(ValidationResult::Rejected);
        }

        Ok(ValidationResult::Pass)
    }
}
