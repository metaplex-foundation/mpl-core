//! Account ownership: mpl-core instructions must reject accounts that carry
//! valid-looking mpl-core data but are owned by another program, have a wrong
//! discriminator, or are otherwise malformed, and must keep enforcing
//! authority and freeze checks on genuine accounts.
//!
//! Mirrors `clients/js/test/accountOwnership.test.ts`. Unlike the JS suite,
//! which can only create zero-filled accounts, these tests fabricate accounts
//! with fully serialized `AssetV1` / `CollectionV1` data (and plugins) under
//! a foreign owner, so the owner check in `SolanaAccount::load` is what
//! rejects them (`ProgramError::InvalidAccountOwner`) rather than the
//! discriminator check.
//!
//! Every instruction here runs against a real executable program account
//! (`with_program_accounts`), and every failure is asserted exactly.

use {
    crate::common::{accounts::core_bytes_account, *},
    mollusk_svm::result::InstructionResult,
    mpl_core_program::{
        error::MplCoreError,
        plugins::{
            FreezeDelegate, PermanentBurnDelegate, PermanentFreezeDelegate,
            PermanentTransferDelegate, Plugin, PluginType,
        },
        state::{Authority, Key, UpdateAuthority},
    },
    solana_account::Account,
    solana_program::{
        instruction::{Instruction, InstructionError},
        program_error::ProgramError,
        pubkey::Pubkey,
    },
    solana_system_interface::program as system_program,
};

/// A fake program ID representing an attacker's program.
const FAKE_PROGRAM_ID: Pubkey = Pubkey::new_from_array([0xAA; 32]);

/// Runs one instruction with the given accounts plus every program account
/// the harness knows about (mpl-core, system, noop, recorder).
fn run(ix: &Instruction, accounts: Vec<(Pubkey, Account)>) -> InstructionResult {
    core_mollusk().process_instruction(ix, &with_program_accounts(accounts))
}

/// `TransferV1` of `asset` to `new_owner`, signed by `payer` as both payer
/// and authority, with no collection.
fn transfer_ix(asset: Pubkey, payer: Pubkey, new_owner: Pubkey) -> Instruction {
    ix::transfer_v1(asset, None, payer, None, new_owner, None, None)
}

/// `BurnV1` of `asset`, signed by `payer` as both payer and authority.
fn burn_ix(asset: Pubkey, payer: Pubkey) -> Instruction {
    ix::burn_v1(asset, None, payer, None, None, None)
}

/// `UpdateV1` renaming `asset` to "X", signed by `payer`.
fn update_ix(asset: Pubkey, payer: Pubkey) -> Instruction {
    ix::update_v1(
        asset,
        None,
        payer,
        None,
        None,
        Some("X".to_string()),
        None,
        None,
    )
}

/// Asserts the resulting account for `key` is byte-for-byte `original`
/// (lamports, data and owner): a rejected instruction must not touch it.
fn assert_untouched(result: &InstructionResult, key: &Pubkey, original: &Account) {
    let account = account_of(result, key);
    assert_eq!(
        account.lamports, original.lamports,
        "account {key} lost or gained lamports"
    );
    assert_eq!(account.owner, original.owner, "account {key} changed owner");
    assert_eq!(account.data, original.data, "account {key} data changed");
}

// ===========================================================================
// Section 1: Fake assets owned by a different program
//
// Valid mpl-core AssetV1 data (correct discriminator byte, valid Borsh
// fields) but owned by a different program.
// ===========================================================================

/// JS: accountOwnership.test.ts :: combined defense: an account owned by a random program as asset is rejected
#[test]
fn transfer_rejects_fake_asset_owned_by_different_program() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    let fake_asset = AssetSpec::new(payer).program_owner(FAKE_PROGRAM_ID).build();

    let result = run(
        &transfer_ix(asset, payer, new_owner),
        vec![
            (asset, fake_asset.clone()),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
            (new_owner, empty_account()),
        ],
    );

    assert_program_err(&result, ProgramError::InvalidAccountOwner);
    assert_untouched(&result, &asset, &fake_asset);
}

/// JS: accountOwnership.test.ts :: combined defense: burn with wrong-program asset does not drain lamports
#[test]
fn burn_rejects_fake_asset_owned_by_different_program() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();

    let fake_asset = AssetSpec::new(payer).program_owner(FAKE_PROGRAM_ID).build();

    let result = run(
        &burn_ix(asset, payer),
        vec![
            (asset, fake_asset.clone()),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
        ],
    );

    assert_program_err(&result, ProgramError::InvalidAccountOwner);
    // The fake asset keeps its lamports; nothing was drained to the payer.
    assert_untouched(&result, &asset, &fake_asset);
    assert_eq!(lamports_of(&result, &payer), ACCOUNT_LAMPORTS);
}

/// JS: accountOwnership.test.ts :: combined defense: update with wrong-program asset cannot modify data
#[test]
fn update_rejects_fake_asset_owned_by_different_program() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();

    let fake_asset = AssetSpec::new(payer).program_owner(FAKE_PROGRAM_ID).build();

    let result = run(
        &update_ix(asset, payer),
        vec![
            (asset, fake_asset.clone()),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
        ],
    );

    assert_program_err(&result, ProgramError::InvalidAccountOwner);
    assert_untouched(&result, &asset, &fake_asset);
}

// ===========================================================================
// Section 2: Fake collections owned by a different program
// ===========================================================================

/// JS: accountOwnership.test.ts :: wrong owner: update collection rejects non-program-owned collection
#[test]
fn update_collection_rejects_fake_collection_owned_by_different_program() {
    let payer = Pubkey::new_unique();
    let collection = Pubkey::new_unique();

    let fake_collection = CollectionSpec::new(payer)
        .program_owner(FAKE_PROGRAM_ID)
        .build();

    let ix = ix::update_collection_v1(
        collection,
        payer,
        None,
        None,
        None,
        Some("X".to_string()),
        None,
    );
    let result = run(
        &ix,
        vec![
            (collection, fake_collection.clone()),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
        ],
    );

    assert_program_err(&result, ProgramError::InvalidAccountOwner);
    assert_untouched(&result, &collection, &fake_collection);
}

// ===========================================================================
// Section 3: Accounts with wrong discriminator
//
// Even with valid Borsh data and the correct owner, an account with the
// wrong discriminator byte is rejected by the explicit `load_key` match.
// ===========================================================================

/// JS: accountOwnership.test.ts :: discriminator: transfer rejects account with uninitialized key owned by mpl-core
#[test]
fn transfer_rejects_account_with_wrong_discriminator() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    // A genuine mpl-core-owned CollectionV1 (key 5) in the asset slot.
    let wrong_disc_account = CollectionSpec::new(payer).build();

    let result = run(
        &transfer_ix(asset, payer, new_owner),
        vec![
            (asset, wrong_disc_account.clone()),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
            (new_owner, empty_account()),
        ],
    );

    assert_core_err(&result, MplCoreError::IncorrectAccount);
    assert_untouched(&result, &asset, &wrong_disc_account);
}

/// JS: accountOwnership.test.ts :: discriminator: burn rejects account with uninitialized key owned by mpl-core
#[test]
fn burn_rejects_account_with_wrong_discriminator() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();

    // Key::Uninitialized (0) followed by zeros: an allocated, never
    // initialized mpl-core account.
    let wrong_disc_account = core_bytes_account(vec![Key::Uninitialized as u8; 100]);

    let result = run(
        &burn_ix(asset, payer),
        vec![
            (asset, wrong_disc_account.clone()),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
        ],
    );

    assert_core_err(&result, MplCoreError::IncorrectAccount);
    assert_untouched(&result, &asset, &wrong_disc_account);
}

// ===========================================================================
// Section 4: Random / garbage data accounts
//
// A first byte that is not a `Key` at all fails in `load_key`.
// ===========================================================================

#[test]
fn transfer_rejects_random_data_account() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    let garbage_account = core_bytes_account(vec![0xFF; 200]);

    let result = run(
        &transfer_ix(asset, payer, new_owner),
        vec![
            (asset, garbage_account.clone()),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
            (new_owner, empty_account()),
        ],
    );

    assert_core_err(&result, MplCoreError::DeserializationError);
    assert_untouched(&result, &asset, &garbage_account);
}

// ===========================================================================
// Section 5: Empty accounts
//
// A zero-length account has no discriminator byte to read.
// ===========================================================================

/// JS: accountOwnership.test.ts :: combined defense: a completely random pubkey as asset fails immediately
#[test]
fn transfer_rejects_empty_account() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    let empty = core_bytes_account(vec![]);

    let result = run(
        &transfer_ix(asset, payer, new_owner),
        vec![
            (asset, empty.clone()),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
            (new_owner, empty_account()),
        ],
    );

    // `load_key` indexes `data[0]` unguarded, so an empty account panics
    // instead of returning a clean error (docs/coverage-roadmap.md, section 6).
    assert_instruction_err(&result, InstructionError::ProgramFailedToComplete);
    assert_untouched(&result, &asset, &empty);
}

// ===========================================================================
// Section 6: Valid accounts work correctly (sanity checks)
// ===========================================================================

/// JS: accountOwnership.test.ts :: baseline: transfer succeeds with correctly program-owned asset
#[test]
fn transfer_succeeds_with_valid_asset() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    let valid_asset = AssetSpec::new(payer).build();

    let result = run(
        &transfer_ix(asset, payer, new_owner),
        vec![
            (asset, valid_asset),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
            (new_owner, empty_account()),
        ],
    );

    assert_ok(&result);
    let transferred = account_of(&result, &asset);
    let core = read_asset(transferred);
    assert_eq!(core.owner, new_owner, "the asset owner must change");
    assert_eq!(
        core.update_authority,
        UpdateAuthority::Address(payer),
        "the update authority must not change"
    );
    assert_registry_consistent(transferred);
}

// ===========================================================================
// Section 7: System-program-owned accounts with an AssetV1 discriminator
// ===========================================================================

/// JS: accountOwnership.test.ts :: wrong owner: transfer rejects asset account owned by system program
#[test]
fn transfer_rejects_system_owned_account_with_asset_discriminator() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    let fake_asset = AssetSpec::new(payer)
        .program_owner(system_program::ID)
        .build();

    let result = run(
        &transfer_ix(asset, payer, new_owner),
        vec![
            (asset, fake_asset.clone()),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
            (new_owner, empty_account()),
        ],
    );

    assert_program_err(&result, ProgramError::InvalidAccountOwner);
    assert_untouched(&result, &asset, &fake_asset);
}

/// JS: accountOwnership.test.ts :: wrong owner: update rejects asset account owned by system program
#[test]
fn update_rejects_system_owned_account_with_asset_discriminator() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();

    let fake_asset = AssetSpec::new(payer)
        .program_owner(system_program::ID)
        .build();

    let result = run(
        &update_ix(asset, payer),
        vec![
            (asset, fake_asset.clone()),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
        ],
    );

    assert_program_err(&result, ProgramError::InvalidAccountOwner);
    assert_untouched(&result, &asset, &fake_asset);
}

// ===========================================================================
// Section 8: Valid asset with a mismatched authority
//
// A genuine asset, but the signer is neither the owner nor a delegate: no
// validator approves, so the program returns `NoApprovals`.
// ===========================================================================

/// JS: accountOwnership.test.ts :: authority: transfer rejects non-owner even with correct program-owned accounts
#[test]
fn transfer_rejects_unauthorized_caller_on_valid_asset() {
    let actual_owner = Pubkey::new_unique();
    let attacker = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    let valid_asset = AssetSpec::new(actual_owner).build();

    let result = run(
        &transfer_ix(asset, attacker, new_owner),
        vec![
            (asset, valid_asset.clone()),
            (attacker, payer_account(ACCOUNT_LAMPORTS)),
            (new_owner, empty_account()),
        ],
    );

    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_untouched(&result, &asset, &valid_asset);
    assert_eq!(read_asset(account_of(&result, &asset)).owner, actual_owner);
}

/// JS: accountOwnership.test.ts :: authority: burn rejects non-owner even with correct program-owned accounts
#[test]
fn burn_rejects_unauthorized_caller_on_valid_asset() {
    let actual_owner = Pubkey::new_unique();
    let attacker = Pubkey::new_unique();
    let asset = Pubkey::new_unique();

    let valid_asset = AssetSpec::new(actual_owner).build();

    let result = run(
        &burn_ix(asset, attacker),
        vec![
            (asset, valid_asset.clone()),
            (attacker, payer_account(ACCOUNT_LAMPORTS)),
        ],
    );

    assert_core_err(&result, MplCoreError::NoApprovals);
    assert_untouched(&result, &asset, &valid_asset);
}

// ===========================================================================
// Section 9: Fake assets with freeze plugins (owned by a different program)
//
// Valid AssetV1 data plus an embedded FreezeDelegate, owned by
// FAKE_PROGRAM_ID. The owner check fires before any plugin is evaluated,
// whatever the freeze state.
// ===========================================================================

#[test]
fn transfer_rejects_fake_frozen_asset_owned_by_different_program() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    let fake_asset = AssetSpec::new(payer)
        .program_owner(FAKE_PROGRAM_ID)
        .plugin(
            Plugin::FreezeDelegate(FreezeDelegate { frozen: true }),
            Authority::Owner,
        )
        .build();

    let result = run(
        &transfer_ix(asset, payer, new_owner),
        vec![
            (asset, fake_asset.clone()),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
            (new_owner, empty_account()),
        ],
    );

    assert_program_err(&result, ProgramError::InvalidAccountOwner);
    assert_untouched(&result, &asset, &fake_asset);
}

#[test]
fn transfer_rejects_fake_unfrozen_asset_owned_by_different_program() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    // Even an "unfrozen" fake must fail.
    let fake_asset = AssetSpec::new(payer)
        .program_owner(FAKE_PROGRAM_ID)
        .plugin(
            Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
            Authority::Owner,
        )
        .build();

    let result = run(
        &transfer_ix(asset, payer, new_owner),
        vec![
            (asset, fake_asset.clone()),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
            (new_owner, empty_account()),
        ],
    );

    assert_program_err(&result, ProgramError::InvalidAccountOwner);
    assert_untouched(&result, &asset, &fake_asset);
}

#[test]
fn burn_rejects_fake_frozen_asset_owned_by_different_program() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();

    let fake_asset = AssetSpec::new(payer)
        .program_owner(FAKE_PROGRAM_ID)
        .plugin(
            Plugin::FreezeDelegate(FreezeDelegate { frozen: true }),
            Authority::Owner,
        )
        .build();

    let result = run(
        &burn_ix(asset, payer),
        vec![
            (asset, fake_asset.clone()),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
        ],
    );

    assert_program_err(&result, ProgramError::InvalidAccountOwner);
    assert_untouched(&result, &asset, &fake_asset);
}

// ===========================================================================
// Section 10: Fake assets with permanent delegate plugins
//
// Permanent delegates can force-approve transfers and burns, but only on a
// genuine account; the owner check fires first.
// ===========================================================================

#[test]
fn transfer_rejects_fake_asset_with_permanent_transfer_delegate() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    let fake_asset = AssetSpec::new(payer)
        .program_owner(FAKE_PROGRAM_ID)
        .plugin(
            Plugin::PermanentTransferDelegate(PermanentTransferDelegate {}),
            Authority::Owner,
        )
        .build();

    let result = run(
        &transfer_ix(asset, payer, new_owner),
        vec![
            (asset, fake_asset.clone()),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
            (new_owner, empty_account()),
        ],
    );

    assert_program_err(&result, ProgramError::InvalidAccountOwner);
    assert_untouched(&result, &asset, &fake_asset);
}

#[test]
fn burn_rejects_fake_asset_with_permanent_burn_delegate() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();

    let fake_asset = AssetSpec::new(payer)
        .program_owner(FAKE_PROGRAM_ID)
        .plugin(
            Plugin::PermanentBurnDelegate(PermanentBurnDelegate {}),
            Authority::Owner,
        )
        .build();

    let result = run(
        &burn_ix(asset, payer),
        vec![
            (asset, fake_asset.clone()),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
        ],
    );

    assert_program_err(&result, ProgramError::InvalidAccountOwner);
    assert_untouched(&result, &asset, &fake_asset);
}

#[test]
fn transfer_rejects_fake_asset_with_permanent_freeze_frozen() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    let fake_asset = AssetSpec::new(payer)
        .program_owner(FAKE_PROGRAM_ID)
        .plugin(
            Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen: true }),
            Authority::Owner,
        )
        .build();

    let result = run(
        &transfer_ix(asset, payer, new_owner),
        vec![
            (asset, fake_asset.clone()),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
            (new_owner, empty_account()),
        ],
    );

    assert_program_err(&result, ProgramError::InvalidAccountOwner);
    assert_untouched(&result, &asset, &fake_asset);
}

// ===========================================================================
// Section 11: Fake assets with multiple conflicting plugins
//
// Even "favorable" plugin combinations (frozen but with a permanent transfer
// delegate) do not matter: the fake account fails first.
// ===========================================================================

#[test]
fn transfer_rejects_fake_asset_frozen_but_with_permanent_transfer() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    let fake_asset = AssetSpec::new(payer)
        .program_owner(FAKE_PROGRAM_ID)
        .plugin(
            Plugin::FreezeDelegate(FreezeDelegate { frozen: true }),
            Authority::Owner,
        )
        .plugin(
            Plugin::PermanentTransferDelegate(PermanentTransferDelegate {}),
            Authority::Owner,
        )
        .build();

    let result = run(
        &transfer_ix(asset, payer, new_owner),
        vec![
            (asset, fake_asset.clone()),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
            (new_owner, empty_account()),
        ],
    );

    assert_program_err(&result, ProgramError::InvalidAccountOwner);
    assert_untouched(&result, &asset, &fake_asset);
}

#[test]
fn burn_rejects_fake_asset_unfrozen_with_permanent_burn() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();

    let fake_asset = AssetSpec::new(payer)
        .program_owner(FAKE_PROGRAM_ID)
        .plugin(
            Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
            Authority::Owner,
        )
        .plugin(
            Plugin::PermanentBurnDelegate(PermanentBurnDelegate {}),
            Authority::Owner,
        )
        .build();

    let result = run(
        &burn_ix(asset, payer),
        vec![
            (asset, fake_asset.clone()),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
        ],
    );

    assert_program_err(&result, ProgramError::InvalidAccountOwner);
    assert_untouched(&result, &asset, &fake_asset);
}

// ===========================================================================
// Section 12: Fake collections with plugins
//
// A genuine asset whose update authority is a collection, with the collection
// account fabricated under FAKE_PROGRAM_ID and carrying favorable plugins.
// The collection address matches, so it is the owner check in
// `CollectionV1::load` (reached from `resolve_pubkey_to_authorities`) that
// rejects it, before any plugin is evaluated.
// ===========================================================================

#[test]
fn transfer_rejects_when_fake_collection_has_permanent_freeze() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    let valid_asset = AssetSpec::new(payer)
        .update_authority(UpdateAuthority::Collection(collection))
        .build();
    let fake_collection = CollectionSpec::new(payer)
        .program_owner(FAKE_PROGRAM_ID)
        .plugin(
            Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen: false }),
            Authority::UpdateAuthority,
        )
        .build();

    let ix = ix::transfer_v1(asset, Some(collection), payer, None, new_owner, None, None);
    let result = run(
        &ix,
        vec![
            (asset, valid_asset.clone()),
            (collection, fake_collection),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
            (new_owner, empty_account()),
        ],
    );

    assert_program_err(&result, ProgramError::InvalidAccountOwner);
    assert_untouched(&result, &asset, &valid_asset);
}

#[test]
fn transfer_rejects_when_fake_collection_has_permanent_transfer_delegate() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    let valid_asset = AssetSpec::new(payer)
        .update_authority(UpdateAuthority::Collection(collection))
        .build();
    let fake_collection = CollectionSpec::new(payer)
        .program_owner(FAKE_PROGRAM_ID)
        .plugin(
            Plugin::PermanentTransferDelegate(PermanentTransferDelegate {}),
            Authority::UpdateAuthority,
        )
        .build();

    let ix = ix::transfer_v1(asset, Some(collection), payer, None, new_owner, None, None);
    let result = run(
        &ix,
        vec![
            (asset, valid_asset.clone()),
            (collection, fake_collection),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
            (new_owner, empty_account()),
        ],
    );

    assert_program_err(&result, ProgramError::InvalidAccountOwner);
    assert_untouched(&result, &asset, &valid_asset);
}

// ===========================================================================
// Section 13: Valid frozen asset sanity checks
//
// Plugin enforcement on genuine accounts: a frozen asset rejects the
// transfer (the FreezeDelegate rejection surfaces as `InvalidAuthority`),
// an unfrozen one with the same plugin transfers.
// ===========================================================================

/// JS: accountOwnership.test.ts :: frozen: transfer rejects even with correct ownership when asset is frozen
#[test]
fn transfer_rejects_valid_frozen_asset() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    let frozen_asset = AssetSpec::new(payer)
        .plugin(
            Plugin::FreezeDelegate(FreezeDelegate { frozen: true }),
            Authority::Owner,
        )
        .build();

    let result = run(
        &transfer_ix(asset, payer, new_owner),
        vec![
            (asset, frozen_asset.clone()),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
            (new_owner, empty_account()),
        ],
    );

    assert_core_err(&result, MplCoreError::InvalidAuthority);
    assert_untouched(&result, &asset, &frozen_asset);
    let (authority, plugin) =
        read_plugin(account_of(&result, &asset), PluginType::FreezeDelegate).unwrap();
    assert_eq!(authority, Authority::Owner);
    assert_eq!(
        plugin,
        Plugin::FreezeDelegate(FreezeDelegate { frozen: true })
    );
}

#[test]
fn transfer_succeeds_valid_unfrozen_asset_with_freeze_plugin() {
    let payer = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    let unfrozen_asset = AssetSpec::new(payer)
        .plugin(
            Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
            Authority::Owner,
        )
        .build();

    let result = run(
        &transfer_ix(asset, payer, new_owner),
        vec![
            (asset, unfrozen_asset),
            (payer, payer_account(ACCOUNT_LAMPORTS)),
            (new_owner, empty_account()),
        ],
    );

    assert_ok(&result);
    let transferred = account_of(&result, &asset);
    assert_eq!(read_asset(transferred).owner, new_owner);
    // The owner-managed plugin survives the transfer and stays owner-managed.
    let (authority, plugin) = read_plugin(transferred, PluginType::FreezeDelegate).unwrap();
    assert_eq!(authority, Authority::Owner);
    assert_eq!(
        plugin,
        Plugin::FreezeDelegate(FreezeDelegate { frozen: false })
    );
    assert_registry_consistent(transferred);
}
