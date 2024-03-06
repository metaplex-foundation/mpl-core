use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use std::{cmp::Ordering, collections::BTreeMap};

use crate::state::{Authority, DataBlob, Key, SolanaAccount};

use super::{CheckResult, PluginType};

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

impl PluginRegistry {
    /// Evaluate create checks for all plugins.
    pub fn check_create(
        &self,
        key: Key,
        result: &mut BTreeMap<PluginType, (Key, CheckResult, RegistryRecord)>,
    ) {
        for record in &self.registry {
            result.insert(
                record.plugin_type,
                (key, record.plugin_type.check_create(), record.clone()),
            );
        }
    }

    /// Evaluate update checks for all plugins.
    pub fn check_update(
        &self,
        key: Key,
        result: &mut BTreeMap<PluginType, (Key, CheckResult, RegistryRecord)>,
    ) {
        for record in &self.registry {
            result.insert(
                record.plugin_type,
                (key, record.plugin_type.check_update(), record.clone()),
            );
        }
    }

    /// Evaluate delete checks for all plugins.
    pub fn check_burn(
        &self,
        key: Key,
        result: &mut BTreeMap<PluginType, (Key, CheckResult, RegistryRecord)>,
    ) {
        for record in &self.registry {
            result.insert(
                record.plugin_type,
                (key, record.plugin_type.check_burn(), record.clone()),
            );
        }
    }

    /// Evaluate transfer checks for all plugins.
    pub fn check_transfer(
        &self,
        key: Key,
        result: &mut BTreeMap<PluginType, (Key, CheckResult, RegistryRecord)>,
    ) {
        for record in &self.registry {
            result.insert(
                record.plugin_type,
                (key, record.plugin_type.check_transfer(), record.clone()),
            );
        }
    }

    /// Evaluate the compress checks for all plugins.
    pub fn check_compress(
        &self,
        key: Key,
        result: &mut BTreeMap<PluginType, (Key, CheckResult, RegistryRecord)>,
    ) {
        for record in &self.registry {
            result.insert(
                record.plugin_type,
                (key, record.plugin_type.check_compress(), record.clone()),
            );
        }
    }

    /// Evaluate the decompress checks for all plugins.
    pub fn check_decompress(
        &self,
        key: Key,
        result: &mut BTreeMap<PluginType, (Key, CheckResult, RegistryRecord)>,
    ) {
        for record in &self.registry {
            result.insert(
                record.plugin_type,
                (key, record.plugin_type.check_decompress(), record.clone()),
            );
        }
    }
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
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct RegistryRecord {
    /// The type of plugin.
    pub plugin_type: PluginType,
    /// The authorities who have permission to utilize a plugin.
    pub authorities: Vec<Authority>,
    /// The offset to the plugin in the account.
    pub offset: usize,
}

impl RegistryRecord {
    /// Associated function for sorting `RegistryRecords` by offset.
    pub fn compare_offsets(a: &RegistryRecord, b: &RegistryRecord) -> Ordering {
        a.offset.cmp(&b.offset)
    }
}

/// A simple type to store the mapping of external Plugin authority to Plugin data.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
pub struct ExternalPluginRecord {
    /// The authority of the external plugin.
    pub authority: Authority,
    /// The offset to the plugin in the account.
    pub offset: usize,
}
