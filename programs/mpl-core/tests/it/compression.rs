//! Compression: `CompressV1`, `DecompressV1`, and the `HashedAssetV1` branches
//! of `BurnV1`, `TransferV1`, `UpdateV1` and `ExecuteV1`.
//!
//! All of these end in `MplCoreError::NotAvailable` — compression is switched
//! off — but they run the real state transitions first: permission validation,
//! `compress_into_account_space` / `rebuild_account_state_from_proof_data`
//! (payer-funded reallocs and a `sol_memcpy` over the account), `verify_proof`,
//! and the `Wrappable::wrap` CPI into SPL Noop. That is the whole of
//! `utils/compression.rs`, `state/compression_proof.rs`,
//! `state/hashable_plugin_schema.rs`, `state/hashed_asset.rs` and the `hash` /
//! `wrap` trait methods, none of which any other instruction reaches.
//!
//! The hashed-asset fixture is computed in-test from the crate's own public
//! `state` types, exactly as `verify_proof` recomputes it, so a change on
//! either side alone breaks these tests. Because both sides walk the same
//! path, a change made to both at once would not; the golden vectors in
//! `hashed_asset_schema_matches_its_golden_vectors` pin the bytes themselves
//! to close that gap.

use {
    crate::common::*,
    mpl_core_program::{
        error::MplCoreError,
        plugins::{Attributes, FreezeDelegate, Plugin},
        state::{AssetV1, Authority, CompressionProof, HashablePluginSchema, Key, UpdateAuthority},
        ID as MPL_CORE_ID,
    },
    solana_account::Account,
    solana_program::{instruction::AccountMeta, pubkey::Pubkey},
};

/// A fixture with a funded payer that also owns the assets.
fn world() -> (Fixture, Pubkey) {
    let fixture = Fixture::new();
    let owner = fixture.fund(ACCOUNT_LAMPORTS);
    (fixture, owner)
}

fn put(fixture: &Fixture, account: Account) -> Pubkey {
    let key = Pubkey::new_unique();
    fixture.store(key, account);
    key
}

/// The `AssetV1` a compression proof describes.
fn proof_asset(owner: Pubkey) -> AssetV1 {
    AssetSpec::new(owner).name("Compressed").uri("uri").core()
}

/// A `CompressionProof` for `owner`'s asset at sequence `seq`.
fn proof_for(owner: Pubkey, seq: u64, plugins: Vec<HashablePluginSchema>) -> CompressionProof {
    CompressionProof::new(proof_asset(owner), seq, plugins)
}

/// The proof in the form the generated client's instruction args take.
fn client_proof(proof: &CompressionProof) -> mpl_core::types::CompressionProof {
    ix::convert(proof)
}

/// Stores the `HashedAssetV1` account that `proof` hashes to and returns its
/// key.
fn put_hashed(fixture: &Fixture, proof: &CompressionProof) -> Pubkey {
    put(fixture, hashed_asset_for_proof(proof))
}

/// Two plugins in a proof, deliberately listed with descending `index` so that
/// the `sort_by(compare_indeces)` in `verify_proof` has something to do.
fn unsorted_plugins() -> Vec<HashablePluginSchema> {
    vec![
        HashablePluginSchema {
            index: 1,
            authority: Authority::UpdateAuthority,
            plugin: Plugin::Attributes(Attributes {
                attribute_list: vec![],
            }),
        },
        HashablePluginSchema {
            index: 0,
            authority: Authority::Owner,
            plugin: Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
        },
    ]
}

// ===========================================================================
// CompressV1
// ===========================================================================

/// `CompressV1` validates, hashes the asset and every registry record into a
/// `CompressionProof`, shrinks the account to the 33-byte `HashedAssetV1`
/// layout, CPIs the proof into SPL Noop and only then reports that compression
/// is unavailable. The transaction reverts, so nothing of that is persisted;
/// what the test pins is that the whole path runs and where it stops.
///
/// JS: compress.test.ts :: it cannot compress an asset because it is not available
#[test]
fn compress_runs_the_full_state_transition_then_returns_not_available() {
    let (f, owner) = world();

    // Without plugins: `compress_into_account_space` skips the registry loop.
    let bare = put(&f, AssetSpec::new(owner).build());
    assert_core_err(
        &f.run(&ix::compress_v1(bare, None, owner, None, Some(SPL_NOOP_ID))),
        MplCoreError::NotAvailable,
    );

    // With plugins: one `HashablePluginSchema` per registry record, sorted by
    // offset before hashing.
    let with_plugins = put(
        &f,
        AssetSpec::new(owner)
            .plugin(
                Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
                Authority::Owner,
            )
            .plugin(
                Plugin::Attributes(Attributes {
                    attribute_list: vec![],
                }),
                Authority::UpdateAuthority,
            )
            .build(),
    );
    assert_core_err(
        &f.run(&ix::compress_v1(
            with_plugins,
            None,
            owner,
            None,
            Some(SPL_NOOP_ID),
        )),
        MplCoreError::NotAvailable,
    );
}

/// Proof that the noop CPI really is executed rather than skipped: the noop
/// program is optional as far as the processor's guards go, but `wrap()`
/// invokes it unconditionally, so leaving it out of the account list fails in
/// the runtime before the `NotAvailable` return is reached. The exact runtime
/// error differs between the native and SBF harnesses, so the assertion is on
/// where the instruction stopped, not on the error code.
#[test]
fn compress_without_the_log_wrapper_account_fails_in_the_noop_cpi() {
    let (f, owner) = world();
    let asset = put(&f, AssetSpec::new(owner).build());

    let result = f.run(&ix::compress_v1(asset, None, owner, None, None));

    // Every error the program itself returns is a `Custom` code, so rejecting
    // the whole `Custom` family is what makes this assertion mean something:
    // it admits only a failure raised by the runtime, which is where a CPI
    // with a missing account dies. `NotAvailable` would say the guard was
    // reached instead of the CPI; any other `MplCoreError` would say the
    // instruction never got as far as `wrap()` at all. Both are failures of
    // this test's premise, so both are named rather than tolerated.
    match &result.program_result {
        mollusk_svm::result::ProgramResult::Success => {
            panic!("CompressV1 must not succeed while compression is unavailable")
        }
        mollusk_svm::result::ProgramResult::Failure(
            solana_program::program_error::ProgramError::Custom(code),
        ) if *code == MplCoreError::NotAvailable as u32 => panic!(
            "the instruction reached its NotAvailable return, so the noop CPI \
             was not attempted"
        ),
        mollusk_svm::result::ProgramResult::Failure(
            solana_program::program_error::ProgramError::Custom(code),
        ) => panic!(
            "expected the noop CPI to fail in the runtime, but the program \
             returned its own error (custom {code}); the instruction stopped \
             before reaching wrap()"
        ),
        // A runtime-level failure: natively `ProgramFailedToComplete`, and an
        // account-resolution error under SBF. The code differs by harness, so
        // what is pinned is that the program did not return it.
        _ => {}
    }
}

#[test]
fn compress_rejections() {
    let (f, owner) = world();
    let asset = put(&f, AssetSpec::new(owner).build());

    // JS: compress.test.ts :: it cannot use an invalid system program
    let mut ix = ix::compress_v1(asset, None, owner, None, Some(SPL_NOOP_ID));
    let impostor = put(&f, payer_account(ACCOUNT_LAMPORTS));
    ix::set_account(&mut ix, 4, AccountMeta::new_readonly(impostor, false));
    assert_core_err(&f.run(&ix), MplCoreError::InvalidSystemProgram);

    // JS: compress.test.ts :: it cannot use an invalid log wrapper program
    let wrong_noop = put(&f, payer_account(ACCOUNT_LAMPORTS));
    assert_core_err(
        &f.run(&ix::compress_v1(asset, None, owner, None, Some(wrong_noop))),
        MplCoreError::InvalidLogWrapperProgram,
    );

    // An account that is already a hashed asset.
    let hashed = put(&f, hashed_asset_placeholder());
    assert_core_err(
        &f.run(&ix::compress_v1(
            hashed,
            None,
            owner,
            None,
            Some(SPL_NOOP_ID),
        )),
        MplCoreError::AlreadyCompressed,
    );

    // A collection in the asset slot.
    let collection = put(&f, CollectionSpec::new(owner).build());
    assert_core_err(
        &f.run(&ix::compress_v1(
            collection,
            None,
            owner,
            None,
            Some(SPL_NOOP_ID),
        )),
        MplCoreError::IncorrectAccount,
    );

    // Only the owner may compress.
    let stranger = f.fund(ACCOUNT_LAMPORTS);
    assert_core_err(
        &f.run(&ix::compress_v1(
            asset,
            None,
            stranger,
            None,
            Some(SPL_NOOP_ID),
        )),
        MplCoreError::NoApprovals,
    );
}

// ===========================================================================
// DecompressV1
// ===========================================================================

/// `DecompressV1` verifies the proof, rebuilds the whole `AssetV1` (and every
/// plugin in the proof) in account space — growing the account from 33 bytes
/// and charging the payer for the rent — and then reports that decompression
/// is unavailable.
#[test]
fn decompress_rebuilds_the_account_then_returns_not_available() {
    let (f, owner) = world();

    // No plugins: `rebuild_account_state_from_proof_data` skips the plugin
    // loop entirely.
    let proof = proof_for(owner, 7, vec![]);
    let asset = put_hashed(&f, &proof);
    assert_core_err(
        &f.run(&ix::decompress_v1(
            asset,
            None,
            owner,
            None,
            Some(SPL_NOOP_ID),
            client_proof(&proof),
        )),
        MplCoreError::NotAvailable,
    );

    // Two plugins, handed over unsorted: `verify_proof` sorts them by index
    // before hashing, and the rebuild re-initializes each one.
    let proof = proof_for(owner, 3, unsorted_plugins());
    let asset = put_hashed(&f, &proof);
    assert_core_err(
        &f.run(&ix::decompress_v1(
            asset,
            None,
            owner,
            None,
            Some(SPL_NOOP_ID),
            client_proof(&proof),
        )),
        MplCoreError::NotAvailable,
    );
}

#[test]
fn decompress_rejects_a_proof_that_does_not_hash_to_the_account() {
    let (f, owner) = world();
    let proof = proof_for(owner, 1, vec![]);
    let asset = put_hashed(&f, &proof);

    // Same asset, different name: a different hash.
    let mut tampered = proof.clone();
    tampered.name = "Renamed".to_string();
    assert_core_err(
        &f.run(&ix::decompress_v1(
            asset,
            None,
            owner,
            None,
            Some(SPL_NOOP_ID),
            client_proof(&tampered),
        )),
        MplCoreError::IncorrectAssetHash,
    );

    // The plugin hashes are part of the schema too: dropping one changes it.
    let proof = proof_for(owner, 1, unsorted_plugins());
    let asset = put_hashed(&f, &proof);
    let mut without_plugins = proof.clone();
    without_plugins.plugins.clear();
    assert_core_err(
        &f.run(&ix::decompress_v1(
            asset,
            None,
            owner,
            None,
            Some(SPL_NOOP_ID),
            client_proof(&without_plugins),
        )),
        MplCoreError::IncorrectAssetHash,
    );
}

#[test]
fn decompress_rejections() {
    let (f, owner) = world();
    let proof = proof_for(owner, 1, vec![]);
    let asset = put_hashed(&f, &proof);

    // JS: decompress.test.ts :: it cannot use an invalid system program
    let mut ix = ix::decompress_v1(
        asset,
        None,
        owner,
        None,
        Some(SPL_NOOP_ID),
        client_proof(&proof),
    );
    let impostor = put(&f, payer_account(ACCOUNT_LAMPORTS));
    ix::set_account(&mut ix, 4, AccountMeta::new_readonly(impostor, false));
    assert_core_err(&f.run(&ix), MplCoreError::InvalidSystemProgram);

    // JS: decompress.test.ts :: it cannot use an invalid noop program
    let wrong_noop = put(&f, payer_account(ACCOUNT_LAMPORTS));
    assert_core_err(
        &f.run(&ix::decompress_v1(
            asset,
            None,
            owner,
            None,
            Some(wrong_noop),
            client_proof(&proof),
        )),
        MplCoreError::InvalidLogWrapperProgram,
    );

    // A plain asset account.
    let plain = put(&f, AssetSpec::new(owner).build());
    assert_core_err(
        &f.run(&ix::decompress_v1(
            plain,
            None,
            owner,
            None,
            Some(SPL_NOOP_ID),
            client_proof(&proof),
        )),
        MplCoreError::AlreadyDecompressed,
    );

    // A collection account.
    let collection = put(&f, CollectionSpec::new(owner).build());
    assert_core_err(
        &f.run(&ix::decompress_v1(
            collection,
            None,
            owner,
            None,
            Some(SPL_NOOP_ID),
            client_proof(&proof),
        )),
        MplCoreError::IncorrectAccount,
    );

    // The account is rebuilt *before* permissions are checked, so a stranger
    // gets all the way to `validate_asset_permissions` (roadmap section 13,
    // note 10) and is rejected there.
    let stranger = f.fund(ACCOUNT_LAMPORTS);
    assert_core_err(
        &f.run(&ix::decompress_v1(
            asset,
            None,
            stranger,
            None,
            Some(SPL_NOOP_ID),
            client_proof(&proof),
        )),
        MplCoreError::NoApprovals,
    );
}

// ===========================================================================
// The hashed-asset branches of burn, transfer, update and execute
// ===========================================================================

/// Sets the optional `system_program` slot at `index` back to the
/// optional-account sentinel, which the program reads as `None`.
/// `BurnV1` has it at 4, `TransferV1` (which carries `new_owner` first) at 5.
fn drop_system_program(ix: &mut solana_program::instruction::Instruction, index: usize) {
    assert_eq!(
        ix.accounts[index].pubkey,
        solana_system_interface::program::ID,
        "account {index} is not the system program"
    );
    ix::set_account(ix, index, AccountMeta::new_readonly(MPL_CORE_ID, false));
}

#[test]
fn burn_hashed_asset_matrix() {
    let (f, owner) = world();
    let proof = proof_for(owner, 5, unsorted_plugins());
    let asset = put_hashed(&f, &proof);

    // No proof at all.
    assert_core_err(
        &f.run(&ix::burn_v1(
            asset,
            None,
            owner,
            None,
            Some(SPL_NOOP_ID),
            None,
        )),
        MplCoreError::MissingCompressionProof,
    );

    // A proof but no system program.
    let mut ix = ix::burn_v1(
        asset,
        None,
        owner,
        None,
        Some(SPL_NOOP_ID),
        Some(client_proof(&proof)),
    );
    drop_system_program(&mut ix, 4);
    assert_core_err(&f.run(&ix), MplCoreError::MissingSystemProgram);

    // A proof that does not match the account hash.
    let mut tampered = proof.clone();
    tampered.seq = 6;
    assert_core_err(
        &f.run(&ix::burn_v1(
            asset,
            None,
            owner,
            None,
            Some(SPL_NOOP_ID),
            Some(client_proof(&tampered)),
        )),
        MplCoreError::IncorrectAssetHash,
    );

    // The matching proof: rebuild, bump the sequence, wrap into noop, stop.
    assert_core_err(
        &f.run(&ix::burn_v1(
            asset,
            None,
            owner,
            None,
            Some(SPL_NOOP_ID),
            Some(client_proof(&proof)),
        )),
        MplCoreError::NotAvailable,
    );
}

#[test]
fn transfer_hashed_asset_matrix() {
    let (f, owner) = world();
    let new_owner = f.fund(ACCOUNT_LAMPORTS);
    let proof = proof_for(owner, 2, vec![]);
    let asset = put_hashed(&f, &proof);

    assert_core_err(
        &f.run(&ix::transfer_v1(
            asset,
            None,
            owner,
            None,
            new_owner,
            Some(SPL_NOOP_ID),
            None,
        )),
        MplCoreError::MissingCompressionProof,
    );

    let mut ix = ix::transfer_v1(
        asset,
        None,
        owner,
        None,
        new_owner,
        Some(SPL_NOOP_ID),
        Some(client_proof(&proof)),
    );
    drop_system_program(&mut ix, 5);
    assert_core_err(&f.run(&ix), MplCoreError::MissingSystemProgram);

    let mut tampered = proof.clone();
    tampered.uri = "other".to_string();
    assert_core_err(
        &f.run(&ix::transfer_v1(
            asset,
            None,
            owner,
            None,
            new_owner,
            Some(SPL_NOOP_ID),
            Some(client_proof(&tampered)),
        )),
        MplCoreError::IncorrectAssetHash,
    );

    // The transfer path rebuilds the account with the new owner already set
    // and then stops; the `HashedAssetV1` re-serialization after the early
    // return (`transfer.rs:118-136`) is dead code.
    assert_core_err(
        &f.run(&ix::transfer_v1(
            asset,
            None,
            owner,
            None,
            new_owner,
            Some(SPL_NOOP_ID),
            Some(client_proof(&proof)),
        )),
        MplCoreError::NotAvailable,
    );
}

/// `UpdateV1` and `ExecuteV1` refuse hashed assets up front, without touching
/// the proof machinery (they take no compression proof at all).
#[test]
fn update_and_execute_reject_hashed_assets_immediately() {
    let (f, owner) = world();
    let hashed = put(&f, hashed_asset_placeholder());

    assert_core_err(
        &f.run(&ix::update_v1(
            hashed,
            None,
            owner,
            None,
            None,
            Some("X".to_string()),
            None,
            None,
        )),
        MplCoreError::NotAvailable,
    );

    let (signer, _) = asset_signer_pda(&hashed);
    f.store(signer, empty_account());
    assert_core_err(
        &f.run(&ix::execute_v1(
            hashed,
            None,
            signer,
            owner,
            None,
            RECORDER_ID,
            vec![1, 2, 3],
            &[],
        )),
        MplCoreError::NotAvailable,
    );
}

/// The hashed account fixture is only meaningful if it really is what the
/// program writes: `HashedAssetV1` is one discriminator byte plus the 32-byte
/// hash, owned by mpl-core.
#[test]
fn hashed_asset_fixture_has_the_on_chain_layout() {
    let (_f, owner) = world();
    let proof = proof_for(owner, 9, unsorted_plugins());
    let account = hashed_asset_for_proof(&proof);

    assert_eq!(account.owner, MPL_CORE_ID);
    assert_eq!(account.data.len(), 33);
    assert_eq!(account.data[0], Key::HashedAssetV1 as u8);
    assert_eq!(
        account.data[1..],
        hashed_asset_schema_for_proof(&proof)[..],
        "the account must carry the schema hash"
    );

    // `AssetV1::from(proof)` is what the asset hash is taken over, and it
    // always carries the proof's sequence number.
    let rebuilt = AssetV1::from(proof.clone());
    assert_eq!(rebuilt.seq, Some(9));
    assert_eq!(rebuilt.owner, owner);
    assert_eq!(rebuilt.update_authority, UpdateAuthority::Address(owner));
}

/// Golden vectors for the on-chain hash of a `HashedAssetV1`.
///
/// Every other fixture in this module derives its expected hash with
/// `hashed_asset_schema_for_proof`, which walks the same `AssetV1::hash`,
/// plugin hash, `compare_indeces` sort and `HashedAssetSchema::hash` path that
/// `verify_proof` walks. That catches a change on one side only. A change made
/// on both sides at once — the realistic shape of an accidental format break,
/// since the helper is written from the crate's own types — would pass
/// unnoticed.
///
/// These two vectors pin the bytes themselves, from inputs spelled out in
/// full. The first fixes the asset hash alone; the second adds two plugins
/// listed out of order, so it also fixes plugin hashing, the index sort and
/// the schema hash over a non-empty plugin list. Either failing means the
/// serialized form of a compressed asset changed, which is a consensus-visible
/// break for anything holding an existing `HashedAssetV1`, not a test to
/// update lightly.
#[test]
fn hashed_asset_schema_matches_its_golden_vectors() {
    let owner = Pubkey::new_from_array([7u8; 32]);
    let update_authority = Pubkey::new_from_array([9u8; 32]);
    let base = CompressionProof {
        owner,
        update_authority: UpdateAuthority::Address(update_authority),
        name: "Golden".to_string(),
        uri: "https://example.com/golden".to_string(),
        seq: 3,
        plugins: vec![],
    };

    assert_eq!(
        hashed_asset_schema_for_proof(&base),
        [
            232, 69, 128, 55, 166, 28, 212, 150, 182, 21, 202, 233, 210, 118, 48, 5, 58, 53, 58, 7,
            115, 161, 163, 244, 180, 29, 186, 122, 236, 182, 154, 0
        ],
        "the hash of a plugin-less compressed asset changed"
    );

    let with_plugins = CompressionProof {
        plugins: vec![
            HashablePluginSchema {
                index: 1,
                authority: Authority::UpdateAuthority,
                plugin: Plugin::FreezeDelegate(FreezeDelegate { frozen: true }),
            },
            HashablePluginSchema {
                index: 0,
                authority: Authority::Owner,
                plugin: Plugin::Attributes(Attributes {
                    attribute_list: vec![],
                }),
            },
        ],
        ..base
    };

    assert_eq!(
        hashed_asset_schema_for_proof(&with_plugins),
        [
            82, 119, 219, 156, 156, 251, 131, 133, 196, 149, 36, 152, 149, 155, 94, 169, 128, 21,
            158, 46, 100, 10, 120, 211, 92, 85, 43, 8, 123, 141, 119, 204
        ],
        "the hash of a compressed asset with plugins changed"
    );
}
