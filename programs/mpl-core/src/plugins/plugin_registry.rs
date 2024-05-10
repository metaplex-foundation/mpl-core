use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};
use std::{cmp::Ordering, collections::BTreeMap};

use crate::{
    plugins::validate_lifecycle_checks,
    state::{Authority, DataBlob, Key, SolanaAccount},
};

use super::{
    CheckResult, ExternalPluginAdapterCheckResult, ExternalPluginAdapterCheckResultBits,
    ExternalPluginAdapterKey, ExternalPluginAdapterType, ExternalPluginAdapterUpdateInfo,
    HookableLifecycleEvent, PluginType,
};

/// The Plugin Registry stores a record of all plugins, their location, and their authorities.
#[repr(C)]
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, ShankAccount)]
pub struct PluginRegistryV1 {
    /// The Discriminator of the header which doubles as a plugin metadata version.
    pub key: Key, // 1
    /// The registry of all plugins.
    pub registry: Vec<RegistryRecord>, // 4
    /// The registry of all adapter, third party, plugins.
    pub external_plugin_adapter_registry: Vec<ExternalPluginAdapterRegistryRecord>, // 4
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

    pub(crate) fn check_adapter_registry(
        &self,
        account: &AccountInfo,
        key: Key,
        lifecycle_event: &HookableLifecycleEvent,
        result: &mut BTreeMap<
            ExternalPluginAdapterKey,
            (
                Key,
                ExternalPluginAdapterCheckResultBits,
                ExternalPluginAdapterRegistryRecord,
            ),
        >,
    ) -> ProgramResult {
        for record in &self.external_plugin_adapter_registry {
            if let Some(lifecycle_checks) = &record.lifecycle_checks {
                for (event, check_result) in lifecycle_checks {
                    if event == lifecycle_event {
                        let plugin_key = ExternalPluginAdapterKey::from_record(account, record)?;

                        result.insert(
                            plugin_key,
                            (
                                key,
                                ExternalPluginAdapterCheckResultBits::from(*check_result),
                                record.clone(),
                            ),
                        );
                    }
                }
            }
        }

        Ok(())
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
pub struct ExternalPluginAdapterRegistryRecord {
    /// The adapter, third party plugin type.
    pub plugin_type: ExternalPluginAdapterType,
    /// The authority of the external plugin adapter.
    pub authority: Authority,
    /// The lifecyle events for which the the external plugin adapter is active.
    pub lifecycle_checks: Option<Vec<(HookableLifecycleEvent, ExternalPluginAdapterCheckResult)>>,
    /// The offset to the plugin in the account.
    pub offset: usize, // 8
    /// For plugins with data, the offset to the data in the account.
    pub data_offset: Option<usize>,
    /// For plugins with data, the length of the data in the account.
    pub data_len: Option<usize>,
}

impl ExternalPluginAdapterRegistryRecord {
    /// Update the adapter registry record with the new info, if relevant.
    pub fn update(&mut self, update_info: &ExternalPluginAdapterUpdateInfo) -> ProgramResult {
        match update_info {
            ExternalPluginAdapterUpdateInfo::LifecycleHook(update_info) => {
                if let Some(checks) = &update_info.lifecycle_checks {
                    validate_lifecycle_checks(checks, false)?;
                    self.lifecycle_checks = update_info.lifecycle_checks.clone()
                }
            }
            ExternalPluginAdapterUpdateInfo::Oracle(update_info) => {
                if let Some(checks) = &update_info.lifecycle_checks {
                    validate_lifecycle_checks(checks, true)?;
                    self.lifecycle_checks = update_info.lifecycle_checks.clone()
                }
            }
            _ => (),
        }

        Ok(())
    }
}
