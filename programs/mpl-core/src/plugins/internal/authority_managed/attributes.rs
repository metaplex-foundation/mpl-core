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

impl Attribute {
    const BASE_LEN: usize = 4 // The length of the Key string
    + 4; // The length of the Value string
}

impl DataBlob for Attribute {
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
    const BASE_LEN: usize = 4; // The length of the attribute list

    /// Initialize the Attributes plugin, unfrozen by default.
    pub fn new() -> Self {
        Self::default()
    }
}

impl DataBlob for Attributes {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attribute_len() {
        let attribute = Attribute {
            key: "test".to_string(),
            value: "test".to_string(),
        };
        let serialized = attribute.try_to_vec().unwrap();
        assert_eq!(serialized.len(), attribute.len());
    }

    #[test]
    fn test_attributes_default_len() {
        let attributes = Attributes::new();
        let serialized = attributes.try_to_vec().unwrap();
        assert_eq!(serialized.len(), attributes.len());
    }

    #[test]
    fn test_attributes_len() {
        let attributes = Attributes {
            attribute_list: vec![
                Attribute {
                    key: "test".to_string(),
                    value: "test".to_string(),
                },
                Attribute {
                    key: "test2".to_string(),
                    value: "test2".to_string(),
                },
            ],
        };
        let serialized = attributes.try_to_vec().unwrap();
        assert_eq!(serialized.len(), attributes.len());
    }
}
