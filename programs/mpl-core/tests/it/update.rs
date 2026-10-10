//! `UpdateV1`, `UpdateV2`, `UpdateCollectionV1` and `UpdateCollectionInfoV1`
//! (`src/processor/update.rs`, `src/processor/update_collection_info.rs`).
//!
//! `process_update` moves the plugin area when the core account grows or
//! shrinks, so every success test here asserts the post-state of the trailing
//! plugins as well as the field that changed: the JS suite only checks the
//! decoded plugin values, while the Rust client's `plugin_shrink_corruption`
//! tests check that the bytes survive. Both are asserted below through
//! [`assert_registry_consistent`] plus explicit plugin comparisons.

use {
    crate::common::{
        accounts::{DEFAULT_ASSET_NAME, DEFAULT_COLLECTION_NAME, DEFAULT_URI},
        *,
    },
    mpl_core::types as client,
    mpl_core_program::{
        error::MplCoreError,
        plugins::{
            Attribute, Attributes, FreezeDelegate, ImmutableMetadata, PermanentBurnDelegate,
            PermanentFreezeDelegate, PermanentTransferDelegate, Plugin, PluginType, UpdateDelegate,
        },
        state::{Authority, UpdateAuthority},
    },
    solana_account::Account,
    solana_program::pubkey::Pubkey,
};

/// The Bubblegum PDA that is the only accepted signer of
/// `UpdateCollectionInfoV1` (`src/state/mod.rs`).
const BUBBLEGUM_SIGNER: Pubkey =
    solana_program::pubkey!("CbNY3JiXdXNE9tPNEk1aRZVEkWdj2v7kfJLNQwZZgpXk");

/// A name long enough that switching to it grows the account.
const LONG_NAME: &str = "a very long asset name that grows the core account";
/// A URI long enough that switching to it grows the account.
const LONG_URI: &str = "https://example.com/a/very/long/uri/that/grows/the/account.json";
/// A short name, for the shrink direction.
const SHORT_NAME: &str = "n";
/// A short URI, for the shrink direction.
const SHORT_URI: &str = "u";

/// The `FreezeDelegate` used as the first plugin of the multi-plugin assets.
fn freeze_delegate() -> Plugin {
    Plugin::FreezeDelegate(FreezeDelegate { frozen: false })
}

/// The `Attributes` plugin used as the trailing plugin: its bytes are what a
/// botched memmove in `process_update` would corrupt.
fn attributes() -> Plugin {
    Plugin::Attributes(Attributes {
        attribute_list: vec![
            Attribute {
                key: "trailing".to_string(),
                value: "must survive the memmove".to_string(),
            },
            Attribute {
                key: "second".to_string(),
                value: "value".to_string(),
            },
        ],
    })
}

/// An `UpdateDelegate` plugin with the given additional delegates.
fn update_delegate(additional: &[Pubkey]) -> Plugin {
    Plugin::UpdateDelegate(UpdateDelegate {
        additional_delegates: additional.to_vec(),
    })
}

/// Asserts the account still holds exactly `plugins` (type, authority and
/// value) in registry order, with a consistent layout.
fn assert_plugins(account: &Account, plugins: &[(Plugin, Authority)]) {
    assert_registry_consistent(account);
    let parsed = parse_asset(&account.data).1;
    let found: Vec<_> = parsed
        .plugins
        .iter()
        .map(|(record, plugin)| (plugin.clone(), record.authority))
        .collect();
    assert_eq!(
        found, plugins,
        "the plugin area should be unchanged by the update"
    );
}

/// Asserts the collection account still holds exactly `plugins`.
fn assert_collection_plugins(account: &Account, plugins: &[(Plugin, Authority)]) {
    assert_registry_consistent(account);
    let parsed = parse_collection(&account.data).1;
    let found: Vec<_> = parsed
        .plugins
        .iter()
        .map(|(record, plugin)| (plugin.clone(), record.authority))
        .collect();
    assert_eq!(found, plugins, "the collection plugins should be unchanged");
}

/// A fixture with a funded payer and a stored account.
fn setup(account_key: Pubkey, account: Account) -> (Fixture, Pubkey) {
    let f = Fixture::new();
    let payer = f.fund(ACCOUNT_LAMPORTS);
    f.store(account_key, account);
    (f, payer)
}

// ===========================================================================
// UpdateV1: name and URI, with and without plugins
// ===========================================================================

/// JS: update.test.ts :: it can update an asset to be larger
#[test]
fn update_v1_grows_name_and_uri() {
    let asset = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let (f, _payer) = setup(asset, AssetSpec::new(owner).build());
    f.store(owner, payer_account(ACCOUNT_LAMPORTS));
    let len_before = f.account(&asset).data.len();

    f.run_ok(&ix::update_v1(
        asset,
        None,
        owner,
        None,
        None,
        Some(LONG_NAME.to_string()),
        Some(LONG_URI.to_string()),
        None,
    ));

    let updated = f.asset(&asset);
    assert_eq!(updated.name, LONG_NAME);
    assert_eq!(updated.uri, LONG_URI);
    assert_eq!(updated.owner, owner);
    assert_eq!(updated.update_authority, UpdateAuthority::Address(owner));
    let account = f.account(&asset);
    assert!(
        account.data.len() > len_before,
        "the account should have grown from {len_before} to more, got {}",
        account.data.len()
    );
    assert_eq!(
        account.data.len(),
        borsh::to_vec(&updated).unwrap().len(),
        "a bare asset must be exactly the size of its core account"
    );
    assert_registry_consistent(&account);
    // The payer funded the extra rent.
    assert!(
        f.lamports(&owner) < ACCOUNT_LAMPORTS,
        "the payer should have paid for the realloc"
    );
}

/// JS: update.test.ts :: it can update an asset to be smaller
#[test]
fn update_v1_shrinks_name_and_uri() {
    let asset = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let (f, _payer) = setup(
        asset,
        AssetSpec::new(owner).name(LONG_NAME).uri(LONG_URI).build(),
    );
    f.store(owner, payer_account(ACCOUNT_LAMPORTS));
    let len_before = f.account(&asset).data.len();

    f.run_ok(&ix::update_v1(
        asset,
        None,
        owner,
        None,
        None,
        Some(SHORT_NAME.to_string()),
        Some(SHORT_URI.to_string()),
        None,
    ));

    let updated = f.asset(&asset);
    assert_eq!(updated.name, SHORT_NAME);
    assert_eq!(updated.uri, SHORT_URI);
    let account = f.account(&asset);
    assert!(
        account.data.len() < len_before,
        "the account should have shrunk from {len_before}, got {}",
        account.data.len()
    );
    assert_registry_consistent(&account);
}

/// JS: update.test.ts :: it can update an asset with plugins to be larger
#[test]
fn update_v1_with_plugins_grows() {
    let asset = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let plugins = vec![
        (freeze_delegate(), Authority::Owner),
        (attributes(), Authority::UpdateAuthority),
    ];
    let mut spec = AssetSpec::new(owner);
    for (plugin, authority) in &plugins {
        spec = spec.plugin(plugin.clone(), *authority);
    }
    let (f, _payer) = setup(asset, spec.build());
    f.store(owner, payer_account(ACCOUNT_LAMPORTS));
    let before = f.account(&asset);
    let registry_offset_before = parse_asset(&before.data)
        .1
        .header
        .unwrap()
        .plugin_registry_offset;

    f.run_ok(&ix::update_v1(
        asset,
        None,
        owner,
        None,
        None,
        Some(LONG_NAME.to_string()),
        Some(LONG_URI.to_string()),
        None,
    ));

    let account = f.account(&asset);
    let updated = read_asset(&account);
    assert_eq!(updated.name, LONG_NAME);
    assert_eq!(updated.uri, LONG_URI);
    assert_plugins(&account, &plugins);
    let parsed = parse_asset(&account.data).1;
    let growth = account.data.len() - before.data.len();
    assert!(growth > 0, "the account should have grown");
    assert_eq!(
        parsed.header.unwrap().plugin_registry_offset,
        registry_offset_before + growth,
        "the registry offset should have moved by exactly the core size delta"
    );
}

/// JS: update.test.ts :: it can update an asset with plugins to be smaller
///
/// Also the Mollusk port of the Rust client regression
/// `plugin_shrink_corruption.rs::test_update_v1_shrink_name_uri_preserves_plugin`:
/// the trailing plugin bytes must survive the memmove-then-realloc order.
#[test]
fn update_v1_with_plugins_shrinks() {
    let asset = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let plugins = vec![
        (freeze_delegate(), Authority::Owner),
        (attributes(), Authority::UpdateAuthority),
    ];
    let mut spec = AssetSpec::new(owner).name(LONG_NAME).uri(LONG_URI);
    for (plugin, authority) in &plugins {
        spec = spec.plugin(plugin.clone(), *authority);
    }
    let (f, _payer) = setup(asset, spec.build());
    f.store(owner, payer_account(ACCOUNT_LAMPORTS));
    let before = f.account(&asset);
    let registry_offset_before = parse_asset(&before.data)
        .1
        .header
        .unwrap()
        .plugin_registry_offset;

    f.run_ok(&ix::update_v1(
        asset,
        None,
        owner,
        None,
        None,
        Some(SHORT_NAME.to_string()),
        Some(SHORT_URI.to_string()),
        None,
    ));

    let account = f.account(&asset);
    let updated = read_asset(&account);
    assert_eq!(updated.name, SHORT_NAME);
    assert_eq!(updated.uri, SHORT_URI);
    assert_plugins(&account, &plugins);
    let parsed = parse_asset(&account.data).1;
    let shrink = before.data.len() - account.data.len();
    assert!(shrink > 0, "the account should have shrunk");
    assert_eq!(
        parsed.header.unwrap().plugin_registry_offset,
        registry_offset_before - shrink,
        "the registry offset should have moved by exactly the core size delta"
    );
}

/// A same-length rename takes neither realloc branch (`size_diff == 0`).
#[test]
fn update_v1_same_size_keeps_layout() {
    let asset = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let plugins = vec![(attributes(), Authority::UpdateAuthority)];
    let (f, _payer) = setup(
        asset,
        AssetSpec::new(owner)
            .plugin(attributes(), Authority::UpdateAuthority)
            .build(),
    );
    f.store(owner, payer_account(ACCOUNT_LAMPORTS));
    let before = f.account(&asset);
    let same_length_name: String = "x".repeat(DEFAULT_ASSET_NAME.len());

    f.run_ok(&ix::update_v1(
        asset,
        None,
        owner,
        None,
        None,
        Some(same_length_name.clone()),
        None,
        None,
    ));

    let account = f.account(&asset);
    assert_eq!(read_asset(&account).name, same_length_name);
    assert_eq!(
        account.data.len(),
        before.data.len(),
        "a same-length rename must not resize the account"
    );
    assert_plugins(&account, &plugins);
}

/// An asset whose plugins were all removed keeps an empty header and registry;
/// `copy_len` is then zero and the memmove is skipped.
#[test]
fn update_v1_with_empty_registry() {
    let asset = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let (f, _payer) = setup(asset, AssetSpec::new(owner).with_empty_meta().build());
    f.store(owner, payer_account(ACCOUNT_LAMPORTS));

    f.run_ok(&ix::update_v1(
        asset,
        None,
        owner,
        None,
        None,
        Some(LONG_NAME.to_string()),
        None,
        None,
    ));

    let account = f.account(&asset);
    assert_eq!(read_asset(&account).name, LONG_NAME);
    let parsed = parse_asset(&account.data).1;
    assert!(parsed.has_meta(), "the empty header must be preserved");
    assert!(parsed.plugins.is_empty());
    assert_registry_consistent(&account);
}

// ===========================================================================
// UpdateV1: update authority
// ===========================================================================

/// JS: update.test.ts :: it can update an asset update authority
#[test]
fn update_v1_changes_update_authority() {
    let asset = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let new_authority = Pubkey::new_unique();
    let (f, _payer) = setup(asset, AssetSpec::new(owner).build());
    f.store(owner, payer_account(ACCOUNT_LAMPORTS));

    f.run_ok(&ix::update_v1(
        asset,
        None,
        owner,
        None,
        None,
        None,
        None,
        Some(client::UpdateAuthority::Address(new_authority)),
    ));

    assert_eq!(
        f.asset(&asset).update_authority,
        UpdateAuthority::Address(new_authority)
    );
    assert_registry_consistent(&f.account(&asset));
}

/// JS: update.test.ts :: it can update an asset update authority to None
#[test]
fn update_v1_changes_update_authority_to_none() {
    let asset = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let (f, _payer) = setup(asset, AssetSpec::new(owner).build());
    f.store(owner, payer_account(ACCOUNT_LAMPORTS));

    f.run_ok(&ix::update_v1(
        asset,
        None,
        owner,
        None,
        None,
        None,
        None,
        Some(client::UpdateAuthority::None),
    ));

    assert_eq!(f.asset(&asset).update_authority, UpdateAuthority::None);
}

/// JS: update.test.ts :: it can update an asset with plugins update authority to None
#[test]
fn update_v1_with_plugins_changes_update_authority_to_none() {
    let asset = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let plugins = vec![(freeze_delegate(), Authority::Owner)];
    let (f, _payer) = setup(
        asset,
        AssetSpec::new(owner)
            .plugin(freeze_delegate(), Authority::Owner)
            .build(),
    );
    f.store(owner, payer_account(ACCOUNT_LAMPORTS));
    let len_before = f.account(&asset).data.len();

    f.run_ok(&ix::update_v1(
        asset,
        None,
        owner,
        None,
        None,
        None,
        None,
        Some(client::UpdateAuthority::None),
    ));

    let account = f.account(&asset);
    assert_eq!(read_asset(&account).update_authority, UpdateAuthority::None);
    assert_plugins(&account, &plugins);
    assert!(
        account.data.len() < len_before,
        "UpdateAuthority::None is smaller than Address(..), so the account shrinks"
    );
}

/// JS: update.test.ts :: it cannot update an asset update authority to be part of a collection using updateV1
#[test]
fn update_v1_cannot_add_asset_to_collection() {
    let asset = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    let (f, _payer) = setup(asset, AssetSpec::new(owner).build());
    f.store(owner, payer_account(ACCOUNT_LAMPORTS));
    f.store(collection, CollectionSpec::new(owner).build());

    let result = f.run(&ix::update_v1(
        asset,
        None,
        owner,
        None,
        None,
        None,
        None,
        Some(client::UpdateAuthority::Collection(collection)),
    ));
    assert_core_err(&result, MplCoreError::NotAvailable);
    assert_eq!(
        f.asset(&asset).update_authority,
        UpdateAuthority::Address(owner),
        "the rejected update must not have changed the asset"
    );
}

/// JS: update.test.ts :: it cannot remove an asset from a collection using updateV1
#[test]
fn update_v1_cannot_remove_asset_from_collection() {
    let asset = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    let (f, _payer) = setup(
        asset,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Collection(collection))
            .build(),
    );
    f.store(owner, payer_account(ACCOUNT_LAMPORTS));
    f.store(collection, CollectionSpec::new(owner).sizes(1, 1).build());

    let result = f.run(&ix::update_v1(
        asset,
        Some(collection),
        owner,
        None,
        None,
        None,
        None,
        Some(client::UpdateAuthority::Address(owner)),
    ));
    assert_core_err(&result, MplCoreError::NotAvailable);
    assert_eq!(f.collection(&collection).current_size, 1);
}

// ===========================================================================
// UpdateV1 / UpdateV2 guards and authority rejections
// ===========================================================================

/// JS: update.test.ts :: it cannot update an asset using wrong authority
#[test]
fn update_rejects_wrong_authority() {
    let asset = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let stranger = Pubkey::new_unique();
    let (f, _payer) = setup(asset, AssetSpec::new(owner).build());
    f.store(stranger, payer_account(ACCOUNT_LAMPORTS));

    let result = f.run(&ix::update_v1(
        asset,
        None,
        stranger,
        None,
        None,
        Some(LONG_NAME.to_string()),
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_eq!(f.asset(&asset).name, DEFAULT_ASSET_NAME);
}

/// JS: update.test.ts :: it cannot update an asset using asset as authority
#[test]
fn update_rejects_asset_as_authority() {
    let asset = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let (f, payer) = setup(asset, AssetSpec::new(owner).build());

    let result = f.run(&ix::update_v1(
        asset,
        None,
        payer,
        Some(asset),
        None,
        Some(LONG_NAME.to_string()),
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
}

/// JS: update.test.ts :: it cannot use an invalid system program for assets
#[test]
fn update_rejects_invalid_system_program() {
    let asset = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let (f, _payer) = setup(asset, AssetSpec::new(owner).build());
    f.store(owner, payer_account(ACCOUNT_LAMPORTS));

    let mut ix = ix::update_v1(
        asset,
        None,
        owner,
        None,
        None,
        Some(LONG_NAME.to_string()),
        None,
        None,
    );
    // Account 4 is the system program on UpdateV1.
    set_account(
        &mut ix,
        4,
        solana_program::instruction::AccountMeta::new_readonly(Pubkey::new_unique(), false),
    );
    assert_core_err(&f.run(&ix), MplCoreError::InvalidSystemProgram);
}

/// JS: update.test.ts :: it cannot use an invalid noop program for assets
#[test]
fn update_rejects_invalid_log_wrapper() {
    let asset = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let (f, _payer) = setup(asset, AssetSpec::new(owner).build());
    f.store(owner, payer_account(ACCOUNT_LAMPORTS));

    let ix = ix::update_v1(
        asset,
        None,
        owner,
        None,
        Some(Pubkey::new_unique()),
        Some(LONG_NAME.to_string()),
        None,
        None,
    );
    assert_core_err(&f.run(&ix), MplCoreError::InvalidLogWrapperProgram);
}

/// JS: updateV2.test.ts :: it cannot use an invalid system program for assets
#[test]
fn update_v2_rejects_invalid_system_program_and_log_wrapper() {
    let asset = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let (f, _payer) = setup(asset, AssetSpec::new(owner).build());
    f.store(owner, payer_account(ACCOUNT_LAMPORTS));

    let mut ix = ix::update_v2(
        asset,
        None,
        owner,
        None,
        None,
        None,
        Some(LONG_NAME.to_string()),
        None,
        None,
    );
    // Account 5 is the system program on UpdateV2.
    set_account(
        &mut ix,
        5,
        solana_program::instruction::AccountMeta::new_readonly(Pubkey::new_unique(), false),
    );
    assert_core_err(&f.run(&ix), MplCoreError::InvalidSystemProgram);

    let ix = ix::update_v2(
        asset,
        None,
        owner,
        None,
        None,
        Some(Pubkey::new_unique()),
        Some(LONG_NAME.to_string()),
        None,
        None,
    );
    assert_core_err(&f.run(&ix), MplCoreError::InvalidLogWrapperProgram);
}

/// The payer must sign. Mollusk verifies no signatures, so the missing signer
/// is expressed by clearing `is_signer`.
#[test]
fn update_rejects_non_signer_payer() {
    let asset = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let (f, _payer) = setup(asset, AssetSpec::new(owner).build());
    f.store(owner, payer_account(ACCOUNT_LAMPORTS));

    let mut ix = ix::update_v1(
        asset,
        None,
        owner,
        None,
        None,
        Some(LONG_NAME.to_string()),
        None,
        None,
    );
    unsign(&mut ix, &owner);
    assert_program_err(
        &f.run(&ix),
        solana_program::program_error::ProgramError::MissingRequiredSignature,
    );
}

/// A compressed asset cannot be updated (`update.rs:105-108`). `load_key`
/// only reads byte 0, so the one-byte placeholder is enough.
#[test]
fn update_rejects_compressed_asset() {
    let asset = Pubkey::new_unique();
    let (f, payer) = setup(asset, hashed_asset_placeholder());

    let result = f.run(&ix::update_v1(
        asset,
        None,
        payer,
        None,
        None,
        Some(LONG_NAME.to_string()),
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::NotAvailable);
}

/// `increment_seq_and_save` is a no-op unless the asset carries a sequence
/// number; only compression sets one, so the asset is crafted with `seq`.
#[test]
fn update_increments_seq_when_present() {
    let asset = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let (f, _payer) = setup(asset, AssetSpec::new(owner).seq(5).build());
    f.store(owner, payer_account(ACCOUNT_LAMPORTS));

    f.run_ok(&ix::update_v1(
        asset,
        None,
        owner,
        None,
        None,
        Some(LONG_NAME.to_string()),
        None,
        None,
    ));

    let updated = f.asset(&asset);
    assert_eq!(updated.seq, Some(6));
    assert_eq!(updated.name, LONG_NAME);
}

/// `ImmutableMetadata` rejects every name/URI change.
///
/// JS: plugins/asset/immutableMetadata.test.ts :: it can prevent the asset from metadata updating
#[test]
fn update_rejected_by_immutable_metadata() {
    let asset = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let (f, _payer) = setup(
        asset,
        AssetSpec::new(owner)
            .plugin(
                Plugin::ImmutableMetadata(ImmutableMetadata {}),
                Authority::UpdateAuthority,
            )
            .build(),
    );
    f.store(owner, payer_account(ACCOUNT_LAMPORTS));

    let result = f.run(&ix::update_v1(
        asset,
        None,
        owner,
        None,
        None,
        Some(LONG_NAME.to_string()),
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(f.asset(&asset).name, DEFAULT_ASSET_NAME);
}

// ===========================================================================
// UpdateV1 through delegates
// ===========================================================================

/// An asset-level `UpdateDelegate`, as its record authority and as an
/// additional delegate, may rename the asset.
///
/// JS: plugins/asset/updateDelegate.test.ts :: an updateDelegate can update an asset
/// JS: plugins/asset/updateDelegate.test.ts :: an updateDelegate additionalDelegate can update an asset
#[test]
fn update_v1_via_asset_update_delegate() {
    for use_additional_delegate in [false, true] {
        let asset = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let delegate = Pubkey::new_unique();
        let (plugin, record_authority) = if use_additional_delegate {
            (update_delegate(&[delegate]), Authority::UpdateAuthority)
        } else {
            (
                update_delegate(&[]),
                Authority::Address { address: delegate },
            )
        };
        let (f, _payer) = setup(
            asset,
            AssetSpec::new(owner)
                .plugin(plugin.clone(), record_authority)
                .build(),
        );
        f.store(delegate, payer_account(ACCOUNT_LAMPORTS));

        f.run_ok(&ix::update_v1(
            asset,
            None,
            delegate,
            None,
            None,
            Some(LONG_NAME.to_string()),
            None,
            None,
        ));

        let account = f.account(&asset);
        assert_eq!(read_asset(&account).name, LONG_NAME);
        assert_plugins(&account, &[(plugin, record_authority)]);
    }
}

/// A collection-level `UpdateDelegate` may rename a member asset.
///
/// JS: plugins/collection/updateDelegate.test.ts :: an updateDelegate on collection can update an asset
/// JS: plugins/collection/updateDelegate.test.ts :: an updateDelegate additionalDelegate on collection can update an asset
#[test]
fn update_v1_via_collection_update_delegate() {
    for use_additional_delegate in [false, true] {
        let authority = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let delegate = Pubkey::new_unique();
        let record = if use_additional_delegate {
            (update_delegate(&[delegate]), Authority::UpdateAuthority)
        } else {
            (
                update_delegate(&[]),
                Authority::Address { address: delegate },
            )
        };
        let (f, asset, collection) = asset_in_collection(authority, owner, &[record]);
        f.store(delegate, payer_account(ACCOUNT_LAMPORTS));

        f.run_ok(&ix::update_v1(
            asset,
            Some(collection),
            delegate,
            None,
            None,
            Some(LONG_NAME.to_string()),
            Some(LONG_URI.to_string()),
            None,
        ));

        let updated = f.asset(&asset);
        assert_eq!(updated.name, LONG_NAME);
        assert_eq!(updated.uri, LONG_URI);
        assert_eq!(
            updated.update_authority,
            UpdateAuthority::Collection(collection),
            "renaming must not move the asset"
        );
        assert_eq!(f.collection(&collection).current_size, 1);
    }
}

/// The collection update authority may rename a member asset (the asset itself
/// abstains, the collection approves).
#[test]
fn update_v1_renames_member_asset_as_collection_authority() {
    let authority = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let (f, asset, collection) = asset_in_collection(authority, owner, &[]);
    f.store(authority, payer_account(ACCOUNT_LAMPORTS));

    f.run_ok(&ix::update_v1(
        asset,
        Some(collection),
        authority,
        None,
        None,
        Some(SHORT_NAME.to_string()),
        None,
        None,
    ));

    assert_eq!(f.asset(&asset).name, SHORT_NAME);
}

/// With no `new_name`, `new_uri` or `new_update_authority`, `dirty` stays false
/// and `process_update` is never called: the account is untouched, but the
/// permission check still has to pass.
#[test]
fn update_v1_without_arguments_is_a_no_op() {
    let asset = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let (f, _payer) = setup(
        asset,
        AssetSpec::new(owner)
            .plugin(attributes(), Authority::UpdateAuthority)
            .build(),
    );
    f.store(owner, payer_account(ACCOUNT_LAMPORTS));
    let before = f.account(&asset);

    f.run_ok(&ix::update_v1(
        asset, None, owner, None, None, None, None, None,
    ));

    let after = f.account(&asset);
    assert_eq!(
        after.data, before.data,
        "nothing should have been rewritten"
    );
    assert_eq!(after.lamports, before.lamports);

    // The same call from a stranger is still rejected.
    let stranger = f.fund(ACCOUNT_LAMPORTS);
    let result = f.run(&ix::update_v1(
        asset, None, stranger, None, None, None, None, None,
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
}

// ===========================================================================
// UpdateV2: removing an asset from a collection
// ===========================================================================

/// Stores an asset in a collection of size 1 and returns (fixture, asset,
/// collection). The collection update authority is `collection_authority`.
fn asset_in_collection(
    collection_authority: Pubkey,
    owner: Pubkey,
    collection_plugins: &[(Plugin, Authority)],
) -> (Fixture, Pubkey, Pubkey) {
    let f = Fixture::new();
    let asset = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Collection(collection))
            .build(),
    );
    let mut spec = CollectionSpec::new(collection_authority).sizes(1, 1);
    for (plugin, authority) in collection_plugins {
        spec = spec.plugin(plugin.clone(), *authority);
    }
    f.store(collection, spec.build());
    (f, asset, collection)
}

/// JS: updateV2.test.ts :: it can remove an asset from a collection using update
#[test]
fn update_v2_removes_asset_from_collection() {
    let authority = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let (f, asset, collection) = asset_in_collection(authority, owner, &[]);
    f.store(authority, payer_account(ACCOUNT_LAMPORTS));

    f.run_ok(&ix::update_v2(
        asset,
        Some(collection),
        authority,
        None,
        None,
        None,
        None,
        None,
        Some(client::UpdateAuthority::Address(authority)),
    ));

    assert_eq!(
        f.asset(&asset).update_authority,
        UpdateAuthority::Address(authority)
    );
    assert_eq!(
        f.collection(&collection).current_size,
        0,
        "the collection size must be decremented"
    );
    assert_eq!(
        f.collection(&collection).num_minted,
        1,
        "num_minted is not touched by a removal"
    );
}

/// JS: updateV2.test.ts :: it cannot remove an asset from a collection if not collection update auth
#[test]
fn update_v2_remove_rejects_wrong_authority() {
    let authority = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let (f, asset, collection) = asset_in_collection(authority, owner, &[]);
    let stranger = Pubkey::new_unique();
    f.store(stranger, payer_account(ACCOUNT_LAMPORTS));

    let result = f.run(&ix::update_v2(
        asset,
        Some(collection),
        stranger,
        None,
        None,
        None,
        None,
        None,
        Some(client::UpdateAuthority::Address(stranger)),
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_eq!(f.collection(&collection).current_size, 1);
}

/// JS: updateV2.test.ts :: it cannot remove an asset from a collection when missing collection account
#[test]
fn update_v2_remove_rejects_missing_collection() {
    let authority = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let (f, asset, _collection) = asset_in_collection(authority, owner, &[]);
    f.store(authority, payer_account(ACCOUNT_LAMPORTS));

    let result = f.run(&ix::update_v2(
        asset,
        None,
        authority,
        None,
        None,
        None,
        None,
        None,
        Some(client::UpdateAuthority::Address(authority)),
    ));
    assert_core_err(&result, MplCoreError::MissingCollection);
}

/// JS: updateV2.test.ts :: it cannot remove an asset from a collection when using incorrect collection account
#[test]
fn update_v2_remove_rejects_incorrect_collection() {
    let authority = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let (f, asset, _collection) = asset_in_collection(authority, owner, &[]);
    let other = Pubkey::new_unique();
    f.store(other, CollectionSpec::new(authority).build());
    f.store(authority, payer_account(ACCOUNT_LAMPORTS));

    let result = f.run(&ix::update_v2(
        asset,
        Some(other),
        authority,
        None,
        None,
        None,
        None,
        None,
        Some(client::UpdateAuthority::Address(authority)),
    ));
    assert_core_err(&result, MplCoreError::InvalidCollection);
}

/// JS: updateV2.test.ts :: it can remove an asset from collection using update delegate
#[test]
fn update_v2_removes_asset_using_collection_update_delegate() {
    let authority = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let delegate = Pubkey::new_unique();
    let (f, asset, collection) = asset_in_collection(
        authority,
        owner,
        &[(
            update_delegate(&[]),
            Authority::Address { address: delegate },
        )],
    );
    f.store(delegate, payer_account(ACCOUNT_LAMPORTS));

    f.run_ok(&ix::update_v2(
        asset,
        Some(collection),
        delegate,
        None,
        None,
        None,
        None,
        None,
        Some(client::UpdateAuthority::Address(delegate)),
    ));

    assert_eq!(
        f.asset(&asset).update_authority,
        UpdateAuthority::Address(delegate)
    );
    assert_eq!(f.collection(&collection).current_size, 0);
}

/// JS: updateV2.test.ts :: it can remove an asset from collection using additional update delegate
#[test]
fn update_v2_removes_asset_using_additional_delegate() {
    let authority = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let delegate = Pubkey::new_unique();
    let (f, asset, collection) = asset_in_collection(
        authority,
        owner,
        &[(update_delegate(&[delegate]), Authority::UpdateAuthority)],
    );
    f.store(delegate, payer_account(ACCOUNT_LAMPORTS));

    f.run_ok(&ix::update_v2(
        asset,
        Some(collection),
        delegate,
        None,
        None,
        None,
        None,
        None,
        Some(client::UpdateAuthority::Address(delegate)),
    ));

    assert_eq!(f.collection(&collection).current_size, 0);
}

/// An `UpdateDelegate` on the *asset* may not move the asset out of its
/// collection, even though the same plugin on the collection may
/// (`update_delegate.rs:181-198`).
///
/// JS: updateV2.test.ts :: it cannot remove an asset from collection using update delegate on the asset
#[test]
fn update_v2_remove_rejects_asset_level_update_delegate() {
    let authority = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let delegate = Pubkey::new_unique();
    let f = Fixture::new();
    let asset = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Collection(collection))
            .plugin(
                update_delegate(&[]),
                Authority::Address { address: delegate },
            )
            .build(),
    );
    f.store(
        collection,
        CollectionSpec::new(authority).sizes(1, 1).build(),
    );
    f.store(delegate, payer_account(ACCOUNT_LAMPORTS));

    let result = f.run(&ix::update_v2(
        asset,
        Some(collection),
        delegate,
        None,
        None,
        None,
        None,
        None,
        Some(client::UpdateAuthority::Address(delegate)),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(f.collection(&collection).current_size, 1);
}

/// The collection authority may still remove the asset when the asset carries
/// an `UpdateDelegate` held by someone else.
///
/// JS: updateV2.test.ts :: it can remove an asset from collection as Collection authority with update delegate on asset
#[test]
fn update_v2_removes_asset_as_collection_authority_with_asset_delegate() {
    let authority = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let delegate = Pubkey::new_unique();
    let f = Fixture::new();
    let asset = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Collection(collection))
            .plugin(
                update_delegate(&[]),
                Authority::Address { address: delegate },
            )
            .build(),
    );
    f.store(
        collection,
        CollectionSpec::new(authority).sizes(1, 1).build(),
    );
    f.store(authority, payer_account(ACCOUNT_LAMPORTS));

    f.run_ok(&ix::update_v2(
        asset,
        Some(collection),
        authority,
        None,
        None,
        None,
        None,
        None,
        Some(client::UpdateAuthority::Address(authority)),
    ));

    assert_eq!(f.collection(&collection).current_size, 0);
    assert_plugins(
        &f.account(&asset),
        &[(
            update_delegate(&[]),
            Authority::Address { address: delegate },
        )],
    );
}

// ===========================================================================
// UpdateV2: adding an asset to a collection
// ===========================================================================

/// JS: updateV2.test.ts :: it can add asset to collection using update
#[test]
fn update_v2_adds_asset_to_collection() {
    let authority = Pubkey::new_unique();
    let f = Fixture::new();
    let asset = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    f.store(asset, AssetSpec::new(authority).build());
    f.store(
        collection,
        CollectionSpec::new(authority).sizes(5, 0).build(),
    );
    f.store(authority, payer_account(ACCOUNT_LAMPORTS));

    f.run_ok(&ix::update_v2(
        asset,
        None,
        authority,
        None,
        Some(collection),
        None,
        None,
        None,
        Some(client::UpdateAuthority::Collection(collection)),
    ));

    assert_eq!(
        f.asset(&asset).update_authority,
        UpdateAuthority::Collection(collection)
    );
    let updated = f.collection(&collection);
    assert_eq!(updated.current_size, 1, "the new collection grows by one");
    assert_eq!(
        updated.num_minted, 5,
        "moving an asset in does not mint a new one"
    );
}

/// JS: updateV2.test.ts :: it cannot add asset to collection when missing collection account
#[test]
fn update_v2_add_rejects_missing_new_collection() {
    let authority = Pubkey::new_unique();
    let f = Fixture::new();
    let asset = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    f.store(asset, AssetSpec::new(authority).build());
    f.store(collection, CollectionSpec::new(authority).build());
    f.store(authority, payer_account(ACCOUNT_LAMPORTS));

    let result = f.run(&ix::update_v2(
        asset,
        None,
        authority,
        None,
        None,
        None,
        None,
        None,
        Some(client::UpdateAuthority::Collection(collection)),
    ));
    assert_core_err(&result, MplCoreError::MissingCollection);
}

/// JS: updateV2.test.ts :: it cannot add asset to collection when using incorrect collection account
#[test]
fn update_v2_add_rejects_mismatched_new_collection() {
    let authority = Pubkey::new_unique();
    let f = Fixture::new();
    let asset = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    let other = Pubkey::new_unique();
    f.store(asset, AssetSpec::new(authority).build());
    f.store(collection, CollectionSpec::new(authority).build());
    f.store(other, CollectionSpec::new(authority).build());
    f.store(authority, payer_account(ACCOUNT_LAMPORTS));

    let result = f.run(&ix::update_v2(
        asset,
        None,
        authority,
        None,
        Some(other),
        None,
        None,
        None,
        Some(client::UpdateAuthority::Collection(collection)),
    ));
    assert_core_err(&result, MplCoreError::InvalidCollection);
}

/// The signer is the new collection's authority but not the asset's, so the
/// asset-level check fails first.
///
/// JS: updateV2.test.ts :: it cannot add asset to collection using only new collection authority
#[test]
fn update_v2_add_rejects_collection_only_authority() {
    let asset_authority = Pubkey::new_unique();
    let collection_authority = Pubkey::new_unique();
    let f = Fixture::new();
    let asset = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    f.store(asset, AssetSpec::new(asset_authority).build());
    f.store(
        collection,
        CollectionSpec::new(collection_authority).build(),
    );
    f.store(collection_authority, payer_account(ACCOUNT_LAMPORTS));

    let result = f.run(&ix::update_v2(
        asset,
        None,
        collection_authority,
        None,
        Some(collection),
        None,
        None,
        None,
        Some(client::UpdateAuthority::Collection(collection)),
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_eq!(f.collection(&collection).current_size, 0);
}

/// The signer is the asset's update authority but not the new collection's, so
/// the new-collection branch rejects (`update.rs:231-236`).
///
/// JS: updateV2.test.ts :: it cannot add asset to collection if not both asset and collection auth
#[test]
fn update_v2_add_rejects_asset_only_authority() {
    let asset_authority = Pubkey::new_unique();
    let collection_authority = Pubkey::new_unique();
    let f = Fixture::new();
    let asset = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    f.store(asset, AssetSpec::new(asset_authority).build());
    f.store(
        collection,
        CollectionSpec::new(collection_authority).build(),
    );
    f.store(asset_authority, payer_account(ACCOUNT_LAMPORTS));

    let result = f.run(&ix::update_v2(
        asset,
        None,
        asset_authority,
        None,
        Some(collection),
        None,
        None,
        None,
        Some(client::UpdateAuthority::Collection(collection)),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(f.collection(&collection).current_size, 0);
}

/// JS: updateV2.test.ts :: it can add asset to collection using additional update delegate on new collection
#[test]
fn update_v2_adds_asset_using_additional_delegate_on_new_collection() {
    let authority = Pubkey::new_unique();
    let collection_authority = Pubkey::new_unique();
    let f = Fixture::new();
    let asset = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    f.store(asset, AssetSpec::new(authority).build());
    f.store(
        collection,
        CollectionSpec::new(collection_authority)
            .plugin(update_delegate(&[authority]), Authority::UpdateAuthority)
            .build(),
    );
    f.store(authority, payer_account(ACCOUNT_LAMPORTS));

    f.run_ok(&ix::update_v2(
        asset,
        None,
        authority,
        None,
        Some(collection),
        None,
        None,
        None,
        Some(client::UpdateAuthority::Collection(collection)),
    ));

    assert_eq!(
        f.asset(&asset).update_authority,
        UpdateAuthority::Collection(collection)
    );
    assert_eq!(f.collection(&collection).current_size, 1);
}

/// The new collection's `UpdateDelegate` record authority may also approve the
/// move (`update.rs:208-230`, the `assert_collection_authority` clause).
#[test]
fn update_v2_adds_asset_using_update_delegate_record_authority() {
    let authority = Pubkey::new_unique();
    let collection_authority = Pubkey::new_unique();
    let f = Fixture::new();
    let asset = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    f.store(asset, AssetSpec::new(authority).build());
    f.store(
        collection,
        CollectionSpec::new(collection_authority)
            .plugin(
                update_delegate(&[]),
                Authority::Address { address: authority },
            )
            .build(),
    );
    f.store(authority, payer_account(ACCOUNT_LAMPORTS));

    f.run_ok(&ix::update_v2(
        asset,
        None,
        authority,
        None,
        Some(collection),
        None,
        None,
        None,
        Some(client::UpdateAuthority::Collection(collection)),
    ));

    assert_eq!(f.collection(&collection).current_size, 1);
}

/// With an `UpdateDelegate` present on the new collection but held by someone
/// else, a signer who is neither delegate nor collection authority is rejected
/// at `update.rs:229`.
#[test]
fn update_v2_add_rejects_non_delegate_when_update_delegate_present() {
    let asset_authority = Pubkey::new_unique();
    let collection_authority = Pubkey::new_unique();
    let other_delegate = Pubkey::new_unique();
    let f = Fixture::new();
    let asset = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    f.store(asset, AssetSpec::new(asset_authority).build());
    f.store(
        collection,
        CollectionSpec::new(collection_authority)
            .plugin(
                update_delegate(&[]),
                Authority::Address {
                    address: other_delegate,
                },
            )
            .build(),
    );
    f.store(asset_authority, payer_account(ACCOUNT_LAMPORTS));

    let result = f.run(&ix::update_v2(
        asset,
        None,
        asset_authority,
        None,
        Some(collection),
        None,
        None,
        None,
        Some(client::UpdateAuthority::Collection(collection)),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
}

/// JS: updateV2.test.ts :: it cannot add asset to collection if new collection contains permanent freeze delegate
#[test]
fn update_v2_add_rejects_collection_with_permanent_delegates() {
    for permanent in [
        Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen: false }),
        Plugin::PermanentTransferDelegate(PermanentTransferDelegate {}),
        Plugin::PermanentBurnDelegate(PermanentBurnDelegate {}),
    ] {
        let authority = Pubkey::new_unique();
        let f = Fixture::new();
        let asset = Pubkey::new_unique();
        let collection = Pubkey::new_unique();
        f.store(asset, AssetSpec::new(authority).build());
        f.store(
            collection,
            CollectionSpec::new(authority)
                .plugin(permanent.clone(), Authority::UpdateAuthority)
                .build(),
        );
        f.store(authority, payer_account(ACCOUNT_LAMPORTS));

        let result = f.run(&ix::update_v2(
            asset,
            None,
            authority,
            None,
            Some(collection),
            None,
            None,
            None,
            Some(client::UpdateAuthority::Collection(collection)),
        ));
        assert_core_err(&result, MplCoreError::PermanentDelegatesPreventMove);
        assert_eq!(
            f.collection(&collection).current_size,
            0,
            "{:?} must not have been added to",
            PluginType::from(&permanent)
        );
    }
}

/// JS: updateV2.test.ts :: it can change an asset collection using same update authority
#[test]
fn update_v2_changes_collection() {
    let authority = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let (f, asset, old_collection) = asset_in_collection(authority, owner, &[]);
    let new_collection = Pubkey::new_unique();
    f.store(
        new_collection,
        CollectionSpec::new(authority).sizes(3, 3).build(),
    );
    f.store(authority, payer_account(ACCOUNT_LAMPORTS));

    f.run_ok(&ix::update_v2(
        asset,
        Some(old_collection),
        authority,
        None,
        Some(new_collection),
        None,
        None,
        None,
        Some(client::UpdateAuthority::Collection(new_collection)),
    ));

    assert_eq!(
        f.asset(&asset).update_authority,
        UpdateAuthority::Collection(new_collection)
    );
    assert_eq!(f.collection(&old_collection).current_size, 0);
    assert_eq!(f.collection(&new_collection).current_size, 4);
    assert_eq!(f.collection(&new_collection).num_minted, 3);
}

/// JS: updateV2.test.ts :: it cannot change an asset collection if not both asset and collection auth
#[test]
fn update_v2_change_collection_rejects_wrong_new_collection_authority() {
    let authority = Pubkey::new_unique();
    let owner = Pubkey::new_unique();
    let (f, asset, old_collection) = asset_in_collection(authority, owner, &[]);
    let new_collection = Pubkey::new_unique();
    f.store(
        new_collection,
        CollectionSpec::new(Pubkey::new_unique()).build(),
    );
    f.store(authority, payer_account(ACCOUNT_LAMPORTS));

    let result = f.run(&ix::update_v2(
        asset,
        Some(old_collection),
        authority,
        None,
        Some(new_collection),
        None,
        None,
        None,
        Some(client::UpdateAuthority::Collection(new_collection)),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(
        f.collection(&old_collection).current_size,
        1,
        "the failed move must not have decremented the old collection"
    );
}

/// `UpdateV2` also updates name and URI while moving the asset, going through
/// `process_update` with the new (longer) `UpdateAuthority::Collection`.
///
/// JS: updateV2.test.ts :: it can update an asset to be larger
#[test]
fn update_v2_updates_name_and_moves_asset() {
    let authority = Pubkey::new_unique();
    let f = Fixture::new();
    let asset = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    f.store(asset, AssetSpec::new(authority).build());
    f.store(collection, CollectionSpec::new(authority).build());
    f.store(authority, payer_account(ACCOUNT_LAMPORTS));

    f.run_ok(&ix::update_v2(
        asset,
        None,
        authority,
        None,
        Some(collection),
        None,
        Some(LONG_NAME.to_string()),
        Some(LONG_URI.to_string()),
        Some(client::UpdateAuthority::Collection(collection)),
    ));

    let updated = f.asset(&asset);
    assert_eq!(updated.name, LONG_NAME);
    assert_eq!(updated.uri, LONG_URI);
    assert_eq!(
        updated.update_authority,
        UpdateAuthority::Collection(collection)
    );
    assert_registry_consistent(&f.account(&asset));
}

// ===========================================================================
// UpdateCollectionV1
// ===========================================================================

/// The `CollectionV1` monomorphization of `process_update` on a bare account.
#[test]
fn update_collection_name_and_uri() {
    let collection = Pubkey::new_unique();
    let authority = Pubkey::new_unique();
    let (f, _payer) = setup(collection, CollectionSpec::new(authority).build());
    f.store(authority, payer_account(ACCOUNT_LAMPORTS));
    let len_before = f.account(&collection).data.len();

    f.run_ok(&ix::update_collection_v1(
        collection,
        authority,
        None,
        None,
        None,
        Some(LONG_NAME.to_string()),
        Some(LONG_URI.to_string()),
    ));

    let updated = f.collection(&collection);
    assert_eq!(updated.name, LONG_NAME);
    assert_eq!(updated.uri, LONG_URI);
    assert_eq!(updated.update_authority, authority);
    assert!(f.account(&collection).data.len() > len_before);
    assert_registry_consistent(&f.account(&collection));
}

/// The collection variant of `process_update` is a separate monomorphization;
/// grow and shrink with a trailing plugin, the Mollusk port of
/// `plugin_shrink_corruption.rs::test_update_collection_v1_shrink_name_uri_preserves_plugin`.
#[test]
fn update_collection_with_plugins_grows_and_shrinks() {
    let collection = Pubkey::new_unique();
    let authority = Pubkey::new_unique();
    let plugins = vec![(attributes(), Authority::UpdateAuthority)];
    let (f, _payer) = setup(
        collection,
        CollectionSpec::new(authority)
            .plugin(attributes(), Authority::UpdateAuthority)
            .build(),
    );
    f.store(authority, payer_account(ACCOUNT_LAMPORTS));
    let len_before = f.account(&collection).data.len();

    // Grow.
    f.run_ok(&ix::update_collection_v1(
        collection,
        authority,
        None,
        None,
        None,
        Some(LONG_NAME.to_string()),
        Some(LONG_URI.to_string()),
    ));
    let grown = f.account(&collection);
    assert_eq!(read_collection(&grown).name, LONG_NAME);
    assert!(grown.data.len() > len_before);
    assert_collection_plugins(&grown, &plugins);

    // Shrink back.
    f.run_ok(&ix::update_collection_v1(
        collection,
        authority,
        None,
        None,
        None,
        Some(SHORT_NAME.to_string()),
        Some(SHORT_URI.to_string()),
    ));
    let shrunk = f.account(&collection);
    assert_eq!(read_collection(&shrunk).name, SHORT_NAME);
    assert_eq!(read_collection(&shrunk).uri, SHORT_URI);
    assert!(shrunk.data.len() < grown.data.len());
    assert_collection_plugins(&shrunk, &plugins);
}

/// `UpdateCollectionV1` with account 3 present sets a new update authority.
#[test]
fn update_collection_changes_update_authority() {
    let collection = Pubkey::new_unique();
    let authority = Pubkey::new_unique();
    let new_authority = Pubkey::new_unique();
    let (f, _payer) = setup(collection, CollectionSpec::new(authority).build());
    f.store(authority, payer_account(ACCOUNT_LAMPORTS));

    f.run_ok(&ix::update_collection_v1(
        collection,
        authority,
        None,
        Some(new_authority),
        None,
        None,
        None,
    ));

    assert_eq!(f.collection(&collection).update_authority, new_authority);
}

/// JS: update.test.ts :: it cannot use an invalid system program for collections
#[test]
fn update_collection_rejects_invalid_system_program() {
    let collection = Pubkey::new_unique();
    let authority = Pubkey::new_unique();
    let (f, _payer) = setup(collection, CollectionSpec::new(authority).build());
    f.store(authority, payer_account(ACCOUNT_LAMPORTS));

    let mut ix = ix::update_collection_v1(
        collection,
        authority,
        None,
        None,
        None,
        Some(LONG_NAME.to_string()),
        None,
    );
    // Account 4 is the system program on UpdateCollectionV1.
    set_account(
        &mut ix,
        4,
        solana_program::instruction::AccountMeta::new_readonly(Pubkey::new_unique(), false),
    );
    assert_core_err(&f.run(&ix), MplCoreError::InvalidSystemProgram);
}

/// JS: update.test.ts :: it cannot use an invalid noop program for collections
#[test]
fn update_collection_rejects_invalid_log_wrapper() {
    let collection = Pubkey::new_unique();
    let authority = Pubkey::new_unique();
    let (f, _payer) = setup(collection, CollectionSpec::new(authority).build());
    f.store(authority, payer_account(ACCOUNT_LAMPORTS));

    let ix = ix::update_collection_v1(
        collection,
        authority,
        None,
        None,
        Some(Pubkey::new_unique()),
        Some(LONG_NAME.to_string()),
        None,
    );
    assert_core_err(&f.run(&ix), MplCoreError::InvalidLogWrapperProgram);
}

/// A stranger cannot rename a collection; unlike the asset path, the
/// collection path reports `InvalidAuthority` rather than `NoApprovals`
/// (`utils/mod.rs`, see roadmap section 6).
#[test]
fn update_collection_rejects_wrong_authority() {
    let collection = Pubkey::new_unique();
    let authority = Pubkey::new_unique();
    let stranger = Pubkey::new_unique();
    let (f, _payer) = setup(collection, CollectionSpec::new(authority).build());
    f.store(stranger, payer_account(ACCOUNT_LAMPORTS));

    let result = f.run(&ix::update_collection_v1(
        collection,
        stranger,
        None,
        None,
        None,
        Some(LONG_NAME.to_string()),
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(f.collection(&collection).name, DEFAULT_COLLECTION_NAME);
}

/// JS: plugins/collection/immutableMetadata.test.ts :: it prevents both collection and asset from their meta updating when ImmutableMetadata is added
#[test]
fn update_collection_rejected_by_immutable_metadata() {
    let collection = Pubkey::new_unique();
    let authority = Pubkey::new_unique();
    let (f, _payer) = setup(
        collection,
        CollectionSpec::new(authority)
            .plugin(
                Plugin::ImmutableMetadata(ImmutableMetadata {}),
                Authority::UpdateAuthority,
            )
            .build(),
    );
    f.store(authority, payer_account(ACCOUNT_LAMPORTS));

    let result = f.run(&ix::update_collection_v1(
        collection,
        authority,
        None,
        None,
        None,
        Some(LONG_NAME.to_string()),
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    // The name is the field the instruction tried to change, so it is the one
    // that proves the rejection took effect; the uri confirms nothing else moved.
    let unchanged = f.collection(&collection);
    assert_eq!(unchanged.name, DEFAULT_COLLECTION_NAME);
    assert_eq!(unchanged.uri, DEFAULT_URI);
}

/// A collection-level `UpdateDelegate` may rename the collection.
///
/// JS: plugins/collection/updateDelegate.test.ts :: it can update collection details as an updateDelegate additional delegate
#[test]
fn update_collection_via_update_delegate() {
    let collection = Pubkey::new_unique();
    let authority = Pubkey::new_unique();
    let delegate = Pubkey::new_unique();
    let (f, _payer) = setup(
        collection,
        CollectionSpec::new(authority)
            .plugin(
                update_delegate(&[]),
                Authority::Address { address: delegate },
            )
            .build(),
    );
    f.store(delegate, payer_account(ACCOUNT_LAMPORTS));

    f.run_ok(&ix::update_collection_v1(
        collection,
        delegate,
        None,
        None,
        None,
        Some(LONG_NAME.to_string()),
        None,
    ));

    assert_eq!(f.collection(&collection).name, LONG_NAME);
}

// ===========================================================================
// UpdateCollectionInfoV1
// ===========================================================================

/// Only the Bubblegum PDA may adjust the counters; Mollusk verifies no
/// signatures, so the PDA is simply listed as a signer.
#[test]
fn update_collection_info_mint_add_and_remove() {
    let collection = Pubkey::new_unique();
    let authority = Pubkey::new_unique();
    let (f, _payer) = setup(collection, CollectionSpec::new(authority).build());

    f.run_ok(&ix::update_collection_info_v1(
        collection,
        BUBBLEGUM_SIGNER,
        client::UpdateType::Mint,
        7,
    ));
    let minted = f.collection(&collection);
    assert_eq!(minted.num_minted, 7);
    assert_eq!(minted.current_size, 7);

    f.run_ok(&ix::update_collection_info_v1(
        collection,
        BUBBLEGUM_SIGNER,
        client::UpdateType::Add,
        3,
    ));
    let added = f.collection(&collection);
    assert_eq!(added.num_minted, 7, "Add does not mint");
    assert_eq!(added.current_size, 10);

    // `Remove` saturates at zero instead of failing (roadmap section 6).
    f.run_ok(&ix::update_collection_info_v1(
        collection,
        BUBBLEGUM_SIGNER,
        client::UpdateType::Remove,
        100,
    ));
    let removed = f.collection(&collection);
    assert_eq!(removed.num_minted, 7);
    assert_eq!(removed.current_size, 0, "Remove saturates at zero");
}

/// Rust client: update_collection_info.rs::test_cannot_update_collection_info_with_incorrect_signer
#[test]
fn update_collection_info_rejects_wrong_signer_and_non_signer() {
    let collection = Pubkey::new_unique();
    let authority = Pubkey::new_unique();
    let (f, _payer) = setup(collection, CollectionSpec::new(authority).build());

    let stranger = Pubkey::new_unique();
    let result = f.run(&ix::update_collection_info_v1(
        collection,
        stranger,
        client::UpdateType::Mint,
        1,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    let mut ix =
        ix::update_collection_info_v1(collection, BUBBLEGUM_SIGNER, client::UpdateType::Mint, 1);
    unsign(&mut ix, &BUBBLEGUM_SIGNER);
    assert_program_err(
        &f.run(&ix),
        solana_program::program_error::ProgramError::MissingRequiredSignature,
    );
    assert_eq!(f.collection(&collection).num_minted, 0);
}
