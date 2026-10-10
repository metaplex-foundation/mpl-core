//! Internal plugin management: `AddPluginV1`, `RemovePluginV1`,
//! `UpdatePluginV1`, `ApprovePluginAuthorityV1`, `RevokePluginAuthorityV1`
//! and their five collection counterparts.
//!
//! The nine processor functions share one guard prologue (payer signer,
//! system program, log wrapper, `HashedAssetV1`, `Groups`), so those are
//! covered once with the instruction list in [`instructions`] and asserted
//! for every instruction; the rest of the module covers the paths that differ
//! per instruction.

use {
    crate::common::{assert::describe, read, *},
    mollusk_svm::result::{InstructionResult, ProgramResult},
    mpl_core::types as client,
    mpl_core_program::{
        error::MplCoreError,
        plugins::{
            AddBlocker, Attribute, Attributes, BubblegumV2, Creator, Edition, FreezeDelegate,
            Groups, MasterEdition, PermanentFreezeDelegate, Plugin, PluginType, Royalties, RuleSet,
            TransferDelegate, UpdateDelegate,
        },
        state::{Authority, UpdateAuthority},
    },
    solana_account::Account,
    solana_program::{
        instruction::{AccountMeta, Instruction, InstructionError},
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

// ---------------------------------------------------------------------------
// Plugin values used throughout
// ---------------------------------------------------------------------------

fn freeze(frozen: bool) -> Plugin {
    Plugin::FreezeDelegate(FreezeDelegate { frozen })
}

fn attributes(value: &str) -> Plugin {
    Plugin::Attributes(Attributes {
        attribute_list: vec![Attribute {
            key: "key".to_string(),
            value: value.to_string(),
        }],
    })
}

fn transfer_delegate() -> Plugin {
    Plugin::TransferDelegate(TransferDelegate {})
}

fn update_delegate(additional: &[Pubkey]) -> Plugin {
    Plugin::UpdateDelegate(UpdateDelegate {
        additional_delegates: additional.to_vec(),
    })
}

fn groups_plugin() -> Plugin {
    Plugin::Groups(Groups { groups: vec![] })
}

/// The client-side value of a program plugin.
fn plugin(plugin: &Plugin) -> client::Plugin {
    convert(plugin)
}

/// The client-side value of a plugin type.
fn plugin_type(plugin_type: PluginType) -> client::PluginType {
    convert(&plugin_type)
}

/// The client-side value of an authority.
fn plugin_authority(authority: Authority) -> client::PluginAuthority {
    convert(&authority)
}

fn address(key: Pubkey) -> Authority {
    Authority::Address { address: key }
}

// ---------------------------------------------------------------------------
// Assertions with the case name in the message
// ---------------------------------------------------------------------------

fn expect_core_err(case: &str, result: &InstructionResult, expected: MplCoreError) {
    let code = expected.clone() as u32;
    match &result.program_result {
        ProgramResult::Failure(ProgramError::Custom(actual)) if *actual == code => {}
        other => panic!(
            "{case}: expected MplCoreError::{expected:?}, got {}",
            describe(other)
        ),
    }
}

fn expect_program_err(case: &str, result: &InstructionResult, expected: ProgramError) {
    match &result.program_result {
        ProgramResult::Failure(err) if *err == expected => {}
        other => panic!("{case}: expected {expected:?}, got {}", describe(other)),
    }
}

/// Reads the registry authority and value of `plugin_type`, asserting the
/// layout is consistent first.
fn read(account: &Account, ty: PluginType) -> (Authority, Plugin) {
    assert_registry_consistent(account);
    read_plugin(account, ty).unwrap_or_else(|| panic!("{ty:?} should be in the registry"))
}

/// The plugins of an asset or collection account in registry order.
fn plugins_of(account: &Account) -> Vec<(Plugin, Authority)> {
    assert_registry_consistent(account);
    read::parse_any(&account.data)
        .plugins
        .iter()
        .map(|(record, plugin)| (plugin.clone(), record.authority))
        .collect()
}

// ---------------------------------------------------------------------------
// The nine instructions, for the shared guard tests
// ---------------------------------------------------------------------------

/// One plugin-management instruction as the guard tests need it.
struct Ix {
    /// Name used in assertion messages.
    name: &'static str,
    /// Index of the system-program account in the instruction.
    system_program: usize,
    /// Asset instructions carry the `HashedAssetV1` guard; collection ones do not.
    asset: bool,
    /// Builds the instruction against `target`, paid and signed by `payer`.
    build: fn(Pubkey, Pubkey, Option<Pubkey>) -> Instruction,
    /// The same instruction with a `Groups` plugin or plugin type in the args.
    groups: fn(Pubkey, Pubkey) -> Instruction,
}

fn instructions() -> Vec<Ix> {
    vec![
        Ix {
            name: "AddPluginV1",
            system_program: 4,
            asset: true,
            build: |asset, payer, log_wrapper| {
                ix::add_plugin_v1(
                    asset,
                    None,
                    payer,
                    None,
                    log_wrapper,
                    plugin(&freeze(false)),
                    None,
                )
            },
            groups: |asset, payer| {
                ix::add_plugin_v1(
                    asset,
                    None,
                    payer,
                    None,
                    None,
                    plugin(&groups_plugin()),
                    None,
                )
            },
        },
        Ix {
            name: "AddCollectionPluginV1",
            system_program: 3,
            asset: false,
            build: |collection, payer, log_wrapper| {
                ix::add_collection_plugin_v1(
                    collection,
                    payer,
                    None,
                    log_wrapper,
                    plugin(&update_delegate(&[])),
                    None,
                )
            },
            groups: |collection, payer| {
                ix::add_collection_plugin_v1(
                    collection,
                    payer,
                    None,
                    None,
                    plugin(&groups_plugin()),
                    None,
                )
            },
        },
        Ix {
            name: "RemovePluginV1",
            system_program: 4,
            asset: true,
            build: |asset, payer, log_wrapper| {
                ix::remove_plugin_v1(
                    asset,
                    None,
                    payer,
                    None,
                    log_wrapper,
                    plugin_type(PluginType::Attributes),
                )
            },
            groups: |asset, payer| {
                ix::remove_plugin_v1(
                    asset,
                    None,
                    payer,
                    None,
                    None,
                    plugin_type(PluginType::Groups),
                )
            },
        },
        Ix {
            name: "RemoveCollectionPluginV1",
            system_program: 3,
            asset: false,
            build: |collection, payer, log_wrapper| {
                ix::remove_collection_plugin_v1(
                    collection,
                    payer,
                    None,
                    log_wrapper,
                    plugin_type(PluginType::Attributes),
                )
            },
            groups: |collection, payer| {
                ix::remove_collection_plugin_v1(
                    collection,
                    payer,
                    None,
                    None,
                    plugin_type(PluginType::Groups),
                )
            },
        },
        Ix {
            name: "UpdatePluginV1",
            system_program: 4,
            asset: true,
            build: |asset, payer, log_wrapper| {
                ix::update_plugin_v1(
                    asset,
                    None,
                    payer,
                    None,
                    log_wrapper,
                    plugin(&attributes("new")),
                )
            },
            groups: |asset, payer| {
                ix::update_plugin_v1(asset, None, payer, None, None, plugin(&groups_plugin()))
            },
        },
        Ix {
            name: "UpdateCollectionPluginV1",
            system_program: 3,
            asset: false,
            build: |collection, payer, log_wrapper| {
                ix::update_collection_plugin_v1(
                    collection,
                    payer,
                    None,
                    log_wrapper,
                    plugin(&attributes("new")),
                )
            },
            groups: |collection, payer| {
                ix::update_collection_plugin_v1(
                    collection,
                    payer,
                    None,
                    None,
                    plugin(&groups_plugin()),
                )
            },
        },
        Ix {
            name: "ApprovePluginAuthorityV1",
            system_program: 4,
            asset: true,
            build: |asset, payer, log_wrapper| {
                ix::approve_plugin_authority_v1(
                    asset,
                    None,
                    payer,
                    None,
                    log_wrapper,
                    plugin_type(PluginType::Attributes),
                    plugin_authority(address(Pubkey::new_unique())),
                )
            },
            groups: |asset, payer| {
                ix::approve_plugin_authority_v1(
                    asset,
                    None,
                    payer,
                    None,
                    None,
                    plugin_type(PluginType::Groups),
                    plugin_authority(Authority::UpdateAuthority),
                )
            },
        },
        Ix {
            name: "ApproveCollectionPluginAuthorityV1",
            system_program: 3,
            asset: false,
            build: |collection, payer, log_wrapper| {
                ix::approve_collection_plugin_authority_v1(
                    collection,
                    payer,
                    None,
                    log_wrapper,
                    plugin_type(PluginType::Attributes),
                    plugin_authority(address(Pubkey::new_unique())),
                )
            },
            groups: |collection, payer| {
                ix::approve_collection_plugin_authority_v1(
                    collection,
                    payer,
                    None,
                    None,
                    plugin_type(PluginType::Groups),
                    plugin_authority(Authority::UpdateAuthority),
                )
            },
        },
        Ix {
            name: "RevokePluginAuthorityV1",
            system_program: 4,
            asset: true,
            build: |asset, payer, log_wrapper| {
                ix::revoke_plugin_authority_v1(
                    asset,
                    None,
                    payer,
                    None,
                    log_wrapper,
                    plugin_type(PluginType::Attributes),
                )
            },
            groups: |asset, payer| {
                ix::revoke_plugin_authority_v1(
                    asset,
                    None,
                    payer,
                    None,
                    None,
                    plugin_type(PluginType::Groups),
                )
            },
        },
        Ix {
            name: "RevokeCollectionPluginAuthorityV1",
            system_program: 3,
            asset: false,
            build: |collection, payer, log_wrapper| {
                ix::revoke_collection_plugin_authority_v1(
                    collection,
                    payer,
                    None,
                    log_wrapper,
                    plugin_type(PluginType::Attributes),
                )
            },
            groups: |collection, payer| {
                ix::revoke_collection_plugin_authority_v1(
                    collection,
                    payer,
                    None,
                    None,
                    plugin_type(PluginType::Groups),
                )
            },
        },
    ]
}

/// A fixture holding the asset or collection each guard case acts on, plus a
/// funded payer that is also the owner and update authority.
fn guard_setup(case: &Ix) -> (Fixture, Pubkey, Pubkey) {
    let f = Fixture::new();
    let payer = f.fund(ACCOUNT_LAMPORTS);
    let target = Pubkey::new_unique();
    let account = if case.asset {
        AssetSpec::new(payer)
            .plugin(attributes("v"), Authority::UpdateAuthority)
            .build()
    } else {
        CollectionSpec::new(payer)
            .plugin(attributes("v"), Authority::UpdateAuthority)
            .build()
    };
    f.store(target, account);
    (f, target, payer)
}

// ===========================================================================
// Shared guards
// ===========================================================================

/// JS: addPlugin.test.ts :: it cannot use an invalid system program for assets
#[test]
fn guards_reject_invalid_system_program() {
    for case in instructions() {
        let (f, target, payer) = guard_setup(&case);
        let mut ix = (case.build)(target, payer, None);
        set_account(
            &mut ix,
            case.system_program,
            AccountMeta::new_readonly(Pubkey::new_unique(), false),
        );
        expect_core_err(case.name, &f.run(&ix), MplCoreError::InvalidSystemProgram);
    }
}

/// JS: addPlugin.test.ts :: it cannot use an invalid noop program for assets
#[test]
fn guards_reject_invalid_log_wrapper() {
    for case in instructions() {
        let (f, target, payer) = guard_setup(&case);
        let ix = (case.build)(target, payer, Some(Pubkey::new_unique()));
        expect_core_err(
            case.name,
            &f.run(&ix),
            MplCoreError::InvalidLogWrapperProgram,
        );
    }
}

/// The payer must sign every plugin-management instruction.
#[test]
fn guards_reject_non_signer_payer() {
    for case in instructions() {
        let (f, target, payer) = guard_setup(&case);
        let mut ix = (case.build)(target, payer, None);
        unsign(&mut ix, &payer);
        expect_program_err(
            case.name,
            &f.run(&ix),
            ProgramError::MissingRequiredSignature,
        );
    }
}

/// The five asset instructions refuse to touch a compressed asset. `load_key`
/// reads only byte 0, so a one-byte account is enough to reach the guard.
#[test]
fn asset_instructions_reject_compressed_asset() {
    for case in instructions().into_iter().filter(|case| case.asset) {
        let f = Fixture::new();
        let payer = f.fund(ACCOUNT_LAMPORTS);
        let asset = Pubkey::new_unique();
        f.store(asset, hashed_asset_placeholder());
        let ix = (case.build)(asset, payer, None);
        expect_core_err(case.name, &f.run(&ix), MplCoreError::NotAvailable);
    }
}

/// The `Groups` plugin is reserved for the group instructions and is rejected
/// by all ten generic plugin instructions.
///
/// JS: groupsPluginBlocking.test.ts :: it blocks generic asset plugin operations for Groups
/// JS: groupsPluginBlocking.test.ts :: it blocks generic collection plugin operations for Groups
#[test]
fn groups_plugin_rejected_by_all_instructions() {
    for case in instructions() {
        let (f, target, payer) = guard_setup(&case);
        let ix = (case.groups)(target, payer);
        expect_core_err(case.name, &f.run(&ix), MplCoreError::InvalidPlugin);
    }
}

// ===========================================================================
// AddPluginV1
// ===========================================================================

/// JS: addPlugin.test.ts :: it can add a plugin to an asset
#[test]
fn add_first_plugin_to_bare_asset() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(asset, AssetSpec::new(owner).build());
    let len_before = f.account(&asset).data.len();

    f.run_ok(&ix::add_plugin_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin(&freeze(false)),
        None,
    ));

    let account = f.account(&asset);
    let parsed = parse_asset(&account.data).1;
    assert!(
        parsed.has_meta(),
        "the first plugin must create the header and registry"
    );
    assert_eq!(
        plugins_of(&account),
        vec![(freeze(false), Authority::Owner)]
    );
    assert!(account.data.len() > len_before);
    assert_eq!(
        read_asset(&account).owner,
        owner,
        "the core account is untouched"
    );
}

/// JS: addPlugin.test.ts :: it can add plugin to asset with a plugin
/// JS: addPlugin.test.ts :: it can add a plugin to an asset with a different authority than the default
#[test]
fn add_second_plugin_with_init_authority() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let delegate = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .plugin(freeze(false), Authority::Owner)
            .build(),
    );

    f.run_ok(&ix::add_plugin_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin(&attributes("v")),
        Some(plugin_authority(address(delegate))),
    ));

    let account = f.account(&asset);
    assert_eq!(
        plugins_of(&account),
        vec![
            (freeze(false), Authority::Owner),
            (attributes("v"), address(delegate)),
        ],
        "the new plugin is appended with the requested authority"
    );
}

/// The core only approves the manager of the plugin being added.
///
/// JS: addPlugin.test.ts :: it cannot add authority-managed plugin to an asset by owner
#[test]
fn add_plugin_rejects_wrong_manager() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let update_authority = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Address(update_authority))
            .build(),
    );

    // The owner cannot add an update-authority-managed plugin.
    let result = f.run(&ix::add_plugin_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin(&attributes("v")),
        None,
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);

    // And the update authority cannot add an owner-managed one.
    let result = f.run(&ix::add_plugin_v1(
        asset,
        None,
        update_authority,
        None,
        None,
        plugin(&freeze(false)),
        None,
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert!(
        !f.parsed(&asset).has_meta(),
        "no plugin should have been added"
    );
}

/// JS: addPlugin.test.ts :: it can add a plugin to an asset that is part of a collection
/// JS: addPlugin.test.ts :: it cannot add a plugin to an asset if the collection is wrong
/// JS: addPlugin.test.ts :: it cannot add a plugin to an asset if the collection is missing
#[test]
fn add_plugin_to_asset_in_collection() {
    let f = Fixture::new();
    let authority = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    let other_collection = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(authority)
            .update_authority(UpdateAuthority::Collection(collection))
            .build(),
    );
    f.store(
        collection,
        CollectionSpec::new(authority).sizes(1, 1).build(),
    );
    f.store(
        other_collection,
        CollectionSpec::new(authority).sizes(0, 0).build(),
    );

    // Missing collection account.
    let result = f.run(&ix::add_plugin_v1(
        asset,
        None,
        authority,
        None,
        None,
        plugin(&attributes("v")),
        None,
    ));
    assert_core_err(&result, MplCoreError::MissingCollection);

    // Wrong collection account.
    let result = f.run(&ix::add_plugin_v1(
        asset,
        Some(other_collection),
        authority,
        None,
        None,
        plugin(&attributes("v")),
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidCollection);

    // The collection update authority may add an update-authority-managed plugin.
    f.run_ok(&ix::add_plugin_v1(
        asset,
        Some(collection),
        authority,
        None,
        None,
        plugin(&attributes("v")),
        None,
    ));
    assert_eq!(
        plugins_of(&f.account(&asset)),
        vec![(attributes("v"), Authority::UpdateAuthority)]
    );
}

/// JS: plugins/collection/updateDelegate.test.ts :: it can update a non-updateDelegate plugin on an asset as collection update additional delegate
/// (the add direction: a collection `UpdateDelegate` additional delegate may
/// add an update-authority-managed plugin to a member asset.)
#[test]
fn add_plugin_via_collection_update_delegate() {
    let f = Fixture::new();
    let authority = Pubkey::new_unique();
    let delegate = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(authority)
            .update_authority(UpdateAuthority::Collection(collection))
            .build(),
    );
    f.store(
        collection,
        CollectionSpec::new(authority)
            .sizes(1, 1)
            .plugin(update_delegate(&[delegate]), Authority::UpdateAuthority)
            .build(),
    );

    f.run_ok(&ix::add_plugin_v1(
        asset,
        Some(collection),
        delegate,
        None,
        None,
        plugin(&attributes("v")),
        None,
    ));
    assert_eq!(
        plugins_of(&f.account(&asset)),
        vec![(attributes("v"), Authority::UpdateAuthority)]
    );

    // An owner-managed plugin is still out of reach for the delegate.
    let result = f.run(&ix::add_plugin_v1(
        asset,
        Some(collection),
        delegate,
        None,
        None,
        plugin(&freeze(false)),
        None,
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
}

/// `AddBlocker` vetoes update-authority-managed plugins but not owner-managed
/// ones.
///
/// JS: plugins/asset/addBlocker.test.ts :: it cannot add UA-managed plugin if addBlocker had been added on creation
#[test]
fn add_plugin_rejected_by_add_blocker() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .plugin(
                Plugin::AddBlocker(AddBlocker {}),
                Authority::UpdateAuthority,
            )
            .build(),
    );

    let result = f.run(&ix::add_plugin_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin(&attributes("v")),
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    // Owner-managed plugins are unaffected.
    f.run_ok(&ix::add_plugin_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin(&freeze(false)),
        None,
    ));
    assert_eq!(
        plugins_of(&f.account(&asset)).len(),
        2,
        "the owner-managed plugin should have been added"
    );
}

/// Plugins that only exist at creation time reject their own addition
/// (`add_plugin.rs:76-78`); `MasterEdition` is rejected even earlier, by the
/// plugin-type guard.
///
/// JS: plugins/collection/masterEdition.test.ts :: it cannot add masterEdition to asset
#[test]
fn add_plugin_rejects_creation_only_plugins() {
    let creation_only = [
        Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen: false }),
        Plugin::Edition(Edition { number: 1 }),
        Plugin::BubblegumV2(BubblegumV2 {}),
    ];
    for creation_only_plugin in creation_only {
        let f = Fixture::new();
        let owner = f.fund(ACCOUNT_LAMPORTS);
        let asset = Pubkey::new_unique();
        f.store(asset, AssetSpec::new(owner).build());
        let result = f.run(&ix::add_plugin_v1(
            asset,
            None,
            owner,
            None,
            None,
            plugin(&creation_only_plugin),
            None,
        ));
        expect_core_err(
            &format!("{:?}", PluginType::from(&creation_only_plugin)),
            &result,
            MplCoreError::InvalidAuthority,
        );
    }

    // `MasterEdition` is asset-forbidden and hits the plugin-type guard first.
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(asset, AssetSpec::new(owner).build());
    let result = f.run(&ix::add_plugin_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin(&Plugin::MasterEdition(MasterEdition {
            max_supply: None,
            name: None,
            uri: None,
        })),
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidPlugin);
}

/// JS: plugins/asset/freeze.test.ts :: it cannot add multiple freeze plugins to an asset
#[test]
fn add_plugin_rejects_duplicate() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .plugin(freeze(false), Authority::Owner)
            .build(),
    );

    let result = f.run(&ix::add_plugin_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin(&freeze(true)),
        None,
    ));
    assert_core_err(&result, MplCoreError::PluginAlreadyExists);
    assert_eq!(
        plugins_of(&f.account(&asset)),
        vec![(freeze(false), Authority::Owner)],
        "the existing plugin must be untouched"
    );
}

/// JS: addPlugin.test.ts :: it can add a plugin to a collection
/// JS: addPlugin.test.ts :: it can add a plugin to a collection with a plugin
#[test]
fn add_collection_plugin_first_and_second() {
    let f = Fixture::new();
    let authority = f.fund(ACCOUNT_LAMPORTS);
    let collection = Pubkey::new_unique();
    f.store(collection, CollectionSpec::new(authority).build());

    f.run_ok(&ix::add_collection_plugin_v1(
        collection,
        authority,
        None,
        None,
        plugin(&attributes("v")),
        None,
    ));
    assert_eq!(
        plugins_of(&f.account(&collection)),
        vec![(attributes("v"), Authority::UpdateAuthority)]
    );

    f.run_ok(&ix::add_collection_plugin_v1(
        collection,
        authority,
        None,
        None,
        plugin(&update_delegate(&[])),
        None,
    ));
    assert_eq!(
        plugins_of(&f.account(&collection)),
        vec![
            (attributes("v"), Authority::UpdateAuthority),
            (update_delegate(&[]), Authority::UpdateAuthority),
        ]
    );
    assert_eq!(
        f.collection(&collection).update_authority,
        authority,
        "the core account is untouched"
    );
}

/// JS: addPlugin.test.ts :: it cannot add an owner-managed plugin to a collection
#[test]
fn add_collection_plugin_rejects_owner_managed() {
    let f = Fixture::new();
    let authority = f.fund(ACCOUNT_LAMPORTS);
    let collection = Pubkey::new_unique();
    f.store(collection, CollectionSpec::new(authority).build());

    let result = f.run(&ix::add_collection_plugin_v1(
        collection,
        authority,
        None,
        None,
        plugin(&freeze(false)),
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
}

/// Only the collection update authority (or a delegate) may add a plugin.
#[test]
fn add_collection_plugin_rejects_wrong_signer() {
    let f = Fixture::new();
    let authority = Pubkey::new_unique();
    let stranger = f.fund(ACCOUNT_LAMPORTS);
    let collection = Pubkey::new_unique();
    f.store(collection, CollectionSpec::new(authority).build());

    let result = f.run(&ix::add_collection_plugin_v1(
        collection,
        stranger,
        None,
        None,
        plugin(&attributes("v")),
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert!(!f.parsed(&collection).has_meta());
}

// ===========================================================================
// RemovePluginV1
// ===========================================================================

/// A bare asset has no registry at all; an asset with other plugins has one
/// that does not carry the requested type. Both report `PluginNotFound`.
#[test]
fn remove_plugin_not_found() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let bare = Pubkey::new_unique();
    let with_other = Pubkey::new_unique();
    f.store(bare, AssetSpec::new(owner).build());
    f.store(
        with_other,
        AssetSpec::new(owner)
            .plugin(attributes("v"), Authority::UpdateAuthority)
            .build(),
    );

    for asset in [bare, with_other] {
        let result = f.run(&ix::remove_plugin_v1(
            asset,
            None,
            owner,
            None,
            None,
            plugin_type(PluginType::FreezeDelegate),
        ));
        assert_core_err(&result, MplCoreError::PluginNotFound);
    }
}

/// JS: removePlugin.test.ts :: it can remove a plugin from an asset
/// JS: removePlugin.test.ts :: it cannot remove an owner plugin from an asset if not the owner
#[test]
fn remove_plugin_owner_and_update_authority() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let update_authority = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Address(update_authority))
            .plugin(freeze(false), Authority::Owner)
            .plugin(attributes("v"), Authority::UpdateAuthority)
            .build(),
    );

    // The update authority cannot remove the owner-managed plugin.
    let result = f.run(&ix::remove_plugin_v1(
        asset,
        None,
        update_authority,
        None,
        None,
        plugin_type(PluginType::FreezeDelegate),
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);

    // The owner can.
    f.run_ok(&ix::remove_plugin_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin_type(PluginType::FreezeDelegate),
    ));
    assert_eq!(
        plugins_of(&f.account(&asset)),
        vec![(attributes("v"), Authority::UpdateAuthority)]
    );

    // And the update authority removes the update-authority-managed one.
    f.run_ok(&ix::remove_plugin_v1(
        asset,
        None,
        update_authority,
        None,
        None,
        plugin_type(PluginType::Attributes),
    ));
    let account = f.account(&asset);
    assert!(plugins_of(&account).is_empty());
    assert!(
        parse_asset(&account.data).1.has_meta(),
        "removing the last plugin leaves an empty header and registry"
    );
}

/// Removing a plugin that is not last must memmove the trailing plugins and
/// bump their registry offsets (`plugins/utils.rs::delete_plugin`).
///
/// JS: removePlugin.test.ts :: it can remove a plugin from asset with existing plugins
#[test]
fn remove_middle_plugin_bumps_trailing_offsets() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .plugin(
                attributes("a long attribute value"),
                Authority::UpdateAuthority,
            )
            .plugin(freeze(false), Authority::Owner)
            .plugin(transfer_delegate(), Authority::Owner)
            .build(),
    );
    let before = f.account(&asset);
    let attributes_len = {
        let parsed = parse_asset(&before.data).1;
        let records = &parsed.registry.as_ref().unwrap().registry;
        records[1].offset - records[0].offset
    };
    let freeze_offset_before = parse_asset(&before.data).1.registry.unwrap().registry[1].offset;

    f.run_ok(&ix::remove_plugin_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin_type(PluginType::Attributes),
    ));

    let account = f.account(&asset);
    assert_eq!(
        plugins_of(&account),
        vec![
            (freeze(false), Authority::Owner),
            (transfer_delegate(), Authority::Owner),
        ],
        "the trailing plugins must survive the memmove with their values intact"
    );
    let registry_before = parse_asset(&before.data).1.registry.unwrap();
    let registry_after = parse_asset(&account.data).1.registry.unwrap();
    assert_eq!(
        registry_after.registry[0].offset,
        freeze_offset_before - attributes_len,
        "the trailing records must be bumped by the removed plugin's size"
    );
    // The account loses the plugin bytes and the registry record that pointed
    // at them.
    let record_len = borsh::to_vec(&registry_before).unwrap().len()
        - borsh::to_vec(&registry_after).unwrap().len();
    assert_eq!(
        account.data.len(),
        before.data.len() - attributes_len - record_len
    );
}

/// A frozen asset, a frozen collection and an `Authority::None` record all
/// block removal.
///
/// JS: removePlugin.test.ts :: it cannot remove a plugin from a frozen asset
/// JS: removePlugin.test.ts :: it cannot remove a plugin from an asset with a frozen collection
/// JS: removePlugin.test.ts :: it cannot remove an authority managed plugin when the authority is None
#[test]
fn remove_plugin_rejected_when_frozen_or_authority_none() {
    // Frozen asset.
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let frozen = Pubkey::new_unique();
    f.store(
        frozen,
        AssetSpec::new(owner)
            .plugin(freeze(true), Authority::Owner)
            .build(),
    );
    let result = f.run(&ix::remove_plugin_v1(
        frozen,
        None,
        owner,
        None,
        None,
        plugin_type(PluginType::FreezeDelegate),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    // Authority::None on the record.
    let immutable = Pubkey::new_unique();
    f.store(
        immutable,
        AssetSpec::new(owner)
            .plugin(attributes("v"), Authority::None)
            .build(),
    );
    let result = f.run(&ix::remove_plugin_v1(
        immutable,
        None,
        owner,
        None,
        None,
        plugin_type(PluginType::Attributes),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    // Frozen collection.
    let member = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    f.store(
        member,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Collection(collection))
            .plugin(attributes("v"), Authority::UpdateAuthority)
            .build(),
    );
    f.store(
        collection,
        CollectionSpec::new(owner)
            .sizes(1, 1)
            .plugin(
                Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen: true }),
                Authority::UpdateAuthority,
            )
            .build(),
    );
    let result = f.run(&ix::remove_plugin_v1(
        member,
        Some(collection),
        owner,
        None,
        None,
        plugin_type(PluginType::Attributes),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
}

/// JS: removePlugin.test.ts :: it can remove authority managed plugin from collection
#[test]
fn remove_collection_plugin() {
    let f = Fixture::new();
    let authority = f.fund(ACCOUNT_LAMPORTS);
    let bare = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    f.store(bare, CollectionSpec::new(authority).build());
    f.store(
        collection,
        CollectionSpec::new(authority)
            .plugin(attributes("v"), Authority::UpdateAuthority)
            .plugin(update_delegate(&[]), Authority::UpdateAuthority)
            .build(),
    );

    // No plugin metadata at all.
    let result = f.run(&ix::remove_collection_plugin_v1(
        bare,
        authority,
        None,
        None,
        plugin_type(PluginType::Attributes),
    ));
    assert_core_err(&result, MplCoreError::PluginNotFound);

    // A stranger cannot remove.
    let stranger = f.fund(ACCOUNT_LAMPORTS);
    let result = f.run(&ix::remove_collection_plugin_v1(
        collection,
        stranger,
        None,
        None,
        plugin_type(PluginType::Attributes),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    // The update authority can, and the trailing plugin survives.
    f.run_ok(&ix::remove_collection_plugin_v1(
        collection,
        authority,
        None,
        None,
        plugin_type(PluginType::Attributes),
    ));
    assert_eq!(
        plugins_of(&f.account(&collection)),
        vec![(update_delegate(&[]), Authority::UpdateAuthority)]
    );
}

/// JS: plugins/collection/permanentFreeze.test.ts :: it cannot remove permanentFreezeDelegate from collection when frozen
#[test]
fn remove_collection_plugin_rejected_when_frozen() {
    let f = Fixture::new();
    let authority = f.fund(ACCOUNT_LAMPORTS);
    let collection = Pubkey::new_unique();
    f.store(
        collection,
        CollectionSpec::new(authority)
            .plugin(
                Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen: true }),
                Authority::UpdateAuthority,
            )
            .build(),
    );

    let result = f.run(&ix::remove_collection_plugin_v1(
        collection,
        authority,
        None,
        None,
        plugin_type(PluginType::PermanentFreezeDelegate),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(plugins_of(&f.account(&collection)).len(), 1);
}

// ===========================================================================
// UpdatePluginV1
// ===========================================================================

/// The plugin has to exist before it can be updated.
#[test]
fn update_plugin_not_found() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .plugin(attributes("v"), Authority::UpdateAuthority)
            .build(),
    );

    let result = f.run(&ix::update_plugin_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin(&freeze(true)),
    ));
    assert_core_err(&result, MplCoreError::PluginNotFound);
}

/// Only the plugin's own registry authority may update it: the core always
/// abstains for `UpdatePluginV1`.
///
/// JS: plugins/asset/freeze.test.ts :: it can freeze and unfreeze an asset
#[test]
fn update_plugin_freeze_by_owner_only() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let update_authority = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Address(update_authority))
            .plugin(freeze(false), Authority::Owner)
            .build(),
    );

    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin(&freeze(true)),
    ));
    assert_eq!(
        read(&f.account(&asset), PluginType::FreezeDelegate),
        (Authority::Owner, freeze(true))
    );

    // The update authority cannot unfreeze an owner-managed plugin.
    let result = f.run(&ix::update_plugin_v1(
        asset,
        None,
        update_authority,
        None,
        None,
        plugin(&freeze(false)),
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);

    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin(&freeze(false)),
    ));
    assert_eq!(
        read(&f.account(&asset), PluginType::FreezeDelegate).1,
        freeze(false)
    );
}

/// Growing and shrinking a plugin moves everything behind it.
///
/// Rust client: plugin_shrink_corruption.rs::test_update_plugin_shrink_attributes_preserves_trailing_plugins
#[test]
fn update_plugin_grow_and_shrink_with_trailing_plugin() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .plugin(attributes("small"), Authority::UpdateAuthority)
            .plugin(freeze(true), Authority::Owner)
            .build(),
    );
    let len_before = f.account(&asset).data.len();

    // Grow.
    let long = "a substantially longer attribute value than before";
    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin(&attributes(long)),
    ));
    let grown = f.account(&asset);
    assert_eq!(
        plugins_of(&grown),
        vec![
            (attributes(long), Authority::UpdateAuthority),
            (freeze(true), Authority::Owner),
        ]
    );
    assert_eq!(grown.data.len(), len_before + long.len() - "small".len());

    // Shrink.
    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin(&attributes("s")),
    ));
    let shrunk = f.account(&asset);
    assert_eq!(
        plugins_of(&shrunk),
        vec![
            (attributes("s"), Authority::UpdateAuthority),
            (freeze(true), Authority::Owner),
        ],
        "the trailing FreezeDelegate must keep its frozen flag through the shrink"
    );
    assert_eq!(shrunk.data.len(), len_before - "small".len() + 1);

    // Same size: neither realloc branch runs.
    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin(&attributes("t")),
    ));
    let same = f.account(&asset);
    assert_eq!(same.data.len(), shrunk.data.len());
    assert_eq!(
        plugins_of(&same),
        vec![
            (attributes("t"), Authority::UpdateAuthority),
            (freeze(true), Authority::Owner),
        ]
    );
}

/// A delegate holding the plugin's registry authority may update it, and an
/// `UpdateDelegate` additional delegate may update other update-authority
/// managed plugins.
///
/// JS: plugins/asset/updateDelegate.test.ts :: it can update a non-updateDelegate plugin as additional delegate
#[test]
fn update_plugin_by_delegates() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let delegate = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .plugin(freeze(false), address(delegate))
            .plugin(update_delegate(&[delegate]), Authority::UpdateAuthority)
            .plugin(attributes("v"), Authority::UpdateAuthority)
            .build(),
    );

    // The record authority updates its own plugin.
    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        delegate,
        None,
        None,
        plugin(&freeze(true)),
    ));
    assert_eq!(
        read(&f.account(&asset), PluginType::FreezeDelegate),
        (address(delegate), freeze(true))
    );

    // As an additional update delegate it may also update the Attributes.
    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        delegate,
        None,
        None,
        plugin(&attributes("updated")),
    ));
    assert_eq!(
        read(&f.account(&asset), PluginType::Attributes).1,
        attributes("updated")
    );
}

/// The `CollectionV1` monomorphization of `process_update_plugin`, plus the
/// two error paths of the collection variant.
#[test]
fn update_collection_plugin_by_authority_and_stranger() {
    let f = Fixture::new();
    let authority = f.fund(ACCOUNT_LAMPORTS);
    let stranger = f.fund(ACCOUNT_LAMPORTS);
    let collection = Pubkey::new_unique();
    f.store(
        collection,
        CollectionSpec::new(authority)
            .plugin(attributes("v"), Authority::UpdateAuthority)
            .plugin(update_delegate(&[]), Authority::UpdateAuthority)
            .build(),
    );

    f.run_ok(&ix::update_collection_plugin_v1(
        collection,
        authority,
        None,
        None,
        plugin(&attributes("a much longer value")),
    ));
    assert_eq!(
        plugins_of(&f.account(&collection)),
        vec![
            (
                attributes("a much longer value"),
                Authority::UpdateAuthority
            ),
            (update_delegate(&[]), Authority::UpdateAuthority),
        ]
    );

    let result = f.run(&ix::update_collection_plugin_v1(
        collection,
        stranger,
        None,
        None,
        plugin(&attributes("hijacked")),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    let result = f.run(&ix::update_collection_plugin_v1(
        collection,
        authority,
        None,
        None,
        plugin(&freeze(true)),
    ));
    assert_core_err(&result, MplCoreError::PluginNotFound);
}

// ===========================================================================
// ApprovePluginAuthorityV1
// ===========================================================================

/// JS: approveAuthority.test.ts :: it can add an authority to a plugin
#[test]
fn approve_plugin_authority_on_asset() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let update_authority = f.fund(ACCOUNT_LAMPORTS);
    let delegate = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Address(update_authority))
            .plugin(freeze(false), Authority::Owner)
            .plugin(attributes("v"), Authority::UpdateAuthority)
            .build(),
    );
    let len_before = f.account(&asset).data.len();

    // The owner delegates the owner-managed plugin: the record grows from a
    // one-byte authority to a 33-byte one.
    f.run_ok(&ix::approve_plugin_authority_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin_type(PluginType::FreezeDelegate),
        plugin_authority(address(delegate)),
    ));
    let account = f.account(&asset);
    assert_eq!(
        read(&account, PluginType::FreezeDelegate),
        (address(delegate), freeze(false))
    );
    assert_eq!(
        account.data.len(),
        len_before + 32,
        "Address(..) is 32 bytes larger than the default authority"
    );

    // The update authority delegates the update-authority-managed plugin.
    let other = Pubkey::new_unique();
    f.run_ok(&ix::approve_plugin_authority_v1(
        asset,
        None,
        update_authority,
        None,
        None,
        plugin_type(PluginType::Attributes),
        plugin_authority(address(other)),
    ));
    assert_eq!(
        read(&f.account(&asset), PluginType::Attributes).0,
        address(other)
    );
}

/// Approving `Authority::None` makes the plugin immutable and does not change
/// the account size.
///
/// JS: revokeAuthority.test.ts :: it can remove the default authority from a plugin to make it immutable
#[test]
fn approve_plugin_authority_none_keeps_size() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .plugin(freeze(false), Authority::Owner)
            .build(),
    );
    let len_before = f.account(&asset).data.len();

    f.run_ok(&ix::approve_plugin_authority_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin_type(PluginType::FreezeDelegate),
        plugin_authority(Authority::None),
    ));

    let account = f.account(&asset);
    assert_eq!(
        read(&account, PluginType::FreezeDelegate).0,
        Authority::None
    );
    assert_eq!(
        account.data.len(),
        len_before,
        "None and Owner are both one byte, so no realloc happens"
    );
}

/// JS: approveAuthority.test.ts :: it cannot reassign authority of a plugin while already delegated
#[test]
fn approve_plugin_authority_errors() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let delegate = Pubkey::new_unique();

    // Already delegated.
    let delegated = Pubkey::new_unique();
    f.store(
        delegated,
        AssetSpec::new(owner)
            .plugin(freeze(false), address(delegate))
            .build(),
    );
    let result = f.run(&ix::approve_plugin_authority_v1(
        delegated,
        None,
        owner,
        None,
        None,
        plugin_type(PluginType::FreezeDelegate),
        plugin_authority(address(Pubkey::new_unique())),
    ));
    assert_core_err(&result, MplCoreError::CannotRedelegate);

    // Frozen.
    let frozen = Pubkey::new_unique();
    f.store(
        frozen,
        AssetSpec::new(owner)
            .plugin(freeze(true), Authority::Owner)
            .build(),
    );
    let result = f.run(&ix::approve_plugin_authority_v1(
        frozen,
        None,
        owner,
        None,
        None,
        plugin_type(PluginType::FreezeDelegate),
        plugin_authority(address(delegate)),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    // No such plugin: `fetch_wrapped_plugin` is called with `None`, so a bare
    // asset gives a clean error here (unlike revoke, see
    // `revoke_plugin_authority_on_bare_asset_panics`).
    let bare = Pubkey::new_unique();
    f.store(bare, AssetSpec::new(owner).build());
    let result = f.run(&ix::approve_plugin_authority_v1(
        bare,
        None,
        owner,
        None,
        None,
        plugin_type(PluginType::FreezeDelegate),
        plugin_authority(address(delegate)),
    ));
    assert_core_err(&result, MplCoreError::PluginNotFound);
}

/// JS: plugins/collection/updateDelegate.test.ts :: it can add updateDelegate to collection and then approve
#[test]
fn approve_collection_plugin_authority() {
    let f = Fixture::new();
    let authority = f.fund(ACCOUNT_LAMPORTS);
    let stranger = f.fund(ACCOUNT_LAMPORTS);
    let delegate = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    f.store(
        collection,
        CollectionSpec::new(authority)
            .plugin(update_delegate(&[]), Authority::UpdateAuthority)
            .build(),
    );

    let result = f.run(&ix::approve_collection_plugin_authority_v1(
        collection,
        stranger,
        None,
        None,
        plugin_type(PluginType::UpdateDelegate),
        plugin_authority(address(delegate)),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    f.run_ok(&ix::approve_collection_plugin_authority_v1(
        collection,
        authority,
        None,
        None,
        plugin_type(PluginType::UpdateDelegate),
        plugin_authority(address(delegate)),
    ));
    assert_eq!(
        read(&f.account(&collection), PluginType::UpdateDelegate).0,
        address(delegate)
    );

    // A second approval is a redelegation.
    let result = f.run(&ix::approve_collection_plugin_authority_v1(
        collection,
        authority,
        None,
        None,
        plugin_type(PluginType::UpdateDelegate),
        plugin_authority(address(Pubkey::new_unique())),
    ));
    assert_core_err(&result, MplCoreError::CannotRedelegate);
}

// ===========================================================================
// RevokePluginAuthorityV1
// ===========================================================================

/// JS: revokeAuthority.test.ts :: it can remove an authority from a plugin
/// JS: revokeAuthority.test.ts :: it can remove a pubkey authority from an owner-managed plugin if that pubkey is the signer authority
#[test]
fn revoke_plugin_authority_on_asset() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let update_authority = f.fund(ACCOUNT_LAMPORTS);
    let freeze_delegate = f.fund(ACCOUNT_LAMPORTS);
    let attributes_delegate = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Address(update_authority))
            .plugin(freeze(false), address(freeze_delegate))
            .plugin(attributes("v"), address(attributes_delegate))
            .build(),
    );
    let len_before = f.account(&asset).data.len();
    let payer_before = f.lamports(&owner);

    // The owner revokes the delegate of the owner-managed plugin; the freed
    // rent goes back to the payer.
    f.run_ok(&ix::revoke_plugin_authority_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin_type(PluginType::FreezeDelegate),
    ));
    let account = f.account(&asset);
    assert_eq!(
        read(&account, PluginType::FreezeDelegate),
        (Authority::Owner, freeze(false)),
        "the authority falls back to the plugin's manager"
    );
    assert_eq!(account.data.len(), len_before - 32);
    assert!(
        f.lamports(&owner) > payer_before,
        "the payer should have received the freed rent"
    );

    // A delegate revoking itself is paid by the asset, so the lamports stay
    // locked in the asset account (roadmap section 9, finding 5).
    let asset_lamports_before = f.lamports(&asset);
    let delegate_lamports_before = f.lamports(&attributes_delegate);
    f.run_ok(&ix::revoke_plugin_authority_v1(
        asset,
        None,
        attributes_delegate,
        None,
        None,
        plugin_type(PluginType::Attributes),
    ));
    let account = f.account(&asset);
    assert_eq!(
        read(&account, PluginType::Attributes).0,
        Authority::UpdateAuthority
    );
    assert_eq!(
        f.lamports(&asset),
        asset_lamports_before,
        "the asset pays itself, so its balance does not change"
    );
    assert_eq!(
        f.lamports(&attributes_delegate),
        delegate_lamports_before,
        "the signer is not refunded when it is not the plugin manager"
    );
}

/// JS: revokeAuthority.test.ts :: it cannot remove a none authority from a plugin
#[test]
fn revoke_plugin_authority_errors() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);

    // `Authority::None` cannot be revoked.
    let immutable = Pubkey::new_unique();
    f.store(
        immutable,
        AssetSpec::new(owner)
            .plugin(attributes("v"), Authority::None)
            .build(),
    );
    let result = f.run(&ix::revoke_plugin_authority_v1(
        immutable,
        None,
        owner,
        None,
        None,
        plugin_type(PluginType::Attributes),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    // A plugin that is not in the registry.
    let other = Pubkey::new_unique();
    f.store(
        other,
        AssetSpec::new(owner)
            .plugin(attributes("v"), Authority::UpdateAuthority)
            .build(),
    );
    let result = f.run(&ix::revoke_plugin_authority_v1(
        other,
        None,
        owner,
        None,
        None,
        plugin_type(PluginType::FreezeDelegate),
    ));
    assert_core_err(&result, MplCoreError::PluginNotFound);
}

/// Regression for roadmap section 9, finding 1: `RevokePluginAuthorityV1`
/// passes the loaded asset to `fetch_wrapped_plugin`, which skips the
/// "account has no plugin metadata" check and indexes past the end of the
/// account in `PluginHeaderV1::load`. The panic surfaces as
/// `ProgramFailedToComplete` instead of the `PluginNotFound` that every other
/// instruction returns for the same input.
#[test]
fn revoke_plugin_authority_on_bare_asset_panics() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(asset, AssetSpec::new(owner).build());

    let result = f.run(&ix::revoke_plugin_authority_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin_type(PluginType::FreezeDelegate),
    ));
    assert_instruction_err(&result, InstructionError::ProgramFailedToComplete);
}

/// The collection variant of the same panic.
#[test]
fn revoke_collection_plugin_authority_on_bare_collection_panics() {
    let f = Fixture::new();
    let authority = f.fund(ACCOUNT_LAMPORTS);
    let collection = Pubkey::new_unique();
    f.store(collection, CollectionSpec::new(authority).build());

    let result = f.run(&ix::revoke_collection_plugin_authority_v1(
        collection,
        authority,
        None,
        None,
        plugin_type(PluginType::Attributes),
    ));
    assert_instruction_err(&result, InstructionError::ProgramFailedToComplete);
}

/// JS: revokeAuthority.test.ts :: it can remove an authority from a plugin
/// (collection variant: the update authority revokes, then the delegate
/// revokes itself and the collection account pays.)
#[test]
fn revoke_collection_plugin_authority() {
    let f = Fixture::new();
    let authority = f.fund(ACCOUNT_LAMPORTS);
    let delegate = f.fund(ACCOUNT_LAMPORTS);
    let stranger = f.fund(ACCOUNT_LAMPORTS);
    let collection = Pubkey::new_unique();
    f.store(
        collection,
        CollectionSpec::new(authority)
            .plugin(attributes("v"), address(delegate))
            .plugin(update_delegate(&[]), address(delegate))
            .build(),
    );

    let result = f.run(&ix::revoke_collection_plugin_authority_v1(
        collection,
        stranger,
        None,
        None,
        plugin_type(PluginType::Attributes),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    let len_before = f.account(&collection).data.len();
    f.run_ok(&ix::revoke_collection_plugin_authority_v1(
        collection,
        authority,
        None,
        None,
        plugin_type(PluginType::Attributes),
    ));
    let account = f.account(&collection);
    assert_eq!(
        read(&account, PluginType::Attributes),
        (Authority::UpdateAuthority, attributes("v"))
    );
    assert_eq!(account.data.len(), len_before - 32);

    // The delegate revokes its own record on the trailing plugin.
    let collection_lamports = f.lamports(&collection);
    f.run_ok(&ix::revoke_collection_plugin_authority_v1(
        collection,
        delegate,
        None,
        None,
        plugin_type(PluginType::UpdateDelegate),
    ));
    let account = f.account(&collection);
    assert_eq!(
        plugins_of(&account),
        vec![
            (attributes("v"), Authority::UpdateAuthority),
            (update_delegate(&[]), Authority::UpdateAuthority),
        ]
    );
    assert_eq!(
        f.lamports(&collection),
        collection_lamports,
        "the collection pays itself when the signer is not the plugin manager"
    );
}

// ===========================================================================
// The UpdateDelegate authority matrix
// ===========================================================================

/// An asset-level `UpdateDelegate` (root authority or additional delegate) may
/// add update-authority-managed plugins but not owner-managed ones
/// (`update_delegate.rs:66-86`).
///
/// JS: addPlugin.test.ts :: it can add an authority-managed plugin to an asset via delegate authority
/// JS: addPlugin.test.ts :: it cannot add a owner-managed plugin to an asset via delegate authority
/// JS: plugins/asset/updateDelegate.test.ts :: an updateDelegate can add a plugin to an asset
#[test]
fn add_plugin_via_asset_update_delegate() {
    for use_additional_delegate in [false, true] {
        let f = Fixture::new();
        let owner = Pubkey::new_unique();
        let update_authority = Pubkey::new_unique();
        let delegate = f.fund(ACCOUNT_LAMPORTS);
        let asset = Pubkey::new_unique();
        let (delegate_plugin, record_authority) = if use_additional_delegate {
            (update_delegate(&[delegate]), Authority::UpdateAuthority)
        } else {
            (update_delegate(&[]), address(delegate))
        };
        f.store(
            asset,
            AssetSpec::new(owner)
                .update_authority(UpdateAuthority::Address(update_authority))
                .plugin(delegate_plugin.clone(), record_authority)
                .build(),
        );

        f.run_ok(&ix::add_plugin_v1(
            asset,
            None,
            delegate,
            None,
            None,
            plugin(&attributes("v")),
            None,
        ));
        assert_eq!(
            plugins_of(&f.account(&asset)),
            vec![
                (delegate_plugin.clone(), record_authority),
                (attributes("v"), Authority::UpdateAuthority),
            ]
        );

        let result = f.run(&ix::add_plugin_v1(
            asset,
            None,
            delegate,
            None,
            None,
            plugin(&freeze(false)),
            None,
        ));
        assert_core_err(&result, MplCoreError::NoApprovals);
    }
}

/// The same delegate may remove what it could add, and the collection update
/// authority may remove an update-authority-managed plugin from a member asset
/// (but only with the right collection account).
///
/// JS: plugins/asset/updateDelegate.test.ts :: an updateDelegate can remove a plugin from an asset
/// JS: removePlugin.test.ts :: it can remove authority managed plugin from asset in collection using update auth
/// JS: removePlugin.test.ts :: it cannot use an invalid collection to remove a plugin on an asset
#[test]
fn remove_plugin_via_update_delegate_and_collection_authority() {
    // Asset-level UpdateDelegate.
    let f = Fixture::new();
    let owner = Pubkey::new_unique();
    let delegate = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .plugin(update_delegate(&[]), address(delegate))
            .plugin(attributes("v"), Authority::UpdateAuthority)
            .build(),
    );
    f.run_ok(&ix::remove_plugin_v1(
        asset,
        None,
        delegate,
        None,
        None,
        plugin_type(PluginType::Attributes),
    ));
    assert_eq!(
        plugins_of(&f.account(&asset)),
        vec![(update_delegate(&[]), address(delegate))]
    );

    // Collection update authority on a member asset.
    let authority = f.fund(ACCOUNT_LAMPORTS);
    let member = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    let other_collection = Pubkey::new_unique();
    f.store(
        member,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Collection(collection))
            .plugin(attributes("v"), Authority::UpdateAuthority)
            .build(),
    );
    f.store(
        collection,
        CollectionSpec::new(authority).sizes(1, 1).build(),
    );
    f.store(other_collection, CollectionSpec::new(authority).build());

    let result = f.run(&ix::remove_plugin_v1(
        member,
        Some(other_collection),
        authority,
        None,
        None,
        plugin_type(PluginType::Attributes),
    ));
    assert_core_err(&result, MplCoreError::InvalidCollection);

    f.run_ok(&ix::remove_plugin_v1(
        member,
        Some(collection),
        authority,
        None,
        None,
        plugin_type(PluginType::Attributes),
    ));
    assert!(plugins_of(&f.account(&member)).is_empty());
}

/// Only the signer that the target plugin's registry authority resolves to may
/// update it; the other plugins on the asset are irrelevant. The four cases
/// are the `updatePlugin.test.ts` matrix.
///
/// JS: updatePlugin.test.ts :: it cannot update a plugin with Owner authority on an Asset if a plugin with UpdateAuthority authority is present
/// JS: updatePlugin.test.ts :: it can update a plugin with Owner authority on an Asset if a plugin with UpdateAuthority authority is present
/// JS: updatePlugin.test.ts :: it cannot update a plugin with UpdateAuthority authority on an Asset if a plugin with Owner authority is present
/// JS: updatePlugin.test.ts :: it can update a plugin with UpdateAuthority authority on an Asset if a plugin with Owner authority is present
#[test]
fn update_plugin_authority_matrix() {
    let stranger = Pubkey::new_unique();

    // (freeze authority, attributes authority, signer is the owner, target,
    //  expected success)
    let cases: [(Authority, Authority, bool, PluginType, bool); 4] = [
        // The update authority cannot touch a plugin delegated elsewhere.
        (
            address(stranger),
            Authority::UpdateAuthority,
            false,
            PluginType::FreezeDelegate,
            false,
        ),
        // It can touch one whose record names the update authority.
        (
            Authority::UpdateAuthority,
            Authority::UpdateAuthority,
            false,
            PluginType::FreezeDelegate,
            true,
        ),
        // The owner cannot touch an update-authority-managed plugin.
        (
            Authority::Owner,
            Authority::UpdateAuthority,
            true,
            PluginType::Attributes,
            false,
        ),
        // But it can when the record names the owner.
        (
            Authority::Owner,
            Authority::Owner,
            true,
            PluginType::Attributes,
            true,
        ),
    ];

    for (freeze_authority, attributes_authority, signer_is_owner, target, expect_ok) in cases {
        let f = Fixture::new();
        let owner = f.fund(ACCOUNT_LAMPORTS);
        let update_authority = f.fund(ACCOUNT_LAMPORTS);
        let asset = Pubkey::new_unique();
        f.store(
            asset,
            AssetSpec::new(owner)
                .update_authority(UpdateAuthority::Address(update_authority))
                .plugin(freeze(false), freeze_authority)
                .plugin(attributes("v"), attributes_authority)
                .build(),
        );
        let signer = if signer_is_owner {
            owner
        } else {
            update_authority
        };
        let new_plugin = match target {
            PluginType::FreezeDelegate => freeze(true),
            PluginType::Attributes => attributes("updated"),
            other => panic!("unexpected target {other:?}"),
        };

        let result = f.run(&ix::update_plugin_v1(
            asset,
            None,
            signer,
            None,
            None,
            plugin(&new_plugin),
        ));
        let case = format!(
            "{target:?} by {}",
            if signer_is_owner { "owner" } else { "UA" }
        );
        if expect_ok {
            assert_ok(&result);
            assert_eq!(
                read(&f.account(&asset), target).1,
                new_plugin,
                "{case}: the plugin should have been updated"
            );
        } else {
            expect_core_err(&case, &result, MplCoreError::NoApprovals);
        }
    }
}

/// The same matrix with a frozen `PermanentFreezeDelegate` on the collection:
/// the collection plugin neither approves nor blocks the update of an
/// asset-level plugin.
///
/// JS: updatePlugin.test.ts :: it cannot update a plugin with Owner authority on an Asset if a plugin with UpdateAuthority authority is present on a Collection
/// JS: updatePlugin.test.ts :: it can update a plugin with Owner authority on an Asset if a plugin with UpdateAuthority authority is present on a Collection
#[test]
fn update_plugin_on_asset_in_collection_with_permanent_freeze() {
    for (freeze_authority, expect_ok) in [
        (address(Pubkey::new_unique()), false),
        (Authority::UpdateAuthority, true),
    ] {
        let f = Fixture::new();
        let authority = f.fund(ACCOUNT_LAMPORTS);
        let asset = Pubkey::new_unique();
        let collection = Pubkey::new_unique();
        f.store(
            asset,
            AssetSpec::new(authority)
                .update_authority(UpdateAuthority::Collection(collection))
                .plugin(freeze(false), freeze_authority)
                .build(),
        );
        f.store(
            collection,
            CollectionSpec::new(authority)
                .sizes(1, 1)
                .plugin(
                    Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen: true }),
                    Authority::UpdateAuthority,
                )
                .build(),
        );

        let result = f.run(&ix::update_plugin_v1(
            asset,
            Some(collection),
            authority,
            None,
            None,
            plugin(&freeze(false)),
        ));
        if expect_ok {
            assert_ok(&result);
            assert_eq!(
                read(&f.account(&asset), PluginType::FreezeDelegate).0,
                Authority::UpdateAuthority
            );
        } else {
            expect_core_err(
                "delegated FreezeDelegate",
                &result,
                MplCoreError::NoApprovals,
            );
        }
    }
}

/// `Royalties::validate_update_plugin` validates the incoming royalty data and
/// returns `InvalidPluginSetting` outright (the roadmap expected the softer
/// `InvalidAuthority` that a `Rejected` result would produce; the validator
/// errors instead). See roadmap section 6.
///
/// JS: plugins/asset/royalties.test.ts :: it cannot update royalty basis points greater than 10000
#[test]
fn update_plugin_rejects_invalid_royalties() {
    let f = Fixture::new();
    let authority = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    let creator = || Creator {
        address: authority,
        percentage: 100,
    };
    let valid = Plugin::Royalties(Royalties {
        basis_points: 500,
        creators: vec![creator()],
        rule_set: RuleSet::None,
    });
    f.store(
        asset,
        AssetSpec::new(authority)
            .plugin(valid.clone(), Authority::UpdateAuthority)
            .build(),
    );

    let invalid = Plugin::Royalties(Royalties {
        basis_points: 10_001,
        creators: vec![creator()],
        rule_set: RuleSet::None,
    });
    let result = f.run(&ix::update_plugin_v1(
        asset,
        None,
        authority,
        None,
        None,
        plugin(&invalid),
    ));
    assert_core_err(&result, MplCoreError::InvalidPluginSetting);
    assert_eq!(read(&f.account(&asset), PluginType::Royalties).1, valid);

    // A valid change goes through.
    let larger = Plugin::Royalties(Royalties {
        basis_points: 1000,
        creators: vec![
            creator(),
            Creator {
                address: Pubkey::new_unique(),
                percentage: 0,
            },
        ],
        rule_set: RuleSet::None,
    });
    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        authority,
        None,
        None,
        plugin(&larger),
    ));
    assert_eq!(read(&f.account(&asset), PluginType::Royalties).1, larger);
}

/// Regression for the operator-precedence fix in
/// `UpdateDelegate::validate_revoke_plugin_authority`: an additional delegate
/// may revoke update-authority-managed plugins only, never owner-managed ones.
///
/// JS: plugins/asset/updateDelegateRevokeBug.test.ts :: it should NOT allow update authority to revoke authority on owner-managed plugins via UpdateDelegate
/// JS: plugins/asset/updateDelegateRevokeBug.test.ts :: it should NOT allow delegated update delegate to revoke authority on owner-managed plugins
/// JS: plugins/asset/updateDelegateRevokeBug.test.ts :: it should allow update authority to revoke authority on UpdateAuthority-managed plugins via UpdateDelegate
/// JS: plugins/asset/updateDelegate.test.ts :: it can approve/revoke the plugin authority of non-updateDelegate plugins as additional delegate
#[test]
fn revoke_via_update_delegate_only_for_update_authority_managed() {
    let f = Fixture::new();
    let owner = Pubkey::new_unique();
    let update_authority = f.fund(ACCOUNT_LAMPORTS);
    let delegate = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Address(update_authority))
            .plugin(update_delegate(&[delegate]), Authority::UpdateAuthority)
            .plugin(freeze(false), address(Pubkey::new_unique()))
            .plugin(attributes("v"), address(Pubkey::new_unique()))
            .build(),
    );

    // The owner-managed FreezeDelegate is out of reach for both the update
    // authority and its additional delegate.
    for signer in [update_authority, delegate] {
        let result = f.run(&ix::revoke_plugin_authority_v1(
            asset,
            None,
            signer,
            None,
            None,
            plugin_type(PluginType::FreezeDelegate),
        ));
        assert_core_err(&result, MplCoreError::NoApprovals);
    }

    // The update-authority-managed Attributes is not.
    f.run_ok(&ix::revoke_plugin_authority_v1(
        asset,
        None,
        delegate,
        None,
        None,
        plugin_type(PluginType::Attributes),
    ));
    assert_eq!(
        read(&f.account(&asset), PluginType::Attributes).0,
        Authority::UpdateAuthority
    );
}

/// An additional delegate may delegate other update-authority-managed plugins
/// but not the `UpdateDelegate` plugin itself
/// (`update_delegate.rs:111-136`).
///
/// JS: plugins/asset/updateDelegate.test.ts :: it cannot approve the update delegate plugin authority as additional delegate
/// JS: plugins/asset/updateDelegate.test.ts :: it can approve/revoke the plugin authority of non-updateDelegate plugins as additional delegate
#[test]
fn approve_as_additional_delegate() {
    let f = Fixture::new();
    let owner = Pubkey::new_unique();
    let update_authority = Pubkey::new_unique();
    let delegate = f.fund(ACCOUNT_LAMPORTS);
    let new_delegate = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Address(update_authority))
            .plugin(update_delegate(&[delegate]), Authority::UpdateAuthority)
            .plugin(attributes("v"), Authority::UpdateAuthority)
            .build(),
    );

    let result = f.run(&ix::approve_plugin_authority_v1(
        asset,
        None,
        delegate,
        None,
        None,
        plugin_type(PluginType::UpdateDelegate),
        plugin_authority(address(new_delegate)),
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);

    f.run_ok(&ix::approve_plugin_authority_v1(
        asset,
        None,
        delegate,
        None,
        None,
        plugin_type(PluginType::Attributes),
        plugin_authority(address(new_delegate)),
    ));
    assert_eq!(
        read(&f.account(&asset), PluginType::Attributes).0,
        address(new_delegate)
    );
    assert_eq!(
        read(&f.account(&asset), PluginType::UpdateDelegate).0,
        Authority::UpdateAuthority,
        "the UpdateDelegate record is untouched"
    );
}

// ===========================================================================
// Sequence numbers
// ===========================================================================

/// `increment_seq_and_save` runs on every asset-side plugin instruction but is
/// a no-op unless the asset carries a sequence number (only compression sets
/// one, so the asset is crafted with `seq`).
#[test]
fn asset_plugin_instructions_increment_seq() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let delegate = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    f.store(asset, AssetSpec::new(owner).seq(5).build());

    f.run_ok(&ix::add_plugin_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin(&freeze(false)),
        None,
    ));
    assert_eq!(f.asset(&asset).seq, Some(6));

    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin(&freeze(true)),
    ));
    assert_eq!(f.asset(&asset).seq, Some(7));

    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin(&freeze(false)),
    ));
    f.run_ok(&ix::approve_plugin_authority_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin_type(PluginType::FreezeDelegate),
        plugin_authority(address(delegate)),
    ));
    assert_eq!(f.asset(&asset).seq, Some(9));

    f.run_ok(&ix::revoke_plugin_authority_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin_type(PluginType::FreezeDelegate),
    ));
    assert_eq!(f.asset(&asset).seq, Some(10));

    f.run_ok(&ix::remove_plugin_v1(
        asset,
        None,
        owner,
        None,
        None,
        plugin_type(PluginType::FreezeDelegate),
    ));
    assert_eq!(f.asset(&asset).seq, Some(11));
    assert!(plugins_of(&f.account(&asset)).is_empty());
}
