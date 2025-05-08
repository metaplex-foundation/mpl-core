use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::{
    plugins::{
        abstain, reject, Plugin, PluginType, PluginValidation, PluginValidationContext,
        ValidationResult,
    },
    state::DataBlob,
};

/// The Bubblegum V2 plugin allows a Core collection to contain Compressed NFTs (cNFTs)
/// from the Bubblegum program.  The authority for this plugin can only be the Bubblegum
/// program.
#[repr(C)]
#[derive(Clone, Copy, BorshSerialize, BorshDeserialize, Debug, Default, PartialEq, Eq)]
pub struct BubblegumV2 {}

impl BubblegumV2 {
    /// List of other plugins allowed on collections with the Bubblegum V2 plugin.
    /// The BubblegumV2 plugin limits what can be on the collection to plugins that are
    /// supported and validated at runtime by the Bubblegum program.  Other plugins may
    /// be added in the future but for now this subset was chosen.
    pub(crate) const ALLOW_LIST: [PluginType; 6] = [
        PluginType::Attributes,
        PluginType::PermanentFreezeDelegate,
        PluginType::PermanentTransferDelegate,
        PluginType::PermanentBurnDelegate,
        PluginType::Royalties,
        PluginType::UpdateDelegate,
    ];
}

impl DataBlob for BubblegumV2 {
    fn len(&self) -> usize {
        // Stateless data blob
        0
    }
}

impl PluginValidation for BubblegumV2 {
    fn validate_add_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        if let Some(target_plugin) = ctx.target_plugin {
            let plugin_type = PluginType::from(target_plugin);
            if Self::ALLOW_LIST.contains(&plugin_type) {
                abstain!()
            } else if plugin_type == PluginType::BubblegumV2 {
                // This plugin can only be added at creation time, so we
                // always reject it.
                reject!()
            } else {
                // All other plugins are not allowed on Bubblegum
                // collections.
                reject!()
            }
        } else {
            abstain!()
        }
    }

    fn validate_remove_plugin(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // This plugin cannot be removed so always reject it.
        match ctx.target_plugin {
            Some(Plugin::BubblegumV2(_)) => {
                reject!()
            }
            _ => abstain!(),
        }
    }

    fn validate_add_external_plugin_adapter(
        &self,
        _ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // If the BubblegumV2 plugin is present, no external plugin adapters
        // can be added.  The BubblegumV2 plugin limits what can be on the
        // collection to plugins that are supported and validated at runtime
        // by the Bubblegum program.  External plugin adapters may be added
        // in the future but for now this subset was chosen.
        reject!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bubblegum_v2_len() {
        let bubblegum_v2 = BubblegumV2::default();
        let serialized = bubblegum_v2.try_to_vec().unwrap();
        assert_eq!(serialized.len(), bubblegum_v2.len());
    }
}
