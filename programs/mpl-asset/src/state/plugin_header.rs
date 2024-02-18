use crate::{state::Key, utils::DataBlob};
use borsh::{BorshDeserialize, BorshSerialize};

#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
pub struct PluginHeader {
    pub version: u8,              // 1
    pub plugin_map_offset: usize, // 8
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
