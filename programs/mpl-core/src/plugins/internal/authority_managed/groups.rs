use crate::{
    plugins::{abstain, reject, PluginValidation, PluginValidationContext, ValidationResult},
    state::{DataBlob, Key},
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey};

/// Groups plugin for collections. Stores the immediate parent group accounts this collection
/// belongs to. Relationship updates are handled by specialized group instructions, and this
/// plugin overrides `validate_burn` to reject burning the group member itself while the group
/// set is non-empty.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, Default)]
pub struct Groups {
    /// The list of parent group accounts for this collection.
    pub groups: Vec<Pubkey>, // 4 + len * 32
}

impl Groups {
    const BASE_LEN: usize = 4; // length of the groups vector
}

impl DataBlob for Groups {
    fn len(&self) -> usize {
        Self::BASE_LEN + self.groups.len() * 32
    }
}

impl PluginValidation for Groups {
    fn validate_burn(
        &self,
        ctx: &PluginValidationContext,
    ) -> Result<ValidationResult, ProgramError> {
        // Only block the burn when the entity that actually owns this Groups plugin (and is
        // therefore the group member) is the one being burned. The plugin can also be reached
        // when burning an asset that merely belongs to a member collection: in that case the
        // plugin is inherited from the parent collection (`self_key == CollectionV1`) while an
        // asset is the burn target (`asset_info` is set), and burning the asset does not remove
        // the collection from any group, so we abstain.
        let member_is_burn_target = match ctx.self_key {
            // Plugin lives on the asset being burned (asset added directly to a group).
            Key::AssetV1 => true,
            // Plugin lives on the collection. This is the burn target only when no asset is
            // being burned (i.e. a collection burn).
            Key::CollectionV1 => ctx.asset_info.is_none(),
            _ => false,
        };

        if member_is_burn_target && !self.groups.is_empty() {
            reject!()
        } else {
            abstain!()
        }
    }
}
