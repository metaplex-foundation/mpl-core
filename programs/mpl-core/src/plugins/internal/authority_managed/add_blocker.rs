use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::{
    error::MplCoreError,
    plugins::{
        abstain, reject, AssetValidationCommon, AssetValidationContext, PluginType,
        PluginValidation, PluginValidationContext, ValidationResult,
    },
    state::{Authority, DataBlob},
};

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
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let AssetValidationContext::AddPlugin { new_plugin } = asset_ctx {
            if new_plugin.manager() == Authority::Owner
                || PluginType::from(new_plugin) == PluginType::AddBlocker
            {
                return abstain!();
            }

            reject!()
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }
}
