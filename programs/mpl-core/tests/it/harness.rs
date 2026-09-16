//! Self-checks for the shared test library: the raw account builders must
//! reproduce what the program writes byte for byte, the type conversion must
//! round-trip, and the auxiliary builtins must be invocable.

use {
    crate::common::*,
    mpl_core::types as client,
    mpl_core_program::{
        plugins::{
            AppData, Attribute, Attributes, Creator, ExternalCheckResult, ExternalPluginAdapter,
            ExternalPluginAdapterInitInfo, ExternalPluginAdapterKey, ExternalPluginAdapterSchema,
            ExternalPluginAdapterType, FreezeDelegate, HookableLifecycleEvent, Oracle, Plugin,
            PluginType, Royalties, RuleSet, ValidationResultsOffset,
        },
        state::{Authority, UpdateAuthority},
    },
    solana_program::{
        instruction::{AccountMeta, Instruction, InstructionError},
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

/// Owner-managed `FreezeDelegate { frozen: false }` as the program and the
/// client see it.
fn freeze_plugin() -> (Plugin, client::PluginAuthorityPair) {
    let plugin = Plugin::FreezeDelegate(FreezeDelegate { frozen: false });
    let pair = client::PluginAuthorityPair {
        plugin: convert(&plugin),
        authority: Some(client::PluginAuthority::Owner),
    };
    (plugin, pair)
}

/// `Attributes` with two entries, managed by the update authority.
fn attributes_plugin() -> (Plugin, client::PluginAuthorityPair) {
    let plugin = Plugin::Attributes(Attributes {
        attribute_list: vec![
            Attribute {
                key: "key0".to_string(),
                value: "value0".to_string(),
            },
            Attribute {
                key: "key1".to_string(),
                value: "value1".to_string(),
            },
        ],
    });
    let pair = client::PluginAuthorityPair {
        plugin: convert(&plugin),
        authority: Some(client::PluginAuthority::UpdateAuthority),
    };
    (plugin, pair)
}

#[test]
fn raw_asset_builder_matches_program_layout() {
    let f = Fixture::new();
    let payer = f.fund(ACCOUNT_LAMPORTS);
    let owner = Pubkey::new_unique();
    let data_authority = Pubkey::new_unique();

    let (freeze, freeze_pair) = freeze_plugin();
    let (attributes, attributes_pair) = attributes_plugin();
    let app_data_init =
        ExternalPluginAdapterInitInfo::AppData(mpl_core_program::plugins::AppDataInitInfo {
            data_authority: Authority::Address {
                address: data_authority,
            },
            init_plugin_authority: None,
            schema: Some(ExternalPluginAdapterSchema::Json),
        });

    let asset = f.create_asset(CreateAssetArgs {
        payer: Some(payer),
        owner: Some(owner),
        plugins: vec![freeze_pair, attributes_pair],
        adapters: vec![convert(&app_data_init)],
        ..CreateAssetArgs::default()
    });

    let app_data = ExternalAdapterSpec::new(ExternalPluginAdapter::AppData(AppData {
        data_authority: Authority::Address {
            address: data_authority,
        },
        schema: ExternalPluginAdapterSchema::Json,
    }));
    let spec = AssetSpec::new(owner)
        .update_authority(UpdateAuthority::Address(payer))
        .plugin(freeze, Authority::Owner)
        .plugin(attributes, Authority::UpdateAuthority)
        .adapter(app_data.clone());

    let created = f.account(&asset);
    assert_registry_consistent(&created);
    assert_eq!(
        created.data,
        spec.build().data,
        "AssetSpec must reproduce the CreateV2 layout byte for byte"
    );

    // Write 16 bytes into the app data and compare again.
    let bytes: Vec<u8> = (0u8..16).collect();
    f.run_ok(&ix::write_external_plugin_adapter_data_v1(
        asset,
        None,
        payer,
        Some(data_authority),
        None,
        None,
        client::ExternalPluginAdapterKey::AppData(client::PluginAuthority::Address {
            address: data_authority,
        }),
        Some(bytes.clone()),
    ));

    let written = f.account(&asset);
    assert_registry_consistent(&written);
    let spec = AssetSpec {
        adapters: vec![app_data.data(bytes.clone())],
        ..spec
    };
    assert_eq!(
        written.data,
        spec.build().data,
        "AssetSpec with data must reproduce the WriteExternalPluginAdapterDataV1 layout byte for byte"
    );

    // The readers see the same thing.
    let parsed = f.parsed(&asset);
    let (authority, plugin) = parsed.plugin(PluginType::FreezeDelegate).unwrap();
    assert_eq!(authority, Authority::Owner);
    assert_eq!(
        plugin,
        Plugin::FreezeDelegate(FreezeDelegate { frozen: false })
    );
    let (record, _, data) = parsed
        .adapter(&ExternalPluginAdapterKey::AppData(Authority::Address {
            address: data_authority,
        }))
        .unwrap();
    assert_eq!(record.plugin_type, ExternalPluginAdapterType::AppData);
    assert_eq!(record.data_len, Some(16));
    assert_eq!(data, Some(bytes));
}

#[test]
fn raw_collection_builder_matches_program_layout() {
    let f = Fixture::new();
    let payer = f.fund(ACCOUNT_LAMPORTS);
    let creator = Pubkey::new_unique();
    let oracle_base = Pubkey::new_unique();

    let royalties = Plugin::Royalties(Royalties {
        basis_points: 500,
        creators: vec![Creator {
            address: creator,
            percentage: 100,
        }],
        rule_set: RuleSet::None,
    });
    let royalties_pair = client::PluginAuthorityPair {
        plugin: convert(&royalties),
        authority: None,
    };
    let (attributes, attributes_pair) = attributes_plugin();
    let oracle_init =
        ExternalPluginAdapterInitInfo::Oracle(mpl_core_program::plugins::OracleInitInfo {
            base_address: oracle_base,
            init_plugin_authority: None,
            lifecycle_checks: vec![(
                HookableLifecycleEvent::Transfer,
                ExternalCheckResult { flags: 0x4 },
            )],
            base_address_config: None,
            results_offset: Some(ValidationResultsOffset::Anchor),
        });

    let collection = f.create_collection(CreateCollectionArgs {
        payer: Some(payer),
        plugins: vec![royalties_pair, attributes_pair],
        adapters: vec![convert(&oracle_init)],
        ..CreateCollectionArgs::default()
    });

    let spec = CollectionSpec::new(payer)
        .plugin(royalties, Authority::UpdateAuthority)
        .plugin(attributes, Authority::UpdateAuthority)
        .adapter(
            ExternalAdapterSpec::new(ExternalPluginAdapter::Oracle(Oracle {
                base_address: oracle_base,
                base_address_config: None,
                results_offset: ValidationResultsOffset::Anchor,
            }))
            .lifecycle_checks(vec![(
                HookableLifecycleEvent::Transfer,
                ExternalCheckResult { flags: 0x4 },
            )]),
        );

    let created = f.account(&collection);
    assert_registry_consistent(&created);
    assert_eq!(
        created.data,
        spec.build().data,
        "CollectionSpec must reproduce the CreateCollectionV2 layout byte for byte"
    );
    assert_eq!(f.collection(&collection).update_authority, payer);
    assert!(read_plugin(&created, PluginType::Royalties).is_some());
    assert!(read_adapter(&created, &ExternalPluginAdapterKey::Oracle(oracle_base)).is_some());
}

#[test]
fn bare_and_empty_meta_specs_are_consistent() {
    let owner = Pubkey::new_unique();
    let bare = AssetSpec::new(owner).build();
    assert_registry_consistent(&bare);
    assert!(!parse_asset(&bare.data).1.has_meta());

    let empty = AssetSpec::new(owner).with_empty_meta().build();
    assert_registry_consistent(&empty);
    let (_, parsed) = parse_asset(&empty.data);
    assert!(parsed.has_meta());
    assert!(parsed.plugins.is_empty() && parsed.adapters.is_empty());

    let collection = CollectionSpec::new(owner).with_empty_meta().build();
    assert_registry_consistent(&collection);
    assert!(parse_collection(&collection.data).1.has_meta());
}

#[test]
fn convert_round_trips() {
    let plugin = Plugin::FreezeDelegate(FreezeDelegate { frozen: true });
    let client_plugin: client::Plugin = convert(&plugin);
    assert_eq!(
        client_plugin,
        client::Plugin::FreezeDelegate(client::FreezeDelegate { frozen: true })
    );
    let back: Plugin = convert(&client_plugin);
    assert_eq!(back, plugin);

    let init_info =
        ExternalPluginAdapterInitInfo::AppData(mpl_core_program::plugins::AppDataInitInfo {
            data_authority: Authority::Owner,
            init_plugin_authority: Some(Authority::Address {
                address: Pubkey::new_unique(),
            }),
            schema: Some(ExternalPluginAdapterSchema::MsgPack),
        });
    let client_init: client::ExternalPluginAdapterInitInfo = convert(&init_info);
    assert!(matches!(
        client_init,
        client::ExternalPluginAdapterInitInfo::AppData(_)
    ));
    let back: ExternalPluginAdapterInitInfo = convert(&client_init);
    assert_eq!(back, init_info);
}

#[test]
fn noop_builtin_is_invocable() {
    let mollusk = core_mollusk();
    let ix = Instruction {
        program_id: SPL_NOOP_ID,
        accounts: vec![],
        data: vec![1, 2, 3, 4],
    };
    let result = mollusk.process_instruction(&ix, &with_program_accounts(vec![]));
    assert_ok(&result);
}

#[test]
fn recorder_captures_invocation() {
    let mollusk = core_mollusk();
    let signer = Pubkey::new_unique();
    let readonly = Pubkey::new_unique();
    let ix = Instruction {
        program_id: RECORDER_ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new_readonly(readonly, false),
        ],
        data: vec![9, 8, 7],
    };
    let accounts = with_program_accounts(vec![
        (signer, payer_account(ACCOUNT_LAMPORTS)),
        (readonly, payer_account(0)),
    ]);

    clear_recorded();
    let result = mollusk.process_instruction(&ix, &accounts);
    assert_ok(&result);
    assert_eq!(
        take_recorded(),
        vec![RecordedInvocation {
            program_id: RECORDER_ID,
            accounts: vec![
                RecordedAccount {
                    pubkey: signer,
                    is_signer: true,
                    is_writable: true,
                },
                RecordedAccount {
                    pubkey: readonly,
                    is_signer: false,
                    is_writable: false,
                },
            ],
            data: vec![9, 8, 7],
        }]
    );
    assert!(
        take_recorded().is_empty(),
        "take_recorded must clear the log"
    );

    set_recorder_result(Err(InstructionError::Custom(42)));
    let result = mollusk.process_instruction(&ix, &accounts);
    assert_program_err(&result, ProgramError::Custom(42));
    assert_eq!(
        take_recorded().len(),
        1,
        "failed invocations are recorded too"
    );
    set_recorder_result(Ok(()));
}

#[test]
fn program_accounts_are_executable_in_both_modes() {
    let accounts = keyed_program_accounts();
    assert_eq!(accounts.len(), 4);
    for (key, account) in &accounts {
        assert!(
            account.executable,
            "program account {key} must be executable"
        );
    }
    // `with_program_accounts` replaces bogus entries instead of duplicating them.
    let fixed = with_program_accounts(vec![
        (mpl_core_program::ID, solana_account::Account::default()),
        (Pubkey::new_unique(), payer_account(1)),
    ]);
    assert_eq!(fixed.len(), 5);
    assert!(fixed
        .iter()
        .all(|(key, account)| *key != mpl_core_program::ID || account.executable));
}
