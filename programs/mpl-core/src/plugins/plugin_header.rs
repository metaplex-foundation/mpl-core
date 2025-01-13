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

impl PluginHeaderV1 {
    const BASE_LEN: usize = 1 // Key
    + 8; // Offset
}

impl DataBlob for PluginHeaderV1 {
    fn len(&self) -> usize {
        Self::BASE_LEN
    }
}

impl SolanaAccount for PluginHeaderV1 {
    fn key() -> Key {
        Key::PluginHeaderV1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_header_v1_len() {
        let header = PluginHeaderV1 {
            key: Key::PluginHeaderV1,
            plugin_registry_offset: 0,
        };
        let serialized = header.try_to_vec().unwrap();
        assert_eq!(serialized.len(), header.len());
    }
}
