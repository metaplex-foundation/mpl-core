//! `CreateV1` / `CreateV2` (`src/processor/create.rs`) and
//! `CreateCollectionV1` / `CreateCollectionV2`
//! (`src/processor/create_collection.rs`), plus the dispatcher's rejection of
//! an unknown discriminator (`src/processor/mod.rs`).
//!
//! The existing Mollusk tests only ever created a plugin-less asset with an
//! `AgentIdentity` adapter, so everything here is about the branches around
//! that: creating into a collection, the internal-plugin loop with its
//! create-time validators, and the plugin kinds each of the two processors
//! forbids. The oracle-at-create case (roadmap section 8, test 16) needs the
//! oracle fixture and belongs to the external-adapter section.

use {
    crate::common::{
        accounts::{DEFAULT_ASSET_NAME, DEFAULT_COLLECTION_NAME, DEFAULT_URI},
        *,
    },
    mpl_core::types as client,
    mpl_core_program::{
        error::MplCoreError,
        plugins::{
            AppDataInitInfo, Attribute, Attributes, Autograph, AutographSignature, BubblegumV2,
            Creator, DataSectionInitInfo, Edition, ExternalPluginAdapterInitInfo,
            ExternalPluginAdapterSchema, FreezeDelegate, Groups, LinkedAppDataInitInfo,
            LinkedDataKey, LinkedLifecycleHookInitInfo, MasterEdition, Plugin, PluginType,
            Royalties, RuleSet, UpdateDelegate, VerifiedCreators, VerifiedCreatorsSignature,
        },
        state::{Authority, UpdateAuthority},
    },
    solana_program::{instruction::AccountMeta, program_error::ProgramError, pubkey::Pubkey},
};

/// A `PluginAuthorityPair` for the client, with the default authority.
fn pair(plugin: &Plugin) -> client::PluginAuthorityPair {
    client::PluginAuthorityPair {
        plugin: convert(plugin),
        authority: None,
    }
}

/// A `PluginAuthorityPair` with an explicit authority.
fn pair_with_authority(plugin: &Plugin, authority: Authority) -> client::PluginAuthorityPair {
    client::PluginAuthorityPair {
        plugin: convert(plugin),
        authority: Some(convert(&authority)),
    }
}

fn attributes() -> Plugin {
    Plugin::Attributes(Attributes {
        attribute_list: vec![Attribute {
            key: "k".to_string(),
            value: "v".to_string(),
        }],
    })
}

fn royalties(basis_points: u16, creators: Vec<Creator>) -> Plugin {
    Plugin::Royalties(Royalties {
        basis_points,
        creators,
        rule_set: RuleSet::None,
    })
}

fn creator(address: Pubkey, percentage: u8) -> Creator {
    Creator {
        address,
        percentage,
    }
}

fn valid_royalties(address: Pubkey) -> Plugin {
    royalties(500, vec![creator(address, 100)])
}

fn adapter(init_info: &ExternalPluginAdapterInitInfo) -> client::ExternalPluginAdapterInitInfo {
    convert(init_info)
}

/// A fixture with a funded payer and an empty account for the asset or
/// collection about to be created.
fn setup() -> (Fixture, Pubkey, Pubkey) {
    let f = Fixture::new();
    let payer = f.fund(ACCOUNT_LAMPORTS);
    let key = Pubkey::new_unique();
    f.store(key, empty_account());
    (f, payer, key)
}

/// Asserts the asset holds exactly `plugins` (value and authority) in registry
/// order.
fn assert_plugins(f: &Fixture, asset: &Pubkey, plugins: &[(Plugin, Authority)]) {
    let account = f.account(asset);
    assert_registry_consistent(&account);
    let found: Vec<_> = f
        .parsed(asset)
        .plugins
        .iter()
        .map(|(record, plugin)| (plugin.clone(), record.authority))
        .collect();
    assert_eq!(found, plugins);
}

// ===========================================================================
// Dispatcher
// ===========================================================================

/// `MplAssetInstruction::try_from_slice` rejects a byte that is not a
/// discriminator before any account is touched.
#[test]
fn dispatch_rejects_unknown_discriminator() {
    let f = Fixture::new();
    let payer = f.fund(ACCOUNT_LAMPORTS);
    let result = f.run(&raw_bytes(vec![0xFF], vec![AccountMeta::new(payer, true)]));
    assert_program_err(&result, ProgramError::BorshIoError);
}

// ===========================================================================
// CreateV1 / CreateV2: the plugin-less paths
// ===========================================================================

/// Rust client: create.rs::create_asset_in_account_state
#[test]
fn create_v1_in_account_state() {
    let (f, payer, asset) = setup();

    f.run_ok(&ix::create_v1(
        asset,
        None,
        None,
        payer,
        None,
        None,
        None,
        client::DataState::AccountState,
        DEFAULT_ASSET_NAME,
        DEFAULT_URI,
        None,
    ));

    let created = f.asset(&asset);
    assert_eq!(created.owner, payer);
    assert_eq!(created.update_authority, UpdateAuthority::Address(payer));
    assert_eq!(created.name, DEFAULT_ASSET_NAME);
    assert_eq!(created.uri, DEFAULT_URI);
    assert_eq!(created.seq, None);
    let account = f.account(&asset);
    assert_registry_consistent(&account);
    assert!(
        !f.parsed(&asset).has_meta(),
        "an asset created without plugins has no plugin metadata"
    );
    // The account holds its rent plus the create fee.
    let rent = &f.ctx.mollusk.sysvars.rent;
    let create_fee = rent.minimum_balance(87) + 3600;
    assert_eq!(
        account.lamports,
        rent.minimum_balance(account.data.len()) + create_fee,
        "the asset is funded with rent + the create fee"
    );
}

/// JS: create.test.ts :: it cannot create a new asset in ledger state because it is not available
#[test]
fn create_ledger_state_not_available() {
    let (f, payer, asset) = setup();

    let result = f.run(&ix::create_v2_with_data_state(
        asset,
        None,
        None,
        payer,
        None,
        None,
        None,
        client::DataState::LedgerState,
        DEFAULT_ASSET_NAME,
        DEFAULT_URI,
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::NotAvailable);
    assert!(
        f.account(&asset).data.is_empty(),
        "the rejected create must not have allocated the account"
    );
}

/// JS: create.test.ts :: it cannot use an invalid system program for assets
/// JS: create.test.ts :: it cannot use an invalid noop program assets
#[test]
fn create_rejects_invalid_system_program_and_log_wrapper() {
    let (f, payer, asset) = setup();

    // Accounts: asset, collection, authority, payer, owner, update authority,
    // system program, log wrapper.
    let mut ix = ix::create_v2(
        asset,
        None,
        None,
        payer,
        None,
        None,
        None,
        DEFAULT_ASSET_NAME,
        DEFAULT_URI,
        None,
        None,
        &[],
    );
    set_account(
        &mut ix,
        6,
        AccountMeta::new_readonly(Pubkey::new_unique(), false),
    );
    assert_core_err(&f.run(&ix), MplCoreError::InvalidSystemProgram);

    let ix = ix::create_v2(
        asset,
        None,
        None,
        payer,
        None,
        None,
        Some(Pubkey::new_unique()),
        DEFAULT_ASSET_NAME,
        DEFAULT_URI,
        None,
        None,
        &[],
    );
    assert_core_err(&f.run(&ix), MplCoreError::InvalidLogWrapperProgram);
}

/// JS: create.test.ts :: it cannot create a new asset with an update authority that is not the collection
#[test]
fn create_rejects_conflicting_authority() {
    let (f, payer, asset) = setup();
    let collection = Pubkey::new_unique();
    f.store(collection, CollectionSpec::new(payer).build());

    let result = f.run(&ix::create_v2(
        asset,
        Some(collection),
        None,
        payer,
        None,
        Some(Pubkey::new_unique()),
        None,
        DEFAULT_ASSET_NAME,
        DEFAULT_URI,
        None,
        None,
        &[],
    ));
    assert_core_err(&result, MplCoreError::ConflictingAuthority);
}

/// The system-program CPI fails with `SystemError::AccountAlreadyInUse`
/// (custom code 0, which shares its number with `MplCoreError` variant 0 and
/// so decodes misleadingly in error dumps).
///
/// JS: create.test.ts :: it cannot create a new asset if the address is already in use
/// JS: create.test.ts :: it cannot create a new asset if the address is not owned by the system program
#[test]
fn create_rejects_address_in_use() {
    let f = Fixture::new();
    let payer = f.fund(ACCOUNT_LAMPORTS);

    // An account that already holds lamports cannot be created again.
    let funded = Pubkey::new_unique();
    f.store(funded, payer_account(ACCOUNT_LAMPORTS));
    let result = f.run(&ix::create_v2(
        funded,
        None,
        None,
        payer,
        None,
        None,
        None,
        DEFAULT_ASSET_NAME,
        DEFAULT_URI,
        None,
        None,
        &[],
    ));
    assert_program_err(&result, ProgramError::Custom(0));

    // Neither can one that is already owned by mpl-core.
    let existing = Pubkey::new_unique();
    f.store(existing, AssetSpec::new(payer).build());
    let result = f.run(&ix::create_v2(
        existing,
        None,
        None,
        payer,
        None,
        None,
        None,
        DEFAULT_ASSET_NAME,
        DEFAULT_URI,
        None,
        None,
        &[],
    ));
    assert_program_err(&result, ProgramError::Custom(0));
    assert_eq!(
        read::key_of(&f.account(&existing).data),
        mpl_core_program::state::Key::AssetV1,
        "the existing asset must be untouched"
    );
}

/// The asset account is the address being allocated and must sign.
#[test]
fn create_rejects_non_signer_asset() {
    let (f, payer, asset) = setup();

    let mut ix = ix::create_v2(
        asset,
        None,
        None,
        payer,
        None,
        None,
        None,
        DEFAULT_ASSET_NAME,
        DEFAULT_URI,
        None,
        None,
        &[],
    );
    unsign(&mut ix, &asset);
    assert_program_err(&f.run(&ix), ProgramError::MissingRequiredSignature);
}

/// JS: create.test.ts :: it can create a new asset in account state with a different update authority
#[test]
fn create_with_separate_owner_and_update_authority() {
    let (f, payer, asset) = setup();
    let owner = Pubkey::new_unique();
    let update_authority = Pubkey::new_unique();

    f.run_ok(&ix::create_v2(
        asset,
        None,
        None,
        payer,
        Some(owner),
        Some(update_authority),
        None,
        DEFAULT_ASSET_NAME,
        DEFAULT_URI,
        None,
        None,
        &[],
    ));

    let created = f.asset(&asset);
    assert_eq!(created.owner, owner);
    assert_eq!(
        created.update_authority,
        UpdateAuthority::Address(update_authority)
    );
}

// ===========================================================================
// CreateV2 into a collection
// ===========================================================================

/// JS: createCollection.test.ts :: it can create a new asset with a collection
#[test]
fn create_in_collection_as_collection_authority() {
    let (f, payer, asset) = setup();
    let collection = Pubkey::new_unique();
    f.store(collection, CollectionSpec::new(payer).sizes(2, 1).build());

    f.run_ok(&ix::create_v2(
        asset,
        Some(collection),
        None,
        payer,
        None,
        None,
        None,
        DEFAULT_ASSET_NAME,
        DEFAULT_URI,
        None,
        None,
        &[],
    ));

    assert_eq!(
        f.asset(&asset).update_authority,
        UpdateAuthority::Collection(collection)
    );
    let updated = f.collection(&collection);
    assert_eq!(updated.num_minted, 3);
    assert_eq!(updated.current_size, 2);
}

/// JS: plugins/collection/updateDelegate.test.ts :: it can create a new asset with a collection if it is the collection updateDelegate
/// JS: plugins/collection/updateDelegate.test.ts :: it can create a new asset with a collection if it is a collection updateDelegate additionalDelegate
#[test]
fn create_in_collection_as_update_delegate() {
    for (record_authority, additional) in [
        (Authority::UpdateAuthority, true),
        (
            Authority::Address {
                address: Pubkey::new_unique(),
            },
            false,
        ),
    ] {
        let (f, delegate, asset) = setup();
        let collection_authority = Pubkey::new_unique();
        let collection = Pubkey::new_unique();
        // Either the delegate is an additional delegate of a UA-held record,
        // or it holds the record itself.
        let (plugin, record_authority) = if additional {
            (
                Plugin::UpdateDelegate(UpdateDelegate {
                    additional_delegates: vec![delegate],
                }),
                record_authority,
            )
        } else {
            (
                Plugin::UpdateDelegate(UpdateDelegate {
                    additional_delegates: vec![],
                }),
                Authority::Address { address: delegate },
            )
        };
        f.store(
            collection,
            CollectionSpec::new(collection_authority)
                .plugin(plugin, record_authority)
                .build(),
        );

        f.run_ok(&ix::create_v2(
            asset,
            Some(collection),
            None,
            delegate,
            None,
            None,
            None,
            DEFAULT_ASSET_NAME,
            DEFAULT_URI,
            None,
            None,
            &[],
        ));

        assert_eq!(
            f.asset(&asset).update_authority,
            UpdateAuthority::Collection(collection)
        );
        assert_eq!(f.collection(&collection).current_size, 1);
    }
}

/// JS: createCollection.test.ts :: it cannot create a new asset with a collection if it is not the collection auth
#[test]
fn create_in_collection_rejects_wrong_authority() {
    let (f, payer, asset) = setup();
    let collection = Pubkey::new_unique();
    f.store(
        collection,
        CollectionSpec::new(Pubkey::new_unique()).build(),
    );

    let result = f.run(&ix::create_v2(
        asset,
        Some(collection),
        None,
        payer,
        None,
        None,
        None,
        DEFAULT_ASSET_NAME,
        DEFAULT_URI,
        None,
        None,
        &[],
    ));
    // The roadmap predicted `InvalidAuthority` here (create.rs:176); the
    // program actually reports `NoApprovals`, because nothing in
    // `validate_asset_permissions` approves: the asset abstains (its update
    // authority is the collection) and the collection abstains (the signer is
    // not its update authority). The JS suite asserts the same. See roadmap
    // section 6.
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_eq!(
        f.collection(&collection).num_minted,
        0,
        "the rejected create must not have counted a mint"
    );
}

// ===========================================================================
// CreateV2 with internal plugins
// ===========================================================================

/// JS: create.test.ts :: it can create a new asset in account state with plugins
/// Rust client: create.rs::create_asset_with_plugins
#[test]
fn create_with_internal_plugins() {
    let (f, payer, asset) = setup();
    let delegate = Pubkey::new_unique();

    f.run_ok(&ix::create_v2(
        asset,
        None,
        None,
        payer,
        None,
        None,
        None,
        DEFAULT_ASSET_NAME,
        DEFAULT_URI,
        Some(vec![
            pair(&Plugin::FreezeDelegate(FreezeDelegate { frozen: true })),
            pair_with_authority(&attributes(), Authority::Address { address: delegate }),
            pair(&Plugin::UpdateDelegate(UpdateDelegate {
                additional_delegates: vec![delegate],
            })),
        ]),
        None,
        &[],
    ));

    assert_plugins(
        &f,
        &asset,
        &[
            (
                Plugin::FreezeDelegate(FreezeDelegate { frozen: true }),
                Authority::Owner,
            ),
            (attributes(), Authority::Address { address: delegate }),
            (
                Plugin::UpdateDelegate(UpdateDelegate {
                    additional_delegates: vec![delegate],
                }),
                Authority::UpdateAuthority,
            ),
        ],
    );
}

/// JS: plugins/collection/masterEdition.test.ts :: it cannot create asset with masterEdition
/// JS: groupsPluginBlocking.test.ts :: it cannot create an asset with a Groups plugin
#[test]
fn create_rejects_collection_only_plugins() {
    for forbidden in [
        Plugin::MasterEdition(MasterEdition {
            max_supply: None,
            name: None,
            uri: None,
        }),
        Plugin::BubblegumV2(BubblegumV2 {}),
        Plugin::Groups(Groups { groups: vec![] }),
    ] {
        let (f, payer, asset) = setup();
        let result = f.run(&ix::create_v2(
            asset,
            None,
            None,
            payer,
            None,
            None,
            None,
            DEFAULT_ASSET_NAME,
            DEFAULT_URI,
            Some(vec![pair(&forbidden)]),
            None,
            &[],
        ));
        assert_core_err(&result, MplCoreError::InvalidPlugin);
    }
}

/// `Royalties::validate_create` is the only place royalty data is checked at
/// creation time.
///
/// JS: plugins/asset/royalties.test.ts :: it cannot create royalty basis points greater than 10000
/// JS: plugins/asset/royalties.test.ts :: it cannot create royalty percentages that dont add up to 100
#[test]
fn create_rejects_invalid_royalties() {
    let first = Pubkey::new_unique();
    let invalid = [
        // Basis points above 100%.
        royalties(10_001, vec![creator(first, 100)]),
        // Creator percentages that do not add up to 100.
        royalties(500, vec![creator(first, 50)]),
        // The same creator twice.
        royalties(500, vec![creator(first, 50), creator(first, 50)]),
    ];
    for plugin in invalid {
        let (f, payer, asset) = setup();
        let result = f.run(&ix::create_v2(
            asset,
            None,
            None,
            payer,
            None,
            None,
            None,
            DEFAULT_ASSET_NAME,
            DEFAULT_URI,
            Some(vec![pair(&plugin)]),
            None,
            &[],
        ));
        assert_core_err(&result, MplCoreError::InvalidPluginSetting);
    }

    // The valid shape is accepted.
    let (f, payer, asset) = setup();
    f.run_ok(&ix::create_v2(
        asset,
        None,
        None,
        payer,
        None,
        None,
        None,
        DEFAULT_ASSET_NAME,
        DEFAULT_URI,
        Some(vec![pair(&valid_royalties(first))]),
        None,
        &[],
    ));
    assert_plugins(
        &f,
        &asset,
        &[(valid_royalties(first), Authority::UpdateAuthority)],
    );
}

/// Only the signing authority may add its own autograph at creation.
///
/// JS: plugins/asset/autograph.test.ts :: it cannot create asset with autograph plugin and unauthorized signature
/// JS: plugins/asset/autograph.test.ts :: it can create asset with autograph plugin with authorized signature
#[test]
fn create_autograph_signature_rules() {
    let (f, payer, asset) = setup();
    let signature = |address: Pubkey| {
        Plugin::Autograph(Autograph {
            signatures: vec![AutographSignature {
                address,
                message: "hi".to_string(),
            }],
        })
    };

    // Somebody else's signature.
    let result = f.run(&ix::create_v2(
        asset,
        None,
        None,
        payer,
        None,
        None,
        None,
        DEFAULT_ASSET_NAME,
        DEFAULT_URI,
        Some(vec![pair(&signature(Pubkey::new_unique()))]),
        None,
        &[],
    ));
    assert_core_err(&result, MplCoreError::MissingSigner);

    // The payer's own signature.
    f.run_ok(&ix::create_v2(
        asset,
        None,
        None,
        payer,
        None,
        None,
        None,
        DEFAULT_ASSET_NAME,
        DEFAULT_URI,
        Some(vec![pair(&signature(payer))]),
        None,
        &[],
    ));
    assert_plugins(&f, &asset, &[(signature(payer), Authority::Owner)]);
}

/// A `VerifiedCreators` entry may only be created verified for the signer.
///
/// JS: plugins/asset/verifiedCreators.test.ts :: it cannot create asset with verified creators plugin and unauthorized signature
/// JS: plugins/asset/verifiedCreators.test.ts :: it can create asset with verified creators plugin with authorized signature
#[test]
fn create_verified_creators_signature_rules() {
    let (f, payer, asset) = setup();
    let entry = |address: Pubkey, verified: bool| {
        Plugin::VerifiedCreators(VerifiedCreators {
            signatures: vec![VerifiedCreatorsSignature { address, verified }],
        })
    };

    // Verified for somebody else: the added signature is not the signer's, so
    // `validate_verified_creators_as_plugin_authority` reports `MissingSigner`
    // (the roadmap listed `InvalidPluginOperation`, which is what the *change*
    // and *removal* paths return).
    let result = f.run(&ix::create_v2(
        asset,
        None,
        None,
        payer,
        None,
        None,
        None,
        DEFAULT_ASSET_NAME,
        DEFAULT_URI,
        Some(vec![pair(&entry(Pubkey::new_unique(), true))]),
        None,
        &[],
    ));
    assert_core_err(&result, MplCoreError::MissingSigner);

    // Verified for the signer, plus an unverified third party.
    let plugin = Plugin::VerifiedCreators(VerifiedCreators {
        signatures: vec![
            VerifiedCreatorsSignature {
                address: payer,
                verified: true,
            },
            VerifiedCreatorsSignature {
                address: Pubkey::new_unique(),
                verified: false,
            },
        ],
    });
    f.run_ok(&ix::create_v2(
        asset,
        None,
        None,
        payer,
        None,
        None,
        None,
        DEFAULT_ASSET_NAME,
        DEFAULT_URI,
        Some(vec![pair(&plugin)]),
        None,
        &[],
    ));
    assert_plugins(&f, &asset, &[(plugin, Authority::UpdateAuthority)]);
}

/// Linked adapters live on the collection and a `DataSection` is only ever
/// created by a write, so neither may be initialized on an asset.
///
/// JS: externalPlugins/linkedAppData.test.ts :: it cannot create an asset with linked app data
/// JS: externalPlugins/dataSection.test.ts :: it cannot create an asset with a DataSection
#[test]
fn create_rejects_linked_adapters_and_data_section() {
    let cases = [
        (
            ExternalPluginAdapterInitInfo::LinkedAppData(LinkedAppDataInitInfo {
                data_authority: Authority::UpdateAuthority,
                init_plugin_authority: None,
                schema: Some(ExternalPluginAdapterSchema::Binary),
            }),
            MplCoreError::InvalidPluginAdapterTarget,
        ),
        (
            ExternalPluginAdapterInitInfo::LinkedLifecycleHook(LinkedLifecycleHookInitInfo {
                hooked_program: Pubkey::new_unique(),
                init_plugin_authority: None,
                lifecycle_checks: vec![],
                extra_accounts: None,
                data_authority: None,
                schema: None,
            }),
            MplCoreError::InvalidPluginAdapterTarget,
        ),
        (
            ExternalPluginAdapterInitInfo::DataSection(DataSectionInitInfo {
                parent_key: LinkedDataKey::LinkedAppData(Authority::UpdateAuthority),
                schema: ExternalPluginAdapterSchema::Binary,
            }),
            MplCoreError::CannotAddDataSection,
        ),
    ];

    for (init_info, expected) in cases {
        let (f, payer, asset) = setup();
        let result = f.run(&ix::create_v2(
            asset,
            None,
            None,
            payer,
            None,
            None,
            None,
            DEFAULT_ASSET_NAME,
            DEFAULT_URI,
            None,
            Some(vec![adapter(&init_info)]),
            &[],
        ));
        assert_core_err(&result, expected);
    }
}

// ===========================================================================
// CreateCollectionV1 / CreateCollectionV2
// ===========================================================================

/// Rust client: create_collection.rs::test_create_collection
#[test]
fn create_collection_v1() {
    let (f, payer, collection) = setup();

    f.run_ok(&ix::create_collection_v1(
        collection,
        None,
        payer,
        DEFAULT_COLLECTION_NAME,
        DEFAULT_URI,
        None,
    ));

    let created = f.collection(&collection);
    assert_eq!(created.update_authority, payer);
    assert_eq!(created.name, DEFAULT_COLLECTION_NAME);
    assert_eq!(created.uri, DEFAULT_URI);
    assert_eq!(created.num_minted, 0);
    assert_eq!(created.current_size, 0);
    let account = f.account(&collection);
    assert!(!f.parsed(&collection).has_meta());
    // Unlike assets, collections are created with no extra fee.
    assert_eq!(
        account.lamports,
        f.ctx
            .mollusk
            .sysvars
            .rent
            .minimum_balance(account.data.len()),
        "a collection is funded with exactly its rent"
    );
}

/// JS: createCollection.test.ts :: it cannot use an invalid system program
#[test]
fn create_collection_rejects_invalid_system_program_and_address_in_use() {
    let (f, payer, collection) = setup();

    // Accounts: collection, update authority, payer, system program.
    let mut ix = ix::create_collection_v2(
        collection,
        None,
        payer,
        DEFAULT_COLLECTION_NAME,
        DEFAULT_URI,
        None,
        None,
        &[],
    );
    set_account(
        &mut ix,
        3,
        AccountMeta::new_readonly(Pubkey::new_unique(), false),
    );
    assert_core_err(&f.run(&ix), MplCoreError::InvalidSystemProgram);

    let taken = Pubkey::new_unique();
    f.store(taken, CollectionSpec::new(payer).build());
    let result = f.run(&ix::create_collection_v2(
        taken,
        None,
        payer,
        DEFAULT_COLLECTION_NAME,
        DEFAULT_URI,
        None,
        None,
        &[],
    ));
    // `SystemError::AccountAlreadyInUse`, see `create_rejects_address_in_use`.
    assert_program_err(&result, ProgramError::Custom(0));
}

/// JS: createCollection.test.ts :: it can create a new collection with plugins
/// Rust client: create_collection.rs::create_collection_with_plugins
#[test]
fn create_collection_with_plugins() {
    let (f, payer, collection) = setup();
    let delegate = Pubkey::new_unique();
    let update_delegate = Plugin::UpdateDelegate(UpdateDelegate {
        additional_delegates: vec![delegate],
    });

    f.run_ok(&ix::create_collection_v2(
        collection,
        None,
        payer,
        DEFAULT_COLLECTION_NAME,
        DEFAULT_URI,
        Some(vec![
            pair(&valid_royalties(payer)),
            pair_with_authority(&attributes(), Authority::Address { address: delegate }),
            pair(&update_delegate),
        ]),
        None,
        &[],
    ));

    let account = f.account(&collection);
    assert_registry_consistent(&account);
    let found: Vec<_> = f
        .parsed(&collection)
        .plugins
        .iter()
        .map(|(record, plugin)| (plugin.clone(), record.authority))
        .collect();
    assert_eq!(
        found,
        vec![
            (valid_royalties(payer), Authority::UpdateAuthority),
            (attributes(), Authority::Address { address: delegate }),
            (update_delegate, Authority::UpdateAuthority),
        ]
    );
}

/// JS: createCollection.test.ts :: it cannot create a collection with an owner managed plugin
/// JS: groupsPluginBlocking.test.ts :: it cannot create a collection with a Groups plugin
#[test]
fn create_collection_rejects_owner_managed_and_asset_only_plugins() {
    let cases = [
        (
            Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
            MplCoreError::InvalidAuthority,
        ),
        (
            Plugin::Edition(Edition { number: 1 }),
            MplCoreError::InvalidPlugin,
        ),
        (
            Plugin::Groups(Groups { groups: vec![] }),
            MplCoreError::InvalidPlugin,
        ),
    ];
    for (plugin, expected) in cases {
        let (f, payer, collection) = setup();
        let result = f.run(&ix::create_collection_v2(
            collection,
            None,
            payer,
            DEFAULT_COLLECTION_NAME,
            DEFAULT_URI,
            Some(vec![pair(&plugin)]),
            None,
            &[],
        ));
        assert_core_err(&result, expected);
    }
}

/// A collection with `BubblegumV2` only accepts the plugins on its allow list,
/// takes no external adapters, and pins the plugin's authority.
///
/// JS: plugins/collection/bubblegumV2.test.ts :: it can create collection with BubblegumV2 plugin and other allow-listed plugins
/// JS: plugins/collection/bubblegumV2.test.ts :: it cannot create collection with BubblegumV2 plugin and non-allow-listed plugins
/// JS: plugins/collection/bubblegumV2.test.ts :: it cannot create collection with BubblegumV2 plugin using wrong authority
/// JS: plugins/collection/bubblegumV2.test.ts :: it cannot create collection with BubblegumV2 plugin and external plugin
#[test]
fn create_collection_bubblegum_v2_allow_list() {
    let bubblegum = Plugin::BubblegumV2(BubblegumV2 {});

    // Allow-listed: Royalties (see `BubblegumV2::ALLOW_LIST`).
    let (f, payer, collection) = setup();
    f.run_ok(&ix::create_collection_v2(
        collection,
        None,
        payer,
        DEFAULT_COLLECTION_NAME,
        DEFAULT_URI,
        Some(vec![pair(&bubblegum), pair(&valid_royalties(payer))]),
        None,
        &[],
    ));
    let types: Vec<_> = f
        .parsed(&collection)
        .plugins
        .iter()
        .map(|(record, _)| record.plugin_type)
        .collect();
    assert_eq!(
        types,
        vec![PluginType::BubblegumV2, PluginType::Royalties],
        "both plugins should be in the registry"
    );

    // Not allow-listed: `MasterEdition` is a valid collection plugin but is not
    // in `BubblegumV2::ALLOW_LIST` (`Attributes` is, so it would be accepted).
    let (f, payer, collection) = setup();
    let result = f.run(&ix::create_collection_v2(
        collection,
        None,
        payer,
        DEFAULT_COLLECTION_NAME,
        DEFAULT_URI,
        Some(vec![
            pair(&bubblegum),
            pair(&Plugin::MasterEdition(MasterEdition {
                max_supply: None,
                name: None,
                uri: None,
            })),
        ]),
        None,
        &[],
    ));
    assert_core_err(&result, MplCoreError::BlockedByBubblegumV2);

    // A fixed authority is required.
    let (f, payer, collection) = setup();
    let result = f.run(&ix::create_collection_v2(
        collection,
        None,
        payer,
        DEFAULT_COLLECTION_NAME,
        DEFAULT_URI,
        Some(vec![pair_with_authority(
            &bubblegum,
            Authority::Address { address: payer },
        )]),
        None,
        &[],
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    // External adapters are blocked entirely.
    let (f, payer, collection) = setup();
    let app_data = ExternalPluginAdapterInitInfo::AppData(AppDataInitInfo {
        data_authority: Authority::UpdateAuthority,
        init_plugin_authority: None,
        schema: Some(ExternalPluginAdapterSchema::Binary),
    });
    let result = f.run(&ix::create_collection_v2(
        collection,
        None,
        payer,
        DEFAULT_COLLECTION_NAME,
        DEFAULT_URI,
        Some(vec![pair(&bubblegum)]),
        Some(vec![adapter(&app_data)]),
        &[],
    ));
    assert_core_err(&result, MplCoreError::BlockedByBubblegumV2);
}

/// JS: externalPlugins/dataSection.test.ts :: it cannot create a collection with a DataSection
#[test]
fn create_collection_rejects_data_section() {
    let (f, payer, collection) = setup();
    let data_section = ExternalPluginAdapterInitInfo::DataSection(DataSectionInitInfo {
        parent_key: LinkedDataKey::LinkedAppData(Authority::UpdateAuthority),
        schema: ExternalPluginAdapterSchema::Binary,
    });

    let result = f.run(&ix::create_collection_v2(
        collection,
        None,
        payer,
        DEFAULT_COLLECTION_NAME,
        DEFAULT_URI,
        None,
        Some(vec![adapter(&data_section)]),
        &[],
    ));
    assert_core_err(&result, MplCoreError::CannotAddDataSection);
}

/// The collection update authority may be an account other than the payer.
///
/// JS: createCollection.test.ts :: it can create a new collection
#[test]
fn create_collection_with_separate_update_authority() {
    let (f, payer, collection) = setup();
    let update_authority = Pubkey::new_unique();

    f.run_ok(&ix::create_collection_v2(
        collection,
        Some(update_authority),
        payer,
        DEFAULT_COLLECTION_NAME,
        DEFAULT_URI,
        None,
        None,
        &[],
    ));

    assert_eq!(f.collection(&collection).update_authority, update_authority);
}
