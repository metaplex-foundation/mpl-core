//! Internal plugin validations.
//!
//! Ports of `clients/js/test/plugins/asset/*.test.ts` and
//! `clients/js/test/plugins/collection/*.test.ts`. Each test drives a real
//! instruction so that the plugin callback is reached through the same
//! `check_*` table and `validate_plugin_checks` aggregation the chain uses,
//! and asserts the exact `MplCoreError` (which is what identifies the arm
//! that fired) plus the resulting on-chain state.
//!
//! Assets and collections are laid out directly with [`AssetSpec`] /
//! [`CollectionSpec`] wherever the starting state is a given; the flows that
//! the JS tests build step by step (approve then revoke, freeze then remove)
//! run every step through the program on a [`Fixture`] so the intermediate
//! post-state is asserted too.

use {
    crate::common::*,
    mpl_core::types as client,
    mpl_core_program::{
        error::MplCoreError,
        plugins::{
            AddBlocker, Attributes, Autograph, AutographSignature, BubblegumV2, BurnDelegate,
            Creator, Edition, FreezeDelegate, FreezeExecute, ImmutableMetadata,
            PermanentBurnDelegate, PermanentFreezeDelegate, PermanentFreezeExecute,
            PermanentTransferDelegate, Plugin, PluginType, Royalties, RuleSet, TransferDelegate,
            UpdateDelegate, VerifiedCreators, VerifiedCreatorsSignature,
        },
        state::{Authority, UpdateAuthority},
    },
    solana_account::Account,
    solana_program::{pubkey::Pubkey, rent::Rent},
};

// ---------------------------------------------------------------------------
// Conversions and small builders
// ---------------------------------------------------------------------------

/// The generated client's view of a program plugin value.
fn client_plugin(plugin: &Plugin) -> client::Plugin {
    convert(plugin)
}

/// The generated client's view of a `PluginType`.
fn client_type(plugin_type: PluginType) -> client::PluginType {
    convert(&plugin_type)
}

/// The generated client's view of an `Authority`.
fn client_authority(authority: &Authority) -> client::PluginAuthority {
    convert(authority)
}

/// A `(plugin, authority)` pair for the `plugins` argument of `CreateV2`.
fn plugin_pair(plugin: &Plugin, authority: Option<Authority>) -> client::PluginAuthorityPair {
    client::PluginAuthorityPair {
        plugin: client_plugin(plugin),
        authority: authority.as_ref().map(client_authority),
    }
}

/// `Authority::Address { address }`.
fn address(address: Pubkey) -> Authority {
    Authority::Address { address }
}

/// A `FreezeDelegate` plugin.
fn freeze(frozen: bool) -> Plugin {
    Plugin::FreezeDelegate(FreezeDelegate { frozen })
}

/// A `FreezeExecute` plugin.
fn freeze_execute(frozen: bool) -> Plugin {
    Plugin::FreezeExecute(FreezeExecute { frozen })
}

/// A `PermanentFreezeDelegate` plugin.
fn permanent_freeze(frozen: bool) -> Plugin {
    Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen })
}

/// A `PermanentFreezeExecute` plugin.
fn permanent_freeze_execute(frozen: bool) -> Plugin {
    Plugin::PermanentFreezeExecute(PermanentFreezeExecute { frozen })
}

/// An `UpdateDelegate` plugin with the given additional delegates.
fn update_delegate(additional: &[Pubkey]) -> Plugin {
    Plugin::UpdateDelegate(UpdateDelegate {
        additional_delegates: additional.to_vec(),
    })
}

/// An `Attributes` plugin with no attributes.
fn attributes() -> Plugin {
    Plugin::Attributes(Attributes {
        attribute_list: vec![],
    })
}

/// A valid `Royalties` plugin: 500 bp, one 100% creator, the given rule set.
fn royalties(creator: Pubkey, rule_set: RuleSet) -> Plugin {
    Plugin::Royalties(Royalties {
        basis_points: 500,
        creators: vec![Creator {
            address: creator,
            percentage: 100,
        }],
        rule_set,
    })
}

/// The authority and plugin stored under `plugin_type`; panics when absent.
fn stored(account: &Account, plugin_type: PluginType) -> (Authority, Plugin) {
    read_plugin(account, plugin_type)
        .unwrap_or_else(|| panic!("{plugin_type:?} should be in the registry"))
}

/// Asserts the registry is consistent and `plugin_type` is stored with the
/// given authority and value.
fn assert_plugin(
    account: &Account,
    plugin_type: PluginType,
    authority: &Authority,
    plugin: &Plugin,
) {
    assert_registry_consistent(account);
    let (stored_authority, stored_plugin) = stored(account, plugin_type);
    assert_eq!(
        &stored_authority, authority,
        "{plugin_type:?} has the wrong registry authority"
    );
    assert_eq!(
        &stored_plugin, plugin,
        "{plugin_type:?} holds the wrong data"
    );
}

/// A funded payer, an owner and an update authority, all distinct.
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

/// Stores an asset owned by `owner` with update authority `update_authority`
/// and the given plugins, and returns its key.
fn store_asset(
    f: &Fixture,
    owner: Pubkey,
    update_authority: Pubkey,
    plugins: &[(Plugin, Authority)],
) -> Pubkey {
    let asset = Pubkey::new_unique();
    let mut spec =
        AssetSpec::new(owner).update_authority(UpdateAuthority::Address(update_authority));
    for (plugin, authority) in plugins {
        spec = spec.plugin(plugin.clone(), *authority);
    }
    f.store(asset, spec.build());
    asset
}

/// Stores a collection controlled by `update_authority` with the given
/// plugins, and returns its key.
fn store_collection(
    f: &Fixture,
    update_authority: Pubkey,
    plugins: &[(Plugin, Authority)],
) -> Pubkey {
    let collection = Pubkey::new_unique();
    let mut spec = CollectionSpec::new(update_authority).sizes(1, 1);
    for (plugin, authority) in plugins {
        spec = spec.plugin(plugin.clone(), *authority);
    }
    f.store(collection, spec.build());
    collection
}

/// Stores a collection and a member asset (its update authority is the
/// collection) and returns `(collection, asset)`.
fn store_collection_with_member(
    f: &Fixture,
    update_authority: Pubkey,
    owner: Pubkey,
    collection_plugins: &[(Plugin, Authority)],
    asset_plugins: &[(Plugin, Authority)],
) -> (Pubkey, Pubkey) {
    let collection = store_collection(f, update_authority, collection_plugins);
    let asset = Pubkey::new_unique();
    let mut spec = AssetSpec::new(owner).update_authority(UpdateAuthority::Collection(collection));
    for (plugin, authority) in asset_plugins {
        spec = spec.plugin(plugin.clone(), *authority);
    }
    f.store(asset, spec.build());
    (collection, asset)
}

// ===========================================================================
// updateDelegateRevokeBug.test.ts
//
// The regression suite for the operator-precedence fix in
// `UpdateDelegate::validate_revoke_plugin_authority`: the manager check now
// binds to both branches, so an UpdateDelegate (whether it is the update
// authority through `resolved_authorities`, or a delegated address) can only
// revoke authority on UpdateAuthority-managed plugins.
// ===========================================================================

/// JS: plugins/asset/updateDelegateRevokeBug.test.ts :: it should NOT allow update authority to revoke authority on owner-managed plugins via UpdateDelegate
#[test]
fn update_delegate_cannot_revoke_freeze_delegate_authority() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let freeze_authority = Pubkey::new_unique();

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[
            (update_delegate(&[]), Authority::UpdateAuthority),
            (freeze(false), Authority::Owner),
        ],
    );

    // The owner delegates the FreezeDelegate authority: this is the state the
    // JS test sets up before the revoke attempt.
    f.run_ok(&ix::approve_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_type(PluginType::FreezeDelegate),
        client_authority(&address(freeze_authority)),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeDelegate,
        &address(freeze_authority),
        &freeze(false),
    );

    // The update authority is in `resolved_authorities` for the
    // UpdateDelegate record, but FreezeDelegate is owner-managed, so nothing
    // approves the revoke.
    let result = f.run(&ix::revoke_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::FreezeDelegate),
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);

    // The delegated authority is still in place.
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeDelegate,
        &address(freeze_authority),
        &freeze(false),
    );
}

/// JS: plugins/asset/updateDelegateRevokeBug.test.ts :: it should allow update authority to revoke authority on UpdateAuthority-managed plugins via UpdateDelegate
#[test]
fn update_delegate_can_revoke_edition_authority() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let edition_authority = Pubkey::new_unique();
    let edition = Plugin::Edition(Edition { number: 1 });

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[
            (update_delegate(&[]), Authority::UpdateAuthority),
            (edition.clone(), Authority::UpdateAuthority),
        ],
    );

    f.run_ok(&ix::approve_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::Edition),
        client_authority(&address(edition_authority)),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::Edition,
        &address(edition_authority),
        &edition,
    );

    // Edition is UpdateAuthority-managed, so the UpdateDelegate approves.
    f.run_ok(&ix::revoke_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::Edition),
    ));
    // Revoking resets the authority to the plugin type's manager.
    assert_plugin(
        &f.account(&asset),
        PluginType::Edition,
        &Authority::UpdateAuthority,
        &edition,
    );
}

/// JS: plugins/asset/updateDelegateRevokeBug.test.ts :: it should NOT allow update authority to revoke authority on TransferDelegate via UpdateDelegate
#[test]
fn update_delegate_cannot_revoke_transfer_delegate_authority() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let transfer_authority = Pubkey::new_unique();

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[
            (update_delegate(&[]), Authority::UpdateAuthority),
            (
                Plugin::TransferDelegate(TransferDelegate {}),
                Authority::Owner,
            ),
        ],
    );

    f.run_ok(&ix::approve_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_type(PluginType::TransferDelegate),
        client_authority(&address(transfer_authority)),
    ));

    let result = f.run(&ix::revoke_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::TransferDelegate),
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_plugin(
        &f.account(&asset),
        PluginType::TransferDelegate,
        &address(transfer_authority),
        &Plugin::TransferDelegate(TransferDelegate {}),
    );
}

/// JS: plugins/asset/updateDelegateRevokeBug.test.ts :: it should NOT allow delegated update delegate to revoke authority on owner-managed plugins
#[test]
fn delegated_update_delegate_cannot_revoke_freeze_delegate_authority() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let update_delegate_authority = f.fund(ACCOUNT_LAMPORTS);
    let freeze_authority = Pubkey::new_unique();

    // The owner is also the update authority here (as in the JS test), so the
    // only thing that could approve the revoke is the UpdateDelegate record
    // delegated to `update_delegate_authority`.
    let asset = store_asset(
        &f,
        actors.owner,
        actors.owner,
        &[
            (update_delegate(&[]), address(update_delegate_authority)),
            (freeze(false), Authority::Owner),
        ],
    );

    f.run_ok(&ix::approve_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_type(PluginType::FreezeDelegate),
        client_authority(&address(freeze_authority)),
    ));

    let result = f.run(&ix::revoke_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(update_delegate_authority),
        None,
        client_type(PluginType::FreezeDelegate),
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeDelegate,
        &address(freeze_authority),
        &freeze(false),
    );
}

/// JS: plugins/asset/updateDelegateRevokeBug.test.ts :: it should allow the owner to revoke authority on owner-managed plugins
#[test]
fn owner_can_revoke_freeze_delegate_authority() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let freeze_authority = Pubkey::new_unique();

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(freeze(false), Authority::Owner)],
    );

    f.run_ok(&ix::approve_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_type(PluginType::FreezeDelegate),
        client_authority(&address(freeze_authority)),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeDelegate,
        &address(freeze_authority),
        &freeze(false),
    );
    let delegated_len = f.account(&asset).data.len();

    f.run_ok(&ix::revoke_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_type(PluginType::FreezeDelegate),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeDelegate,
        &Authority::Owner,
        &freeze(false),
    );
    // `Address` -> `Owner` drops the 32-byte pubkey from the registry record.
    assert_eq!(
        f.account(&asset).data.len(),
        delegated_len - 32,
        "revoking an Address authority should shrink the registry by a pubkey"
    );
}

// ===========================================================================
// updateDelegate.test.ts
// ===========================================================================

/// JS: plugins/asset/updateDelegate.test.ts :: an updateDelegate can update an asset
#[test]
fn update_delegate_can_update_an_asset() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(update_delegate(&[]), address(delegate))],
    );

    f.run_ok(&ix::update_v1(
        asset,
        None,
        actors.payer,
        Some(delegate),
        None,
        Some("Renamed".to_string()),
        None,
        None,
    ));
    assert_eq!(f.asset(&asset).name, "Renamed");
    assert_registry_consistent(&f.account(&asset));
}

/// JS: plugins/asset/updateDelegate.test.ts :: an updateDelegate additionalDelegate can update an asset
#[test]
fn update_delegate_additional_delegate_can_update_an_asset() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let additional = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(update_delegate(&[additional]), Authority::UpdateAuthority)],
    );

    f.run_ok(&ix::update_v1(
        asset,
        None,
        actors.payer,
        Some(additional),
        None,
        None,
        Some("https://example.com/new".to_string()),
        None,
    ));
    assert_eq!(f.asset(&asset).uri, "https://example.com/new");
}

/// JS: plugins/asset/updateDelegate.test.ts :: an updateDelegate cannot update an asset after delegate authority revoked
#[test]
fn update_delegate_cannot_update_after_revoke() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(update_delegate(&[]), address(delegate))],
    );

    // The update authority revokes the delegate: the record authority falls
    // back to `UpdateAuthority`.
    f.run_ok(&ix::revoke_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::UpdateDelegate),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::UpdateDelegate,
        &Authority::UpdateAuthority,
        &update_delegate(&[]),
    );

    let result = f.run(&ix::update_v1(
        asset,
        None,
        actors.payer,
        Some(delegate),
        None,
        Some("Renamed".to_string()),
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_eq!(f.asset(&asset).name, "Test Asset");
}

/// JS: plugins/asset/updateDelegate.test.ts :: an updateDelegate can add a plugin to an asset
#[test]
fn update_delegate_can_add_authority_managed_plugin() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(update_delegate(&[]), address(delegate))],
    );

    f.run_ok(&ix::add_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(delegate),
        None,
        client_plugin(&attributes()),
        None,
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::Attributes,
        &Authority::UpdateAuthority,
        &attributes(),
    );
}

/// JS: plugins/asset/updateDelegate.test.ts :: it cannot add updateDelegate plugin with additional delegate as additional delegate
#[test]
fn stranger_cannot_add_update_delegate_plugin() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let stranger = f.fund(ACCOUNT_LAMPORTS);

    // A bare asset: nothing on it can approve the add, and the signer is
    // neither the owner nor the update authority.
    let asset = store_asset(&f, actors.owner, actors.update_authority, &[]);

    let result = f.run(&ix::add_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(stranger),
        None,
        client_plugin(&update_delegate(&[stranger])),
        None,
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
}

/// JS: plugins/asset/updateDelegate.test.ts :: an updateDelegate can remove a plugin from an asset
#[test]
fn update_delegate_can_remove_authority_managed_plugin() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[
            (update_delegate(&[]), address(delegate)),
            (attributes(), Authority::UpdateAuthority),
        ],
    );

    f.run_ok(&ix::remove_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(delegate),
        None,
        client_type(PluginType::Attributes),
    ));
    let account = f.account(&asset);
    assert_registry_consistent(&account);
    assert!(
        read_plugin(&account, PluginType::Attributes).is_none(),
        "Attributes should be gone from the registry"
    );
    // The UpdateDelegate record survives the removal of the trailing plugin.
    assert_plugin(
        &account,
        PluginType::UpdateDelegate,
        &address(delegate),
        &update_delegate(&[]),
    );
}

/// JS: plugins/asset/updateDelegate.test.ts :: it can remove additional delegate as additional delegate if self
#[test]
fn additional_delegate_can_remove_itself() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let first = f.fund(ACCOUNT_LAMPORTS);
    let second = Pubkey::new_unique();

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(
            update_delegate(&[first, second]),
            Authority::UpdateAuthority,
        )],
    );

    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(first),
        None,
        client_plugin(&update_delegate(&[second])),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::UpdateDelegate,
        &Authority::UpdateAuthority,
        &update_delegate(&[second]),
    );
}

/// JS: plugins/asset/updateDelegate.test.ts :: it cannot add additional delegate as additional delegate
#[test]
fn additional_delegate_cannot_add_another_delegate() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let first = f.fund(ACCOUNT_LAMPORTS);
    let second = Pubkey::new_unique();

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(update_delegate(&[first]), Authority::UpdateAuthority)],
    );

    let result = f.run(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(first),
        None,
        client_plugin(&update_delegate(&[first, second])),
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_plugin(
        &f.account(&asset),
        PluginType::UpdateDelegate,
        &Authority::UpdateAuthority,
        &update_delegate(&[first]),
    );
}

/// JS: plugins/asset/updateDelegate.test.ts :: it cannot remove another additional delegate as additional delegate
#[test]
fn additional_delegate_cannot_remove_another_delegate() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let first = f.fund(ACCOUNT_LAMPORTS);
    let second = Pubkey::new_unique();

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(
            update_delegate(&[first, second]),
            Authority::UpdateAuthority,
        )],
    );

    // The diff is "remove `second`", not "remove self", so the self-removal
    // arm does not apply and nothing else approves.
    let result = f.run(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(first),
        None,
        client_plugin(&update_delegate(&[first])),
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_plugin(
        &f.account(&asset),
        PluginType::UpdateDelegate,
        &Authority::UpdateAuthority,
        &update_delegate(&[first, second]),
    );
}

/// JS: plugins/asset/updateDelegate.test.ts :: it cannot approve the update delegate plugin authority as additional delegate
#[test]
fn additional_delegate_cannot_approve_update_delegate_authority() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let additional = f.fund(ACCOUNT_LAMPORTS);
    let new_authority = Pubkey::new_unique();

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(update_delegate(&[additional]), Authority::UpdateAuthority)],
    );

    let result = f.run(&ix::approve_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(additional),
        None,
        client_type(PluginType::UpdateDelegate),
        client_authority(&address(new_authority)),
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_plugin(
        &f.account(&asset),
        PluginType::UpdateDelegate,
        &Authority::UpdateAuthority,
        &update_delegate(&[additional]),
    );
}

/// JS: plugins/asset/updateDelegate.test.ts :: it cannot revoke the update delegate plugin authority as additional delegate
#[test]
fn additional_delegate_cannot_revoke_update_delegate_authority() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let additional = f.fund(ACCOUNT_LAMPORTS);
    let delegated = Pubkey::new_unique();

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(update_delegate(&[additional]), address(delegated))],
    );

    let result = f.run(&ix::revoke_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(additional),
        None,
        client_type(PluginType::UpdateDelegate),
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_plugin(
        &f.account(&asset),
        PluginType::UpdateDelegate,
        &address(delegated),
        &update_delegate(&[additional]),
    );
}

/// JS: plugins/asset/updateDelegate.test.ts :: it can approve/revoke the plugin authority of non-updateDelegate plugins as additional delegate
#[test]
fn additional_delegate_can_approve_and_revoke_other_plugin_authority() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let additional = f.fund(ACCOUNT_LAMPORTS);
    let new_authority = Pubkey::new_unique();
    let edition = Plugin::Edition(Edition { number: 1 });

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[
            (update_delegate(&[additional]), Authority::UpdateAuthority),
            (edition.clone(), Authority::UpdateAuthority),
        ],
    );

    f.run_ok(&ix::approve_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(additional),
        None,
        client_type(PluginType::Edition),
        client_authority(&address(new_authority)),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::Edition,
        &address(new_authority),
        &edition,
    );

    f.run_ok(&ix::revoke_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(additional),
        None,
        client_type(PluginType::Edition),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::Edition,
        &Authority::UpdateAuthority,
        &edition,
    );
}

/// JS: plugins/asset/updateDelegate.test.ts :: it can update an authority-managed plugin on an asset as additional delegate
#[test]
fn additional_delegate_can_update_authority_managed_plugin() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let additional = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[
            (permanent_freeze(true), Authority::UpdateAuthority),
            (update_delegate(&[additional]), Authority::UpdateAuthority),
        ],
    );

    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(additional),
        None,
        client_plugin(&permanent_freeze(false)),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::PermanentFreezeDelegate,
        &Authority::UpdateAuthority,
        &permanent_freeze(false),
    );
}

/// JS: plugins/asset/updateDelegate.test.ts :: it cannot update an authority-managed plugin on an asset as additional delegate if the plugin authority is not UpdateAuthority
#[test]
fn additional_delegate_cannot_update_plugin_delegated_elsewhere() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let additional = f.fund(ACCOUNT_LAMPORTS);
    let other = Pubkey::new_unique();

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[
            (permanent_freeze(true), address(other)),
            (update_delegate(&[additional]), Authority::UpdateAuthority),
        ],
    );

    let result = f.run(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(additional),
        None,
        client_plugin(&permanent_freeze(false)),
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_plugin(
        &f.account(&asset),
        PluginType::PermanentFreezeDelegate,
        &address(other),
        &permanent_freeze(true),
    );
}

/// JS: plugins/asset/updateDelegate.test.ts :: it cannot update an owner-managed plugin on an asset as collection update additional delegate
#[test]
fn additional_delegate_cannot_update_owner_managed_plugin() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let additional = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[
            (freeze(true), Authority::Owner),
            (update_delegate(&[additional]), Authority::UpdateAuthority),
        ],
    );

    let result = f.run(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(additional),
        None,
        client_plugin(&freeze(false)),
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeDelegate,
        &Authority::Owner,
        &freeze(true),
    );
}

/// JS: plugins/asset/updateDelegate.test.ts :: it can update an owner-managed plugin on an asset as collection update additional delegate if the plugin authority is UpdateAuthority
#[test]
fn additional_delegate_can_update_owner_managed_plugin_held_by_update_authority() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let additional = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[
            (freeze(true), Authority::UpdateAuthority),
            (update_delegate(&[additional]), Authority::UpdateAuthority),
        ],
    );

    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(additional),
        None,
        client_plugin(&freeze(false)),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeDelegate,
        &Authority::UpdateAuthority,
        &freeze(false),
    );
}

/// The asset-level `UpdateDelegate` reject arm
/// (`update_delegate.rs:182-198`): an asset-level delegate may not re-parent
/// the asset out of its collection. JS documents this with `UpdateV2`
/// ('it cannot remove an asset from collection using update delegate on the
/// asset'); `UpdateV1` reaches the same validation and would only afterwards
/// refuse the re-parenting itself with `NotAvailable`, so the `InvalidAuthority`
/// here proves the plugin rejected first.
#[test]
fn asset_update_delegate_cannot_move_asset_out_of_collection() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = f.fund(ACCOUNT_LAMPORTS);

    let (collection, asset) = store_collection_with_member(
        &f,
        actors.update_authority,
        actors.owner,
        &[],
        &[(update_delegate(&[]), address(delegate))],
    );

    let result = f.run(&ix::update_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(delegate),
        None,
        None,
        None,
        Some(convert(&UpdateAuthority::Address(actors.owner))),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(
        f.asset(&asset).update_authority,
        UpdateAuthority::Collection(collection)
    );
}

/// JS: plugins/asset/updateDelegate.test.ts :: it can update the update authority of the asset as an updateDelegate additional delegate
#[test]
fn additional_delegate_can_change_the_update_authority() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let additional = f.fund(ACCOUNT_LAMPORTS);
    let new_update_authority = Pubkey::new_unique();

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(update_delegate(&[additional]), Authority::UpdateAuthority)],
    );

    f.run_ok(&ix::update_v1(
        asset,
        None,
        actors.payer,
        Some(additional),
        None,
        None,
        None,
        Some(convert(&UpdateAuthority::Address(new_update_authority))),
    ));
    assert_eq!(
        f.asset(&asset).update_authority,
        UpdateAuthority::Address(new_update_authority)
    );
}

// ===========================================================================
// freeze.test.ts — FreezeDelegate
// ===========================================================================

/// JS: plugins/asset/freeze.test.ts :: it can freeze and unfreeze an asset
#[test]
fn freeze_delegate_can_freeze_and_unfreeze() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(freeze(false), address(delegate))],
    );

    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(delegate),
        None,
        client_plugin(&freeze(true)),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeDelegate,
        &address(delegate),
        &freeze(true),
    );

    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(delegate),
        None,
        client_plugin(&freeze(false)),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeDelegate,
        &address(delegate),
        &freeze(false),
    );
}

/// JS: plugins/asset/freeze.test.ts :: it owner cannot unfreeze frozen asset
#[test]
fn owner_cannot_unfreeze_delegate_frozen_asset() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = Pubkey::new_unique();

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(freeze(true), address(delegate))],
    );

    let result = f.run(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_plugin(&freeze(false)),
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeDelegate,
        &address(delegate),
        &freeze(true),
    );
}

/// JS: plugins/asset/freeze.test.ts :: owner cannot undelegate a freeze plugin with a delegate
#[test]
fn owner_cannot_revoke_freeze_delegate_while_frozen() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = Pubkey::new_unique();

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(freeze(true), address(delegate))],
    );

    let result = f.run(&ix::revoke_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_type(PluginType::FreezeDelegate),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeDelegate,
        &address(delegate),
        &freeze(true),
    );
}

/// JS: plugins/asset/freeze.test.ts :: owner cannot approve to reassign authority back to owner if frozen
///
/// The wrapper in `lifecycle.rs` refuses to re-delegate an already delegated
/// plugin before `FreezeDelegate::validate_approve_plugin_authority` runs,
/// so the error is `CannotRedelegate`, not `InvalidAuthority`.
#[test]
fn owner_cannot_redelegate_frozen_freeze_delegate() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = Pubkey::new_unique();

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(freeze(true), address(delegate))],
    );

    let result = f.run(&ix::approve_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_type(PluginType::FreezeDelegate),
        client_authority(&Authority::Owner),
    ));
    assert_core_err(&result, MplCoreError::CannotRedelegate);
}

/// `FreezeDelegate::validate_approve_plugin_authority` rejects while the
/// plugin is frozen and still owner-managed, which is the only way to reach
/// that arm (once it is delegated, the wrapper's `CannotRedelegate` fires
/// first — see [`owner_cannot_redelegate_frozen_freeze_delegate`]). No JS
/// test exercises this; roadmap section 12 flags it as new.
#[test]
fn owner_cannot_delegate_a_self_frozen_freeze_delegate() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let new_delegate = Pubkey::new_unique();

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(freeze(true), Authority::Owner)],
    );

    let result = f.run(&ix::approve_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_type(PluginType::FreezeDelegate),
        client_authority(&address(new_delegate)),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeDelegate,
        &Authority::Owner,
        &freeze(true),
    );
}

/// JS: plugins/asset/freeze.test.ts :: it delegate cannot freeze after delegate has been revoked
#[test]
fn owner_can_revoke_unfrozen_freeze_delegate_and_delegate_loses_access() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(freeze(false), address(delegate))],
    );

    // Unfrozen: the plugin abstains and the owner's own approval carries the
    // revoke.
    f.run_ok(&ix::revoke_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_type(PluginType::FreezeDelegate),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeDelegate,
        &Authority::Owner,
        &freeze(false),
    );

    let result = f.run(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(delegate),
        None,
        client_plugin(&freeze(true)),
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
}

/// JS: revokeAuthority.test.ts :: it can remove a pubkey authority from an owner-managed plugin if that pubkey is the signer authority
#[test]
fn freeze_delegate_can_revoke_itself() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(freeze(false), address(delegate))],
    );

    // `FreezeDelegate::validate_revoke_plugin_authority` approves because the
    // signer resolves to the record's own authority.
    f.run_ok(&ix::revoke_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(delegate),
        None,
        client_type(PluginType::FreezeDelegate),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeDelegate,
        &Authority::Owner,
        &freeze(false),
    );
}

/// JS: plugins/asset/freeze.test.ts :: it cannot remove freeze plugin if update authority and frozen
#[test]
fn cannot_remove_freeze_delegate_while_frozen() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(freeze(true), Authority::Owner)],
    );

    let result = f.run(&ix::remove_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::FreezeDelegate),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeDelegate,
        &Authority::Owner,
        &freeze(true),
    );
}

/// A frozen `FreezeDelegate` blocks the removal of *any* plugin, not only its
/// own (`freeze_delegate.rs:97-98`, `ctx.target_plugin.is_some()`).
/// `removePlugin.test.ts` documents the same with a TransferDelegate target.
#[test]
fn frozen_freeze_delegate_blocks_removing_other_plugins() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[
            (freeze(true), Authority::Owner),
            (attributes(), Authority::UpdateAuthority),
        ],
    );

    let result = f.run(&ix::remove_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::Attributes),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    // Unfreezing first (as the owner, who holds the record) lets the same
    // removal through, which is the `abstain` arm.
    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_plugin(&freeze(false)),
    ));
    f.run_ok(&ix::remove_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::Attributes),
    ));
    let account = f.account(&asset);
    assert_registry_consistent(&account);
    assert!(read_plugin(&account, PluginType::Attributes).is_none());
    assert_plugin(
        &account,
        PluginType::FreezeDelegate,
        &Authority::Owner,
        &freeze(false),
    );
}

/// JS: plugins/asset/freeze.test.ts :: it cannot add multiple freeze plugins to an asset
#[test]
fn cannot_add_a_second_freeze_delegate() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(freeze(false), Authority::Owner)],
    );

    let result = f.run(&ix::add_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_plugin(&freeze(true)),
        None,
    ));
    assert_core_err(&result, MplCoreError::PluginAlreadyExists);
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeDelegate,
        &Authority::Owner,
        &freeze(false),
    );
}

/// A frozen `FreezeDelegate` rejects `BurnV1`
/// (`freeze_delegate.rs:43-48`); unfrozen it abstains and the owner's own
/// approval carries the burn.
#[test]
fn frozen_asset_cannot_be_burned_then_can_after_unfreezing() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(freeze(true), Authority::Owner)],
    );

    let result = f.run(&ix::burn_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(f.asset(&asset).owner, actors.owner);

    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_plugin(&freeze(false)),
    ));
    f.run_ok(&ix::burn_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        None,
    ));
    f.assert_burned(&asset);
}

// ===========================================================================
// burnDelegate.test.ts / delegateTransfer.test.ts
// ===========================================================================

/// JS: plugins/asset/burnDelegate.test.ts :: a burnDelegate can burn an asset
#[test]
fn burn_delegate_can_burn() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(Plugin::BurnDelegate(BurnDelegate {}), address(delegate))],
    );
    let payer_before = f.lamports(&actors.payer);
    let asset_before = f.lamports(&asset);
    let data_len = f.account(&asset).data.len();

    f.run_ok(&ix::burn_v1(
        asset,
        None,
        actors.payer,
        Some(delegate),
        None,
        None,
    ));
    f.assert_burned(&asset);
    // `close_program_account` refunds the rent of the closed size minus the
    // rent of the one byte it leaves behind; anything the account held above
    // its rent-exempt minimum stays with the husk.
    let rent = Rent::default();
    let refund = rent.minimum_balance(data_len) - rent.minimum_balance(1);
    assert_eq!(f.lamports(&actors.payer), payer_before + refund);
    assert_eq!(f.lamports(&asset), asset_before - refund);
}

/// JS: plugins/asset/burnDelegate.test.ts :: an burnDelegate cannot burn an asset after delegate authority revoked
#[test]
fn burn_delegate_cannot_burn_after_revoke() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(Plugin::BurnDelegate(BurnDelegate {}), address(delegate))],
    );

    f.run_ok(&ix::revoke_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_type(PluginType::BurnDelegate),
    ));

    let result = f.run(&ix::burn_v1(
        asset,
        None,
        actors.payer,
        Some(delegate),
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_eq!(f.asset(&asset).owner, actors.owner);
}

/// JS: plugins/asset/delegateTransfer.test.ts :: a delegate can transfer the asset
#[test]
fn transfer_delegate_can_transfer() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = f.fund(ACCOUNT_LAMPORTS);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(
            Plugin::TransferDelegate(TransferDelegate {}),
            address(delegate),
        )],
    );

    let len_before = f.account(&asset).data.len();

    f.run_ok(&ix::transfer_v1(
        asset,
        None,
        actors.payer,
        Some(delegate),
        new_owner,
        None,
        None,
    ));
    assert_eq!(f.asset(&asset).owner, new_owner);

    // `TransferV1` resets every owner-managed record to `Authority::Owner`,
    // so the new owner controls the delegate.
    let account = f.account(&asset);
    let (authority, plugin) = stored(&account, PluginType::TransferDelegate);
    assert_eq!(authority, Authority::Owner);
    assert_eq!(plugin, Plugin::TransferDelegate(TransferDelegate {}));

    // `transfer.rs:100-112` rewrites the registry in place without
    // reallocating, so dropping the 32-byte `Address` pubkey leaves that many
    // stale bytes after the registry. `assert_registry_consistent` would
    // fail here; this asserts the real layout instead. See roadmap section 6
    // (findings) — the state still parses, it just wastes rent.
    assert_eq!(
        account.data.len(),
        len_before,
        "the account keeps its pre-transfer size"
    );
    let parsed = parse_asset(&account.data).1;
    let registry_offset = parsed
        .header
        .as_ref()
        .expect("the asset has plugin metadata")
        .plugin_registry_offset;
    let registry_len = borsh::to_vec(parsed.registry.as_ref().expect("registry"))
        .expect("registry serializes")
        .len();
    assert_eq!(
        account.data.len() - (registry_offset + registry_len),
        32,
        "the shrunk Address authority leaves 32 trailing bytes"
    );
}

/// JS: plugins/asset/delegateTransfer.test.ts :: it cannot transfer after delegate authority has been revoked
#[test]
fn transfer_delegate_cannot_transfer_after_revoke() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = f.fund(ACCOUNT_LAMPORTS);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(
            Plugin::TransferDelegate(TransferDelegate {}),
            address(delegate),
        )],
    );

    f.run_ok(&ix::revoke_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_type(PluginType::TransferDelegate),
    ));

    let result = f.run(&ix::transfer_v1(
        asset,
        None,
        actors.payer,
        Some(delegate),
        new_owner,
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_eq!(f.asset(&asset).owner, actors.owner);
}

/// JS: plugins/asset/delegateTransfer.test.ts :: it can transfer using delegated update authority
#[test]
fn transfer_delegate_delegated_to_update_authority_can_transfer() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(
            Plugin::TransferDelegate(TransferDelegate {}),
            Authority::UpdateAuthority,
        )],
    );

    f.run_ok(&ix::transfer_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        new_owner,
        None,
        None,
    ));
    assert_eq!(f.asset(&asset).owner, new_owner);
}

// ===========================================================================
// permanentBurn / permanentTransfer / permanentFreeze
// ===========================================================================

/// JS: plugins/asset/permanentBurn.test.ts :: it can burn an assets as a delegate
#[test]
fn permanent_burn_delegate_can_burn() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(
            Plugin::PermanentBurnDelegate(PermanentBurnDelegate {}),
            address(delegate),
        )],
    );

    f.run_ok(&ix::burn_v1(
        asset,
        None,
        actors.payer,
        Some(delegate),
        None,
        None,
    ));
    f.assert_burned(&asset);
}

/// JS: plugins/asset/permanentBurn.test.ts :: it can add another plugin on asset with permanent burn plugin
#[test]
fn can_add_another_plugin_next_to_permanent_burn_delegate() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(
            Plugin::PermanentBurnDelegate(PermanentBurnDelegate {}),
            Authority::UpdateAuthority,
        )],
    );

    // `PermanentBurnDelegate::validate_add_plugin` abstains for a different
    // target, so the update authority's own approval carries the add.
    f.run_ok(&ix::add_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&attributes()),
        None,
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::Attributes,
        &Authority::UpdateAuthority,
        &attributes(),
    );
}

/// JS: plugins/asset/permanentBurn.test.ts :: it can burn an assets as an owner
#[test]
fn owner_can_burn_asset_with_permanent_burn_delegate() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    // The owner is not the plugin authority, so the plugin abstains and
    // `AssetV1::validate_burn` approves.
    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(
            Plugin::PermanentBurnDelegate(PermanentBurnDelegate {}),
            Authority::UpdateAuthority,
        )],
    );

    f.run_ok(&ix::burn_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        None,
    ));
    f.assert_burned(&asset);
}

/// JS: plugins/asset/pluginValidationOverrides.test.ts :: it can burn a frozen asset using PermanentBurnDelegate (ForceApproved overrides freeze)
#[test]
fn permanent_burn_delegate_force_approves_over_a_frozen_asset() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[
            (
                Plugin::PermanentBurnDelegate(PermanentBurnDelegate {}),
                address(delegate),
            ),
            (freeze(true), Authority::Owner),
        ],
    );

    // FreezeDelegate rejects the burn; `ForceApproved` short-circuits the
    // whole aggregation (`lifecycle.rs:733`).
    f.run_ok(&ix::burn_v1(
        asset,
        None,
        actors.payer,
        Some(delegate),
        None,
        None,
    ));
    f.assert_burned(&asset);
}

/// JS: plugins/asset/pluginValidationOverrides.test.ts :: it can burn a frozen asset using collection PermanentBurnDelegate (ForceApproved overrides freeze)
#[test]
fn collection_permanent_burn_delegate_force_approves_over_a_frozen_asset() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let (collection, asset) = store_collection_with_member(
        &f,
        actors.update_authority,
        actors.owner,
        &[(
            Plugin::PermanentBurnDelegate(PermanentBurnDelegate {}),
            Authority::UpdateAuthority,
        )],
        &[(freeze(true), Authority::Owner)],
    );
    let size_before = f.collection(&collection).current_size;

    f.run_ok(&ix::burn_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(actors.update_authority),
        None,
        None,
    ));
    f.assert_burned(&asset);
    assert_eq!(f.collection(&collection).current_size, size_before - 1);
}

/// JS: plugins/asset/permanentTransfer.test.ts :: it can transfer an asset as not the owner
#[test]
fn permanent_transfer_delegate_can_transfer() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = f.fund(ACCOUNT_LAMPORTS);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(
            Plugin::PermanentTransferDelegate(PermanentTransferDelegate {}),
            address(delegate),
        )],
    );

    f.run_ok(&ix::transfer_v1(
        asset,
        None,
        actors.payer,
        Some(delegate),
        new_owner,
        None,
        None,
    ));
    assert_eq!(f.asset(&asset).owner, new_owner);
}

/// JS: plugins/asset/permanentTransfer.test.ts :: it can permanent transfer asset that is frozen as a delegate
#[test]
fn permanent_transfer_delegate_force_approves_over_a_frozen_asset() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = f.fund(ACCOUNT_LAMPORTS);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[
            (
                Plugin::PermanentTransferDelegate(PermanentTransferDelegate {}),
                address(delegate),
            ),
            (freeze(true), Authority::Owner),
        ],
    );

    f.run_ok(&ix::transfer_v1(
        asset,
        None,
        actors.payer,
        Some(delegate),
        new_owner,
        None,
        None,
    ));
    assert_eq!(f.asset(&asset).owner, new_owner);
}

/// JS: plugins/asset/permanentTransfer.test.ts :: it cannot transfer asset that is frozen with permanent transfer by owner
#[test]
fn owner_cannot_transfer_a_frozen_asset_with_permanent_transfer_delegate() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = Pubkey::new_unique();
    let new_owner = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[
            (
                Plugin::PermanentTransferDelegate(PermanentTransferDelegate {}),
                address(delegate),
            ),
            (freeze(true), Authority::Owner),
        ],
    );

    // The owner is not the permanent delegate, so that plugin abstains and
    // FreezeDelegate's rejection stands.
    let result = f.run(&ix::transfer_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        new_owner,
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(f.asset(&asset).owner, actors.owner);
}

/// JS: plugins/asset/pluginValidationOverrides.test.ts :: it can transfer with PermanentTransferDelegate even when collection Royalties would reject
#[test]
fn permanent_transfer_delegate_force_approves_over_collection_royalties() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = f.fund(ACCOUNT_LAMPORTS);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);

    // The allow list holds only the recorder program, so neither the signer
    // nor the new owner (both system-owned) are on it and Royalties rejects.
    let (collection, asset) = store_collection_with_member(
        &f,
        actors.update_authority,
        actors.owner,
        &[(
            royalties(
                actors.update_authority,
                RuleSet::ProgramAllowList(vec![RECORDER_ID]),
            ),
            Authority::UpdateAuthority,
        )],
        &[(
            Plugin::PermanentTransferDelegate(PermanentTransferDelegate {}),
            address(delegate),
        )],
    );

    f.run_ok(&ix::transfer_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(delegate),
        new_owner,
        None,
        None,
    ));
    assert_eq!(f.asset(&asset).owner, new_owner);
}

/// JS: plugins/asset/permanentFreeze.test.ts :: it cannot be transferred while frozen
#[test]
fn permanently_frozen_asset_cannot_be_transferred() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(permanent_freeze(true), Authority::UpdateAuthority)],
    );

    let result = f.run(&ix::transfer_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        new_owner,
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(f.asset(&asset).owner, actors.owner);
}

/// JS: plugins/asset/permanentFreeze.test.ts :: it cannot move asset in a permanently frozen collection
#[test]
fn asset_in_a_permanently_frozen_collection_cannot_be_transferred() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);

    let (collection, asset) = store_collection_with_member(
        &f,
        actors.update_authority,
        actors.owner,
        &[(permanent_freeze(true), Authority::UpdateAuthority)],
        &[],
    );

    let result = f.run(&ix::transfer_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(actors.owner),
        new_owner,
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(f.asset(&asset).owner, actors.owner);
}

/// JS: plugins/asset/pluginValidationOverrides.test.ts :: asset PermanentFreezeDelegate(unfrozen) overrides collection PermanentFreezeDelegate(frozen) for transfer
#[test]
fn unfrozen_asset_plugin_shadows_a_frozen_collection_plugin() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);

    // `checks` is keyed by `PluginType`: the asset registry overwrites the
    // collection's entry, so only the asset's unfrozen plugin is evaluated.
    let (collection, asset) = store_collection_with_member(
        &f,
        actors.update_authority,
        actors.owner,
        &[(permanent_freeze(true), Authority::UpdateAuthority)],
        &[(permanent_freeze(false), Authority::UpdateAuthority)],
    );

    f.run_ok(&ix::transfer_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(actors.owner),
        new_owner,
        None,
        None,
    ));
    assert_eq!(f.asset(&asset).owner, new_owner);
}

/// JS: plugins/asset/pluginValidationOverrides.test.ts :: asset PermanentFreezeDelegate(frozen) blocks transfer even when collection PermanentFreezeDelegate is unfrozen
#[test]
fn frozen_asset_plugin_shadows_an_unfrozen_collection_plugin() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);

    let (collection, asset) = store_collection_with_member(
        &f,
        actors.update_authority,
        actors.owner,
        &[(permanent_freeze(false), Authority::UpdateAuthority)],
        &[(permanent_freeze(true), Authority::UpdateAuthority)],
    );

    let result = f.run(&ix::transfer_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(actors.owner),
        new_owner,
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(f.asset(&asset).owner, actors.owner);
}

/// JS: plugins/asset/permanentFreeze.test.ts :: it cannot add permanentFreeze after creation
#[test]
fn cannot_add_permanent_freeze_delegate_after_creation() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(permanent_freeze(false), Authority::UpdateAuthority)],
    );

    // The new plugin validates itself in `add_plugin.rs` and rejects.
    let result = f.run(&ix::add_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&permanent_freeze(true)),
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
}

/// JS: plugins/asset/permanentFreeze.test.ts :: it cannot remove permanent freeze plugin if update authority and frozen
/// JS: plugins/asset/permanentFreeze.test.ts :: it can remove permanent freeze plugin if update authority and unfrozen
#[test]
fn permanent_freeze_delegate_removal_depends_on_frozen() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(permanent_freeze(true), Authority::UpdateAuthority)],
    );

    let result = f.run(&ix::remove_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::PermanentFreezeDelegate),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&permanent_freeze(false)),
    ));
    f.run_ok(&ix::remove_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::PermanentFreezeDelegate),
    ));
    let account = f.account(&asset);
    assert_registry_consistent(&account);
    assert!(read_plugin(&account, PluginType::PermanentFreezeDelegate).is_none());
}

/// JS: plugins/asset/permanentTransfer.test.ts :: it cannot add permanentTransfer after creation
/// JS: plugins/collection/permanentBurn.test.ts :: it cannot add permanentBurnDelegate to collection after creation
#[test]
fn permanent_delegates_cannot_be_added_after_creation() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let asset = store_asset(&f, actors.owner, actors.update_authority, &[]);
    let collection = store_collection(&f, actors.update_authority, &[]);

    for plugin in [
        Plugin::PermanentTransferDelegate(PermanentTransferDelegate {}),
        Plugin::PermanentBurnDelegate(PermanentBurnDelegate {}),
        permanent_freeze(false),
        permanent_freeze_execute(false),
        Plugin::Edition(Edition { number: 1 }),
    ] {
        let result = f.run(&ix::add_plugin_v1(
            asset,
            None,
            actors.payer,
            Some(actors.update_authority),
            None,
            client_plugin(&plugin),
            None,
        ));
        assert_core_err(&result, MplCoreError::InvalidAuthority);
    }

    for plugin in [
        Plugin::PermanentTransferDelegate(PermanentTransferDelegate {}),
        Plugin::PermanentBurnDelegate(PermanentBurnDelegate {}),
        permanent_freeze(false),
        permanent_freeze_execute(false),
    ] {
        let result = f.run(&ix::add_collection_plugin_v1(
            collection,
            actors.payer,
            Some(actors.update_authority),
            None,
            client_plugin(&plugin),
            None,
        ));
        assert_core_err(&result, MplCoreError::InvalidAuthority);
    }
}

// ===========================================================================
// freezeExecute / freezeExecuteRemoval / permanentFreezeExecute
// ===========================================================================

/// Instruction data forwarded to the recorder builtin by the execute tests.
const EXECUTE_DATA: [u8; 2] = [0x01, 0x02];

/// `ExecuteV1` on `asset` targeting the recorder builtin, signed by
/// `authority`. The asset signer PDA is stored and passed as a remaining
/// account, exactly as `tests/it/execution_delegate.rs` does.
fn execute_ix(
    f: &Fixture,
    asset: Pubkey,
    collection: Option<Pubkey>,
    payer: Pubkey,
    authority: Pubkey,
) -> solana_program::instruction::Instruction {
    let (asset_signer, _) = asset_signer_pda(&asset);
    if !f.has_account(&asset_signer) {
        f.store(asset_signer, payer_account(ACCOUNT_LAMPORTS));
    }
    let ix = ix::execute_v1(
        asset,
        collection,
        asset_signer,
        payer,
        Some(authority),
        RECORDER_ID,
        EXECUTE_DATA.to_vec(),
        &[],
    );
    with_remaining(
        ix,
        [solana_program::instruction::AccountMeta::new_readonly(
            asset_signer,
            false,
        )],
    )
}

/// JS: plugins/asset/freezeExecute.test.ts :: it covers the freeze execute backed NFT flow
#[test]
fn freeze_execute_blocks_and_allows_execute() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(freeze_execute(true), Authority::Owner)],
    );

    clear_recorded();
    let result = f.run(&execute_ix(&f, asset, None, actors.payer, actors.owner));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert!(
        take_recorded().is_empty(),
        "a rejected execute must not reach the target program"
    );

    // Unfreezing makes `validate_execute` abstain and the owner's own
    // approval carries the instruction through to the CPI.
    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_plugin(&freeze_execute(false)),
    ));
    clear_recorded();
    f.run_ok(&execute_ix(&f, asset, None, actors.payer, actors.owner));
    let recorded = take_recorded();
    assert_eq!(
        recorded.len(),
        1,
        "the target program should be invoked once"
    );
    assert_eq!(recorded[0].data, EXECUTE_DATA);
}

/// JS: plugins/asset/freezeExecuteRemoval.test.ts :: it cannot remove FreezeExecute while frozen
#[test]
fn cannot_remove_freeze_execute_while_frozen() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[
            (freeze_execute(true), Authority::Owner),
            (attributes(), Authority::UpdateAuthority),
        ],
    );

    let result = f.run(&ix::remove_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_type(PluginType::FreezeExecute),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    // Unlike FreezeDelegate, a frozen FreezeExecute only blocks the removal
    // of itself (`freeze_execute.rs:85-88`), so another plugin still goes.
    f.run_ok(&ix::remove_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::Attributes),
    ));
    let account = f.account(&asset);
    assert_registry_consistent(&account);
    assert!(read_plugin(&account, PluginType::Attributes).is_none());
    assert_plugin(
        &account,
        PluginType::FreezeExecute,
        &Authority::Owner,
        &freeze_execute(true),
    );
}

/// JS: plugins/asset/freezeExecuteRemoval.test.ts :: it cannot revoke FreezeExecute as the owner while frozen
#[test]
fn cannot_revoke_freeze_execute_while_frozen() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = Pubkey::new_unique();

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(freeze_execute(true), address(delegate))],
    );

    let result = f.run(&ix::revoke_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_type(PluginType::FreezeExecute),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeExecute,
        &address(delegate),
        &freeze_execute(true),
    );
}

/// JS: plugins/asset/freezeExecuteRemoval.test.ts :: it cannot approve a new authority for FreezeExecute as the owner while frozen
#[test]
fn cannot_approve_freeze_execute_authority_while_frozen() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let new_delegate = Pubkey::new_unique();

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(freeze_execute(true), Authority::Owner)],
    );

    let result = f.run(&ix::approve_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_type(PluginType::FreezeExecute),
        client_authority(&address(new_delegate)),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeExecute,
        &Authority::Owner,
        &freeze_execute(true),
    );
}

/// JS: plugins/asset/freezeExecuteRemoval.test.ts :: delegate can unfreeze FreezeExecute
/// JS: plugins/asset/freezeExecuteRemoval.test.ts :: owner can remove FreezeExecute after delegate unfreezes it
#[test]
fn delegate_unfreezes_freeze_execute_then_revokes_and_owner_removes_it() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(freeze_execute(true), address(delegate))],
    );

    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(delegate),
        None,
        client_plugin(&freeze_execute(false)),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeExecute,
        &address(delegate),
        &freeze_execute(false),
    );

    // Unfrozen, the delegate's own signature approves the revoke
    // (`freeze_execute.rs:69-76`).
    f.run_ok(&ix::revoke_plugin_authority_v1(
        asset,
        None,
        actors.payer,
        Some(delegate),
        None,
        client_type(PluginType::FreezeExecute),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeExecute,
        &Authority::Owner,
        &freeze_execute(false),
    );

    f.run_ok(&ix::remove_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_type(PluginType::FreezeExecute),
    ));
    let account = f.account(&asset);
    assert_registry_consistent(&account);
    assert!(read_plugin(&account, PluginType::FreezeExecute).is_none());
}

/// JS: plugins/asset/permanentFreezeExecute.test.ts :: PermanentFreezeExecute blocks execute but allows burn
#[test]
fn permanent_freeze_execute_blocks_execute_but_allows_burn() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(permanent_freeze_execute(true), Authority::UpdateAuthority)],
    );

    let result = f.run(&execute_ix(&f, asset, None, actors.payer, actors.owner));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    // `check_burn` has no entry for PermanentFreezeExecute, so the burn is
    // unaffected.
    f.run_ok(&ix::burn_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        None,
    ));
    f.assert_burned(&asset);
}

/// JS: plugins/asset/permanentFreezeExecute.test.ts :: it cannot remove PermanentFreezeExecute plugin if frozen
/// JS: plugins/asset/permanentFreezeExecute.test.ts :: it can remove PermanentFreezeExecute plugin if unfrozen
#[test]
fn permanent_freeze_execute_removal_depends_on_frozen() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(permanent_freeze_execute(true), Authority::UpdateAuthority)],
    );

    let result = f.run(&ix::remove_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::PermanentFreezeExecute),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&permanent_freeze_execute(false)),
    ));
    f.run_ok(&ix::remove_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::PermanentFreezeExecute),
    ));
    let account = f.account(&asset);
    assert_registry_consistent(&account);
    assert!(read_plugin(&account, PluginType::PermanentFreezeExecute).is_none());
}

/// JS: plugins/collection/permanentFreezeExecute.test.ts :: assets inherit PermanentFreezeExecute plugin from collection and execute is blocked when frozen
#[test]
fn collection_permanent_freeze_execute_blocks_member_execute() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let (collection, asset) = store_collection_with_member(
        &f,
        actors.update_authority,
        actors.owner,
        &[(permanent_freeze_execute(true), Authority::UpdateAuthority)],
        &[],
    );

    let result = f.run(&execute_ix(
        &f,
        asset,
        Some(collection),
        actors.payer,
        actors.owner,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
}

/// JS: plugins/collection/permanentFreezeExecute.test.ts :: asset-level PermanentFreezeExecute overrides collection-level plugin when unfrozen
#[test]
fn unfrozen_asset_freeze_execute_shadows_frozen_collection_plugin() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let (collection, asset) = store_collection_with_member(
        &f,
        actors.update_authority,
        actors.owner,
        &[(permanent_freeze_execute(true), Authority::UpdateAuthority)],
        &[(permanent_freeze_execute(false), Authority::UpdateAuthority)],
    );

    clear_recorded();
    f.run_ok(&execute_ix(
        &f,
        asset,
        Some(collection),
        actors.payer,
        actors.owner,
    ));
    assert_eq!(take_recorded().len(), 1);
}

/// JS: plugins/asset/permanentFreezeExecute.test.ts :: it cannot add PermanentFreezeExecute after creation
/// (asset variant; see [`permanent_delegates_cannot_be_added_after_creation`]
/// for the whole matrix) — this one checks the `abstain` arm for a different
/// target next to it.
#[test]
fn can_add_another_plugin_next_to_permanent_freeze_execute() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(permanent_freeze_execute(true), Authority::UpdateAuthority)],
    );

    f.run_ok(&ix::add_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&attributes()),
        None,
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::Attributes,
        &Authority::UpdateAuthority,
        &attributes(),
    );
}

// ===========================================================================
// royalties.test.ts
// ===========================================================================

/// `Royalties` with the given basis points and creators, rule set `None`.
fn royalties_raw(basis_points: u16, creators: Vec<(Pubkey, u8)>) -> Plugin {
    Plugin::Royalties(Royalties {
        basis_points,
        creators: creators
            .into_iter()
            .map(|(address, percentage)| Creator {
                address,
                percentage,
            })
            .collect(),
        rule_set: RuleSet::None,
    })
}

/// JS: plugins/asset/royalties.test.ts :: it cannot create royalty basis points greater than 10000
/// JS: plugins/asset/royalties.test.ts :: it cannot create royalty percentages that dont add up to 100
/// JS: plugins/asset/royalties.test.ts :: it cannot create royalty with duplicate creators
#[test]
fn cannot_create_an_asset_with_invalid_royalties() {
    let f = Fixture::new();
    let payer = f.fund(ACCOUNT_LAMPORTS);
    let creator = Pubkey::new_unique();
    let other = Pubkey::new_unique();

    for plugin in [
        royalties_raw(10001, vec![(creator, 100)]),
        royalties_raw(500, vec![(creator, 50), (other, 40)]),
        royalties_raw(500, vec![(creator, 50), (creator, 50)]),
    ] {
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
            Some(vec![plugin_pair(&plugin, None)]),
            None,
            &[],
        ));
        assert_core_err(&result, MplCoreError::InvalidPluginSetting);
    }
}

/// JS: plugins/asset/royalties.test.ts :: it cannot add royalty basis points greater than 10000
/// JS: plugins/asset/royalties.test.ts :: it cannot update royalty basis points greater than 10000
#[test]
fn cannot_add_or_update_invalid_royalties() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let creator = Pubkey::new_unique();

    let bare = store_asset(&f, actors.owner, actors.update_authority, &[]);
    let result = f.run(&ix::add_plugin_v1(
        bare,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&royalties_raw(10001, vec![(creator, 100)])),
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidPluginSetting);

    let valid = royalties(creator, RuleSet::None);
    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(valid.clone(), Authority::UpdateAuthority)],
    );
    let result = f.run(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&royalties_raw(10001, vec![(creator, 100)])),
    ));
    assert_core_err(&result, MplCoreError::InvalidPluginSetting);
    assert_plugin(
        &f.account(&asset),
        PluginType::Royalties,
        &Authority::UpdateAuthority,
        &valid,
    );
}

/// `Royalties::validate_update_plugin` abstains when the signer is not the
/// plugin authority, and nothing else approves an `UpdatePluginV1` from the
/// owner of an authority-managed plugin. Roadmap section 12 lists this arm as
/// having no JS counterpart.
#[test]
fn owner_cannot_update_royalties() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let creator = Pubkey::new_unique();
    let stored_royalties = royalties(creator, RuleSet::None);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(stored_royalties.clone(), Authority::UpdateAuthority)],
    );

    let result = f.run(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_plugin(&royalties(creator, RuleSet::ProgramDenyList(vec![]))),
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_plugin(
        &f.account(&asset),
        PluginType::Royalties,
        &Authority::UpdateAuthority,
        &stored_royalties,
    );
}

/// `Royalties::validate_add_plugin` abstains for a non-Royalties target.
/// Roadmap section 12 lists this arm as having no JS counterpart.
#[test]
fn can_add_another_plugin_next_to_royalties() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let creator = Pubkey::new_unique();

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(
            royalties(creator, RuleSet::None),
            Authority::UpdateAuthority,
        )],
    );

    f.run_ok(&ix::add_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&attributes()),
        None,
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::Attributes,
        &Authority::UpdateAuthority,
        &attributes(),
    );
}

/// JS: plugins/asset/royalties.test.ts :: it can transfer an asset with royalties
/// JS: plugins/asset/royalties.test.ts :: it can transfer an asset with royalties to an allowlisted program address
/// JS: plugins/asset/royalties.test.ts :: it can transfer an asset with royalties to a program address not on the denylist
#[test]
fn royalties_rule_sets_that_allow_a_transfer() {
    let creator = Pubkey::new_unique();
    // A wallet account is owned by the system program, which is what the
    // allow / deny lists are matched against.
    let system = solana_system_interface::program::ID;

    for rule_set in [
        RuleSet::None,
        RuleSet::ProgramAllowList(vec![system]),
        RuleSet::ProgramDenyList(vec![]),
        RuleSet::ProgramDenyList(vec![RECORDER_ID]),
    ] {
        let f = Fixture::new();
        let actors = Actors::new(&f);
        let new_owner = f.fund(ACCOUNT_LAMPORTS);
        let asset = store_asset(
            &f,
            actors.owner,
            actors.update_authority,
            &[(
                royalties(creator, rule_set.clone()),
                Authority::UpdateAuthority,
            )],
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
        assert_eq!(
            f.asset(&asset).owner,
            new_owner,
            "{rule_set:?} should allow the transfer"
        );
    }
}

/// JS: plugins/asset/royalties.test.ts :: it cannot transfer an asset with royalties to a program address not on the allowlist
/// JS: plugins/asset/royalties.test.ts :: it cannot transfer an asset with royalties to a denylisted program
#[test]
fn royalties_rule_sets_that_reject_a_transfer() {
    let creator = Pubkey::new_unique();
    let system = solana_system_interface::program::ID;

    for rule_set in [
        // Neither the signer nor the new owner is owned by the recorder.
        RuleSet::ProgramAllowList(vec![RECORDER_ID]),
        // The signer's wallet is system-owned, which is denied.
        RuleSet::ProgramDenyList(vec![system]),
    ] {
        let f = Fixture::new();
        let actors = Actors::new(&f);
        let new_owner = f.fund(ACCOUNT_LAMPORTS);
        let asset = store_asset(
            &f,
            actors.owner,
            actors.update_authority,
            &[(
                royalties(creator, rule_set.clone()),
                Authority::UpdateAuthority,
            )],
        );

        let result = f.run(&ix::transfer_v1(
            asset,
            None,
            actors.payer,
            Some(actors.owner),
            new_owner,
            None,
            None,
        ));
        assert_core_err(&result, MplCoreError::InvalidAuthority);
        assert_eq!(
            f.asset(&asset).owner,
            actors.owner,
            "{rule_set:?} should reject the transfer"
        );
    }
}

/// The deny list is matched against the *new owner's* owning program too, not
/// just the signer's (`royalties.rs:131-135`).
#[test]
fn royalties_deny_list_rejects_a_program_owned_new_owner() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let creator = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();
    f.store(
        new_owner,
        Account {
            lamports: ACCOUNT_LAMPORTS,
            data: vec![],
            owner: RECORDER_ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(
            royalties(creator, RuleSet::ProgramDenyList(vec![RECORDER_ID])),
            Authority::UpdateAuthority,
        )],
    );

    let result = f.run(&ix::transfer_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        new_owner,
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(f.asset(&asset).owner, actors.owner);
}

/// JS: plugins/asset/royalties.test.ts :: it cannot transfer an asset with collection royalties to a program address not on allowlist
#[test]
fn collection_royalties_reject_a_member_transfer() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let creator = Pubkey::new_unique();
    let new_owner = f.fund(ACCOUNT_LAMPORTS);

    let (collection, asset) = store_collection_with_member(
        &f,
        actors.update_authority,
        actors.owner,
        &[(
            royalties(creator, RuleSet::ProgramAllowList(vec![RECORDER_ID])),
            Authority::UpdateAuthority,
        )],
        &[],
    );

    let result = f.run(&ix::transfer_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(actors.owner),
        new_owner,
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(f.asset(&asset).owner, actors.owner);
}

// ===========================================================================
// verifiedCreators.test.ts
// ===========================================================================

/// A `VerifiedCreators` plugin from `(address, verified)` pairs.
fn verified_creators(signatures: &[(Pubkey, bool)]) -> Plugin {
    Plugin::VerifiedCreators(VerifiedCreators {
        signatures: signatures
            .iter()
            .map(|(address, verified)| VerifiedCreatorsSignature {
                address: *address,
                verified: *verified,
            })
            .collect(),
    })
}

/// JS: plugins/asset/verifiedCreators.test.ts :: it can create asset with verified creators plugin
/// JS: plugins/asset/verifiedCreators.test.ts :: it can create asset with verified creators plugin with authorized signature
#[test]
fn can_create_an_asset_with_verified_creators() {
    let f = Fixture::new();
    let payer = f.fund(ACCOUNT_LAMPORTS);
    let other = Pubkey::new_unique();

    // The payer is the create authority, so it may add itself verified and
    // anyone else unverified.
    let plugin = verified_creators(&[(payer, true), (other, false)]);
    let asset = Pubkey::new_unique();
    f.store(asset, empty_account());
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
        Some(vec![plugin_pair(&plugin, None)]),
        None,
        &[],
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::VerifiedCreators,
        &Authority::UpdateAuthority,
        &plugin,
    );
}

/// JS: plugins/asset/verifiedCreators.test.ts :: it cannot create asset with verified creators plugin and unauthorized signature
#[test]
fn cannot_create_an_asset_verifying_a_third_party() {
    let f = Fixture::new();
    let payer = f.fund(ACCOUNT_LAMPORTS);
    let other = Pubkey::new_unique();

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
        Some(vec![plugin_pair(
            &verified_creators(&[(other, true)]),
            None,
        )]),
        None,
        &[],
    ));
    assert_core_err(&result, MplCoreError::MissingSigner);
}

/// JS: plugins/asset/verifiedCreators.test.ts :: it cannot add duplicate verified creator signatures
#[test]
fn cannot_create_an_asset_with_duplicate_verified_creators() {
    let f = Fixture::new();
    let payer = f.fund(ACCOUNT_LAMPORTS);

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
        Some(vec![plugin_pair(
            &verified_creators(&[(payer, false), (payer, false)]),
            None,
        )]),
        None,
        &[],
    ));
    assert_core_err(&result, MplCoreError::InvalidPluginSetting);
}

/// JS: plugins/asset/verifiedCreators.test.ts :: it can create asset with verified creators plugin with unverified signatures and then verify
/// JS: plugins/asset/verifiedCreators.test.ts :: it can unverify signature verified creator plugin
#[test]
fn a_creator_can_verify_and_unverify_itself() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let creator = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(
            verified_creators(&[(actors.update_authority, true), (creator, false)]),
            Authority::UpdateAuthority,
        )],
    );

    let verified = verified_creators(&[(actors.update_authority, true), (creator, true)]);
    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(creator),
        None,
        client_plugin(&verified),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::VerifiedCreators,
        &Authority::UpdateAuthority,
        &verified,
    );

    let unverified = verified_creators(&[(actors.update_authority, true), (creator, false)]);
    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(creator),
        None,
        client_plugin(&unverified),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::VerifiedCreators,
        &Authority::UpdateAuthority,
        &unverified,
    );
}

/// JS: plugins/asset/verifiedCreators.test.ts :: it cannot verify a verified creator plugin with unauthorized signature
#[test]
fn a_creator_cannot_verify_someone_else() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let creator = f.fund(ACCOUNT_LAMPORTS);
    let other = Pubkey::new_unique();
    let stored_plugin = verified_creators(&[(creator, false), (other, false)]);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(stored_plugin.clone(), Authority::UpdateAuthority)],
    );

    let result = f.run(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(creator),
        None,
        client_plugin(&verified_creators(&[(creator, false), (other, true)])),
    ));
    assert_core_err(&result, MplCoreError::MissingSigner);
    assert_plugin(
        &f.account(&asset),
        PluginType::VerifiedCreators,
        &Authority::UpdateAuthority,
        &stored_plugin,
    );
}

/// JS: plugins/asset/verifiedCreators.test.ts :: it cannot remove verified creator plugin signture with unauthorized signature
#[test]
fn a_creator_cannot_remove_an_entry() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let creator = f.fund(ACCOUNT_LAMPORTS);
    let other = Pubkey::new_unique();
    let stored_plugin = verified_creators(&[(creator, false), (other, false)]);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(stored_plugin.clone(), Authority::UpdateAuthority)],
    );

    let result = f.run(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(creator),
        None,
        client_plugin(&verified_creators(&[(creator, false)])),
    ));
    assert_core_err(&result, MplCoreError::MissingSigner);
    assert_plugin(
        &f.account(&asset),
        PluginType::VerifiedCreators,
        &Authority::UpdateAuthority,
        &stored_plugin,
    );
}

/// JS: plugins/asset/verifiedCreators.test.ts :: it cannot remove verified creator plugin signature with update auth
/// JS: plugins/asset/verifiedCreators.test.ts :: it cannot unverify verified creator plugin signature with update auth
#[test]
fn the_plugin_authority_cannot_touch_someone_elses_verified_entry() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let creator = Pubkey::new_unique();
    let stored_plugin = verified_creators(&[(creator, true)]);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(stored_plugin.clone(), Authority::UpdateAuthority)],
    );

    // Removing a verified entry that is not the signer.
    let result = f.run(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&verified_creators(&[])),
    ));
    assert_core_err(&result, MplCoreError::InvalidPluginOperation);

    // Flipping someone else's flag.
    let result = f.run(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&verified_creators(&[(creator, false)])),
    ));
    assert_core_err(&result, MplCoreError::InvalidPluginOperation);
    assert_plugin(
        &f.account(&asset),
        PluginType::VerifiedCreators,
        &Authority::UpdateAuthority,
        &stored_plugin,
    );
}

/// JS: plugins/asset/verifiedCreators.test.ts :: it can remove and add unverified creator plugin signature with update auth
#[test]
fn the_plugin_authority_can_edit_unverified_entries() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let first = Pubkey::new_unique();
    let second = Pubkey::new_unique();

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(
            verified_creators(&[(first, false)]),
            Authority::UpdateAuthority,
        )],
    );

    let replaced = verified_creators(&[(second, false)]);
    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&replaced),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::VerifiedCreators,
        &Authority::UpdateAuthority,
        &replaced,
    );
}

// ===========================================================================
// autograph.test.ts
// ===========================================================================

/// An `Autograph` plugin from `(address, message)` pairs.
fn autograph(signatures: &[(Pubkey, &str)]) -> Plugin {
    Plugin::Autograph(Autograph {
        signatures: signatures
            .iter()
            .map(|(address, message)| AutographSignature {
                address: *address,
                message: message.to_string(),
            })
            .collect(),
    })
}

/// JS: plugins/asset/autograph.test.ts :: it can create asset with autograph plugin with authorized signature
#[test]
fn can_create_an_asset_with_an_own_autograph() {
    let f = Fixture::new();
    let payer = f.fund(ACCOUNT_LAMPORTS);

    let plugin = autograph(&[(payer, "hi")]);
    let asset = Pubkey::new_unique();
    f.store(asset, empty_account());
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
        Some(vec![plugin_pair(&plugin, None)]),
        None,
        &[],
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::Autograph,
        &Authority::Owner,
        &plugin,
    );
}

/// JS: plugins/asset/autograph.test.ts :: it cannot create asset with autograph plugin and unauthorized signature
#[test]
fn cannot_create_an_asset_with_a_third_party_autograph() {
    let f = Fixture::new();
    let payer = f.fund(ACCOUNT_LAMPORTS);
    let other = Pubkey::new_unique();

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
        Some(vec![plugin_pair(&autograph(&[(other, "hi")]), None)]),
        None,
        &[],
    ));
    assert_core_err(&result, MplCoreError::MissingSigner);
}

/// JS: plugins/asset/autograph.test.ts :: it cannot add duplicate autographs
#[test]
fn cannot_create_an_asset_with_duplicate_autographs() {
    let f = Fixture::new();
    let payer = f.fund(ACCOUNT_LAMPORTS);

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
        Some(vec![plugin_pair(
            &autograph(&[(payer, "a"), (payer, "b")]),
            None,
        )]),
        None,
        &[],
    ));
    assert_core_err(&result, MplCoreError::InvalidPluginSetting);
}

/// JS: plugins/asset/autograph.test.ts :: it can add additional autograph to asset via update by 3rd party
#[test]
fn a_third_party_can_append_its_own_autograph() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let third_party = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(autograph(&[(actors.owner, "first")]), Authority::Owner)],
    );

    let appended = autograph(&[(actors.owner, "first"), (third_party, "second")]);
    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(third_party),
        None,
        client_plugin(&appended),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::Autograph,
        &Authority::Owner,
        &appended,
    );
}

/// JS: plugins/asset/autograph.test.ts :: it cannot modify autograph message as owner
#[test]
fn nobody_can_rewrite_an_existing_autograph_message() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let stored_plugin = autograph(&[(actors.owner, "first")]);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(stored_plugin.clone(), Authority::Owner)],
    );

    let result = f.run(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_plugin(&autograph(&[(actors.owner, "edited")])),
    ));
    assert_core_err(&result, MplCoreError::InvalidPluginOperation);
    assert_plugin(
        &f.account(&asset),
        PluginType::Autograph,
        &Authority::Owner,
        &stored_plugin,
    );
}

/// JS: plugins/asset/autograph.test.ts :: it cannot remove autograph if not owner
/// JS: plugins/asset/autograph.test.ts :: it can remove autograph if owner
#[test]
fn only_the_plugin_authority_can_remove_an_autograph() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let third_party = f.fund(ACCOUNT_LAMPORTS);
    let stored_plugin = autograph(&[(actors.owner, "first"), (third_party, "second")]);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(stored_plugin.clone(), Authority::Owner)],
    );

    let result = f.run(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(third_party),
        None,
        client_plugin(&autograph(&[(third_party, "second")])),
    ));
    assert_core_err(&result, MplCoreError::MissingSigner);
    assert_plugin(
        &f.account(&asset),
        PluginType::Autograph,
        &Authority::Owner,
        &stored_plugin,
    );

    let trimmed = autograph(&[(actors.owner, "first")]);
    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_plugin(&trimmed),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::Autograph,
        &Authority::Owner,
        &trimmed,
    );
}

/// JS: plugins/asset/autograph.test.ts :: it can add autograph plugin to asset by owner
#[test]
fn the_owner_can_add_an_autograph_plugin() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let asset = store_asset(&f, actors.owner, actors.update_authority, &[]);
    let plugin = autograph(&[(actors.owner, "hi")]);

    f.run_ok(&ix::add_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_plugin(&plugin),
        None,
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::Autograph,
        &Authority::Owner,
        &plugin,
    );
}

/// JS: plugins/asset/autograph.test.ts :: it cannot add autograph to asset by unauthorized 3rd party
#[test]
fn a_third_party_cannot_add_an_autograph_plugin_for_someone_else() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let third_party = f.fund(ACCOUNT_LAMPORTS);

    let asset = store_asset(&f, actors.owner, actors.update_authority, &[]);

    let result = f.run(&ix::add_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(third_party),
        None,
        client_plugin(&autograph(&[(actors.owner, "hi")])),
        None,
    ));
    assert_core_err(&result, MplCoreError::MissingSigner);
}

// ===========================================================================
// addBlocker.test.ts / immutableMetadata.test.ts / edition.test.ts
// ===========================================================================

/// JS: plugins/asset/addBlocker.test.ts :: it cannot add UA-managed plugin if addBlocker had been added on creation
/// JS: plugins/asset/addBlocker.test.ts :: it can add owner-managed plugins even if AddBlocker had been added
#[test]
fn add_blocker_rejects_authority_managed_plugins_only() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(
            Plugin::AddBlocker(AddBlocker {}),
            Authority::UpdateAuthority,
        )],
    );

    let result = f.run(&ix::add_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&attributes()),
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    // Owner-managed plugins take the `abstain` arm.
    f.run_ok(&ix::add_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_plugin(&freeze(false)),
        None,
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeDelegate,
        &Authority::Owner,
        &freeze(false),
    );
}

/// JS: plugins/collection/addBlocker.test.ts :: it can add addBlocker to collection
/// JS: plugins/collection/addBlocker.test.ts :: it cannot add UA-managed plugin to an asset in a collection if addBlocker had been added on creation
#[test]
fn collection_add_blocker_blocks_member_assets() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let (collection, asset) =
        store_collection_with_member(&f, actors.update_authority, actors.owner, &[], &[]);

    // Adding AddBlocker itself is allowed: the new plugin self-validates and
    // takes the `PluginType::AddBlocker` abstain arm.
    f.run_ok(&ix::add_collection_plugin_v1(
        collection,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&Plugin::AddBlocker(AddBlocker {})),
        None,
    ));
    assert_plugin(
        &f.account(&collection),
        PluginType::AddBlocker,
        &Authority::UpdateAuthority,
        &Plugin::AddBlocker(AddBlocker {}),
    );

    // The collection's AddBlocker is inherited by its members.
    let result = f.run(&ix::add_plugin_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&attributes()),
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
}

/// JS: plugins/asset/immutableMetadata.test.ts :: it can prevent the asset from metadata updating
#[test]
fn immutable_metadata_blocks_asset_updates() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    // `Authority::None` is the normal configuration: `check_update` is
    // `CanReject`, so the plugin is evaluated anyway.
    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(
            Plugin::ImmutableMetadata(ImmutableMetadata {}),
            Authority::None,
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
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(f.asset(&asset).name, "Test Asset");

    // The rejection also covers a pure authority change, not only name/uri.
    let result = f.run(&ix::update_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        None,
        None,
        Some(convert(&UpdateAuthority::Address(actors.owner))),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
}

/// JS: plugins/collection/immutableMetadata.test.ts :: it can prevent collection assets metadata from being updated
#[test]
fn collection_immutable_metadata_blocks_member_updates() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let (collection, asset) = store_collection_with_member(
        &f,
        actors.update_authority,
        actors.owner,
        &[(
            Plugin::ImmutableMetadata(ImmutableMetadata {}),
            Authority::None,
        )],
        &[],
    );

    let result = f.run(&ix::update_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(actors.update_authority),
        None,
        Some("Renamed".to_string()),
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(f.asset(&asset).name, "Test Asset");

    let result = f.run(&ix::update_collection_v1(
        collection,
        actors.payer,
        Some(actors.update_authority),
        None,
        None,
        Some("Renamed".to_string()),
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(f.collection(&collection).name, "Test Collection");
}

/// JS: plugins/asset/edition.test.ts :: it cannot remove edition plugin
#[test]
fn edition_cannot_be_removed_but_other_plugins_can() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let edition = Plugin::Edition(Edition { number: 1 });

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[
            (edition.clone(), Authority::UpdateAuthority),
            (attributes(), Authority::UpdateAuthority),
        ],
    );

    let result = f.run(&ix::remove_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::Edition),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    // `Edition::validate_remove_plugin` abstains for a different target.
    f.run_ok(&ix::remove_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::Attributes),
    ));
    let account = f.account(&asset);
    assert_registry_consistent(&account);
    assert!(read_plugin(&account, PluginType::Attributes).is_none());
    assert_plugin(
        &account,
        PluginType::Edition,
        &Authority::UpdateAuthority,
        &edition,
    );
}

// ===========================================================================
// bubblegumV2.test.ts (collection)
// ===========================================================================

/// The authority a `BubblegumV2` record always carries.
fn bubblegum_authority() -> Authority {
    address(mpl_bubblegum::ID)
}

/// JS: plugins/collection/bubblegumV2.test.ts :: it cannot add BubblegumV2 to collection after creation
/// JS: plugins/asset/bubblegumV2.test.ts :: it cannot add BubblegumV2 to asset
#[test]
fn bubblegum_v2_cannot_be_added_after_creation() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let collection = store_collection(&f, actors.update_authority, &[]);
    let result = f.run(&ix::add_collection_plugin_v1(
        collection,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&Plugin::BubblegumV2(BubblegumV2 {})),
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    let asset = store_asset(&f, actors.owner, actors.update_authority, &[]);
    let result = f.run(&ix::add_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&Plugin::BubblegumV2(BubblegumV2 {})),
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
}

/// JS: plugins/collection/bubblegumV2.test.ts :: Update Authority cannot remove BubblegumV2 from collection
#[test]
fn bubblegum_v2_cannot_be_removed_from_a_collection() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let plugin = Plugin::BubblegumV2(BubblegumV2 {});

    let collection = store_collection(
        &f,
        actors.update_authority,
        &[
            (plugin.clone(), bubblegum_authority()),
            (attributes(), Authority::UpdateAuthority),
        ],
    );

    let result = f.run(&ix::remove_collection_plugin_v1(
        collection,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::BubblegumV2),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    // `validate_remove_plugin` abstains for any other target.
    f.run_ok(&ix::remove_collection_plugin_v1(
        collection,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::Attributes),
    ));
    let account = f.account(&collection);
    assert_registry_consistent(&account);
    assert!(read_plugin(&account, PluginType::Attributes).is_none());
    assert_plugin(
        &account,
        PluginType::BubblegumV2,
        &bubblegum_authority(),
        &plugin,
    );
}

/// JS: plugins/collection/bubblegumV2.test.ts :: it can add allow-listed plugins to collection with BubblegumV2 plugin
/// JS: plugins/collection/bubblegumV2.test.ts :: it cannot add non-allow-listed plugins to collection with BubblegumV2 plugin
/// JS: plugins/collection/bubblegumV2.test.ts :: it can add non-allow-listed plugin to asset in BubblegumV2 collection
#[test]
fn bubblegum_v2_restricts_collection_plugins_but_not_member_assets() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let (collection, asset) = store_collection_with_member(
        &f,
        actors.update_authority,
        actors.owner,
        &[(Plugin::BubblegumV2(BubblegumV2 {}), bubblegum_authority())],
        &[],
    );

    // Attributes is on `BubblegumV2::ALLOW_LIST`.
    f.run_ok(&ix::add_collection_plugin_v1(
        collection,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&attributes()),
        None,
    ));
    assert_plugin(
        &f.account(&collection),
        PluginType::Attributes,
        &Authority::UpdateAuthority,
        &attributes(),
    );

    // ImmutableMetadata is not.
    let result = f.run(&ix::add_collection_plugin_v1(
        collection,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&Plugin::ImmutableMetadata(ImmutableMetadata {})),
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    // Assets in the collection are unaffected (`asset_info.is_some()` arm).
    f.run_ok(&ix::add_plugin_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&Plugin::ImmutableMetadata(ImmutableMetadata {})),
        None,
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::ImmutableMetadata,
        &Authority::UpdateAuthority,
        &Plugin::ImmutableMetadata(ImmutableMetadata {}),
    );
}

// ===========================================================================
// plugins/collection/updateDelegate.test.ts
// ===========================================================================

/// JS: plugins/collection/updateDelegate.test.ts :: an updateDelegate on collection can update an asset
/// JS: plugins/collection/updateDelegate.test.ts :: an updateDelegate additionalDelegate on collection can update an asset
#[test]
fn collection_update_delegate_can_update_a_member_asset() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = f.fund(ACCOUNT_LAMPORTS);
    let additional = f.fund(ACCOUNT_LAMPORTS);

    let (collection, asset) = store_collection_with_member(
        &f,
        actors.update_authority,
        actors.owner,
        &[(update_delegate(&[additional]), address(delegate))],
        &[],
    );

    f.run_ok(&ix::update_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(delegate),
        None,
        Some("By delegate".to_string()),
        None,
        None,
    ));
    assert_eq!(f.asset(&asset).name, "By delegate");

    f.run_ok(&ix::update_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(additional),
        None,
        Some("By additional".to_string()),
        None,
        None,
    ));
    assert_eq!(f.asset(&asset).name, "By additional");
}

/// JS: plugins/collection/updateDelegate.test.ts :: it can update collection details as an updateDelegate additional delegate
#[test]
fn collection_additional_delegate_can_update_the_collection() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let additional = f.fund(ACCOUNT_LAMPORTS);

    let collection = store_collection(
        &f,
        actors.update_authority,
        &[(update_delegate(&[additional]), Authority::UpdateAuthority)],
    );

    f.run_ok(&ix::update_collection_v1(
        collection,
        actors.payer,
        Some(additional),
        None,
        None,
        Some("Renamed".to_string()),
        None,
    ));
    assert_eq!(f.collection(&collection).name, "Renamed");
    assert_registry_consistent(&f.account(&collection));
}

/// JS: plugins/collection/updateDelegate.test.ts :: it can update an authority-managed plugin on an asset as collection update additional delegate
/// JS: plugins/collection/updateDelegate.test.ts :: it cannot update an owner-managed plugin on an asset as collection update additional delegate
#[test]
fn collection_additional_delegate_can_only_update_authority_managed_member_plugins() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let additional = f.fund(ACCOUNT_LAMPORTS);

    let (collection, asset) = store_collection_with_member(
        &f,
        actors.update_authority,
        actors.owner,
        &[(update_delegate(&[additional]), Authority::UpdateAuthority)],
        &[
            (permanent_freeze(true), Authority::UpdateAuthority),
            (freeze(true), Authority::Owner),
        ],
    );

    f.run_ok(&ix::update_plugin_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(additional),
        None,
        client_plugin(&permanent_freeze(false)),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::PermanentFreezeDelegate,
        &Authority::UpdateAuthority,
        &permanent_freeze(false),
    );

    let result = f.run(&ix::update_plugin_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(additional),
        None,
        client_plugin(&freeze(false)),
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_plugin(
        &f.account(&asset),
        PluginType::FreezeDelegate,
        &Authority::Owner,
        &freeze(true),
    );
}

/// Roadmap section 12, observation 4: an asset-level plugin shadows the
/// collection plugin of the same type, so a collection `UpdateDelegate`
/// cannot act on an asset that carries its own. There is no JS test for this.
#[test]
fn an_asset_level_update_delegate_shadows_the_collection_one() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let collection_delegate = f.fund(ACCOUNT_LAMPORTS);

    let (collection, asset) = store_collection_with_member(
        &f,
        actors.update_authority,
        actors.owner,
        &[(update_delegate(&[]), address(collection_delegate))],
        // The asset's own UpdateDelegate is delegated to nobody in
        // particular, which is what makes the collection's unreachable.
        &[(update_delegate(&[]), Authority::UpdateAuthority)],
    );

    let result = f.run(&ix::update_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(collection_delegate),
        None,
        Some("Renamed".to_string()),
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_eq!(f.asset(&asset).name, "Test Asset");

    // Removing the asset-level plugin (as the update authority, who does
    // approve) lets the collection delegate act again.
    f.run_ok(&ix::remove_plugin_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::UpdateDelegate),
    ));
    f.run_ok(&ix::update_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(collection_delegate),
        None,
        Some("Renamed".to_string()),
        None,
        None,
    ));
    assert_eq!(f.asset(&asset).name, "Renamed");
}

/// JS: plugins/collection/updateDelegate.test.ts :: it can approve/revoke non-updateDelegate plugin on an asset as collection update additional delegate
#[test]
fn collection_additional_delegate_can_approve_and_revoke_member_plugin_authority() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let additional = f.fund(ACCOUNT_LAMPORTS);
    let new_authority = Pubkey::new_unique();
    let edition = Plugin::Edition(Edition { number: 1 });

    let (collection, asset) = store_collection_with_member(
        &f,
        actors.update_authority,
        actors.owner,
        &[(update_delegate(&[additional]), Authority::UpdateAuthority)],
        &[(edition.clone(), Authority::UpdateAuthority)],
    );

    f.run_ok(&ix::approve_plugin_authority_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(additional),
        None,
        client_type(PluginType::Edition),
        client_authority(&address(new_authority)),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::Edition,
        &address(new_authority),
        &edition,
    );

    f.run_ok(&ix::revoke_plugin_authority_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(additional),
        None,
        client_type(PluginType::Edition),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::Edition,
        &Authority::UpdateAuthority,
        &edition,
    );
}

/// JS: plugins/asset/verifiedCreators.test.ts :: it cannot add verified creator plugin to asset by owner
#[test]
fn the_owner_cannot_add_a_verified_creators_plugin() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let asset = store_asset(&f, actors.owner, actors.update_authority, &[]);

    // `VerifiedCreators` is authority-managed, so the owner's approval does
    // not carry the add.
    let result = f.run(&ix::add_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_plugin(&Plugin::VerifiedCreators(VerifiedCreators {
            signatures: vec![VerifiedCreatorsSignature {
                address: actors.owner,
                verified: true,
            }],
        })),
        None,
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);
}

// ===========================================================================
// Delegates resolved through the collection
// ===========================================================================

/// JS: plugins/asset/burnDelegate.test.ts :: a burnDelegate can burn using delegated update authority from collection
#[test]
fn burn_delegate_delegated_to_the_collection_update_authority() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    // The record's authority is `UpdateAuthority`, which for an asset in a
    // collection resolves to the *collection's* update authority.
    let (collection, asset) = store_collection_with_member(
        &f,
        actors.update_authority,
        actors.owner,
        &[],
        &[(
            Plugin::BurnDelegate(BurnDelegate {}),
            Authority::UpdateAuthority,
        )],
    );
    let size_before = f.collection(&collection).current_size;

    f.run_ok(&ix::burn_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(actors.update_authority),
        None,
        None,
    ));
    f.assert_burned(&asset);
    assert_eq!(f.collection(&collection).current_size, size_before - 1);
}

/// JS: plugins/asset/delegateTransfer.test.ts :: it can transfer using delegated update authority from collection
#[test]
fn transfer_delegate_delegated_to_the_collection_update_authority() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);

    let (collection, asset) = store_collection_with_member(
        &f,
        actors.update_authority,
        actors.owner,
        &[],
        &[(
            Plugin::TransferDelegate(TransferDelegate {}),
            Authority::UpdateAuthority,
        )],
    );

    f.run_ok(&ix::transfer_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(actors.update_authority),
        new_owner,
        None,
        None,
    ));
    assert_eq!(f.asset(&asset).owner, new_owner);
}

/// JS: plugins/asset/permanentBurn.test.ts :: it can burn an assets as a delegate for a collection
#[test]
fn collection_permanent_burn_delegate_can_burn_a_member() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = f.fund(ACCOUNT_LAMPORTS);

    let (collection, asset) = store_collection_with_member(
        &f,
        actors.update_authority,
        actors.owner,
        &[(
            Plugin::PermanentBurnDelegate(PermanentBurnDelegate {}),
            address(delegate),
        )],
        &[],
    );

    f.run_ok(&ix::burn_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(delegate),
        None,
        None,
    ));
    f.assert_burned(&asset);
}

/// JS: plugins/asset/permanentTransfer.test.ts :: it can collection permanent transfer asset that is frozen as a collection delegate
#[test]
fn collection_permanent_transfer_delegate_force_approves_a_frozen_member() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let delegate = f.fund(ACCOUNT_LAMPORTS);
    let new_owner = f.fund(ACCOUNT_LAMPORTS);

    let (collection, asset) = store_collection_with_member(
        &f,
        actors.update_authority,
        actors.owner,
        &[(
            Plugin::PermanentTransferDelegate(PermanentTransferDelegate {}),
            address(delegate),
        )],
        &[(freeze(true), Authority::Owner)],
    );

    f.run_ok(&ix::transfer_v1(
        asset,
        Some(collection),
        actors.payer,
        Some(delegate),
        new_owner,
        None,
        None,
    ));
    assert_eq!(f.asset(&asset).owner, new_owner);
}

/// JS: plugins/collection/permanentFreeze.test.ts :: it cannot remove permanentFreezeDelegate from collection when frozen
/// JS: plugins/collection/permanentFreeze.test.ts :: it can remove permanentFreezeDelegate from collection
#[test]
fn collection_permanent_freeze_delegate_removal_depends_on_frozen() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let collection = store_collection(
        &f,
        actors.update_authority,
        &[(permanent_freeze(true), Authority::UpdateAuthority)],
    );

    let result = f.run(&ix::remove_collection_plugin_v1(
        collection,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::PermanentFreezeDelegate),
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    f.run_ok(&ix::update_collection_plugin_v1(
        collection,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&permanent_freeze(false)),
    ));
    f.run_ok(&ix::remove_collection_plugin_v1(
        collection,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::PermanentFreezeDelegate),
    ));
    let account = f.account(&collection);
    assert_registry_consistent(&account);
    assert!(read_plugin(&account, PluginType::PermanentFreezeDelegate).is_none());
}

/// JS: plugins/asset/edition.test.ts :: it can update edition plugin
/// JS: plugins/asset/edition.test.ts :: it cannot update edition plugin as owner
#[test]
fn only_the_update_authority_can_update_the_edition_number() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let asset = store_asset(
        &f,
        actors.owner,
        actors.update_authority,
        &[(
            Plugin::Edition(Edition { number: 1 }),
            Authority::UpdateAuthority,
        )],
    );

    let result = f.run(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_plugin(&Plugin::Edition(Edition { number: 2 })),
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);

    f.run_ok(&ix::update_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&Plugin::Edition(Edition { number: 2 })),
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::Edition,
        &Authority::UpdateAuthority,
        &Plugin::Edition(Edition { number: 2 }),
    );
}

/// JS: plugins/asset/addBlocker.test.ts :: it states that UA is the only one who can add the AddBlocker
#[test]
fn only_the_update_authority_can_add_an_add_blocker() {
    let f = Fixture::new();
    let actors = Actors::new(&f);

    let asset = store_asset(&f, actors.owner, actors.update_authority, &[]);

    let result = f.run(&ix::add_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.owner),
        None,
        client_plugin(&Plugin::AddBlocker(AddBlocker {})),
        None,
    ));
    assert_core_err(&result, MplCoreError::NoApprovals);

    f.run_ok(&ix::add_plugin_v1(
        asset,
        None,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_plugin(&Plugin::AddBlocker(AddBlocker {})),
        None,
    ));
    assert_plugin(
        &f.account(&asset),
        PluginType::AddBlocker,
        &Authority::UpdateAuthority,
        &Plugin::AddBlocker(AddBlocker {}),
    );
}

/// JS: revokeAuthority.test.ts :: it can remove an authority from a plugin
///
/// The collection variants of `approve_authority_on_plugin` and
/// `revoke_authority_on_plugin`, which no other test reaches.
#[test]
fn collection_plugin_authority_can_be_approved_and_revoked() {
    let f = Fixture::new();
    let actors = Actors::new(&f);
    let creator = Pubkey::new_unique();
    let new_authority = Pubkey::new_unique();
    let plugin = royalties(creator, RuleSet::None);

    let collection = store_collection(
        &f,
        actors.update_authority,
        &[(plugin.clone(), Authority::UpdateAuthority)],
    );
    let len_before = f.account(&collection).data.len();

    f.run_ok(&ix::approve_collection_plugin_authority_v1(
        collection,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::Royalties),
        client_authority(&address(new_authority)),
    ));
    assert_plugin(
        &f.account(&collection),
        PluginType::Royalties,
        &address(new_authority),
        &plugin,
    );
    assert_eq!(
        f.account(&collection).data.len(),
        len_before + 32,
        "an Address authority adds a pubkey to the registry record"
    );

    f.run_ok(&ix::revoke_collection_plugin_authority_v1(
        collection,
        actors.payer,
        Some(actors.update_authority),
        None,
        client_type(PluginType::Royalties),
    ));
    assert_plugin(
        &f.account(&collection),
        PluginType::Royalties,
        &Authority::UpdateAuthority,
        &plugin,
    );
    assert_eq!(f.account(&collection).data.len(), len_before);
}
