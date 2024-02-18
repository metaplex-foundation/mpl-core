use borsh::{BorshDeserialize, BorshSerialize};

use crate::{state::Key, utils::DataBlob};

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
pub struct PluginHeader {
    pub key: Key,
    pub plugin_registry_offset: usize, // 8
}

impl DataBlob for PluginHeader {
    fn get_initial_size() -> usize {
        1 + 8
    }

    fn get_size(&self) -> usize {
        1 + 8
    }

    fn key() -> Key {
        Key::PluginHeader
    }
}
