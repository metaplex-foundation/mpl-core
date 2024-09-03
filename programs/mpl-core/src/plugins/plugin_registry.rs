use borsh::{BorshDeserialize, BorshSerialize};
use shank::ShankAccount;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult};
use std::{cmp::Ordering, collections::BTreeMap};

use crate::{
    error::MplCoreError,
    plugins::validate_lifecycle_checks,
    state::{Authority, DataBlob, Key, SolanaAccount},
};

use super::{
    CheckResult, ExternalCheckResult, ExternalCheckResultBits, ExternalPluginAdapterKey,
    ExternalPluginAdapterType, ExternalPluginAdapterUpdateInfo, HookableLifecycleEvent, PluginType,
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
    pub external_registry: Vec<ExternalRegistryRecord>, // 4
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
            (Key, ExternalCheckResultBits, ExternalRegistryRecord),
        >,
    ) -> ProgramResult {
        for record in &self.external_registry {
            if let Some(lifecycle_checks) = &record.lifecycle_checks {
                for (event, check_result) in lifecycle_checks {
                    if event == lifecycle_event {
                        let plugin_key = ExternalPluginAdapterKey::from_record(account, record)?;

                        result.insert(
                            plugin_key,
                            (
                                key,
                                ExternalCheckResultBits::from(*check_result),
                                record.clone(),
                            ),
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Increase the offsets of all plugins after a certain offset.
    pub(crate) fn bump_offsets(&mut self, offset: usize, size_diff: isize) -> ProgramResult {
        for record in &mut self.registry {
            if record.offset > offset {
                record.offset = (record.offset as isize)
                    .checked_add(size_diff)
                    .ok_or(MplCoreError::NumericalOverflow)?
                    as usize;
            }
        }

        for record in &mut self.external_registry {
            if record.offset > offset {
                solana_program::msg!("Bumping Record: {:?}", record);
                record.offset = (record.offset as isize)
                    .checked_add(size_diff)
                    .ok_or(MplCoreError::NumericalOverflow)?
                    as usize;

                if let Some(data_offset) = record.data_offset {
                    if data_offset > offset {
                        solana_program::msg!("Bumping Data: {:?}", record);
                        record.data_offset = Some(
                            (data_offset as isize)
                                .checked_add(size_diff)
                                .ok_or(MplCoreError::NumericalOverflow)?
                                as usize,
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
#[derive(Clone, BorshSerialize, BorshDeserialize, Debug, Eq, PartialEq)]
pub struct ExternalRegistryRecord {
    /// The adapter, third party plugin type.
    pub plugin_type: ExternalPluginAdapterType,
    /// The authority of the external plugin adapter.
    pub authority: Authority,
    /// The lifecyle events for which the the external plugin adapter is active.
    pub lifecycle_checks: Option<Vec<(HookableLifecycleEvent, ExternalCheckResult)>>,
    /// The offset to the plugin in the account.
    pub offset: usize, // 8
    /// For plugins with data, the offset to the data in the account.
    pub data_offset: Option<usize>,
    /// For plugins with data, the length of the data in the account.
    pub data_len: Option<usize>,
}

impl ExternalRegistryRecord {
    /// Update the adapter registry record with the new info, if relevant.
    pub fn update(&mut self, update_info: &ExternalPluginAdapterUpdateInfo) -> ProgramResult {
        match update_info {
            ExternalPluginAdapterUpdateInfo::LifecycleHook(update_info) => {
                if let Some(checks) = &update_info.lifecycle_checks {
                    validate_lifecycle_checks(checks, false)?;
                    self.lifecycle_checks
                        .clone_from(&update_info.lifecycle_checks)
                }
            }
            ExternalPluginAdapterUpdateInfo::Oracle(update_info) => {
                if let Some(checks) = &update_info.lifecycle_checks {
                    validate_lifecycle_checks(checks, true)?;
                    self.lifecycle_checks
                        .clone_from(&update_info.lifecycle_checks)
                }
            }
            _ => (),
        }

        Ok(())
    }
}
