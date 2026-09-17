//! External plugin adapters: Oracle, AppData, LinkedAppData and DataSection.
//!
//! Ports of `clients/js/test/externalPlugins/*.test.ts`.
//!
//! Oracle needs no program: the adapter reads a plain account whose bytes hold
//! a borsh `OracleValidation` at `results_offset`, and never checks its owner,
//! so [`oracle_account`] fabricates it. The account is handed to the
//! instruction as a remaining account, which is how `validate_helper` finds it
//! in `ctx.accounts`.
//!
//! LifecycleHook and LinkedLifecycleHook are refused on-chain
//! (`plugins/utils.rs` returns `NotAvailable`), which the blocked-path tests
//! at the end of this file pin down.

use {
    crate::common::*,
    mpl_core::types as client,
    mpl_core_program::{
        error::MplCoreError,
        plugins::{
            AppData, AppDataInitInfo, Attributes, BubblegumV2, DataSection, DataSectionInitInfo,
            ExternalCheckResult, ExternalPluginAdapter, ExternalPluginAdapterInitInfo,
            ExternalPluginAdapterKey, ExternalPluginAdapterSchema, ExternalPluginAdapterUpdateInfo,
            ExternalValidationResult, ExtraAccount, FreezeDelegate, HookableLifecycleEvent,
            LifecycleHookInitInfo, LinkedAppData, LinkedAppDataInitInfo, LinkedAppDataUpdateInfo,
            LinkedDataKey, LinkedLifecycleHookInitInfo, Oracle, OracleInitInfo, OracleUpdateInfo,
            OracleValidation, Plugin, PluginType, Seed, ValidationResultsOffset,
        },
        state::{Authority, UpdateAuthority},
    },
    solana_program::{instruction::AccountMeta, program_error::ProgramError, pubkey::Pubkey},
};

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// `ExternalCheckResult` flag bits.
const CAN_LISTEN: u32 = 0x1;
const CAN_APPROVE: u32 = 0x2;
const CAN_REJECT: u32 = 0x4;

/// A lifecycle check entry.
fn check(
    event: HookableLifecycleEvent,
    flags: u32,
) -> (HookableLifecycleEvent, ExternalCheckResult) {
    (event, ExternalCheckResult { flags })
}

/// The generated client's view of an init info.
fn client_init(info: &ExternalPluginAdapterInitInfo) -> client::ExternalPluginAdapterInitInfo {
    convert(info)
}

/// The generated client's view of an adapter key.
fn client_key(key: &ExternalPluginAdapterKey) -> client::ExternalPluginAdapterKey {
    convert(key)
}

/// The generated client's view of an update info.
fn client_update(
    info: &ExternalPluginAdapterUpdateInfo,
) -> client::ExternalPluginAdapterUpdateInfo {
    convert(info)
}

/// `Authority::Address { address }`.
fn address(address: Pubkey) -> Authority {
    Authority::Address { address }
}

/// An `OracleValidation::V1` that rejects exactly `event` and passes the rest.
fn rejects(event: HookableLifecycleEvent) -> OracleValidation {
    let pick = |candidate: HookableLifecycleEvent| {
        if candidate == event {
            ExternalValidationResult::Rejected
        } else {
            ExternalValidationResult::Pass
        }
    };
    OracleValidation::V1 {
        create: pick(HookableLifecycleEvent::Create),
        transfer: pick(HookableLifecycleEvent::Transfer),
        burn: pick(HookableLifecycleEvent::Burn),
        update: pick(HookableLifecycleEvent::Update),
    }
}

/// An `OracleValidation::V1` that rejects every event.
fn rejects_everything() -> OracleValidation {
    OracleValidation::V1 {
        create: ExternalValidationResult::Rejected,
        transfer: ExternalValidationResult::Rejected,
        burn: ExternalValidationResult::Rejected,
        update: ExternalValidationResult::Rejected,
    }
}

/// An `OracleValidation::V1` that passes every event.
fn passes_everything() -> OracleValidation {
    OracleValidation::V1 {
        create: ExternalValidationResult::Pass,
        transfer: ExternalValidationResult::Pass,
        burn: ExternalValidationResult::Pass,
        update: ExternalValidationResult::Pass,
    }
}

/// An `Oracle` init info at `base_address` with the given checks.
fn oracle_init(
    base_address: Pubkey,
    lifecycle_checks: Vec<(HookableLifecycleEvent, ExternalCheckResult)>,
) -> ExternalPluginAdapterInitInfo {
    ExternalPluginAdapterInitInfo::Oracle(OracleInitInfo {
        base_address,
        init_plugin_authority: None,
        lifecycle_checks,
        base_address_config: None,
        results_offset: None,
    })
}

/// The stored `Oracle` adapter value.
fn oracle_adapter(
    base_address: Pubkey,
    config: Option<ExtraAccount>,
    results_offset: ValidationResultsOffset,
) -> ExternalPluginAdapter {
    ExternalPluginAdapter::Oracle(Oracle {
        base_address,
        base_address_config: config,
        results_offset,
    })
}

/// An `ExternalAdapterSpec` for an Oracle registered for `event` with
/// `can_reject`.
fn oracle_spec(
    base_address: Pubkey,
    config: Option<ExtraAccount>,
    results_offset: ValidationResultsOffset,
    event: HookableLifecycleEvent,
) -> ExternalAdapterSpec {
    ExternalAdapterSpec::new(oracle_adapter(base_address, config, results_offset))
        .lifecycle_checks(vec![check(event, CAN_REJECT)])
}

/// A read-only remaining account.
fn extra(key: Pubkey) -> AccountMeta {
    AccountMeta::new_readonly(key, false)
}

/// A funded payer, owner and update authority, all distinct.
struct Actors {
    payer: Pubkey,
    owner: Pubkey,
    update_authority: Pubkey,
}

impl Actors {
    fn new(f: &Fixture) -> Self {
        Self {
            payer: f.fund(ACCOUNT_LAMPORTS),
            owner: f.fund(ACCOUNT_LAMPORTS),
            update_authority: f.fund(ACCOUNT_LAMPORTS),
        }
    }
}

/// Stores an asset owned by `owner` carrying the given adapters.
fn store_asset(
    f: &Fixture,
    owner: Pubkey,
    update_authority: Pubkey,
    adapters: Vec<ExternalAdapterSpec>,
) -> Pubkey {
    let asset = Pubkey::new_unique();
    let mut spec =
        AssetSpec::new(owner).update_authority(UpdateAuthority::Address(update_authority));
    for adapter in adapters {
        spec = spec.adapter(adapter);
    }
    f.store(asset, spec.build());
    asset
}

/// Stores a collection with the given adapters plus a member asset, and
/// returns `(collection, asset)`.
fn store_collection_with_member(
    f: &Fixture,
    update_authority: Pubkey,
    owner: Pubkey,
    adapters: Vec<ExternalAdapterSpec>,
) -> (Pubkey, Pubkey) {
    let collection = Pubkey::new_unique();
    let mut spec = CollectionSpec::new(update_authority).sizes(1, 1);
    for adapter in adapters {
        spec = spec.adapter(adapter);
    }
    f.store(collection, spec.build());

    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Collection(collection))
            .build(),
    );
    (collection, asset)
}

// ===========================================================================
// oracle.test.ts — deny paths
// ===========================================================================

/// JS: externalPlugins/oracle.test.ts :: it can use fixed address oracle to deny transfer
#[test]
fn oracle_denies_transfer() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);
    let oracle = Pubkey::new_unique();
    f.store(
        oracle,
        oracle_account(&rejects(HookableLifecycleEvent::Transfer), 0),
    );

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![oracle_spec(
            oracle,
            None,
            ValidationResultsOffset::NoOffset,
            HookableLifecycleEvent::Transfer,
        )],
    );

    let result = f.run(&with_remaining(
        ix::transfer_v1(
            asset,
            None,
            actors.payer,
            Some(actors.owner),
            new_owner,
            None,
            None,
        ),
        [extra(oracle)],
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(f.asset(&asset).owner, actors.owner);
}

/// An oracle whose result is `Pass` lets the transfer through: the `Pass` arm
/// of `validate_external_plugin_adapter_checks`.
#[test]
fn oracle_that_passes_allows_the_transfer() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);
    let oracle = Pubkey::new_unique();
    f.store(oracle, oracle_account(&passes_everything(), 0));

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![oracle_spec(
            oracle,
            None,
            ValidationResultsOffset::NoOffset,
            HookableLifecycleEvent::Transfer,
        )],
    );

    f.run_ok(&with_remaining(
        ix::transfer_v1(
            asset,
            None,
            actors.payer,
            Some(actors.owner),
            new_owner,
            None,
            None,
        ),
        [extra(oracle)],
    ));
    assert_eq!(f.asset(&asset).owner, new_owner);
}

/// JS: externalPlugins/oracle.test.ts :: it can use fixed address oracle to deny burn
#[test]
fn oracle_denies_burn() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let oracle = Pubkey::new_unique();
    f.store(
        oracle,
        oracle_account(&rejects(HookableLifecycleEvent::Burn), 0),
    );

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![oracle_spec(
            oracle,
            None,
            ValidationResultsOffset::NoOffset,
            HookableLifecycleEvent::Burn,
        )],
    );

    let result = f.run(&with_remaining(
        ix::burn_v1(asset, None, actors.payer, Some(actors.owner), None, None),
        [extra(oracle)],
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(f.asset(&asset).owner, actors.owner);
}

/// JS: externalPlugins/oracle.test.ts :: it can use fixed address oracle to deny update
#[test]
fn oracle_denies_update() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let oracle = Pubkey::new_unique();
    f.store(
        oracle,
        oracle_account(&rejects(HookableLifecycleEvent::Update), 0),
    );

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![oracle_spec(
            oracle,
            None,
            ValidationResultsOffset::NoOffset,
            HookableLifecycleEvent::Update,
        )],
    );

    let result = f.run(&with_remaining(
        ix::update_v1(
            asset,
            None,
            actors.payer,
            Some(actors.update_authority),
            None,
            Some("Renamed".to_string()),
            None,
            None,
        ),
        [extra(oracle)],
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(f.asset(&asset).name, "Test Asset");
}

/// JS: externalPlugins/oracle.test.ts :: it can use fixed address oracle to deny update via collection
#[test]
fn collection_oracle_denies_a_member_update() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let oracle = Pubkey::new_unique();
    f.store(
        oracle,
        oracle_account(&rejects(HookableLifecycleEvent::Update), 0),
    );

    let (collection, asset) = store_collection_with_member(
        &f,
        actors.update_authority,
        actors.owner,
        vec![oracle_spec(
            oracle,
            None,
            ValidationResultsOffset::NoOffset,
            HookableLifecycleEvent::Update,
        )],
    );

    let result = f.run(&with_remaining(
        ix::update_v1(
            asset,
            Some(collection),
            actors.payer,
            Some(actors.update_authority),
            None,
            Some("Renamed".to_string()),
            None,
            None,
        ),
        [extra(oracle)],
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(f.asset(&asset).name, "Test Asset");
}

/// JS: externalPlugins/oracle.test.ts :: it can use fixed address oracle to deny create
#[test]
fn oracle_denies_create() {
    let f = Fixture::new();
    let payer = f.fund(ACCOUNT_LAMPORTS);
    let oracle = Pubkey::new_unique();
    f.store(
        oracle,
        oracle_account(&rejects(HookableLifecycleEvent::Create), 0),
    );

    let asset = Pubkey::new_unique();
    f.store(asset, empty_account());
    let result = f.run(&ix::create_v2(
        asset,
        None,
        None,
        payer,
        None,
        None,
        None,
        "Test Asset",
        "https://example.com/test",
        None,
        Some(vec![client_init(&oracle_init(
            oracle,
            vec![check(HookableLifecycleEvent::Create, CAN_REJECT)],
        ))]),
        &[extra(oracle)],
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
}

/// JS: externalPlugins/oracle.test.ts :: it cannot use fixed address oracle to deny transfer if not registered for lifecycle event
#[test]
fn oracle_not_registered_for_the_event_is_not_consulted() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);
    let oracle = Pubkey::new_unique();
    // The account rejects everything, but only `Update` is registered.
    f.store(oracle, oracle_account(&rejects_everything(), 0));

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![oracle_spec(
            oracle,
            None,
            ValidationResultsOffset::NoOffset,
            HookableLifecycleEvent::Update,
        )],
    );

    f.run_ok(&with_remaining(
        ix::transfer_v1(
            asset,
            None,
            actors.payer,
            Some(actors.owner),
            new_owner,
            None,
            None,
        ),
        [extra(oracle)],
    ));
    assert_eq!(f.asset(&asset).owner, new_owner);
}

// ===========================================================================
// oracle.test.ts — oracle account error cases
// ===========================================================================

/// JS: externalPlugins/oracle.test.ts :: it transfer fails but does not panic when oracle account does not exist
/// JS: externalPlugins/oracle.test.ts :: it transfer fails but does not panic when oracle account is too small
/// JS: externalPlugins/oracle.test.ts :: it empty account does not default to valid oracle
#[test]
fn oracle_account_error_cases() {
    // (oracle bytes, or None for "not passed", stored offset, expected error)
    let cases: Vec<(Option<Vec<u8>>, ValidationResultsOffset, MplCoreError)> = vec![
        (
            None,
            ValidationResultsOffset::NoOffset,
            MplCoreError::MissingExternalPluginAdapterAccount,
        ),
        // Fewer than `OracleValidation::serialized_size()` bytes at the offset.
        (
            Some(vec![1, 0, 0]),
            ValidationResultsOffset::NoOffset,
            MplCoreError::InvalidOracleAccountData,
        ),
        // The offset itself is past the end of the account data.
        (
            Some(vec![1, 2, 2, 2, 2]),
            ValidationResultsOffset::Custom(64),
            MplCoreError::InvalidOracleAccountData,
        ),
        // A discriminant that is neither `Uninitialized` (0) nor `V1` (1).
        (
            Some(vec![2, 0, 0, 0, 0]),
            ValidationResultsOffset::NoOffset,
            MplCoreError::InvalidOracleAccountData,
        ),
        // All zeroes deserializes as `Uninitialized`, which is refused rather
        // than treated as "everything passes".
        (
            Some(vec![0; 32]),
            ValidationResultsOffset::NoOffset,
            MplCoreError::UninitializedOracleAccount,
        ),
    ];

    for (bytes, offset, expected) in cases {
        let f = Fixture::new();
        let actors = Actors::new(&f);
        let new_owner = f.fund(ACCOUNT_LAMPORTS);
        let oracle = Pubkey::new_unique();
        let passed = bytes.is_some();
        if let Some(bytes) = bytes {
            f.store(oracle, buffer_account(bytes));
        }

        let asset = store_asset(
            &f,
            actors.owner,
            actors.update_authority,
            vec![oracle_spec(
                oracle,
                None,
                offset,
                HookableLifecycleEvent::Transfer,
            )],
        );

        let ix = ix::transfer_v1(
            asset,
            None,
            actors.payer,
            Some(actors.owner),
            new_owner,
            None,
            None,
        );
        let ix = if passed {
            with_remaining(ix, [extra(oracle)])
        } else {
            ix
        };
        let result = f.run(&ix);
        assert_core_err(&result, expected);
        assert_eq!(f.asset(&asset).owner, actors.owner);
    }
}

/// JS: externalPlugins/oracle.test.ts :: it can use preconfigured asset pda custom offset oracle to deny update
///
/// `NoOffset` reads at 0, `Anchor` skips an 8-byte discriminator and
/// `Custom(n)` skips `n` bytes; all three find the same struct.
#[test]
fn oracle_results_offsets() {
    for offset in [
        ValidationResultsOffset::NoOffset,
        ValidationResultsOffset::Anchor,
        ValidationResultsOffset::Custom(42),
    ] {
        let f = Fixture::new();
        let actors = Actors::new(&f);
        let new_owner = f.fund(ACCOUNT_LAMPORTS);
        let oracle = Pubkey::new_unique();
        f.store(
            oracle,
            oracle_account(
                &rejects(HookableLifecycleEvent::Transfer),
                offset.to_offset_usize(),
            ),
        );

        let asset = store_asset(
            &f,
            actors.owner,
            actors.update_authority,
            vec![oracle_spec(
                oracle,
                None,
                offset,
                HookableLifecycleEvent::Transfer,
            )],
        );

        let result = f.run(&with_remaining(
            ix::transfer_v1(
                asset,
                None,
                actors.payer,
                Some(actors.owner),
                new_owner,
                None,
                None,
            ),
            [extra(oracle)],
        ));
        assert_core_err(&result, MplCoreError::InvalidAuthority);
    }
}

// ===========================================================================
// oracle.test.ts — validate_lifecycle_checks
// ===========================================================================

/// JS: externalPlugins/oracle.test.ts :: it cannot add oracle with no lifecycle checks to asset
/// JS: externalPlugins/oracle.test.ts :: it cannot add oracle to asset that can approve
/// JS: externalPlugins/oracle.test.ts :: it cannot add oracle to asset that can listen
/// JS: externalPlugins/oracle.test.ts :: it cannot add oracle to asset that can approve in addition to reject
#[test]
fn oracle_lifecycle_checks_are_validated_on_add() {
    let cases: Vec<(
        Vec<(HookableLifecycleEvent, ExternalCheckResult)>,
        MplCoreError,
    )> = vec![
        (vec![], MplCoreError::RequiresLifecycleCheck),
        (
            vec![
                check(HookableLifecycleEvent::Transfer, CAN_REJECT),
                check(HookableLifecycleEvent::Transfer, CAN_REJECT),
            ],
            MplCoreError::DuplicateLifecycleChecks,
        ),
        (
            vec![check(HookableLifecycleEvent::Transfer, CAN_APPROVE)],
            MplCoreError::OracleCanRejectOnly,
        ),
        (
            vec![check(HookableLifecycleEvent::Transfer, CAN_LISTEN)],
            MplCoreError::OracleCanRejectOnly,
        ),
        (
            vec![check(
                HookableLifecycleEvent::Transfer,
                CAN_REJECT | CAN_APPROVE,
            )],
            MplCoreError::OracleCanRejectOnly,
        ),
    ];

    let f = Fixture::new();
    let actors = Actors::new(&f);
    let asset = store_asset(&f, actors.owner, actors.update_authority, vec![]);

    for (checks, expected) in cases {
        let oracle = Pubkey::new_unique();
        let result = f.run(&ix::add_external_plugin_adapter_v1(
            asset,
            None,
            actors.payer,
            Some(actors.update_authority),
            None,
            client_init(&oracle_init(oracle, checks)),
        ));
        assert_core_err(&result, expected);
    }
    // Nothing was added by any of the rejected instructions.
    assert_eq!(
        f.account(&asset).data,
        AssetSpec::new(actors.owner)
            .update_authority(UpdateAuthority::Address(actors.update_authority))
            .build()
            .data
    );
}

/// JS: externalPlugins/oracle.test.ts :: it cannot update oracle to listen
#[test]
fn oracle_lifecycle_checks_are_validated_on_update() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let oracle = Pubkey::new_unique();
    let checks = vec![check(HookableLifecycleEvent::Transfer, CAN_REJECT)];

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![oracle_spec(
            oracle,
            None,
            ValidationResultsOffset::NoOffset,
            HookableLifecycleEvent::Transfer,
        )],
    );

    let result = f.run(&ix::update_external_plugin_adapter_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_key(&ExternalPluginAdapterKey::Oracle(oracle)),
        client_update(&ExternalPluginAdapterUpdateInfo::Oracle(OracleUpdateInfo {
            lifecycle_checks: Some(vec![check(HookableLifecycleEvent::Transfer, CAN_LISTEN)]),
            base_address_config: None,
            results_offset: None,
        })),
    ));
    assert_core_err(&result, MplCoreError::OracleCanRejectOnly);

    let (record, _, _) = read_adapter(
        &f.account(&asset),
        &ExternalPluginAdapterKey::Oracle(oracle),
    )
    .expect("the oracle record survives the rejected update");
    assert_eq!(record.lifecycle_checks.as_deref(), Some(checks.as_slice()));
}

// ===========================================================================
// oracle.test.ts — add / update / remove
// ===========================================================================

/// JS: externalPlugins/oracle.test.ts :: it can add oracle to asset for multiple lifecycle events
#[test]
fn can_add_an_oracle_for_several_events() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let oracle = Pubkey::new_unique();
    let checks = vec![
        check(HookableLifecycleEvent::Transfer, CAN_REJECT),
        check(HookableLifecycleEvent::Burn, CAN_REJECT),
    ];

    let asset = store_asset(&f, actors.owner, actors.update_authority, vec![]);
    f.run_ok(&ix::add_external_plugin_adapter_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_init(&oracle_init(oracle, checks.clone())),
    ));

    let account = f.account(&asset);
    assert_registry_consistent(&account);
    let (record, adapter, data) = read_adapter(&account, &ExternalPluginAdapterKey::Oracle(oracle))
        .expect("the oracle should be in the registry");
    assert_eq!(record.authority, Authority::UpdateAuthority);
    assert_eq!(record.lifecycle_checks.as_deref(), Some(checks.as_slice()));
    assert_eq!(
        adapter,
        oracle_adapter(oracle, None, ValidationResultsOffset::NoOffset)
    );
    assert!(data.is_none(), "an Oracle adapter carries no data");
}

/// JS: externalPlugins/oracle.test.ts :: it can update oracle to larger registry record
/// JS: externalPlugins/oracle.test.ts :: it can grow a leading oracle without corrupting trailing oracle metadata
#[test]
fn growing_a_leading_oracle_bumps_the_trailing_record() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let first = Pubkey::new_unique();
    let second = Pubkey::new_unique();

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![
            oracle_spec(
                first,
                None,
                ValidationResultsOffset::NoOffset,
                HookableLifecycleEvent::Transfer,
            ),
            oracle_spec(
                second,
                None,
                ValidationResultsOffset::NoOffset,
                HookableLifecycleEvent::Burn,
            ),
        ],
    );
    let second_offset_before = read_adapter(
        &f.account(&asset),
        &ExternalPluginAdapterKey::Oracle(second),
    )
    .expect("second oracle")
    .0
    .offset;

    // Adding a `base_address_config` and an `Anchor` offset makes the leading
    // record longer, so the trailing one has to move.
    let config = ExtraAccount::Address {
        address: Pubkey::new_unique(),
        is_signer: false,
        is_writable: false,
    };
    f.run_ok(&ix::update_external_plugin_adapter_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_key(&ExternalPluginAdapterKey::Oracle(first)),
        client_update(&ExternalPluginAdapterUpdateInfo::Oracle(OracleUpdateInfo {
            lifecycle_checks: Some(vec![check(HookableLifecycleEvent::Burn, CAN_REJECT)]),
            base_address_config: Some(config.clone()),
            results_offset: Some(ValidationResultsOffset::Anchor),
        })),
    ));

    let account = f.account(&asset);
    assert_registry_consistent(&account);
    let (record, adapter, _) =
        read_adapter(&account, &ExternalPluginAdapterKey::Oracle(first)).expect("first oracle");
    assert_eq!(
        adapter,
        oracle_adapter(first, Some(config), ValidationResultsOffset::Anchor)
    );
    assert_eq!(
        record.lifecycle_checks.as_deref(),
        Some([check(HookableLifecycleEvent::Burn, CAN_REJECT)].as_slice())
    );

    let (second_record, second_adapter, _) =
        read_adapter(&account, &ExternalPluginAdapterKey::Oracle(second))
            .expect("second oracle still parses");
    assert!(
        second_record.offset > second_offset_before,
        "the trailing record should have been pushed back"
    );
    assert_eq!(
        second_adapter,
        oracle_adapter(second, None, ValidationResultsOffset::NoOffset)
    );
}

/// JS: externalPlugins/oracle.test.ts :: it can update oracle to smaller registry record
/// JS: externalPlugins/oracle.test.ts :: it can shrink a leading oracle without corrupting trailing oracle metadata
#[test]
fn shrinking_a_leading_oracle_bumps_the_trailing_record() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let first = Pubkey::new_unique();
    let second = Pubkey::new_unique();
    let config = ExtraAccount::Address {
        address: Pubkey::new_unique(),
        is_signer: false,
        is_writable: false,
    };

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![
            oracle_spec(
                first,
                Some(config),
                ValidationResultsOffset::Custom(999),
                HookableLifecycleEvent::Transfer,
            ),
            oracle_spec(
                second,
                None,
                ValidationResultsOffset::NoOffset,
                HookableLifecycleEvent::Burn,
            ),
        ],
    );
    let len_before = f.account(&asset).data.len();
    let second_offset_before = read_adapter(
        &f.account(&asset),
        &ExternalPluginAdapterKey::Oracle(second),
    )
    .expect("second oracle")
    .0
    .offset;

    // `Custom(999)` -> `NoOffset` drops the 8-byte offset from the record.
    f.run_ok(&ix::update_external_plugin_adapter_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_key(&ExternalPluginAdapterKey::Oracle(first)),
        client_update(&ExternalPluginAdapterUpdateInfo::Oracle(OracleUpdateInfo {
            lifecycle_checks: None,
            base_address_config: None,
            results_offset: Some(ValidationResultsOffset::NoOffset),
        })),
    ));

    let account = f.account(&asset);
    assert_registry_consistent(&account);
    assert!(
        account.data.len() < len_before,
        "the account should have shrunk"
    );
    let (second_record, second_adapter, _) =
        read_adapter(&account, &ExternalPluginAdapterKey::Oracle(second))
            .expect("second oracle still parses");
    assert!(second_record.offset < second_offset_before);
    assert_eq!(
        second_adapter,
        oracle_adapter(second, None, ValidationResultsOffset::NoOffset)
    );
}

/// JS: externalPlugins/oracle.test.ts :: it cannot update oracle using update authority when different from external plugin authority
#[test]
fn cannot_update_an_oracle_from_the_wrong_authority() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let oracle = Pubkey::new_unique();
    let plugin_authority = Pubkey::new_unique();

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![oracle_spec(
            oracle,
            None,
            ValidationResultsOffset::NoOffset,
            HookableLifecycleEvent::Transfer,
        )
        .authority(address(plugin_authority))],
    );

    let result = f.run(&ix::update_external_plugin_adapter_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_key(&ExternalPluginAdapterKey::Oracle(oracle)),
        client_update(&ExternalPluginAdapterUpdateInfo::Oracle(OracleUpdateInfo {
            lifecycle_checks: None,
            base_address_config: None,
            results_offset: Some(ValidationResultsOffset::Anchor),
        })),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
}

/// Removing an Oracle from an asset and from a collection, plus the
/// not-found path for a key that is not in the registry.
#[test]
fn can_remove_an_oracle_from_an_asset_and_a_collection() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let oracle = Pubkey::new_unique();
    let missing = Pubkey::new_unique();
    let spec = oracle_spec(
        oracle,
        None,
        ValidationResultsOffset::NoOffset,
        HookableLifecycleEvent::Transfer,
    );

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![spec.clone()],
    );
    let result = f.run(&ix::remove_external_plugin_adapter_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_key(&ExternalPluginAdapterKey::Oracle(missing)),
    ));
    assert_core_err(&result, MplCoreError::ExternalPluginAdapterNotFound);

    f.run_ok(&ix::remove_external_plugin_adapter_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_key(&ExternalPluginAdapterKey::Oracle(oracle)),
    ));
    let account = f.account(&asset);
    assert_registry_consistent(&account);
    assert!(read_adapter(&account, &ExternalPluginAdapterKey::Oracle(oracle)).is_none());

    let collection = Pubkey::new_unique();
    f.store(
        collection,
        CollectionSpec::new(actors.update_authority)
            .adapter(spec)
            .build(),
    );
    f.run_ok(&ix::remove_collection_external_plugin_adapter_v1(
        collection,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_key(&ExternalPluginAdapterKey::Oracle(oracle)),
    ));
    let account = f.account(&collection);
    assert_registry_consistent(&account);
    assert!(read_adapter(&account, &ExternalPluginAdapterKey::Oracle(oracle)).is_none());
}

/// Roadmap section 11, finding 3: an Oracle may register the `Execute` event
/// with `can_reject`, but `ExternalPluginAdapter::validate_execute` routes
/// Oracle to the abstaining trait default, so the check is never enforced.
/// This pins the current behaviour.
#[test]
fn oracle_execute_check_is_never_enforced() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let oracle = Pubkey::new_unique();
    f.store(oracle, oracle_account(&rejects_everything(), 0));

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![oracle_spec(
            oracle,
            None,
            ValidationResultsOffset::NoOffset,
            HookableLifecycleEvent::Execute,
        )],
    );

    let (asset_signer, _) = asset_signer_pda(&asset);
    f.store(asset_signer, payer_account(ACCOUNT_LAMPORTS));
    let ix = with_remaining(
        ix::execute_v1(
            asset,
            None,
            asset_signer,
            actors.payer,
            Some(actors.owner),
            RECORDER_ID,
            vec![0x01],
            &[],
        ),
        [
            extra(oracle),
            AccountMeta::new_readonly(asset_signer, false),
        ],
    );

    clear_recorded();
    f.run_ok(&ix);
    assert_eq!(
        take_recorded().len(),
        1,
        "the Execute oracle check is not enforced today"
    );
}

// ===========================================================================
// oracle.test.ts — ExtraAccount derivations
// ===========================================================================

/// The PDA the program derives for the `Preconfigured*` variants.
fn mpl_core_pda(base: &Pubkey, extra_seed: Option<&Pubkey>) -> Pubkey {
    let prefix = b"mpl-core";
    match extra_seed {
        None => Pubkey::find_program_address(&[prefix], base).0,
        Some(seed) => Pubkey::find_program_address(&[prefix, seed.as_ref()], base).0,
    }
}

/// JS: externalPlugins/oracle.test.ts :: it can use preconfigured program pda oracle to deny update
/// JS: externalPlugins/oracle.test.ts :: it can use preconfigured asset pda oracle to deny update
/// JS: externalPlugins/oracle.test.ts :: it can use preconfigured collection pda oracle to deny update
#[test]
fn oracle_extra_account_derivations_on_update() {
    let base = Pubkey::new_unique();

    // `None` means the PDA is derived from the base program alone.
    let variants: Vec<(&str, ExtraAccount, bool)> = vec![
        (
            "program",
            ExtraAccount::PreconfiguredProgram {
                is_signer: false,
                is_writable: false,
            },
            false,
        ),
        (
            "asset",
            ExtraAccount::PreconfiguredAsset {
                is_signer: false,
                is_writable: false,
            },
            false,
        ),
        (
            "collection",
            ExtraAccount::PreconfiguredCollection {
                is_signer: false,
                is_writable: false,
            },
            true,
        ),
        (
            "owner",
            ExtraAccount::PreconfiguredOwner {
                is_signer: false,
                is_writable: false,
            },
            false,
        ),
    ];

    for (name, config, needs_collection) in variants {
        let f = Fixture::new();
        let actors = Actors::new(&f);
        let spec = ExternalAdapterSpec::new(oracle_adapter(
            base,
            Some(config.clone()),
            ValidationResultsOffset::NoOffset,
        ))
        .lifecycle_checks(vec![check(HookableLifecycleEvent::Update, CAN_REJECT)]);

        let (collection, asset) = if needs_collection {
            let (collection, asset) =
                store_collection_with_member(&f, actors.update_authority, actors.owner, vec![spec]);
            (Some(collection), asset)
        } else {
            (
                None,
                store_asset(&f, actors.owner, actors.update_authority, vec![spec]),
            )
        };

        let seed = match name {
            "program" => None,
            "asset" => Some(asset),
            "collection" => collection,
            "owner" => Some(actors.owner),
            _ => unreachable!(),
        };
        let oracle = mpl_core_pda(&base, seed.as_ref());
        f.store(
            oracle,
            oracle_account(&rejects(HookableLifecycleEvent::Update), 0),
        );

        let result = f.run(&with_remaining(
            ix::update_v1(
                asset,
                collection,
                actors.payer,
                Some(actors.update_authority),
                None,
                Some("Renamed".to_string()),
                None,
                None,
            ),
            [extra(oracle)],
        ));
        assert_core_err(&result, MplCoreError::InvalidAuthority);
        assert_eq!(
            f.asset(&asset).name,
            "Test Asset",
            "the {name} pda oracle should have rejected the update"
        );
    }
}

/// JS: externalPlugins/oracle.test.ts :: it can use preconfigured recipient pda oracle to deny transfer
#[test]
fn oracle_recipient_pda_derivation_on_transfer() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);
    let base = Pubkey::new_unique();

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![oracle_spec(
            base,
            Some(ExtraAccount::PreconfiguredRecipient {
                is_signer: false,
                is_writable: false,
            }),
            ValidationResultsOffset::NoOffset,
            HookableLifecycleEvent::Transfer,
        )],
    );

    let oracle = mpl_core_pda(&base, Some(&new_owner));
    f.store(
        oracle,
        oracle_account(&rejects(HookableLifecycleEvent::Transfer), 0),
    );

    let result = f.run(&with_remaining(
        ix::transfer_v1(
            asset,
            None,
            actors.payer,
            Some(actors.owner),
            new_owner,
            None,
            None,
        ),
        [extra(oracle)],
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(f.asset(&asset).owner, actors.owner);
}

/// JS: externalPlugins/oracle.test.ts :: it can use custom pda (all seeds) oracle to deny transfer
/// JS: externalPlugins/oracle.test.ts :: it can use custom pda (with custom program ID) oracle to deny transfer
#[test]
fn oracle_custom_pda_derivations_on_transfer() {
    let base = Pubkey::new_unique();
    let custom_program = Pubkey::new_unique();
    let literal = Pubkey::new_unique();

    for custom_program_id in [None, Some(custom_program)] {
        let f = Fixture::new();
        let actors = Actors::new(&f);
        let new_owner = f.fund(ACCOUNT_LAMPORTS);
        let seeds = vec![
            Seed::Bytes(b"prefix".to_vec()),
            Seed::Asset,
            Seed::Owner,
            Seed::Recipient,
            Seed::Address(literal),
        ];

        let asset = store_asset(
            &f,
            actors.owner,
            actors.update_authority,
            vec![oracle_spec(
                base,
                Some(ExtraAccount::CustomPda {
                    seeds: seeds.clone(),
                    custom_program_id,
                    is_signer: false,
                    is_writable: false,
                }),
                ValidationResultsOffset::NoOffset,
                HookableLifecycleEvent::Transfer,
            )],
        );

        let seed_bytes: Vec<Vec<u8>> = vec![
            b"prefix".to_vec(),
            asset.as_ref().to_vec(),
            actors.owner.as_ref().to_vec(),
            new_owner.as_ref().to_vec(),
            literal.as_ref().to_vec(),
        ];
        let slices: Vec<&[u8]> = seed_bytes.iter().map(Vec::as_slice).collect();
        let oracle =
            Pubkey::find_program_address(&slices, custom_program_id.as_ref().unwrap_or(&base)).0;
        f.store(
            oracle,
            oracle_account(&rejects(HookableLifecycleEvent::Transfer), 0),
        );

        let result = f.run(&with_remaining(
            ix::transfer_v1(
                asset,
                None,
                actors.payer,
                Some(actors.owner),
                new_owner,
                None,
                None,
            ),
            [extra(oracle)],
        ));
        assert_core_err(&result, MplCoreError::InvalidAuthority);
        assert_eq!(f.asset(&asset).owner, actors.owner);
    }
}

/// `ExtraAccount::Address` bypasses the derivation entirely.
#[test]
fn oracle_address_extra_account_is_used_verbatim() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);
    let base = Pubkey::new_unique();
    let oracle = Pubkey::new_unique();
    f.store(
        oracle,
        oracle_account(&rejects(HookableLifecycleEvent::Transfer), 0),
    );

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![oracle_spec(
            base,
            Some(ExtraAccount::Address {
                address: oracle,
                is_signer: false,
                is_writable: false,
            }),
            ValidationResultsOffset::NoOffset,
            HookableLifecycleEvent::Transfer,
        )],
    );

    let result = f.run(&with_remaining(
        ix::transfer_v1(
            asset,
            None,
            actors.payer,
            Some(actors.owner),
            new_owner,
            None,
            None,
        ),
        [extra(oracle)],
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
}

/// The derivation error arms: `MissingCollection` when the lifecycle target
/// has no collection, `MissingNewOwner` when the event has no recipient, and
/// `MissingAsset` when the target is a collection.
#[test]
fn oracle_extra_account_derivation_errors() {
    // A `PreconfiguredCollection` config on an asset that is not in a
    // collection.
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let base = Pubkey::new_unique();
    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![oracle_spec(
            base,
            Some(ExtraAccount::PreconfiguredCollection {
                is_signer: false,
                is_writable: false,
            }),
            ValidationResultsOffset::NoOffset,
            HookableLifecycleEvent::Update,
        )],
    );
    let result = f.run(&ix::update_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        Some("Renamed".to_string()),
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::MissingCollection);

    // A `PreconfiguredRecipient` config on an update, which has no recipient.
    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![oracle_spec(
            base,
            Some(ExtraAccount::PreconfiguredRecipient {
                is_signer: false,
                is_writable: false,
            }),
            ValidationResultsOffset::NoOffset,
            HookableLifecycleEvent::Update,
        )],
    );
    let result = f.run(&ix::update_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        Some("Renamed".to_string()),
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::MissingNewOwner);

    // A `PreconfiguredOwner` config on a collection-level update, where
    // `asset_info` is `None`.
    let collection = Pubkey::new_unique();
    f.store(
        collection,
        CollectionSpec::new(actors.update_authority)
            .adapter(oracle_spec(
                base,
                Some(ExtraAccount::PreconfiguredOwner {
                    is_signer: false,
                    is_writable: false,
                }),
                ValidationResultsOffset::NoOffset,
                HookableLifecycleEvent::Update,
            ))
            .build(),
    );
    let result = f.run(&ix::update_collection_v1(
        collection,
        actors.payer,
        Some(actors.update_authority),
        None,
        None,
        Some("Renamed".to_string()),
        None,
    ));
    assert_core_err(&result, MplCoreError::MissingAsset);
}

// ===========================================================================
// appData.test.ts
// ===========================================================================

/// An `AppData` adapter with the given data authority.
fn app_data_adapter(data_authority: Authority) -> ExternalPluginAdapter {
    ExternalPluginAdapter::AppData(AppData {
        data_authority,
        schema: ExternalPluginAdapterSchema::Binary,
    })
}

/// An `AppData` init info with the given data authority.
fn app_data_init(data_authority: Authority) -> ExternalPluginAdapterInitInfo {
    ExternalPluginAdapterInitInfo::AppData(AppDataInitInfo {
        data_authority,
        init_plugin_authority: None,
        schema: None,
    })
}

/// The stored data of an adapter, or `None` when the record carries none.
fn adapter_data(f: &Fixture, key: &Pubkey, adapter: &ExternalPluginAdapterKey) -> Option<Vec<u8>> {
    read_adapter(&f.account(key), adapter)
        .unwrap_or_else(|| panic!("{adapter:?} should be in the registry"))
        .2
}

/// JS: externalPlugins/appData.test.ts :: it can update app data with external plugin authority different than asset update authority
/// JS: externalPlugins/appData.test.ts :: Data offsets are correctly bumped when rewriting other external plugins to be larger
/// JS: externalPlugins/appData.test.ts :: Data offsets are correctly bumped when rewriting other external plugins to be smaller
#[test]
fn app_data_writes_grow_and_shrink_without_corrupting_the_trailing_adapter() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let data_authority = f.fund(ACCOUNT_LAMPORTS);
    let trailing_oracle = Pubkey::new_unique();
    let app_data_key = ExternalPluginAdapterKey::AppData(address(data_authority));
    let oracle_key = ExternalPluginAdapterKey::Oracle(trailing_oracle);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![
            ExternalAdapterSpec::new(app_data_adapter(address(data_authority))),
            oracle_spec(
                trailing_oracle,
                None,
                ValidationResultsOffset::NoOffset,
                HookableLifecycleEvent::Transfer,
            ),
        ],
    );
    assert_eq!(adapter_data(&f, &asset, &app_data_key), Some(vec![]));

    // Grow: 100 bytes have to be made room for, pushing the oracle back.
    let big = vec![0xAB; 100];
    f.run_ok(&ix::write_external_plugin_adapter_data_v1(
        asset,
        None,
        actors.payer,
        Some(data_authority),
        None,
        None,
        client_key(&app_data_key),
        Some(big.clone()),
    ));
    let account = f.account(&asset);
    assert_registry_consistent(&account);
    assert_eq!(adapter_data(&f, &asset, &app_data_key), Some(big));
    let (oracle_record, oracle_adapter_value, _) =
        read_adapter(&account, &oracle_key).expect("the trailing oracle still parses");
    assert_eq!(
        oracle_adapter_value,
        oracle_adapter(trailing_oracle, None, ValidationResultsOffset::NoOffset)
    );
    let grown_oracle_offset = oracle_record.offset;

    // Shrink: 10 bytes pulls the oracle forward again.
    let small = vec![0xCD; 10];
    f.run_ok(&ix::write_external_plugin_adapter_data_v1(
        asset,
        None,
        actors.payer,
        Some(data_authority),
        None,
        None,
        client_key(&app_data_key),
        Some(small.clone()),
    ));
    let account = f.account(&asset);
    assert_registry_consistent(&account);
    assert_eq!(adapter_data(&f, &asset, &app_data_key), Some(small));
    let (oracle_record, oracle_adapter_value, _) =
        read_adapter(&account, &oracle_key).expect("the trailing oracle still parses");
    assert!(oracle_record.offset < grown_oracle_offset);
    assert_eq!(
        oracle_adapter_value,
        oracle_adapter(trailing_oracle, None, ValidationResultsOffset::NoOffset)
    );

    // The trailing oracle is still functional after both moves.
    let new_owner = f.fund(ACCOUNT_LAMPORTS);
    f.store(
        trailing_oracle,
        oracle_account(&rejects(HookableLifecycleEvent::Transfer), 0),
    );
    let result = f.run(&with_remaining(
        ix::transfer_v1(
            asset,
            None,
            actors.payer,
            Some(actors.owner),
            new_owner,
            None,
            None,
        ),
        [extra(trailing_oracle)],
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
}

/// The `buffer` account is the second data source for a write: the bytes come
/// from the account's data instead of the instruction args.
#[test]
fn app_data_can_be_written_from_a_buffer_account() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let data_authority = f.fund(ACCOUNT_LAMPORTS);
    let app_data_key = ExternalPluginAdapterKey::AppData(address(data_authority));
    let buffer = Pubkey::new_unique();
    let bytes = vec![0x11, 0x22, 0x33, 0x44];
    f.store(buffer, buffer_account(bytes.clone()));

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![ExternalAdapterSpec::new(app_data_adapter(address(
            data_authority,
        )))],
    );

    f.run_ok(&ix::write_external_plugin_adapter_data_v1(
        asset,
        None,
        actors.payer,
        Some(data_authority),
        Some(buffer),
        None,
        client_key(&app_data_key),
        None,
    ));
    assert_registry_consistent(&f.account(&asset));
    assert_eq!(adapter_data(&f, &asset, &app_data_key), Some(bytes));
}

/// JS: externalPlugins/appData.test.ts :: it cannot update app data using update authority when different from external plugin authority
///
/// Also covers the two data-source arms: passing both `data` and `buffer` is
/// `TwoDataSources`, passing neither is `NoDataSources`.
#[test]
fn app_data_write_rejects_the_wrong_authority_and_bad_data_sources() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let data_authority = f.fund(ACCOUNT_LAMPORTS);
    let app_data_key = ExternalPluginAdapterKey::AppData(address(data_authority));
    let buffer = Pubkey::new_unique();
    f.store(buffer, buffer_account(vec![1, 2, 3]));

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![ExternalAdapterSpec::new(app_data_adapter(address(
            data_authority,
        )))],
    );

    let result = f.run(&ix::write_external_plugin_adapter_data_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        None,
        client_key(&app_data_key),
        Some(vec![1, 2, 3]),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    let result = f.run(&ix::write_external_plugin_adapter_data_v1(
        asset,
        None,
        actors.payer,
        Some(data_authority),
        Some(buffer),
        None,
        client_key(&app_data_key),
        Some(vec![1, 2, 3]),
    ));
    assert_core_err(&result, MplCoreError::TwoDataSources);

    let result = f.run(&ix::write_external_plugin_adapter_data_v1(
        asset,
        None,
        actors.payer,
        Some(data_authority),
        None,
        None,
        client_key(&app_data_key),
        None,
    ));
    assert_core_err(&result, MplCoreError::NoDataSources);

    assert_eq!(adapter_data(&f, &asset, &app_data_key), Some(vec![]));
}

/// JS: externalPlugins/appData.test.ts :: it can update app data on collection with external plugin authority different than asset update authority
#[test]
fn collection_app_data_can_be_written() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let data_authority = f.fund(ACCOUNT_LAMPORTS);
    let app_data_key = ExternalPluginAdapterKey::AppData(address(data_authority));
    let bytes = vec![0x77; 16];

    let collection = Pubkey::new_unique();
    f.store(
        collection,
        CollectionSpec::new(actors.update_authority)
            .adapter(ExternalAdapterSpec::new(app_data_adapter(address(
                data_authority,
            ))))
            .build(),
    );

    f.run_ok(&ix::write_collection_external_plugin_adapter_data_v1(
        collection,
        actors.payer,
        Some(data_authority),
        None,
        None,
        client_key(&app_data_key),
        Some(bytes.clone()),
    ));
    assert_registry_consistent(&f.account(&collection));
    assert_eq!(adapter_data(&f, &collection, &app_data_key), Some(bytes));
}

/// JS: externalPlugins/appData.test.ts :: Data offsets are correctly bumped when removing other external plugins with data
#[test]
fn removing_an_app_data_with_data_bumps_the_trailing_adapter() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let data_authority = f.fund(ACCOUNT_LAMPORTS);
    let trailing_oracle = Pubkey::new_unique();
    let app_data_key = ExternalPluginAdapterKey::AppData(address(data_authority));
    let oracle_key = ExternalPluginAdapterKey::Oracle(trailing_oracle);
    let payload = vec![0x5A; 48];

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![
            ExternalAdapterSpec::new(app_data_adapter(address(data_authority)))
                .data(payload.clone()),
            oracle_spec(
                trailing_oracle,
                None,
                ValidationResultsOffset::NoOffset,
                HookableLifecycleEvent::Transfer,
            ),
        ],
    );
    assert_eq!(adapter_data(&f, &asset, &app_data_key), Some(payload));
    let len_before = f.account(&asset).data.len();
    let oracle_offset_before = read_adapter(&f.account(&asset), &oracle_key)
        .expect("oracle")
        .0
        .offset;

    f.run_ok(&ix::remove_external_plugin_adapter_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_key(&app_data_key),
    ));
    let account = f.account(&asset);
    assert_registry_consistent(&account);
    assert!(read_adapter(&account, &app_data_key).is_none());
    assert!(
        account.data.len() < len_before,
        "removing the adapter and its data should shrink the account"
    );
    let (oracle_record, oracle_adapter_value, _) =
        read_adapter(&account, &oracle_key).expect("the trailing oracle still parses");
    assert!(oracle_record.offset < oracle_offset_before);
    assert_eq!(
        oracle_adapter_value,
        oracle_adapter(trailing_oracle, None, ValidationResultsOffset::NoOffset)
    );
}

/// JS: externalPlugins/appData.test.ts :: Data offsets are correctly bumped when removing other plugins
///
/// An internal plugin lives before the adapters, so removing it moves the
/// `AppData` record *and* its `data_offset`.
#[test]
fn removing_an_internal_plugin_bumps_the_app_data_offsets() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let data_authority = f.fund(ACCOUNT_LAMPORTS);
    let app_data_key = ExternalPluginAdapterKey::AppData(address(data_authority));
    let payload = vec![0x3C; 24];

    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(actors.owner)
            .update_authority(UpdateAuthority::Address(actors.update_authority))
            .plugin(
                Plugin::Attributes(Attributes {
                    attribute_list: vec![],
                }),
                Authority::UpdateAuthority,
            )
            .adapter(
                ExternalAdapterSpec::new(app_data_adapter(address(data_authority)))
                    .data(payload.clone()),
            )
            .build(),
    );

    f.run_ok(&ix::remove_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        convert(&PluginType::Attributes),
    ));
    let account = f.account(&asset);
    assert_registry_consistent(&account);
    assert_eq!(adapter_data(&f, &asset, &app_data_key), Some(payload));
}

// ===========================================================================
// linkedAppData.test.ts / linkedAppDataMembership.test.ts / dataSection.test.ts
// ===========================================================================

/// A `LinkedAppData` init info with the given data authority.
fn linked_app_data_init(data_authority: Authority) -> ExternalPluginAdapterInitInfo {
    ExternalPluginAdapterInitInfo::LinkedAppData(LinkedAppDataInitInfo {
        data_authority,
        init_plugin_authority: None,
        schema: None,
    })
}

/// JS: externalPlugins/linkedAppDataMembership.test.ts :: LinkedAppData write succeeds for a legitimate collection member
/// JS: externalPlugins/linkedAppData.test.ts :: Data offsets are correctly bumped when rewriting other Data Section to be larger
#[test]
fn a_linked_app_data_write_creates_and_updates_a_data_section_on_the_member() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let data_authority = f.fund(ACCOUNT_LAMPORTS);
    let linked_key = ExternalPluginAdapterKey::LinkedAppData(address(data_authority));
    let section_key = ExternalPluginAdapterKey::DataSection(LinkedDataKey::LinkedAppData(address(
        data_authority,
    )));

    let collection = Pubkey::new_unique();
    f.store(
        collection,
        CollectionSpec::new(actors.update_authority)
            .sizes(1, 1)
            .build(),
    );
    f.run_ok(&ix::add_collection_external_plugin_adapter_v1(
        collection,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_init(&linked_app_data_init(address(data_authority))),
    ));
    assert_registry_consistent(&f.account(&collection));
    assert!(read_adapter(&f.account(&collection), &linked_key).is_some());

    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(actors.owner)
            .update_authority(UpdateAuthority::Collection(collection))
            .build(),
    );

    // The first write creates a `DataSection` on the asset.
    let first = vec![0x01; 8];
    f.run_ok(&ix::write_external_plugin_adapter_data_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(data_authority),
        None,
        None,
        client_key(&linked_key),
        Some(first.clone()),
    ));
    let account = f.account(&asset);
    assert_registry_consistent(&account);
    let (record, adapter, data) =
        read_adapter(&account, &section_key).expect("the DataSection should have been created");
    assert_eq!(
        record.authority,
        Authority::None,
        "a DataSection is always managed internally"
    );
    assert_eq!(
        adapter,
        ExternalPluginAdapter::DataSection(DataSection {
            parent_key: LinkedDataKey::LinkedAppData(address(data_authority)),
            schema: ExternalPluginAdapterSchema::Binary,
        })
    );
    assert_eq!(data, Some(first));

    // The second write goes through the existing section and grows it.
    let second = vec![0x02; 64];
    f.run_ok(&ix::write_external_plugin_adapter_data_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(data_authority),
        None,
        None,
        client_key(&linked_key),
        Some(second.clone()),
    ));
    assert_registry_consistent(&f.account(&asset));
    assert_eq!(adapter_data(&f, &asset, &section_key), Some(second));
}

/// JS: externalPlugins/linkedAppDataMembership.test.ts :: LinkedAppData write is rejected when the asset is not a member of the supplied collection
/// JS: externalPlugins/linkedAppDataMembership.test.ts :: LinkedAppData writes from multiple non-member collections are all rejected
///
/// This is the PR #19 membership check. Both shapes are covered so a
/// refactor cannot silently remove the protection: an asset with an `Address`
/// update authority (no collection at all, where this check is the *only*
/// defence) and an asset that belongs to a different collection (which would
/// otherwise fail later with the same error).
#[test]
fn a_linked_app_data_write_requires_the_asset_to_be_a_member() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let data_authority = f.fund(ACCOUNT_LAMPORTS);
    let linked_key = ExternalPluginAdapterKey::LinkedAppData(address(data_authority));
    let section_key = ExternalPluginAdapterKey::DataSection(LinkedDataKey::LinkedAppData(address(
        data_authority,
    )));

    let linked_spec =
        ExternalAdapterSpec::new(ExternalPluginAdapter::LinkedAppData(LinkedAppData {
            data_authority: address(data_authority),
            schema: ExternalPluginAdapterSchema::Binary,
        }));

    // Shape 1: the asset is in no collection at all.
    let stranger_collection = Pubkey::new_unique();
    f.store(
        stranger_collection,
        CollectionSpec::new(actors.update_authority)
            .adapter(linked_spec.clone())
            .build(),
    );
    let standalone = Pubkey::new_unique();
    f.store(
        standalone,
        AssetSpec::new(actors.owner)
            .update_authority(UpdateAuthority::Address(actors.update_authority))
            .build(),
    );
    let result = f.run(&ix::write_external_plugin_adapter_data_v1(
        standalone,
        Some(stranger_collection),
        actors.payer,
        Some(data_authority),
        None,
        None,
        client_key(&linked_key),
        Some(vec![0xFF; 4]),
    ));
    assert_core_err(&result, MplCoreError::InvalidCollection);
    assert!(
        read_adapter(&f.account(&standalone), &section_key).is_none(),
        "no DataSection may be written onto a non-member asset"
    );

    // Shape 2: the asset belongs to a different collection.
    let real_collection = Pubkey::new_unique();
    f.store(
        real_collection,
        CollectionSpec::new(actors.update_authority)
            .sizes(1, 1)
            .build(),
    );
    let member = Pubkey::new_unique();
    f.store(
        member,
        AssetSpec::new(actors.owner)
            .update_authority(UpdateAuthority::Collection(real_collection))
            .build(),
    );
    let result = f.run(&ix::write_external_plugin_adapter_data_v1(
        member,
        Some(stranger_collection),
        actors.payer,
        Some(data_authority),
        None,
        None,
        client_key(&linked_key),
        Some(vec![0xFF; 4]),
    ));
    assert_core_err(&result, MplCoreError::InvalidCollection);
    assert!(read_adapter(&f.account(&member), &section_key).is_none());
}

/// JS: externalPlugins/linkedAppData.test.ts :: it can update linked app data on collection with external plugin authority different than asset update authority
#[test]
fn linked_app_data_schema_can_be_updated_on_the_collection() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let data_authority = Pubkey::new_unique();
    let linked_key = ExternalPluginAdapterKey::LinkedAppData(address(data_authority));

    let collection = Pubkey::new_unique();
    f.store(
        collection,
        CollectionSpec::new(actors.update_authority)
            .adapter(ExternalAdapterSpec::new(
                ExternalPluginAdapter::LinkedAppData(LinkedAppData {
                    data_authority: address(data_authority),
                    schema: ExternalPluginAdapterSchema::Binary,
                }),
            ))
            .build(),
    );

    f.run_ok(&ix::update_collection_external_plugin_adapter_v1(
        collection,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_key(&linked_key),
        client_update(&ExternalPluginAdapterUpdateInfo::LinkedAppData(
            LinkedAppDataUpdateInfo {
                schema: Some(ExternalPluginAdapterSchema::Json),
            },
        )),
    ));
    let account = f.account(&collection);
    assert_registry_consistent(&account);
    let (_, adapter, _) = read_adapter(&account, &linked_key).expect("linked app data");
    assert_eq!(
        adapter,
        ExternalPluginAdapter::LinkedAppData(LinkedAppData {
            data_authority: address(data_authority),
            schema: ExternalPluginAdapterSchema::Json,
        })
    );
}

/// JS: externalPlugins/linkedAppData.test.ts :: it cannot create an asset with linked app data
/// JS: externalPlugins/linkedAppData.test.ts :: it cannot add linked app data to an asset
/// JS: externalPlugins/dataSection.test.ts :: it cannot create an asset with a DataSection
/// JS: externalPlugins/dataSection.test.ts :: it cannot add a DataSection to an asset
#[test]
fn linked_adapters_and_data_sections_cannot_target_an_asset() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let data_authority = Pubkey::new_unique();

    let data_section_init = ExternalPluginAdapterInitInfo::DataSection(DataSectionInitInfo {
        parent_key: LinkedDataKey::LinkedAppData(address(data_authority)),
        schema: ExternalPluginAdapterSchema::Binary,
    });

    for (init, expected) in [
        (
            linked_app_data_init(address(data_authority)),
            MplCoreError::InvalidPluginAdapterTarget,
        ),
        (
            data_section_init.clone(),
            MplCoreError::CannotAddDataSection,
        ),
    ] {
        let asset = Pubkey::new_unique();
        f.store(asset, empty_account());
        let result = f.run(&ix::create_v2(
            asset,
            None,
            None,
            actors.payer,
            None,
            None,
            None,
            "Test Asset",
            "https://example.com/test",
            None,
            Some(vec![client_init(&init)]),
            &[],
        ));
        assert_core_err(&result, expected);
    }

    let asset = store_asset(&f, actors.owner, actors.update_authority, vec![]);
    for (init, expected) in [
        (
            linked_app_data_init(address(data_authority)),
            MplCoreError::InvalidPluginAdapterTarget,
        ),
        (
            data_section_init.clone(),
            MplCoreError::CannotAddDataSection,
        ),
    ] {
        let result = f.run(&ix::add_external_plugin_adapter_v1(
            asset,
            None,
            actors.payer,
            Some(actors.update_authority),
            None,
            client_init(&init),
        ));
        assert_core_err(&result, expected);
    }

    // A DataSection is refused on a collection too.
    let collection = store_collection(&f, actors.update_authority);
    let result = f.run(&ix::add_collection_external_plugin_adapter_v1(
        collection,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_init(&data_section_init),
    ));
    assert_core_err(&result, MplCoreError::CannotAddDataSection);
}

/// Stores a bare collection controlled by `update_authority`.
fn store_collection(f: &Fixture, update_authority: Pubkey) -> Pubkey {
    let collection = Pubkey::new_unique();
    f.store(collection, CollectionSpec::new(update_authority).build());
    collection
}

// ===========================================================================
// lifecycleHook.test.ts — blocked on-chain
// ===========================================================================

/// The hook adapters are refused by `initialize_external_plugin_adapter`
/// (`plugins/utils.rs`) with `NotAvailable`, after the `From<&InitInfo>`
/// conversions have already run.
#[test]
fn lifecycle_hooks_are_not_available() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let hooked_program = Pubkey::new_unique();

    let hook_init = ExternalPluginAdapterInitInfo::LifecycleHook(LifecycleHookInitInfo {
        hooked_program,
        init_plugin_authority: None,
        lifecycle_checks: vec![check(HookableLifecycleEvent::Transfer, CAN_LISTEN)],
        extra_accounts: None,
        data_authority: Some(Authority::UpdateAuthority),
        schema: None,
    });
    let linked_hook_init =
        ExternalPluginAdapterInitInfo::LinkedLifecycleHook(LinkedLifecycleHookInitInfo {
            hooked_program,
            init_plugin_authority: None,
            lifecycle_checks: vec![check(HookableLifecycleEvent::Transfer, CAN_LISTEN)],
            extra_accounts: None,
            data_authority: Some(Authority::UpdateAuthority),
            schema: None,
        });

    // CreateV2 with a LifecycleHook.
    let asset = Pubkey::new_unique();
    f.store(asset, empty_account());
    let result = f.run(&ix::create_v2(
        asset,
        None,
        None,
        actors.payer,
        None,
        None,
        None,
        "Test Asset",
        "https://example.com/test",
        None,
        Some(vec![client_init(&hook_init)]),
        &[],
    ));
    assert_core_err(&result, MplCoreError::NotAvailable);

    // AddExternalPluginAdapterV1 with a LifecycleHook.
    let asset = store_asset(&f, actors.owner, actors.update_authority, vec![]);
    let result = f.run(&ix::add_external_plugin_adapter_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_init(&hook_init),
    ));
    assert_core_err(&result, MplCoreError::NotAvailable);

    // A LinkedLifecycleHook is only valid on a collection, and blocked there.
    let result = f.run(&ix::add_external_plugin_adapter_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_init(&linked_hook_init),
    ));
    assert_core_err(&result, MplCoreError::InvalidPluginAdapterTarget);

    let collection = store_collection(&f, actors.update_authority);
    let result = f.run(&ix::add_collection_external_plugin_adapter_v1(
        collection,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_init(&linked_hook_init),
    ));
    assert_core_err(&result, MplCoreError::NotAvailable);
}

/// JS: plugins/collection/bubblegumV2.test.ts :: it cannot add external plugin to collection with BubblegumV2 plugin
/// JS: plugins/collection/bubblegumV2.test.ts :: it can add external plugin adapter to asset in BubblegumV2 collection
#[test]
fn bubblegum_v2_blocks_collection_adapters_but_not_member_assets() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let data_authority = Pubkey::new_unique();

    let collection = Pubkey::new_unique();
    f.store(
        collection,
        CollectionSpec::new(actors.update_authority)
            .sizes(1, 1)
            .plugin(
                Plugin::BubblegumV2(BubblegumV2 {}),
                address(mpl_bubblegum::ID),
            )
            .build(),
    );
    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(actors.owner)
            .update_authority(UpdateAuthority::Collection(collection))
            .build(),
    );

    let result = f.run(&ix::add_collection_external_plugin_adapter_v1(
        collection,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_init(&app_data_init(address(data_authority))),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    f.run_ok(&ix::add_external_plugin_adapter_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(actors.update_authority),
        None,
        client_init(&app_data_init(address(data_authority))),
    ));
    let account = f.account(&asset);
    assert_registry_consistent(&account);
    assert!(read_adapter(
        &account,
        &ExternalPluginAdapterKey::AppData(address(data_authority))
    )
    .is_some());
}

// ===========================================================================
// Aggregation, duplicates and crafted state
// ===========================================================================

/// JS: externalPlugins/oracle.test.ts :: it can use one fixed address oracle to deny transfer when a second oracle allows it
#[test]
fn one_denying_oracle_outweighs_an_allowing_one() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);
    let allowing = Pubkey::new_unique();
    let denying = Pubkey::new_unique();
    f.store(allowing, oracle_account(&passes_everything(), 0));
    f.store(
        denying,
        oracle_account(&rejects(HookableLifecycleEvent::Transfer), 0),
    );

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![
            oracle_spec(
                allowing,
                None,
                ValidationResultsOffset::NoOffset,
                HookableLifecycleEvent::Transfer,
            ),
            oracle_spec(
                denying,
                None,
                ValidationResultsOffset::NoOffset,
                HookableLifecycleEvent::Transfer,
            ),
        ],
    );

    let result = f.run(&with_remaining(
        ix::transfer_v1(
            asset,
            None,
            actors.payer,
            Some(actors.owner),
            new_owner,
            None,
            None,
        ),
        [extra(allowing), extra(denying)],
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(f.asset(&asset).owner, actors.owner);
}

/// JS: externalPlugins/oracle.test.ts :: it can add multiple oracles and internal plugins to asset
///
/// Also covers `check_add_external_plugin_adapter` on an asset that carries
/// internal plugins, and the duplicate-key guard in
/// `initialize_external_plugin_adapter`.
#[test]
fn adding_oracles_next_to_internal_plugins_and_rejecting_duplicates() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let first = Pubkey::new_unique();
    let second = Pubkey::new_unique();

    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(actors.owner)
            .update_authority(UpdateAuthority::Address(actors.update_authority))
            .plugin(
                Plugin::Attributes(Attributes {
                    attribute_list: vec![],
                }),
                Authority::UpdateAuthority,
            )
            .build(),
    );

    for oracle in [first, second] {
        f.run_ok(&ix::add_external_plugin_adapter_v1(
            asset,
            None,
            actors.payer,
            Some(actors.update_authority),
            None,
            client_init(&oracle_init(
                oracle,
                vec![check(HookableLifecycleEvent::Transfer, CAN_REJECT)],
            )),
        ));
    }
    let account = f.account(&asset);
    assert_registry_consistent(&account);
    assert!(read_adapter(&account, &ExternalPluginAdapterKey::Oracle(first)).is_some());
    assert!(read_adapter(&account, &ExternalPluginAdapterKey::Oracle(second)).is_some());
    assert!(read_plugin(&account, PluginType::Attributes).is_some());

    // The same base address twice.
    let result = f.run(&ix::add_external_plugin_adapter_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_init(&oracle_init(
            first,
            vec![check(HookableLifecycleEvent::Transfer, CAN_REJECT)],
        )),
    ));
    assert_core_err(&result, MplCoreError::ExternalPluginAdapterAlreadyExists);
}

/// JS: externalPlugins/oracle.test.ts :: it can update oracle with external plugin authority different than asset update authority
#[test]
fn the_adapter_authority_can_update_its_own_oracle() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let oracle = Pubkey::new_unique();
    let plugin_authority = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![oracle_spec(
            oracle,
            None,
            ValidationResultsOffset::NoOffset,
            HookableLifecycleEvent::Transfer,
        )
        .authority(address(plugin_authority))],
    );

    f.run_ok(&ix::update_external_plugin_adapter_v1(
        asset,
        None,
        actors.payer,
        Some(plugin_authority),
        None,
        client_key(&ExternalPluginAdapterKey::Oracle(oracle)),
        client_update(&ExternalPluginAdapterUpdateInfo::Oracle(OracleUpdateInfo {
            lifecycle_checks: None,
            base_address_config: None,
            results_offset: Some(ValidationResultsOffset::Anchor),
        })),
    ));
    let account = f.account(&asset);
    assert_registry_consistent(&account);
    let (record, adapter, _) = read_adapter(&account, &ExternalPluginAdapterKey::Oracle(oracle))
        .expect("the oracle record");
    assert_eq!(record.authority, address(plugin_authority));
    assert_eq!(
        adapter,
        oracle_adapter(oracle, None, ValidationResultsOffset::Anchor)
    );
}

/// A crafted account whose adapter bytes no longer deserialize: the program
/// must surface a clean `DeserializationError` rather than panic. Only
/// reachable from fabricated state, which is what makes it worth pinning for a
/// security fork (roadmap section 11, `ExternalPluginAdapter::load`).
#[test]
fn a_corrupted_adapter_payload_is_a_deserialization_error() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);
    let oracle = Pubkey::new_unique();
    f.store(oracle, oracle_account(&passes_everything(), 0));

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![oracle_spec(
            oracle,
            None,
            ValidationResultsOffset::NoOffset,
            HookableLifecycleEvent::Transfer,
        )],
    );

    // Overwrite the adapter's enum discriminator with an out-of-range value.
    let mut account = f.account(&asset);
    let offset = read_adapter(&account, &ExternalPluginAdapterKey::Oracle(oracle))
        .expect("the oracle record")
        .0
        .offset;
    account.data[offset] = 0xFF;
    f.store(asset, account);

    let result = f.run(&with_remaining(
        ix::transfer_v1(
            asset,
            None,
            actors.payer,
            Some(actors.owner),
            new_owner,
            None,
            None,
        ),
        [extra(oracle)],
    ));
    assert_core_err(&result, MplCoreError::DeserializationError);
}

/// The same for an internal plugin: a `FreezeDelegate` whose `frozen` byte is
/// neither 0 nor 1 fails `Plugin::load` with `DeserializationError`.
#[test]
fn a_corrupted_internal_plugin_payload_is_a_deserialization_error() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);

    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(actors.owner)
            .update_authority(UpdateAuthority::Address(actors.update_authority))
            .plugin(
                Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
                Authority::Owner,
            )
            .build(),
    );

    let mut account = f.account(&asset);
    let (_, parsed) = parse_asset(&account.data);
    let offset = parsed.plugins[0].0.offset;
    // Byte 0 is the `Plugin` discriminator, byte 1 the `frozen` bool.
    account.data[offset + 1] = 0x02;
    f.store(asset, account);

    let result = f.run(&ix::transfer_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        new_owner,
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::DeserializationError);
}

// ===========================================================================
// Pre-existing blocked-adapter state, AgentIdentity hooks and DataSection
// removal
// ===========================================================================

/// The program refuses to *create* a `LifecycleHook`, but it reads one that is
/// already in a registry without complaint: the record is selected for the
/// event, the adapter deserializes, its constant `abstain` runs, and the
/// transfer goes through. It can also still be updated. Nothing on-chain can
/// produce this state today; the test documents what would happen if it did.
#[test]
fn a_pre_existing_lifecycle_hook_record_is_read_and_abstains() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);
    let hooked_program = Pubkey::new_unique();
    let hook_key = ExternalPluginAdapterKey::LifecycleHook(hooked_program);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![
            ExternalAdapterSpec::new(ExternalPluginAdapter::LifecycleHook(
                mpl_core_program::plugins::LifecycleHook {
                    hooked_program,
                    extra_accounts: None,
                    data_authority: Some(Authority::UpdateAuthority),
                    schema: ExternalPluginAdapterSchema::Binary,
                },
            ))
            .lifecycle_checks(vec![check(HookableLifecycleEvent::Transfer, CAN_LISTEN)]),
        ],
    );

    f.run_ok(&ix::transfer_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        new_owner,
        None,
        None,
    ));
    assert_eq!(f.asset(&asset).owner, new_owner);

    f.run_ok(&ix::update_external_plugin_adapter_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_key(&hook_key),
        client_update(&ExternalPluginAdapterUpdateInfo::LifecycleHook(
            mpl_core_program::plugins::LifecycleHookUpdateInfo {
                lifecycle_checks: Some(vec![check(HookableLifecycleEvent::Burn, CAN_LISTEN)]),
                extra_accounts: None,
                schema: Some(ExternalPluginAdapterSchema::Json),
            },
        )),
    ));
    let account = f.account(&asset);
    assert_registry_consistent(&account);
    let (record, adapter, _) = read_adapter(&account, &hook_key).expect("the hook record");
    assert_eq!(
        record.lifecycle_checks.as_deref(),
        Some([check(HookableLifecycleEvent::Burn, CAN_LISTEN)].as_slice()),
        "the LifecycleHook arm of ExternalRegistryRecord::update is honoured"
    );
    assert_eq!(
        adapter,
        ExternalPluginAdapter::LifecycleHook(mpl_core_program::plugins::LifecycleHook {
            hooked_program,
            extra_accounts: None,
            data_authority: Some(Authority::UpdateAuthority),
            schema: ExternalPluginAdapterSchema::Json,
        })
    );
}

/// The `AgentIdentity` adapter abstains from transfer, burn and update when
/// those events are registered, so each instruction succeeds on its own
/// authority. There is no JS counterpart.
#[test]
fn agent_identity_abstains_from_the_transfer_burn_and_update_hooks() {
    let checks = vec![
        check(HookableLifecycleEvent::Transfer, CAN_LISTEN),
        check(HookableLifecycleEvent::Burn, CAN_LISTEN),
        check(HookableLifecycleEvent::Update, CAN_LISTEN),
    ];

    let f = Fixture::new();
    let actors = Actors::new(&f);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);
    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        vec![ExternalAdapterSpec::agent_identity(
            "https://example.com/agent.json",
            checks,
        )],
    );

    f.run_ok(&ix::update_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        Some("Renamed".to_string()),
        None,
        None,
    ));
    assert_eq!(f.asset(&asset).name, "Renamed");

    f.run_ok(&ix::transfer_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        new_owner,
        None,
        None,
    ));
    assert_eq!(f.asset(&asset).owner, new_owner);

    f.run_ok(&ix::burn_v1(
        asset,
        None,
        actors.payer,
        Some(new_owner),
        None,
        None,
    ));
    f.assert_burned(&asset);
}

/// `CreateV2` with an `AgentIdentity` that registers `(Create, can_reject)`
/// is the only way to reach `AgentIdentity::validate_create`: it demands the
/// asset's agent identity PDA as the last account, and as a signer. There is
/// no JS counterpart.
#[test]
fn agent_identity_create_hook_requires_the_identity_pda_to_sign() {
    let checks = vec![check(HookableLifecycleEvent::Create, CAN_REJECT)];
    let init = |checks: Vec<(HookableLifecycleEvent, ExternalCheckResult)>| {
        client_init(&ExternalPluginAdapterInitInfo::AgentIdentity(
            mpl_core_program::plugins::AgentIdentityInitInfo {
                uri: "https://example.com/agent.json".to_string(),
                init_plugin_authority: None,
                lifecycle_checks: checks,
            },
        ))
    };

    let f = Fixture::new();
    let payer = f.fund(ACCOUNT_LAMPORTS);

    // The happy path: the PDA is last and signs.
    let asset = Pubkey::new_unique();
    f.store(asset, empty_account());
    let (pda, _) = agent_identity_pda(&asset);
    f.store(pda, payer_account(ACCOUNT_LAMPORTS));
    f.run_ok(&ix::create_v2(
        asset,
        None,
        None,
        payer,
        None,
        None,
        None,
        "Test Asset",
        "https://example.com/test",
        None,
        Some(vec![init(checks.clone())]),
        &[AccountMeta::new_readonly(pda, true)],
    ));
    assert_registry_consistent(&f.account(&asset));
    let (record, _, _) = read_adapter(&f.account(&asset), &ExternalPluginAdapterKey::AgentIdentity)
        .expect("the agent identity record");
    assert_eq!(record.lifecycle_checks.as_deref(), Some(checks.as_slice()));

    // The same create without the PDA signing.
    let asset = Pubkey::new_unique();
    f.store(asset, empty_account());
    let (pda, _) = agent_identity_pda(&asset);
    f.store(pda, payer_account(ACCOUNT_LAMPORTS));
    let result = f.run(&ix::create_v2(
        asset,
        None,
        None,
        payer,
        None,
        None,
        None,
        "Test Asset",
        "https://example.com/test",
        None,
        Some(vec![init(checks.clone())]),
        &[AccountMeta::new_readonly(pda, false)],
    ));
    // `verify_identity_registry` asserts the signature before it checks the
    // derivation, so an unsigned PDA is a plain `MissingRequiredSignature`.
    assert_program_err(&result, ProgramError::MissingRequiredSignature);

    // ... and with an account that is not the asset's identity PDA.
    let asset = Pubkey::new_unique();
    f.store(asset, empty_account());
    let wrong = Pubkey::new_unique();
    f.store(wrong, payer_account(ACCOUNT_LAMPORTS));
    let result = f.run(&ix::create_v2(
        asset,
        None,
        None,
        payer,
        None,
        None,
        None,
        "Test Asset",
        "https://example.com/test",
        None,
        Some(vec![init(checks)]),
        &[AccountMeta::new_readonly(wrong, true)],
    ));
    assert_core_err(&result, MplCoreError::AgentIdentityMustSign);
}

/// JS: externalPlugins/linkedAppData.test.ts :: Data offsets are correctly bumped when removing Data Section with data
#[test]
fn a_data_section_with_data_can_be_removed_from_the_member_asset() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let data_authority = f.fund(ACCOUNT_LAMPORTS);
    let linked_key = ExternalPluginAdapterKey::LinkedAppData(address(data_authority));
    let section_key = ExternalPluginAdapterKey::DataSection(LinkedDataKey::LinkedAppData(address(
        data_authority,
    )));
    let trailing_oracle = Pubkey::new_unique();
    let oracle_key = ExternalPluginAdapterKey::Oracle(trailing_oracle);

    let collection = Pubkey::new_unique();
    f.store(
        collection,
        CollectionSpec::new(actors.update_authority)
            .sizes(1, 1)
            .adapter(ExternalAdapterSpec::new(
                ExternalPluginAdapter::LinkedAppData(LinkedAppData {
                    data_authority: address(data_authority),
                    schema: ExternalPluginAdapterSchema::Binary,
                }),
            ))
            .build(),
    );
    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(actors.owner)
            .update_authority(UpdateAuthority::Collection(collection))
            .adapter(oracle_spec(
                trailing_oracle,
                None,
                ValidationResultsOffset::NoOffset,
                HookableLifecycleEvent::Burn,
            ))
            .build(),
    );

    f.run_ok(&ix::write_external_plugin_adapter_data_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(data_authority),
        None,
        None,
        client_key(&linked_key),
        Some(vec![0x9A; 40]),
    ));
    let account = f.account(&asset);
    assert_registry_consistent(&account);
    assert!(read_adapter(&account, &section_key).is_some());
    let len_with_section = account.data.len();

    f.run_ok(&ix::remove_external_plugin_adapter_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(actors.update_authority),
        None,
        client_key(&section_key),
    ));
    let account = f.account(&asset);
    assert_registry_consistent(&account);
    assert!(read_adapter(&account, &section_key).is_none());
    assert!(account.data.len() < len_with_section);
    // The trailing oracle survives the removal intact.
    let (_, adapter, _) = read_adapter(&account, &oracle_key).expect("the trailing oracle");
    assert_eq!(
        adapter,
        oracle_adapter(trailing_oracle, None, ValidationResultsOffset::NoOffset)
    );
}
