use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use std::{cmp::Ordering, collections::BTreeMap};

use crate::state::{Authority, DataBlob, Key, SolanaAccount};

use super::{CheckResult, ExternalCheckResult, ExternalPluginKey, LifecycleEvent, PluginType};

/// The Plugin Registry stores a record of all plugins, their location, and their authorities.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct PluginRegistryV1 {
    /// The Discriminator of the header which doubles as a plugin metadata version.
    pub key: Key, // 1
    /// The registry of all plugins.
    pub registry: Vec<RegistryRecord>, // 4
    /// The registry of all external, third party, plugins.
    pub external_plugins: Vec<ExternalPluginRecord>, // 4
}

impl PluginRegistryV1 {
    /// Evaluate checks for all plugins in the registry.
    pub(crate) fn check_registry(
        &self,
        key: Key,
        check_fp: fn(&PluginType) -> CheckResult,
        result: &mut BTreeMap<PluginType, (Key, CheckResult, RegistryRecord)>,
    ) {
        for record in &self.registry {
            result.insert(
                record.plugin_type,
                (key, check_fp(&record.plugin_type), record.clone()),
            );
        }
    }
}

impl DataBlob for PluginRegistryV1 {
    fn get_initial_size() -> usize {
        9
    }

    fn get_size(&self) -> usize {
        9 //TODO: Fix this
    }
}

impl SolanaAccount for PluginRegistryV1 {
    fn key() -> Key {
        Key::PluginRegistryV1
    }
}

/// A simple type to store the mapping of plugin type to plugin data.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
pub struct RegistryRecord {
    /// The type of plugin.
    pub plugin_type: PluginType, // 2
    /// The authority who has permission to utilize a plugin.
    pub authority: Authority, // Variable
    /// The offset to the plugin in the account.
    pub offset: usize, // 8
}

impl RegistryRecord {
    /// Associated function for sorting `RegistryRecords` by offset.
    pub fn compare_offsets(a: &RegistryRecord, b: &RegistryRecord) -> Ordering {
        a.offset.cmp(&b.offset)
    }
}

/// A type to store the mapping of third party plugin type to third party plugin header and data.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug)]
pub struct ExternalPluginRecord {
    /// The third party plugin key.
    pub plugin_key: ExternalPluginKey,
    /// The authority of the external plugin.
    pub authority: Authority,
    /// The lifecyle events for which the the third party plugin is active.
    pub lifecycle_checks: Vec<(LifecycleEvent, ExternalCheckResult)>,
    /// The offset to the external plugin header in the account.
    pub header_offset: usize,
    /// The offset to the external plugin data in the account.
    pub data_offset: Option<usize>,
}
