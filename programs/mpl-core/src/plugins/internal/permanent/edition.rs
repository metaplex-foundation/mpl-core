use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::{
    plugins::{
        abstain, reject, Plugin, PluginValidation, PluginValidationContext, ValidationResult,
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

impl Edition {
    const BASE_LEN: usize = 4; // The edition number
}

impl PluginValidation for Edition {
    fn validate_add_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // This plugin can only be added at creation time, so we
        // always reject it.
        match ctx.target_plugin {
            Some(Plugin::Edition(_edition)) => {
                reject!()
            }
            _ => abstain!(),
        }
    }

    fn validate_remove_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // This plugin cannot be removed
        // always reject it.
        match ctx.target_plugin {
            Some(Plugin::Edition(_edition)) => {
                reject!()
            }
            _ => abstain!(),
        }
    }
}

impl DataBlob for Edition {
    fn len(&self) -> usize {
        Self::BASE_LEN
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edition_len() {
        let edition = Edition { number: 1 };
        let serialized = edition.try_to_vec().unwrap();
        assert_eq!(serialized.len(), edition.len());
    }
}
