//! `BurnV1`, `BurnCollectionV1` (`src/processor/burn.rs`) and the paths of
//! `TransferV1` (`src/processor/transfer.rs`) that no other module reaches.
//!
//! Burning leaves the account at one `Uninitialized` byte and moves
//! `rent(len) - rent(1)` to the payer, so the success tests assert both the
//! remains of the account and the lamport flow.
//!
//! The `Groups`-plugin rejections of `burn.test.ts` / `burnCollection.test.ts`
//! need a `GroupV1` fixture and belong to the groups section of the roadmap;
//! the full compressed-burn and compressed-transfer paths need a hashed-asset
//! fixture with a matching proof (roadmap section 8, tests 29 and 33). Only
//! the guard both share (`MissingCompressionProof`) is covered here.

use {
    crate::common::*,
    mpl_core_program::{
        error::MplCoreError,
        plugins::{
            Attribute, Attributes, BurnDelegate, FreezeDelegate, PermanentBurnDelegate,
            PermanentFreezeDelegate, Plugin, PluginType, TransferDelegate,
        },
        state::{Authority, Key, UpdateAuthority},
    },
    solana_program::{instruction::AccountMeta, program_error::ProgramError, pubkey::Pubkey},
};

fn freeze(frozen: bool) -> Plugin {
    Plugin::FreezeDelegate(FreezeDelegate { frozen })
}

fn permanent_freeze(frozen: bool) -> Plugin {
    Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen })
}

fn attributes() -> Plugin {
    Plugin::Attributes(Attributes {
        attribute_list: vec![Attribute {
            key: "k".to_string(),
            value: "v".to_string(),
        }],
    })
}

fn address(key: Pubkey) -> Authority {
    Authority::Address { address: key }
}

/// The lamports `close_program_account` moves out of an account of `data_len`
/// bytes: the rent of the account minus the rent of the one byte it keeps.
fn burn_refund(f: &Fixture, data_len: usize) -> u64 {
    let rent = &f.ctx.mollusk.sysvars.rent;
    rent.minimum_balance(data_len) - rent.minimum_balance(1)
}

// ===========================================================================
// BurnV1
// ===========================================================================

/// JS: burn.test.ts :: it can burn an asset as the owner
#[test]
fn burn_as_owner() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(asset, AssetSpec::new(owner).build());
    let len = f.account(&asset).data.len();
    let asset_lamports = f.lamports(&asset);
    let refund = burn_refund(&f, len);

    f.run_ok(&ix::burn_v1(asset, None, owner, None, None, None));

    f.assert_burned(&asset);
    assert_eq!(
        f.lamports(&owner),
        ACCOUNT_LAMPORTS + refund,
        "the payer receives rent(len) - rent(1)"
    );
    assert_eq!(f.lamports(&asset), asset_lamports - refund);
}

/// JS: burn.test.ts :: it can burn asset with different payer
/// JS: burn.test.ts :: it can burn using owner authority
#[test]
fn burn_with_separate_payer_and_owner_authority() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let payer = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(asset, AssetSpec::new(owner).build());
    let refund = burn_refund(&f, f.account(&asset).data.len());

    f.run_ok(&ix::burn_v1(asset, None, payer, Some(owner), None, None));

    f.assert_burned(&asset);
    assert_eq!(
        f.lamports(&payer),
        ACCOUNT_LAMPORTS + refund,
        "the refund follows the payer, not the authority"
    );
    assert_eq!(f.lamports(&owner), ACCOUNT_LAMPORTS);
}

/// JS: burn.test.ts :: it cannot burn an asset if not the owner
/// JS: burn.test.ts :: it cannot burn an asset as the authority
#[test]
fn burn_rejects_non_owner_and_update_authority() {
    let f = Fixture::new();
    let owner = Pubkey::new_unique();
    let update_authority = f.fund(ACCOUNT_LAMPORTS);
    let stranger = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Address(update_authority))
            .build(),
    );

    for signer in [update_authority, stranger] {
        let result = f.run(&ix::burn_v1(asset, None, signer, None, None, None));
        assert_core_err(&result, MplCoreError::NoApprovals);
    }
    assert_eq!(
        read::key_of(&f.account(&asset).data),
        Key::AssetV1,
        "the asset must still be alive"
    );
}

/// JS: burn.test.ts :: it cannot use an invalid system program for assets
/// JS: burn.test.ts :: it cannot use an invalid noop program for assets
#[test]
fn burn_rejects_invalid_system_program_and_log_wrapper() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(asset, AssetSpec::new(owner).build());

    // Accounts: asset, collection, payer, authority, system program, log wrapper.
    let mut ix = ix::burn_v1(asset, None, owner, None, None, None);
    set_account(
        &mut ix,
        4,
        AccountMeta::new_readonly(Pubkey::new_unique(), false),
    );
    assert_core_err(&f.run(&ix), MplCoreError::InvalidSystemProgram);

    let ix = ix::burn_v1(asset, None, owner, None, Some(Pubkey::new_unique()), None);
    assert_core_err(&f.run(&ix), MplCoreError::InvalidLogWrapperProgram);
}

/// The payer must sign.
#[test]
fn burn_rejects_non_signer_payer() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(asset, AssetSpec::new(owner).build());

    let mut ix = ix::burn_v1(asset, None, owner, None, None, None);
    unsign(&mut ix, &owner);
    assert_program_err(&f.run(&ix), ProgramError::MissingRequiredSignature);
}

/// An account that is neither an `AssetV1` nor a `HashedAssetV1` is refused by
/// the discriminator match (`burn.rs:82`).
#[test]
fn burn_rejects_wrong_discriminator() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let not_an_asset = Pubkey::new_unique();
    f.store(not_an_asset, CollectionSpec::new(owner).build());

    let result = f.run(&ix::burn_v1(not_an_asset, None, owner, None, None, None));
    assert_core_err(&result, MplCoreError::IncorrectAccount);
}

/// The compressed branch needs a proof before it does anything else. The rest
/// of that branch needs a hashed-asset fixture (roadmap section 8, test 29).
#[test]
fn burn_compressed_requires_compression_proof() {
    let f = Fixture::new();
    let payer = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(asset, hashed_asset_placeholder());

    let result = f.run(&ix::burn_v1(asset, None, payer, None, None, None));
    assert_core_err(&result, MplCoreError::MissingCompressionProof);
}

/// JS: burn.test.ts :: it cannot burn asset in collection if no collection specified
/// JS: burn.test.ts :: it cannot burn an asset with the wrong collection specified
/// JS: collectionSize.test.ts :: it can burn an asset which is the part of a collection
#[test]
fn burn_asset_in_collection() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    let other_collection = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Collection(collection))
            .build(),
    );
    f.store(collection, CollectionSpec::new(owner).sizes(1, 1).build());
    f.store(other_collection, CollectionSpec::new(owner).build());

    let result = f.run(&ix::burn_v1(asset, None, owner, None, None, None));
    assert_core_err(&result, MplCoreError::MissingCollection);

    let result = f.run(&ix::burn_v1(
        asset,
        Some(other_collection),
        owner,
        None,
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidCollection);

    f.run_ok(&ix::burn_v1(
        asset,
        Some(collection),
        owner,
        None,
        None,
        None,
    ));
    f.assert_burned(&asset);
    let updated = f.collection(&collection);
    assert_eq!(updated.current_size, 0, "the collection shrinks by one");
    assert_eq!(updated.num_minted, 1, "num_minted is never decremented");
}

/// A collection whose `current_size` is already zero underflows when a member
/// asset is burned. Only a crafted account can reach this.
#[test]
fn burn_in_collection_rejects_size_underflow() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Collection(collection))
            .build(),
    );
    f.store(collection, CollectionSpec::new(owner).sizes(1, 0).build());

    let result = f.run(&ix::burn_v1(
        asset,
        Some(collection),
        owner,
        None,
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::NumericalOverflowError);
    assert_eq!(
        read::key_of(&f.account(&asset).data),
        Key::AssetV1,
        "the failed burn must leave the asset alone"
    );
}

/// JS: burn.test.ts :: it cannot burn an asset if it is frozen
/// JS: burn.test.ts :: it cannot burn an asset if collection permanently frozen
#[test]
fn burn_rejected_when_frozen() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);

    let frozen = Pubkey::new_unique();
    f.store(
        frozen,
        AssetSpec::new(owner)
            .plugin(freeze(true), Authority::Owner)
            .build(),
    );
    let result = f.run(&ix::burn_v1(frozen, None, owner, None, None, None));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    let member = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    f.store(
        member,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Collection(collection))
            .build(),
    );
    f.store(
        collection,
        CollectionSpec::new(owner)
            .sizes(1, 1)
            .plugin(permanent_freeze(true), Authority::UpdateAuthority)
            .build(),
    );
    let result = f.run(&ix::burn_v1(
        member,
        Some(collection),
        owner,
        None,
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_eq!(f.collection(&collection).current_size, 1);
}

/// The collection-level `PermanentBurnDelegate` case has no JS counterpart.
///
/// JS: plugins/asset/burnDelegate.test.ts :: a burnDelegate can burn an asset
#[test]
fn burn_by_burn_delegates() {
    // An asset-level `BurnDelegate`.
    let f = Fixture::new();
    let owner = Pubkey::new_unique();
    let delegate = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .plugin(Plugin::BurnDelegate(BurnDelegate {}), address(delegate))
            .build(),
    );
    f.run_ok(&ix::burn_v1(asset, None, delegate, None, None, None));
    f.assert_burned(&asset);

    // A collection-level `PermanentBurnDelegate`.
    let member = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    let permanent_delegate = f.fund(ACCOUNT_LAMPORTS);
    f.store(
        member,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Collection(collection))
            .build(),
    );
    f.store(
        collection,
        CollectionSpec::new(owner)
            .sizes(1, 1)
            .plugin(
                Plugin::PermanentBurnDelegate(PermanentBurnDelegate {}),
                address(permanent_delegate),
            )
            .build(),
    );
    f.run_ok(&ix::burn_v1(
        member,
        Some(collection),
        permanent_delegate,
        None,
        None,
        None,
    ));
    f.assert_burned(&member);
    assert_eq!(f.collection(&collection).current_size, 0);
}

// ===========================================================================
// BurnCollectionV1
// ===========================================================================

/// JS: burnCollection.test.ts :: it can burn a collection as the authority
/// JS: burnCollection.test.ts :: it can burn asset with different payer
#[test]
fn burn_collection_as_authority() {
    let f = Fixture::new();
    let authority = f.fund(ACCOUNT_LAMPORTS);
    let payer = f.fund(ACCOUNT_LAMPORTS);
    let collection = Pubkey::new_unique();
    f.store(
        collection,
        CollectionSpec::new(authority)
            .plugin(attributes(), Authority::UpdateAuthority)
            .build(),
    );
    let refund = burn_refund(&f, f.account(&collection).data.len());

    f.run_ok(&ix::burn_collection_v1(
        collection,
        payer,
        Some(authority),
        None,
        None,
    ));

    f.assert_burned(&collection);
    assert_eq!(f.lamports(&payer), ACCOUNT_LAMPORTS + refund);
    assert_eq!(f.lamports(&authority), ACCOUNT_LAMPORTS);
}

/// JS: burnCollection.test.ts :: it cannot burn a collection if it has Assets in it
/// JS: burnCollection.test.ts :: it cannot burn a collection if not the authority
/// JS: burnCollection.test.ts :: it cannot use an invalid noop program for collections
#[test]
fn burn_collection_rejections() {
    let f = Fixture::new();
    let authority = f.fund(ACCOUNT_LAMPORTS);
    let stranger = f.fund(ACCOUNT_LAMPORTS);

    // Non-empty collection: checked before the authority.
    let populated = Pubkey::new_unique();
    f.store(
        populated,
        CollectionSpec::new(authority).sizes(1, 1).build(),
    );
    let result = f.run(&ix::burn_collection_v1(
        populated, authority, None, None, None,
    ));
    assert_core_err(&result, MplCoreError::CollectionMustBeEmpty);

    // Empty collection, wrong authority.
    let collection = Pubkey::new_unique();
    f.store(collection, CollectionSpec::new(authority).build());
    let result = f.run(&ix::burn_collection_v1(
        collection, stranger, None, None, None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);

    // Bad log wrapper, checked before the collection is even loaded.
    let result = f.run(&ix::burn_collection_v1(
        collection,
        authority,
        None,
        Some(Pubkey::new_unique()),
        None,
    ));
    assert_core_err(&result, MplCoreError::InvalidLogWrapperProgram);

    assert_eq!(
        read::key_of(&f.account(&collection).data),
        Key::CollectionV1,
        "none of the rejected burns may have closed the collection"
    );
}

/// A frozen `PermanentFreezeDelegate` on the collection blocks the burn
/// through `validate_collection_permissions`, after the update-authority check.
#[test]
fn burn_collection_rejected_by_permanent_freeze() {
    let f = Fixture::new();
    let authority = f.fund(ACCOUNT_LAMPORTS);
    let collection = Pubkey::new_unique();
    f.store(
        collection,
        CollectionSpec::new(authority)
            .plugin(permanent_freeze(true), Authority::UpdateAuthority)
            .build(),
    );

    let result = f.run(&ix::burn_collection_v1(
        collection, authority, None, None, None,
    ));
    assert_core_err(&result, MplCoreError::InvalidAuthority);
}

// ===========================================================================
// TransferV1
// ===========================================================================

/// JS: transfer.test.ts :: it cannot use an invalid system program
/// JS: transfer.test.ts :: it cannot use an invalid noop program
#[test]
fn transfer_rejects_invalid_system_program_and_log_wrapper() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();
    f.store(asset, AssetSpec::new(owner).build());

    // Accounts: asset, collection, payer, authority, new owner, system program,
    // log wrapper.
    let mut ix = ix::transfer_v1(asset, None, owner, None, new_owner, None, None);
    set_account(
        &mut ix,
        5,
        AccountMeta::new_readonly(Pubkey::new_unique(), false),
    );
    assert_core_err(&f.run(&ix), MplCoreError::InvalidSystemProgram);

    let ix = ix::transfer_v1(
        asset,
        None,
        owner,
        None,
        new_owner,
        Some(Pubkey::new_unique()),
        None,
    );
    assert_core_err(&f.run(&ix), MplCoreError::InvalidLogWrapperProgram);
    assert_eq!(f.asset(&asset).owner, owner);
}

/// The compressed branch of `transfer` needs a proof, like burn's.
#[test]
fn transfer_compressed_requires_compression_proof() {
    let f = Fixture::new();
    let payer = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    f.store(asset, hashed_asset_placeholder());

    let result = f.run(&ix::transfer_v1(
        asset,
        None,
        payer,
        None,
        Pubkey::new_unique(),
        None,
        None,
    ));
    assert_core_err(&result, MplCoreError::MissingCompressionProof);
}

/// The sequence number is only bumped when the asset carries one
/// (`transfer.rs:139`); only compression sets it, so the asset is crafted.
#[test]
fn transfer_increments_seq_when_present() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();
    f.store(asset, AssetSpec::new(owner).seq(5).build());

    f.run_ok(&ix::transfer_v1(
        asset, None, owner, None, new_owner, None, None,
    ));

    let updated = f.asset(&asset);
    assert_eq!(updated.owner, new_owner);
    assert_eq!(updated.seq, Some(6));
}

/// JS: transfer.test.ts :: authorities on owner-managed plugins are reset on transfer
/// JS: transfer.test.ts :: authorities on permanent plugins should not be reset on transfer
/// JS: transfer.test.ts :: authorities on authority-managed plugin should not be reset on transfer
#[test]
fn transfer_resets_owner_managed_authorities() {
    let f = Fixture::new();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let delegate = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .plugin(freeze(false), address(delegate))
            .plugin(
                Plugin::TransferDelegate(TransferDelegate {}),
                address(delegate),
            )
            .plugin(permanent_freeze(false), address(delegate))
            .plugin(attributes(), address(delegate))
            .build(),
    );
    let len_before = f.account(&asset).data.len();

    f.run_ok(&ix::transfer_v1(
        asset, None, owner, None, new_owner, None, None,
    ));

    let account = f.account(&asset);
    let (core, parsed) = parse_asset(&account.data);
    assert_eq!(core.owner, new_owner);
    for owner_managed in [PluginType::FreezeDelegate, PluginType::TransferDelegate] {
        assert_eq!(
            parsed.plugin(owner_managed).unwrap().0,
            Authority::Owner,
            "{owner_managed:?} is owner-managed and must be reset"
        );
    }
    for kept in [PluginType::PermanentFreezeDelegate, PluginType::Attributes] {
        assert_eq!(
            parsed.plugin(kept).unwrap().0,
            address(delegate),
            "{kept:?} is not owner-managed and must keep its delegate"
        );
    }

    // `transfer` rewrites the registry in place and never reallocates, so
    // replacing two 33-byte `Address` authorities with the one-byte `Owner`
    // leaves 64 bytes of slack after the registry: the account keeps its size
    // and `assert_registry_consistent` (which requires the registry to end at
    // the end of the account) does not hold here. Readers are unaffected
    // because the registry is always read from `plugin_registry_offset`.
    // Behaviour pinned deliberately; see roadmap section 6.
    assert_eq!(
        account.data.len(),
        len_before,
        "resetting authorities in place must not resize the account"
    );
    let registry_offset = parsed.header.as_ref().unwrap().plugin_registry_offset;
    let registry_len = borsh::to_vec(parsed.registry.as_ref().unwrap())
        .unwrap()
        .len();
    assert_eq!(
        account.data.len() - (registry_offset + registry_len),
        64,
        "two Address authorities collapsed to Owner leave 64 unused trailing bytes"
    );
}

/// A delegate holding the `TransferDelegate` may transfer, and afterwards its
/// own record has been reset to the new owner.
///
/// JS: plugins/asset/delegateTransfer.test.ts :: a delegate can transfer the asset
#[test]
fn transfer_by_transfer_delegate() {
    let f = Fixture::new();
    let owner = Pubkey::new_unique();
    let delegate = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();
    f.store(
        asset,
        AssetSpec::new(owner)
            .plugin(
                Plugin::TransferDelegate(TransferDelegate {}),
                address(delegate),
            )
            .build(),
    );

    f.run_ok(&ix::transfer_v1(
        asset, None, delegate, None, new_owner, None, None,
    ));

    let account = f.account(&asset);
    assert_eq!(read_asset(&account).owner, new_owner);
    assert_eq!(
        read_plugin(&account, PluginType::TransferDelegate)
            .unwrap()
            .0,
        Authority::Owner,
        "the delegate is dropped as part of the transfer"
    );
}
