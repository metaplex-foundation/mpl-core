use crate::{plugins::PluginValidation, state::DataBlob};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

/// Groups plugin for collections. Stores the immediate parent group accounts this collection
/// belongs to. Validation is deferred to specialized group instructions, so the plugin itself has
/// no special lifecycle behaviour beyond the default `PluginValidation` implementation.
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

impl PluginValidation for Groups {}
