use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::{
    plugins::{
        abstain, reject, PluginType, PluginValidation, PluginValidationContext, ValidationResult,
    },
    state::{Authority, DataBlob},
};

/// The AddBlocker plugin prevents any plugin except for owner-managed plugins from being added.
/// The default authority for this plugin is None.

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, Default)]
pub struct AddBlocker {}

impl DataBlob for AddBlocker {
    fn len(&self) -> usize {
        // Stateless data blob
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_blocker_len() {
        let add_blocker = AddBlocker {};
        let serialized = add_blocker.try_to_vec().unwrap();
        assert_eq!(serialized.len(), add_blocker.len());
    }
}
