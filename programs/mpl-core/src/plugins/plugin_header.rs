use crate::state::{DataBlob, Key, SolanaAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;

/// The plugin header is the first part of the plugin metadata.
/// This field stores the Key
/// And a pointer to the Plugin Registry stored at the end of the account.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct PluginHeaderV1 {
    /// The Discriminator of the header which doubles as a Plugin metadata version.
    pub key: Key, // 1
    /// The offset to the plugin registry stored at the end of the account.
    pub plugin_registry_offset: usize, // 8
}

impl DataBlob for PluginHeaderV1 {
    const BASE_LEN: usize = 1 // Key
    + 8; // Offset

    fn len(&self) -> usize {
        Self::BASE_LEN
    }
}

impl SolanaAccount for PluginHeaderV1 {
    fn key() -> Key {
        Key::PluginHeaderV1
    }
}
