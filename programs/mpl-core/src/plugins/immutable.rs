use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::state::{Authority, DataBlob};

use super::{PluginType, PluginValidation, PluginValidationContext, ValidationResult};

/// The immutable plugin allows its authority to make parts of the asset immutable
/// On the other hand, it can whitelist plugins that can keep mutability
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq)]
pub struct Immutable {
    /// The whitelist of plugins that can be mutable.
    pub whitelist: Vec<PluginType>,
    /// Shows whether metadata is mutable
    pub metadata: bool,
}

impl Immutable {
    /// Initialize the Mutable plugin, locked by default by default.
    pub fn new(whitelist: Vec<PluginType>, metadata: bool) -> Self {
        Self {
            whitelist,
            metadata,
        }
    }
}

impl DataBlob for Immutable {
    fn get_initial_size() -> usize {
        1
    }

    fn get_size(&self) -> usize {
        24 // TODO: heap-located
    }
}

impl PluginValidation for Immutable {
    /// Validate the add plugin lifecycle action.
    fn validate_add_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        let (plugin, plugin_type) = match ctx.target_plugin {
            Some(plugin) => (plugin, PluginType::from(plugin)),
            None => return Ok(ValidationResult::Pass),
        };

        if let Authority::Owner = ctx.self_authority {
            if Authority::Owner != plugin.manager() {
                return Ok(ValidationResult::Rejected);
            }
        }

        if !self.whitelist.contains(&plugin_type) {
            return Ok(ValidationResult::Rejected);
        }

        Ok(ValidationResult::Pass)
    }

    /// Validate the remove plugin lifecycle action.
    fn validate_remove_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        let (plugin, plugin_type) = match ctx.target_plugin {
            Some(plugin) => (plugin, PluginType::from(plugin)),
            None => return Ok(ValidationResult::Pass),
        };

        if let Authority::Owner = ctx.self_authority {
            if Authority::Owner != plugin.manager() {
                return Ok(ValidationResult::Rejected);
            }
        }

        if !self.whitelist.contains(&plugin_type) {
            return Ok(ValidationResult::Rejected);
        }

        Ok(ValidationResult::Pass)
    }

    /// Validate the update lifecycle action.
    fn validate_update(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if self.metadata {
            return Ok(ValidationResult::Rejected);
        }

        Ok(ValidationResult::Pass)
    }
}
