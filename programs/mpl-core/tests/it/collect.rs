//! `Collect` and `UpdateCollectionInfoV1`.
//!
//! `Collect` is the permissionless fee sweep: it pays everything an mpl-core
//! account holds above its rent minimum to two hard-coded recipients and
//! leaves the account at exactly that minimum. `UpdateCollectionInfoV1` is the
//! Bubblegum-only counter adjustment. Neither has any Mollusk or Rust-client
//! coverage today, and the JS `collect.test.ts` runs against the real
//! recipients on a validator, so every case here is new.

use {
    crate::common::*,
    mpl_core::types::UpdateType,
    mpl_core_program::{error::MplCoreError, state::Key, ID as MPL_CORE_ID},
    solana_account::Account,
    solana_program::{instruction::AccountMeta, program_error::ProgramError, pubkey::Pubkey},
    solana_system_interface::program as system_program,
};

/// The hard-coded fee recipients from `src/state/collect.rs`. They are
/// `pub(crate)` in the program, so the tests restate them; a change to either
/// constant makes these tests fail, which is the point.
const COLLECT_RECIPIENT1: Pubkey =
    solana_program::pubkey!("8AT6o8Qk5T9QnZvPThMrF9bcCQLTGkyGvVZZzHgCw11v");
const COLLECT_RECIPIENT2: Pubkey =
    solana_program::pubkey!("MmHsqX4LxTfifxoH8BVRLUKrwDn1LPCac6YcCZTHhwt");

/// The Bubblegum PDA that is the only permitted signer of
/// `UpdateCollectionInfoV1` (`src/processor/update_collection_info.rs`).
/// Mollusk does not verify signatures, so the test simply marks it as a
/// signer; no Bubblegum program is needed.
const BUBBLEGUM_SIGNER: Pubkey =
    solana_program::pubkey!("CbNY3JiXdXNE9tPNEk1aRZVEkWdj2v7kfJLNQwZZgpXk");

/// A fixture with both recipients present and a funded payer.
fn world() -> (Fixture, Pubkey) {
    let fixture = Fixture::new();
    fixture.store(COLLECT_RECIPIENT1, payer_account(0));
    fixture.store(COLLECT_RECIPIENT2, payer_account(0));
    let payer = fixture.fund(ACCOUNT_LAMPORTS);
    (fixture, payer)
}

fn put(fixture: &Fixture, account: Account) -> Pubkey {
    let key = Pubkey::new_unique();
    fixture.store(key, account);
    key
}

fn writable(key: Pubkey) -> AccountMeta {
    AccountMeta::new(key, false)
}

/// An asset account holding exactly `rent(len) + fee` lamports, which is what
/// a `CreateV2` leaves behind once the create fee has been paid into it.
fn asset_with_fee(fixture: &Fixture, owner: Pubkey, fee: u64) -> Pubkey {
    let account = AssetSpec::new(owner).build();
    let lamports = rent_exempt_balance(account.data.len()) + fee;
    put(fixture, AssetSpec::new(owner).lamports(lamports).build())
}

/// A burned account: one `Uninitialized` byte, still owned by mpl-core,
/// holding one byte of rent plus the fee that was never collected.
fn burned_account_with_fee(fee: u64) -> Account {
    Account {
        lamports: rent_exempt_balance(1) + fee,
        data: vec![Key::Uninitialized as u8],
        owner: MPL_CORE_ID,
        executable: false,
        rent_epoch: 0,
    }
}

/// The fee split the program applies: recipient 1 gets `fee / 2`, recipient 2
/// gets the remainder (so the odd lamport goes to recipient 2).
fn split(fee: u64) -> (u64, u64) {
    (fee / 2, fee - fee / 2)
}

// ===========================================================================
// Collect
// ===========================================================================

/// JS: collect.test.ts :: it can collect multiple assets at once
#[test]
fn collect_sweeps_assets_and_burned_accounts() {
    let (f, owner) = world();
    let asset = asset_with_fee(&f, owner, 1_000);
    // A hashed asset is collectible too. The placeholder is one byte, so its
    // rent minimum is rent(1).
    let hashed = put(
        &f,
        Account {
            lamports: rent_exempt_balance(1) + 500,
            ..hashed_asset_placeholder()
        },
    );
    let burned = put(&f, burned_account_with_fee(400));

    let asset_len = f.account(&asset).data.len();

    f.run_ok(&ix::collect(
        COLLECT_RECIPIENT1,
        COLLECT_RECIPIENT2,
        &[writable(asset), writable(hashed), writable(burned)],
    ));

    // Every swept account is left at exactly its rent minimum.
    assert_eq!(f.lamports(&asset), rent_exempt_balance(asset_len));
    assert_eq!(f.lamports(&hashed), rent_exempt_balance(1));
    assert_eq!(f.lamports(&burned), rent_exempt_balance(1));

    // A burned account is additionally handed to the system program, which is
    // a permissionless state change on a program-owned account (roadmap
    // section 8, note 7).
    assert_eq!(
        f.account(&burned).owner,
        system_program::ID,
        "Collect reassigns burned accounts to the system program"
    );
    assert_eq!(f.account(&asset).owner, MPL_CORE_ID);

    let total = 1_000 + 500 + 400;
    let (r1, r2) = split(1_000);
    let (h1, h2) = split(500);
    let (b1, b2) = split(400);
    assert_eq!(f.lamports(&COLLECT_RECIPIENT1), r1 + h1 + b1);
    assert_eq!(f.lamports(&COLLECT_RECIPIENT2), r2 + h2 + b2);
    assert_eq!(
        f.lamports(&COLLECT_RECIPIENT1) + f.lamports(&COLLECT_RECIPIENT2),
        total,
        "every collected lamport reaches a recipient"
    );
}

/// JS: collect.test.ts :: it can collect
#[test]
fn collect_is_idempotent_and_gives_the_odd_lamport_to_recipient_two() {
    let (f, owner) = world();
    // Three lamports above rent: recipient 1 gets one, recipient 2 gets two.
    let asset = asset_with_fee(&f, owner, 3);
    let asset_len = f.account(&asset).data.len();

    f.run_ok(&ix::collect(
        COLLECT_RECIPIENT1,
        COLLECT_RECIPIENT2,
        &[writable(asset)],
    ));
    assert_eq!(f.lamports(&COLLECT_RECIPIENT1), 1);
    assert_eq!(f.lamports(&COLLECT_RECIPIENT2), 2);
    assert_eq!(f.lamports(&asset), rent_exempt_balance(asset_len));

    // Collecting again finds nothing and moves nothing.
    f.run_ok(&ix::collect(
        COLLECT_RECIPIENT1,
        COLLECT_RECIPIENT2,
        &[writable(asset)],
    ));
    assert_eq!(f.lamports(&COLLECT_RECIPIENT1), 1);
    assert_eq!(f.lamports(&COLLECT_RECIPIENT2), 2);
    assert_eq!(f.lamports(&asset), rent_exempt_balance(asset_len));
}

#[test]
fn collect_with_no_remaining_accounts_is_a_no_op() {
    let (f, _owner) = world();

    f.run_ok(&ix::collect(COLLECT_RECIPIENT1, COLLECT_RECIPIENT2, &[]));

    assert_eq!(f.lamports(&COLLECT_RECIPIENT1), 0);
    assert_eq!(f.lamports(&COLLECT_RECIPIENT2), 0);
}

#[test]
fn collect_rejects_wrong_recipients() {
    let (f, owner) = world();
    let asset = asset_with_fee(&f, owner, 100);
    let impostor = put(&f, payer_account(0));

    assert_core_err(
        &f.run(&ix::collect(
            impostor,
            COLLECT_RECIPIENT2,
            &[writable(asset)],
        )),
        MplCoreError::IncorrectAccount,
    );
    assert_core_err(
        &f.run(&ix::collect(
            COLLECT_RECIPIENT1,
            impostor,
            &[writable(asset)],
        )),
        MplCoreError::IncorrectAccount,
    );
    assert_eq!(f.lamports(&COLLECT_RECIPIENT1), 0);
}

#[test]
fn collect_rejects_accounts_it_does_not_own() {
    let (f, _owner) = world();
    let foreign = put(&f, payer_account(ACCOUNT_LAMPORTS));

    assert_core_err(
        &f.run(&ix::collect(
            COLLECT_RECIPIENT1,
            COLLECT_RECIPIENT2,
            &[writable(foreign)],
        )),
        MplCoreError::IncorrectAccount,
    );
    assert_eq!(f.lamports(&foreign), ACCOUNT_LAMPORTS);
}

#[test]
fn collect_rejects_account_types_that_carry_no_fee() {
    let (f, owner) = world();
    // Collections are created without a create fee, so they are not a
    // collectible account type at all.
    let collection = put(&f, CollectionSpec::new(owner).build());

    assert_core_err(
        &f.run(&ix::collect(
            COLLECT_RECIPIENT1,
            COLLECT_RECIPIENT2,
            &[writable(collection)],
        )),
        MplCoreError::IncorrectAccount,
    );
}

/// An account holding less than its rent minimum makes the fee subtraction
/// underflow. The program checks it (`checked_sub`) and returns a clean error.
#[test]
fn collect_rejects_an_account_below_its_rent_minimum() {
    let (f, owner) = world();
    let underfunded = put(&f, AssetSpec::new(owner).lamports(1).build());

    assert_core_err(
        &f.run(&ix::collect(
            COLLECT_RECIPIENT1,
            COLLECT_RECIPIENT2,
            &[writable(underfunded)],
        )),
        MplCoreError::NumericalOverflowError,
    );

    // The same guard on the burned-account branch, which subtracts rent(1).
    let burned = put(
        &f,
        Account {
            lamports: 0,
            ..burned_account_with_fee(0)
        },
    );
    assert_core_err(
        &f.run(&ix::collect(
            COLLECT_RECIPIENT1,
            COLLECT_RECIPIENT2,
            &[writable(burned)],
        )),
        MplCoreError::NumericalOverflowError,
    );
}

// ===========================================================================
// UpdateCollectionInfoV1
// ===========================================================================

#[test]
fn update_collection_info_mint_add_and_remove() {
    let (f, authority) = world();
    let collection = put(&f, CollectionSpec::new(authority).build());

    // Mint bumps both counters.
    f.run_ok(&ix::update_collection_info_v1(
        collection,
        BUBBLEGUM_SIGNER,
        UpdateType::Mint,
        5,
    ));
    let stored = read_collection(&f.account(&collection));
    assert_eq!(stored.num_minted, 5);
    assert_eq!(stored.current_size, 5);

    // Add bumps only the current size.
    f.run_ok(&ix::update_collection_info_v1(
        collection,
        BUBBLEGUM_SIGNER,
        UpdateType::Add,
        2,
    ));
    let stored = read_collection(&f.account(&collection));
    assert_eq!(stored.num_minted, 5);
    assert_eq!(stored.current_size, 7);

    // Remove lowers it.
    f.run_ok(&ix::update_collection_info_v1(
        collection,
        BUBBLEGUM_SIGNER,
        UpdateType::Remove,
        3,
    ));
    assert_eq!(read_collection(&f.account(&collection)).current_size, 4);
}

/// The arithmetic saturates in both directions. Over-removing silently zeroes
/// `current_size`, which then unlocks `BurnCollectionV1` on a collection that
/// still has members (roadmap section 8, note 6); `Mint` with `u32::MAX`
/// clamps rather than wrapping.
#[test]
fn update_collection_info_saturates_instead_of_overflowing() {
    let (f, authority) = world();
    let collection = put(&f, CollectionSpec::new(authority).sizes(10, 10).build());

    f.run_ok(&ix::update_collection_info_v1(
        collection,
        BUBBLEGUM_SIGNER,
        UpdateType::Remove,
        u32::MAX,
    ));
    let stored = read_collection(&f.account(&collection));
    assert_eq!(
        stored.current_size, 0,
        "over-removal clamps to zero instead of underflowing"
    );
    assert_eq!(stored.num_minted, 10, "num_minted is untouched by Remove");

    f.run_ok(&ix::update_collection_info_v1(
        collection,
        BUBBLEGUM_SIGNER,
        UpdateType::Mint,
        u32::MAX,
    ));
    let stored = read_collection(&f.account(&collection));
    assert_eq!(stored.num_minted, u32::MAX);
    assert_eq!(stored.current_size, u32::MAX);

    f.run_ok(&ix::update_collection_info_v1(
        collection,
        BUBBLEGUM_SIGNER,
        UpdateType::Add,
        1,
    ));
    assert_eq!(
        read_collection(&f.account(&collection)).current_size,
        u32::MAX,
        "Add saturates at u32::MAX"
    );
}

/// The instruction does not require the collection to carry the `BubblegumV2`
/// plugin (roadmap section 8, note 6): the Bubblegum signer can adjust the
/// counters of any collection.
#[test]
fn update_collection_info_does_not_require_the_bubblegum_plugin() {
    let (f, authority) = world();
    let plain = put(&f, CollectionSpec::new(authority).build());

    f.run_ok(&ix::update_collection_info_v1(
        plain,
        BUBBLEGUM_SIGNER,
        UpdateType::Mint,
        1,
    ));

    assert_eq!(read_collection(&f.account(&plain)).num_minted, 1);
}

#[test]
fn update_collection_info_rejections() {
    let (f, authority) = world();
    let collection = put(&f, CollectionSpec::new(authority).build());

    // Any other signer.
    let stranger = f.fund(ACCOUNT_LAMPORTS);
    assert_core_err(
        &f.run(&ix::update_collection_info_v1(
            collection,
            stranger,
            UpdateType::Mint,
            1,
        )),
        MplCoreError::InvalidAuthority,
    );

    // The Bubblegum PDA itself, but not signing.
    let mut ix = ix::update_collection_info_v1(collection, BUBBLEGUM_SIGNER, UpdateType::Mint, 1);
    ix::unsign(&mut ix, &BUBBLEGUM_SIGNER);
    assert_program_err(&f.run(&ix), ProgramError::MissingRequiredSignature);

    // An asset in the collection slot.
    let asset = put(&f, AssetSpec::new(authority).build());
    assert_core_err(
        &f.run(&ix::update_collection_info_v1(
            asset,
            BUBBLEGUM_SIGNER,
            UpdateType::Mint,
            1,
        )),
        MplCoreError::DeserializationError,
    );

    assert_eq!(read_collection(&f.account(&collection)).num_minted, 0);
}
