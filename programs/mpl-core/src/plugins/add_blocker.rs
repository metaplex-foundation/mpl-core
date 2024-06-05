use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::{
    plugins::abstain,
    state::{Authority, DataBlob},
};

use super::{reject, PluginType, PluginValidation, PluginValidationContext, ValidationResult};

/// The AddBlocker plugin prevents any plugin except for owner-managed plugins from being added.
/// The default authority for this plugin is None.

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, Default)]
pub struct AddBlocker {}

impl DataBlob for AddBlocker {
    fn get_initial_size() -> usize {
        0
    }

    fn get_size(&self) -> usize {
        0
    }
}

impl PluginValidation for AddBlocker {
    fn validate_add_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(plugin) = ctx.target_plugin {
            if plugin.manager() == Authority::Owner
                || PluginType::from(plugin) == PluginType::AddBlocker
            {
                return abstain!();
            }
        }

        reject!()
    }
}
