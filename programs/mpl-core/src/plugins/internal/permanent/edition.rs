use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::{
    error::MplCoreError,
    plugins::{
        abstain, reject, AssetValidationCommon, AssetValidationContext, Plugin, PluginValidation,
        PluginValidationContext, ValidationResult,
    },
    state::DataBlob,
};

/// The edition plugin allows the creator to set an edition number on the asset
/// The default authority for this plugin is the creator.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, Default, PartialEq, Eq)]
pub struct Edition {
    /// The edition number.
    pub number: u32,
}

impl PluginValidation for Edition {
    fn validate_add_plugin(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // This plugin can only be added at creation time, so we
        // always reject it.
        if let AssetValidationContext::AddPlugin { new_plugin } = asset_ctx {
            match new_plugin {
                Plugin::Edition(_edition) => {
                    reject!()
                }
                _ => abstain!(),
            }
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }

    fn validate_remove_plugin(
        &self,
        _plugin_ctx: &PluginValidationContext,
        _common: &AssetValidationCommon,
        asset_ctx: &AssetValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // This plugin cannot be removed
        // always reject it.
        if let AssetValidationContext::RemovePlugin { plugin_to_remove } = asset_ctx {
            match plugin_to_remove {
                Plugin::Edition(_edition) => {
                    reject!()
                }
                _ => abstain!(),
            }
        } else {
            Err(MplCoreError::InvalidPlugin.into())
        }
    }
}

impl DataBlob for Edition {
    fn get_initial_size() -> usize {
        4
    }

    fn get_size(&self) -> usize {
        4
    }
}
