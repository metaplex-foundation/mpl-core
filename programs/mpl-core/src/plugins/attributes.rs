use super::PluginValidation;
use crate::state::DataBlob;
use borsh::{BorshDeserialize, BorshSerialize};

/// The Attribute type which represent a Key Value pair.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, Default)]
pub struct Attribute {
    /// The Key of the attribute.
    pub key: String, // 4
    /// The Value of the attribute.
    pub value: String, // 4
}

/// The Attributes plugin allows the authority to add arbitrary Key-Value pairs to the asset.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, Default)]
pub struct Attributes {
    /// A vector of Key-Value pairs.
    pub attribute_list: Vec<Attribute>, // 4
}

impl Attributes {
    /// Initialize the Attributes plugin, unfrozen by default.
    pub fn new() -> Self {
        Self::default()
    }
}

impl DataBlob for Attributes {
    fn get_initial_size() -> usize {
        4
    }

    fn get_size(&self) -> usize {
        4 // TODO: Implement this.
    }
}

impl PluginValidation for Attributes {}
