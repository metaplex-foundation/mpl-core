use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;

use crate::state::{Authority, DataBlob, Key, SolanaAccount};

use super::PluginType;

/// The Plugin Registry stores a record of all plugins, their location, and their authorities.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct PluginRegistry {
    /// The Discriminator of the header which doubles as a Plugin metadata version.
    pub key: Key, // 1
    /// The registry of all plugins.
    pub registry: Vec<RegistryRecord>, // 4
    /// The registry of all external, third party, plugins.
    pub external_plugins: Vec<ExternalPluginRecord>, // 4
}

impl DataBlob for PluginRegistry {
    fn get_initial_size() -> usize {
        9
    }

    fn get_size(&self) -> usize {
        9 //TODO: Fix this
    }
}

impl SolanaAccount for PluginRegistry {
    fn key() -> Key {
        Key::PluginRegistry
    }
}

/// A simple type to store the mapping of Plugin type to Plugin data.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
pub struct RegistryRecord {
    /// The type of plugin.
    pub plugin_type: PluginType,
    /// The authorities who have permission to utilize a plugin.
    pub authorities: Vec<Authority>,
    /// The offset to the plugin in the account.
    pub offset: u32,
}

/// A simple type to store the mapping of external Plugin authority to Plugin data.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
pub struct ExternalPluginRecord {
    /// The authority of the external plugin.
    pub authority: Authority,
    /// The offset to the plugin in the account.
    pub offset: u32,
}
