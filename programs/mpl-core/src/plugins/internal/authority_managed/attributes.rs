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
    const BASE_LEN: usize = 4 // The length of the Key string
    + 4; // The length of the Value string

    fn len(&self) -> usize {
        Self::BASE_LEN + self.key.len() + self.value.len()
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
    const BASE_LEN: usize = 4; // The length of the attribute list

    fn len(&self) -> usize {
        Self::BASE_LEN
            + self
                .attribute_list
                .iter()
                .map(|attr| attr.len())
                .sum::<usize>()
    }
}

impl PluginValidation for Attributes {}
