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
    const BASE_LEN: usize = 1 // Key
     + 4 // Registry Length
     + 4; // External Registry Length

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
                    .try_into()
                    .map_err(|_| MplCoreError::NumericalOverflow)?;
            }
        }

        for record in &mut self.external_registry {
            if record.offset > offset {
                record.offset = (record.offset as isize)
                    .checked_add(size_diff)
                    .ok_or(MplCoreError::NumericalOverflow)?
                    .try_into()
                    .map_err(|_| MplCoreError::NumericalOverflow)?;

                if let Some(data_offset) = record.data_offset {
                    if data_offset > offset {
                        record.data_offset = Some(
                            (data_offset as isize)
                                .checked_add(size_diff)
                                .ok_or(MplCoreError::NumericalOverflow)?
                                .try_into()
                                .map_err(|_| MplCoreError::NumericalOverflow)?,
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

impl DataBlob for PluginRegistryV1 {
    fn len(&self) -> usize {
        Self::BASE_LEN
            + self
                .registry
                .iter()
                .map(|record| record.len())
                .sum::<usize>()
            + self
                .external_registry
                .iter()
                .map(|record| record.len())
                .sum::<usize>()
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
    pub plugin_type: PluginType, // 1
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

impl DataBlob for RegistryRecord {
    fn len(&self) -> usize {
        self.plugin_type.len() + self.authority.len() + 8
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
            ExternalPluginAdapterUpdateInfo::AgentIdentity(update_info) => {
                if let Some(checks) = &update_info.lifecycle_checks {
                    validate_lifecycle_checks(checks, false)?;
                    self.lifecycle_checks
                        .clone_from(&update_info.lifecycle_checks)
                }
            }
            _ => (),
        }

        Ok(())
    }
}

impl DataBlob for ExternalRegistryRecord {
    fn len(&self) -> usize {
        let mut len = self.plugin_type.len() + self.authority.len()
            + 1 // Lifecycle checks option
            + 8 // Offset
            + 1 // Data offset option
            + 1; // Data len option

        if let Some(checks) = &self.lifecycle_checks {
            len += 4 // 4 bytes for the length of the checks vector
                + checks.len()
                * (1 // HookableLifecycleEvent is a u8 enum
                    + 4); // ExternalCheckResult is a u32 flags
        }

        if self.data_offset.is_some() {
            len += 8;
        }

        if self.data_len.is_some() {
            len += 8;
        }

        len
    }
}

#[cfg(test)]
mod tests {
    use solana_program::pubkey::Pubkey;

    use super::*;

    #[test]
    fn test_plugin_registry_v1_default_len() {
        let registry = PluginRegistryV1 {
            key: Key::PluginRegistryV1,
            registry: vec![],
            external_registry: vec![],
        };
        let serialized = borsh::to_vec(&registry).unwrap();
        assert_eq!(serialized.len(), registry.len());
    }

    #[test]
    fn test_plugin_registry_v1_different_len() {
        let registry = PluginRegistryV1 {
            key: Key::PluginRegistryV1,
            registry: vec![
                RegistryRecord {
                    plugin_type: PluginType::TransferDelegate,
                    authority: Authority::UpdateAuthority,
                    offset: 0,
                },
                RegistryRecord {
                    plugin_type: PluginType::FreezeDelegate,
                    authority: Authority::Owner,
                    offset: 1,
                },
                RegistryRecord {
                    plugin_type: PluginType::PermanentBurnDelegate,
                    authority: Authority::Address {
                        address: Pubkey::default(),
                    },
                    offset: 2,
                },
            ],
            external_registry: vec![
                ExternalRegistryRecord {
                    plugin_type: ExternalPluginAdapterType::LifecycleHook,
                    authority: Authority::UpdateAuthority,
                    lifecycle_checks: None,
                    offset: 3,
                    data_offset: None,
                    data_len: None,
                },
                ExternalRegistryRecord {
                    plugin_type: ExternalPluginAdapterType::Oracle,
                    authority: Authority::Owner,
                    lifecycle_checks: Some(vec![]),
                    offset: 3,
                    data_offset: Some(4),
                    data_len: None,
                },
                ExternalRegistryRecord {
                    plugin_type: ExternalPluginAdapterType::AppData,
                    authority: Authority::Address {
                        address: Pubkey::default(),
                    },
                    lifecycle_checks: Some(vec![(
                        HookableLifecycleEvent::Create,
                        ExternalCheckResult { flags: 5 },
                    )]),
                    offset: 6,
                    data_offset: Some(7),
                    data_len: Some(8),
                },
            ],
        };
        let serialized = borsh::to_vec(&registry).unwrap();
        assert_eq!(serialized.len(), registry.len());
    }

    #[test]
    fn test_registry_record_len() {
        let records = vec![
            RegistryRecord {
                plugin_type: PluginType::TransferDelegate,
                authority: Authority::UpdateAuthority,
                offset: 0,
            },
            RegistryRecord {
                plugin_type: PluginType::FreezeDelegate,
                authority: Authority::Owner,
                offset: 1,
            },
            RegistryRecord {
                plugin_type: PluginType::PermanentBurnDelegate,
                authority: Authority::Address {
                    address: Pubkey::default(),
                },
                offset: 2,
            },
        ];

        for record in records {
            let serialized = borsh::to_vec(&record).unwrap();
            assert_eq!(serialized.len(), record.len());
        }
    }

    #[test]
    fn test_external_registry_record_len() {
        let records = vec![
            ExternalRegistryRecord {
                plugin_type: ExternalPluginAdapterType::LifecycleHook,
                authority: Authority::UpdateAuthority,
                lifecycle_checks: None,
                offset: 3,
                data_offset: None,
                data_len: None,
            },
            ExternalRegistryRecord {
                plugin_type: ExternalPluginAdapterType::Oracle,
                authority: Authority::Owner,
                lifecycle_checks: Some(vec![]),
                offset: 3,
                data_offset: Some(4),
                data_len: None,
            },
            ExternalRegistryRecord {
                plugin_type: ExternalPluginAdapterType::AppData,
                authority: Authority::Address {
                    address: Pubkey::default(),
                },
                lifecycle_checks: Some(vec![(
                    HookableLifecycleEvent::Create,
                    ExternalCheckResult { flags: 5 },
                )]),
                offset: 6,
                data_offset: Some(7),
                data_len: Some(8),
            },
        ];

        for record in records {
            let serialized = borsh::to_vec(&record).unwrap();
            assert_eq!(serialized.len(), record.len());
        }
    }
}

#[cfg(test)]
mod validation_tests {
    use {
        super::*,
        crate::plugins::{
            AgentIdentityUpdateInfo, AppDataUpdateInfo, LifecycleHookUpdateInfo,
            LinkedLifecycleHookUpdateInfo, OracleUpdateInfo,
        },
        solana_program::{program_error::ProgramError, pubkey::Pubkey},
    };

    fn core_err<T>(result: Result<T, ProgramError>) -> MplCoreError {
        match result {
            Err(ProgramError::Custom(code)) => {
                num_traits::FromPrimitive::from_u32(code).expect("an MplCoreError code")
            }
            Err(other) => panic!("expected a custom program error, got {other:?}"),
            Ok(_) => panic!("expected an error"),
        }
    }

    fn internal(offset: usize) -> RegistryRecord {
        RegistryRecord {
            plugin_type: PluginType::Attributes,
            authority: Authority::UpdateAuthority,
            offset,
        }
    }

    fn external(offset: usize, data_offset: Option<usize>) -> ExternalRegistryRecord {
        ExternalRegistryRecord {
            plugin_type: ExternalPluginAdapterType::AppData,
            authority: Authority::UpdateAuthority,
            lifecycle_checks: None,
            offset,
            data_offset,
            data_len: data_offset.map(|_| 4),
        }
    }

    fn registry() -> PluginRegistryV1 {
        PluginRegistryV1 {
            key: Key::PluginRegistryV1,
            registry: vec![internal(100), internal(200), internal(300)],
            external_registry: vec![external(400, Some(440)), external(500, Some(540))],
        }
    }

    #[test]
    fn bump_offsets_moves_only_records_after_the_pivot() {
        // Growing at 200: 200 itself is not moved (the comparison is strict).
        let mut grown = registry();
        grown.bump_offsets(200, 5).unwrap();
        assert_eq!(
            grown
                .registry
                .iter()
                .map(|record| record.offset)
                .collect::<Vec<_>>(),
            vec![100, 200, 305]
        );
        assert_eq!(
            grown
                .external_registry
                .iter()
                .map(|record| (record.offset, record.data_offset))
                .collect::<Vec<_>>(),
            vec![(405, Some(445)), (505, Some(545))]
        );

        // Shrinking is the same walk with a negative diff.
        let mut shrunk = registry();
        shrunk.bump_offsets(200, -5).unwrap();
        assert_eq!(
            shrunk
                .registry
                .iter()
                .map(|record| record.offset)
                .collect::<Vec<_>>(),
            vec![100, 200, 295]
        );
        assert_eq!(
            shrunk
                .external_registry
                .iter()
                .map(|record| (record.offset, record.data_offset))
                .collect::<Vec<_>>(),
            vec![(395, Some(435)), (495, Some(535))]
        );

        // A pivot after everything leaves the registry untouched.
        let mut untouched = registry();
        untouched.bump_offsets(10_000, 5).unwrap();
        assert_eq!(untouched.registry[2].offset, 300);
        assert_eq!(untouched.external_registry[1].offset, 500);

        // A `data_offset` at or before the pivot stays put while the record's
        // own offset moves.
        let mut split = PluginRegistryV1 {
            key: Key::PluginRegistryV1,
            registry: vec![],
            external_registry: vec![external(400, Some(200))],
        };
        split.bump_offsets(300, 7).unwrap();
        assert_eq!(split.external_registry[0].offset, 407);
        assert_eq!(split.external_registry[0].data_offset, Some(200));
    }

    #[test]
    fn bump_offsets_reports_overflow_on_a_negative_result() {
        // Only reachable from a unit test: the processors never shrink a plugin
        // by more than its own size.
        let mut internal_only = PluginRegistryV1 {
            key: Key::PluginRegistryV1,
            registry: vec![internal(100)],
            external_registry: vec![],
        };
        assert_eq!(
            core_err(internal_only.bump_offsets(10, -200)),
            MplCoreError::NumericalOverflow
        );

        let mut external_only = PluginRegistryV1 {
            key: Key::PluginRegistryV1,
            registry: vec![],
            external_registry: vec![external(100, None)],
        };
        assert_eq!(
            core_err(external_only.bump_offsets(10, -200)),
            MplCoreError::NumericalOverflow
        );

        let mut with_data = PluginRegistryV1 {
            key: Key::PluginRegistryV1,
            registry: vec![],
            external_registry: vec![external(300, Some(100))],
        };
        assert_eq!(
            core_err(with_data.bump_offsets(10, -200)),
            MplCoreError::NumericalOverflow
        );
    }

    #[test]
    fn external_registry_record_update_honours_only_some_update_infos() {
        let reject_only = vec![(
            HookableLifecycleEvent::Transfer,
            ExternalCheckResult { flags: 0x4 },
        )];
        let listen = vec![(
            HookableLifecycleEvent::Transfer,
            ExternalCheckResult { flags: 0x1 },
        )];

        // Oracle: reject-only checks are accepted, anything else is refused.
        let mut record = external(0, None);
        record
            .update(&ExternalPluginAdapterUpdateInfo::Oracle(OracleUpdateInfo {
                lifecycle_checks: Some(reject_only.clone()),
                base_address_config: None,
                results_offset: None,
            }))
            .unwrap();
        assert_eq!(
            record.lifecycle_checks.as_deref(),
            Some(reject_only.as_slice())
        );
        assert_eq!(
            core_err(
                record.update(&ExternalPluginAdapterUpdateInfo::Oracle(OracleUpdateInfo {
                    lifecycle_checks: Some(listen.clone()),
                    base_address_config: None,
                    results_offset: None,
                }))
            ),
            MplCoreError::OracleCanRejectOnly
        );

        // LifecycleHook and AgentIdentity accept any nonzero flags.
        let mut record = external(0, None);
        record
            .update(&ExternalPluginAdapterUpdateInfo::LifecycleHook(
                LifecycleHookUpdateInfo {
                    lifecycle_checks: Some(listen.clone()),
                    extra_accounts: None,
                    schema: None,
                },
            ))
            .unwrap();
        assert_eq!(record.lifecycle_checks.as_deref(), Some(listen.as_slice()));

        let mut record = external(0, None);
        record
            .update(&ExternalPluginAdapterUpdateInfo::AgentIdentity(
                AgentIdentityUpdateInfo {
                    uri: None,
                    lifecycle_checks: Some(listen.clone()),
                },
            ))
            .unwrap();
        assert_eq!(record.lifecycle_checks.as_deref(), Some(listen.as_slice()));

        // Roadmap section 11, finding 5: `LinkedLifecycleHook` falls into the
        // catch-all arm, so its `lifecycle_checks` are silently ignored — and
        // so are AppData's (which has none).
        let mut record = external(0, None);
        record
            .update(&ExternalPluginAdapterUpdateInfo::LinkedLifecycleHook(
                LinkedLifecycleHookUpdateInfo {
                    lifecycle_checks: Some(listen.clone()),
                    extra_accounts: None,
                    schema: None,
                },
            ))
            .unwrap();
        assert!(
            record.lifecycle_checks.is_none(),
            "LinkedLifecycleHook lifecycle checks are not applied"
        );

        record
            .update(&ExternalPluginAdapterUpdateInfo::AppData(
                AppDataUpdateInfo { schema: None },
            ))
            .unwrap();
        assert!(record.lifecycle_checks.is_none());

        // Empty checks are refused for every honoured variant.
        let mut record = external(0, None);
        assert_eq!(
            core_err(
                record.update(&ExternalPluginAdapterUpdateInfo::Oracle(OracleUpdateInfo {
                    lifecycle_checks: Some(vec![]),
                    base_address_config: None,
                    results_offset: None,
                }))
            ),
            MplCoreError::RequiresLifecycleCheck
        );
    }

    #[test]
    fn registry_record_compare_offsets_orders_by_offset() {
        // Only used by the compression path, which is `NotAvailable` on-chain.
        let first = internal(10);
        let second = internal(20);
        assert_eq!(
            RegistryRecord::compare_offsets(&first, &second),
            Ordering::Less
        );
        assert_eq!(
            RegistryRecord::compare_offsets(&second, &first),
            Ordering::Greater
        );
        assert_eq!(
            RegistryRecord::compare_offsets(&first, &internal(10)),
            Ordering::Equal
        );
    }

    #[test]
    fn check_adapter_registry_selects_records_for_the_event() {
        // A record with no lifecycle checks is never selected; one whose checks
        // list the event is, with its flags decoded into bits.
        let mut registry = PluginRegistryV1 {
            key: Key::PluginRegistryV1,
            registry: vec![],
            external_registry: vec![
                ExternalRegistryRecord {
                    plugin_type: ExternalPluginAdapterType::Oracle,
                    authority: Authority::UpdateAuthority,
                    lifecycle_checks: Some(vec![(
                        HookableLifecycleEvent::Transfer,
                        ExternalCheckResult { flags: 0x4 },
                    )]),
                    offset: 0,
                    data_offset: None,
                    data_len: None,
                },
                external(0, None),
            ],
        };

        // The Oracle record's key is read from the account data, so build an
        // account whose bytes hold the adapter at offset 0.
        let oracle = crate::plugins::ExternalPluginAdapter::Oracle(crate::plugins::Oracle {
            base_address: Pubkey::new_unique(),
            base_address_config: None,
            results_offset: crate::plugins::ValidationResultsOffset::NoOffset,
        });
        let mut account =
            crate::plugins::test_ctx::FakeAccount::with_data(borsh::to_vec(&oracle).unwrap());
        let info = account.info();

        let mut selected = BTreeMap::new();
        registry
            .check_adapter_registry(
                &info,
                Key::AssetV1,
                &HookableLifecycleEvent::Transfer,
                &mut selected,
            )
            .unwrap();
        assert_eq!(selected.len(), 1, "only the Oracle record lists Transfer");
        let (key, (owner_key, bits, _)) = selected.iter().next().unwrap();
        assert!(matches!(key, ExternalPluginAdapterKey::Oracle(_)));
        assert_eq!(*owner_key, Key::AssetV1);
        assert!(bits.can_reject());

        // A different event selects nothing.
        let mut selected = BTreeMap::new();
        registry
            .check_adapter_registry(
                &info,
                Key::AssetV1,
                &HookableLifecycleEvent::Burn,
                &mut selected,
            )
            .unwrap();
        assert!(selected.is_empty());

        // `check_registry` copies every internal record, keyed by plugin type.
        registry.registry = vec![internal(10)];
        let mut checks = BTreeMap::new();
        registry.check_registry(Key::AssetV1, PluginType::check_transfer, &mut checks);
        assert_eq!(
            checks.get(&PluginType::Attributes).map(|entry| entry.1),
            Some(CheckResult::None)
        );
    }
}
