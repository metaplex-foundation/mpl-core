use crate::{plugins::PluginValidation, state::DataBlob};
use borsh::{BorshDeserialize, BorshSerialize};

/// The Attribute type which represent a Key Value pair.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, Default)]
pub struct Attribute {
    /// The Key of the attribute.
    pub key: String, // 4 + len
    /// The Value of the attribute.
    pub value: String, // 4 + len
}

impl DataBlob for Attribute {
    fn get_initial_size() -> usize {
        4 + 4
    }

    fn get_size(&self) -> usize {
        4 + self.key.len() + 4 + self.value.len()
    }
}

/// The Attributes plugin allows the authority to add arbitrary Key-Value pairs to the asset.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, PartialEq, Eq, Default)]
pub struct Attributes {
    /// A vector of Key-Value pairs.
    pub attribute_list: Vec<Attribute>, // 4 + len * Attribute
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
        4 + self
            .attribute_list
            .iter()
            .fold(0, |acc, attr| acc + attr.get_size())
    }
}

impl PluginValidation for Attributes {}
