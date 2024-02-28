use crate::state::{DataBlob, Key, SolanaAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;

/// The plugin header is the first part of the plugin metadata.
/// This field stores the Key
/// And a pointer to the Plugin Registry stored at the end of the account.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct PluginHeader {
    /// The Discriminator of the header which doubles as a Plugin metadata version.
    pub key: Key, // 1
    /// The offset to the plugin registry stored at the end of the account.
    pub plugin_registry_offset: u32, // 4
}

impl DataBlob for PluginHeader {
    fn get_initial_size() -> usize {
        1 + 4
    }

    fn get_size(&self) -> usize {
        1 + 4
    }
}

impl SolanaAccount for PluginHeader {
    fn key() -> Key {
        Key::PluginHeader
    }
}
