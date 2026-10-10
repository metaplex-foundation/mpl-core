//! Corrupted and hostile account layouts.
//!
//! Every account here is fabricated: a registry record that points at garbage
//! or claims the wrong plugin type, a valid discriminator over a truncated
//! payload, an account funded below its own rent minimum, a zero-byte account
//! passed where the program expects a discriminator. None of these can be
//! produced by any instruction of the program, so nothing in the JS or
//! Rust-client suites reaches them, and they are the branches a security fork
//! most wants pinned.
//!
//! Several of these paths abort rather than returning a clean error; those
//! tests assert `ProgramFailedToComplete` and name the unguarded access, with
//! a pointer to roadmap section 6. They document the current behaviour: if a
//! bounds check is added, the assertion is what has to change.

use {
    crate::common::{accounts::core_bytes_account, *},
    mpl_core_program::{
        error::MplCoreError,
        plugins::{Attributes, FreezeDelegate, Plugin, PluginType},
        state::{Authority, DataBlob, Key, UpdateAuthority},
    },
    solana_account::Account,
    solana_program::{
        instruction::{AccountMeta, InstructionError},
        program_error::ProgramError,
        pubkey::Pubkey,
    },
    solana_system_interface::error::SystemError,
};

/// `ProgramFailedToComplete` is how a panic inside the program surfaces.
const ABORTED: InstructionError = InstructionError::ProgramFailedToComplete;

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

fn writable(key: Pubkey) -> AccountMeta {
    AccountMeta::new(key, false)
}

fn freeze_delegate() -> Plugin {
    Plugin::FreezeDelegate(FreezeDelegate { frozen: false })
}

fn attributes() -> Plugin {
    Plugin::Attributes(Attributes {
        attribute_list: vec![],
    })
}

/// An asset carrying one `FreezeDelegate`, the base for the corrupt layouts.
fn asset_with_freeze(owner: Pubkey) -> Account {
    AssetSpec::new(owner)
        .plugin(freeze_delegate(), Authority::Owner)
        .build()
}

/// A `TransferV1` that has to walk the whole plugin registry before it can
/// decide anything, which is what makes it the probe for corrupt registries.
fn transfer(
    fixture: &Fixture,
    asset: Pubkey,
    owner: Pubkey,
) -> mollusk_svm::result::InstructionResult {
    let new_owner = fixture.fund(ACCOUNT_LAMPORTS);
    fixture.run(&ix::transfer_v1(
        asset, None, owner, None, new_owner, None, None,
    ))
}

// ===========================================================================
// Registry records that point at the wrong bytes
// ===========================================================================

/// A record whose offset lands inside the plugin area but not on a plugin
/// boundary: the bytes there do not deserialize as a `Plugin`.
#[test]
fn registry_record_pointing_into_garbage_is_rejected() {
    let (f, owner) = world();
    let base = asset_with_freeze(owner);
    let core_len = parse_asset(&base.data).0.len();
    // One byte into the plugin area: the first byte of the plugin is its
    // variant index, so this reads the payload as a variant.
    let corrupt = repoint_registry_record(&base, PluginType::FreezeDelegate, core_len + 10);
    let asset = put(&f, corrupt);

    assert_core_err(
        &transfer(&f, asset, owner),
        MplCoreError::DeserializationError,
    );
}

/// A record whose offset is past the end of the account. `check_plugin_key`
/// and the registry walkers slice `data[offset..]` without a bounds check
/// (roadmap section 6), so the program aborts instead of returning
/// `DeserializationError`.
#[test]
fn registry_record_pointing_past_the_end_aborts() {
    let (f, owner) = world();
    let base = asset_with_freeze(owner);
    let past_the_end = base.data.len() + 1;
    let corrupt = repoint_registry_record(&base, PluginType::FreezeDelegate, past_the_end);
    let asset = put(&f, corrupt);

    assert_instruction_err(&transfer(&f, asset, owner), ABORTED);
}

/// A record that claims one plugin type while the bytes it points at are a
/// different plugin. The lifecycle machinery decides *whether* to consult a
/// plugin from the record's `plugin_type` (`PluginType::check_transfer`) but
/// dispatches the validator on the deserialized plugin, so the two can
/// disagree. The record wins the "is there a check at all" question, which
/// means retyping a record silently disables the plugin it hides.
#[test]
fn registry_record_of_the_wrong_plugin_type_disables_the_plugin() {
    let (f, owner) = world();

    // A *frozen* `FreezeDelegate` (owner-managed, `CanReject` on transfer)
    // presented as `Attributes`, which has no transfer check. The freeze is
    // not enforced and the transfer goes through: a corrupt registry record
    // is enough to bypass a freeze. Nothing in the program can write this
    // state, but it is worth pinning that the plugin bytes alone are not what
    // enforces the check.
    let frozen = retype_registry_record(
        &AssetSpec::new(owner)
            .plugin(
                Plugin::FreezeDelegate(FreezeDelegate { frozen: true }),
                Authority::Owner,
            )
            .build(),
        PluginType::FreezeDelegate,
        PluginType::Attributes,
    );
    let asset = put(&f, frozen);
    assert_ok(&transfer(&f, asset, owner));

    // The reverse direction: `Attributes` bytes behind a `FreezeDelegate`
    // record. Here the check table does consult the plugin, and the validator
    // dispatched from the deserialized `Attributes` abstains, so the transfer
    // is allowed rather than erroring.
    let mislabelled = retype_registry_record(
        &AssetSpec::new(owner)
            .plugin(attributes(), Authority::UpdateAuthority)
            .build(),
        PluginType::Attributes,
        PluginType::FreezeDelegate,
    );
    let asset = put(&f, mislabelled);
    assert_ok(&transfer(&f, asset, owner));
}

/// The same asset with an *intact* registry record does reject the transfer,
/// which is what makes the case above a behavioural difference rather than a
/// fixture mistake.
#[test]
fn frozen_asset_with_an_intact_registry_rejects_the_transfer() {
    let (f, owner) = world();
    let asset = put(
        &f,
        AssetSpec::new(owner)
            .plugin(
                Plugin::FreezeDelegate(FreezeDelegate { frozen: true }),
                Authority::Owner,
            )
            .build(),
    );

    assert_core_err(&transfer(&f, asset, owner), MplCoreError::InvalidAuthority);
}

// ===========================================================================
// Truncated payloads behind a valid discriminator
// ===========================================================================

#[test]
fn truncated_asset_payload_is_rejected() {
    let (f, owner) = world();
    let full = AssetSpec::new(owner).build();

    // Valid `Key::AssetV1` byte, then half of the payload.
    let asset = put(&f, truncate_account(&full, full.data.len() / 2));
    assert_core_err(
        &transfer(&f, asset, owner),
        MplCoreError::DeserializationError,
    );

    // Just the discriminator.
    let asset = put(&f, truncate_account(&full, 1));
    assert_core_err(
        &transfer(&f, asset, owner),
        MplCoreError::DeserializationError,
    );

    // No data at all: `load_key` indexes `data[0]` unguarded (roadmap
    // section 6), so this aborts rather than failing cleanly.
    let asset = put(&f, core_bytes_account(vec![]));
    assert_instruction_err(&transfer(&f, asset, owner), ABORTED);
}

/// An asset whose data ends in the middle of the plugin header: the core
/// account deserializes, `data.len() != core.len()` so a header is expected,
/// and there are not enough bytes for one.
#[test]
fn truncated_plugin_header_is_rejected() {
    let (f, owner) = world();
    let full = asset_with_freeze(owner);
    let core_len = parse_asset(&full.data).0.len();

    let asset = put(&f, truncate_account(&full, core_len + 4));
    assert_core_err(
        &transfer(&f, asset, owner),
        MplCoreError::DeserializationError,
    );
}

/// A plugin header whose `plugin_registry_offset` is past the end of the
/// account. `PluginRegistryV1::load` slices from that offset.
#[test]
fn plugin_registry_offset_past_the_end() {
    let (f, owner) = world();
    let full = asset_with_freeze(owner);
    let core_len = parse_asset(&full.data).0.len();

    // The header is `Key` + a little-endian u64 offset.
    let mut offset_bytes = [0u8; 8];
    offset_bytes.copy_from_slice(&(full.data.len() as u64 + 8).to_le_bytes());
    let corrupt = overwrite_bytes(&full, core_len + 1, &offset_bytes);
    let asset = put(&f, corrupt);

    assert_instruction_err(&transfer(&f, asset, owner), ABORTED);
}

// ===========================================================================
// Accounts funded below their own rent minimum
// ===========================================================================

/// `close_program_account` computes the refund from rent minimums rather than
/// the actual balance and debits it with an unchecked `-=`
/// (`utils/account.rs:29`). With `overflow-checks = true` an account holding
/// less than `rent(len) - rent(1)` makes the burn abort (roadmap section 13,
/// note 6) instead of returning an error.
#[test]
fn burning_an_underfunded_account_aborts() {
    let (f, owner) = world();
    let asset = put(&f, AssetSpec::new(owner).lamports(1).build());

    assert_instruction_err(
        &f.run(&ix::burn_v1(asset, None, owner, None, None, None)),
        ABORTED,
    );
}

/// The same account at exactly the rent minimum burns cleanly and keeps
/// `rent(1)`, which is the invariant the unchecked subtraction relies on.
#[test]
fn burning_an_account_at_exactly_its_rent_minimum_succeeds() {
    let (f, owner) = world();
    let bare = AssetSpec::new(owner).build();
    let rent = rent_exempt_balance(bare.data.len());
    let asset = put(&f, AssetSpec::new(owner).lamports(rent).build());

    f.run_ok(&ix::burn_v1(asset, None, owner, None, None, None));

    f.assert_burned(&asset);
    assert_eq!(
        f.lamports(&asset),
        rent_exempt_balance(1),
        "exactly one byte of rent is left behind"
    );
}

// ===========================================================================
// Wrong account types in the collection slot
// ===========================================================================

/// An asset whose `UpdateAuthority::Collection` names another `AssetV1`
/// account. Passing that account as the collection reaches the discriminator
/// check in `SolanaAccount::load` with a *valid but different* key, which is
/// the only way to hit that arm (`state/traits.rs:27`).
#[test]
fn collection_slot_holding_an_asset_is_rejected() {
    let (f, owner) = world();
    let impostor = put(&f, AssetSpec::new(owner).build());
    let asset = put(
        &f,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Collection(impostor))
            .build(),
    );
    let new_owner = f.fund(ACCOUNT_LAMPORTS);

    assert_core_err(
        &f.run(&ix::transfer_v1(
            asset,
            Some(impostor),
            owner,
            None,
            new_owner,
            None,
            None,
        )),
        MplCoreError::DeserializationError,
    );
}

/// A group account in the collection slot: same arm, different key.
#[test]
fn collection_slot_holding_a_group_is_rejected() {
    let (f, owner) = world();
    let group = put(&f, GroupSpec::new(owner).build());
    let asset = put(
        &f,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Collection(group))
            .build(),
    );
    let new_owner = f.fund(ACCOUNT_LAMPORTS);

    assert_core_err(
        &f.run(&ix::transfer_v1(
            asset,
            Some(group),
            owner,
            None,
            new_owner,
            None,
            None,
        )),
        MplCoreError::DeserializationError,
    );
}

// ===========================================================================
// Zero-byte accounts where the program reads a discriminator
// ===========================================================================

/// `load_key` reads `data[0]` with no length check, and the group
/// instructions call it on caller-supplied remaining accounts (roadmap
/// section 10, note 1). A plain system wallet — zero bytes — therefore aborts
/// the transaction instead of producing `IncorrectAccount`.
#[test]
fn group_instructions_abort_on_zero_byte_remaining_accounts() {
    let (f, payer) = world();
    let wallet = put(&f, payer_account(ACCOUNT_LAMPORTS));

    // AddAssetsToGroupV1 classifies every remaining account by discriminator.
    let group = put(&f, GroupSpec::new(payer).build());
    assert_instruction_err(
        &f.run(&ix::add_assets_to_group_v1(
            group,
            payer,
            None,
            &[writable(wallet)],
        )),
        ABORTED,
    );

    // RemoveAssetsFromGroupV1 checks the supplemental accounts the same way.
    let asset = Pubkey::new_unique();
    let group = put(&f, GroupSpec::new(payer).assets(vec![asset]).build());
    f.store(asset, AssetSpec::new(payer).build());
    assert_instruction_err(
        &f.run(&ix::remove_assets_from_group_v1(
            group,
            payer,
            None,
            vec![asset],
            &[writable(asset), writable(wallet)],
        )),
        ABORTED,
    );

    // And so does CreateGroupV1's supplemental-account loop.
    let new_group = put(&f, empty_account());
    assert_instruction_err(
        &f.run(&ix::create_group_v1(
            new_group,
            None,
            payer,
            "G",
            "uri",
            vec![mpl_core::types::RelationshipEntry {
                kind: mpl_core::types::RelationshipKind::Asset,
                key: asset,
            }],
            &[writable(asset), writable(wallet)],
        )),
        ABORTED,
    );
}

/// An mpl-core-owned account whose first byte is not a valid `Key`: `load_key`
/// maps it through `Key::from_u8` and returns a clean error.
#[test]
fn unknown_discriminator_is_rejected() {
    let (f, owner) = world();
    let asset = put(&f, core_bytes_account(vec![0xFF; 64]));

    assert_core_err(
        &transfer(&f, asset, owner),
        MplCoreError::DeserializationError,
    );
}

/// An account whose discriminator says `Uninitialized` (a burned asset) is
/// refused by the instruction's own key match, not by a deserialization error.
#[test]
fn burned_asset_cannot_be_transferred_again() {
    let (f, owner) = world();
    let asset = put(&f, core_bytes_account(vec![Key::Uninitialized as u8]));

    assert_core_err(&transfer(&f, asset, owner), MplCoreError::IncorrectAccount);
}

// ===========================================================================
// Payers that cannot fund the reallocs the instruction needs
// ===========================================================================

/// Every group membership change reallocates both the group account and the
/// member's plugin area upward, paid for by the payer through a system
/// transfer (`utils/account.rs:58-63`). A payer that cannot cover it makes the
/// CPI fail, which is the only way to reach that error path.
#[test]
fn group_membership_fails_when_the_payer_cannot_fund_the_realloc() {
    let (f, authority) = world();
    // A signer with no lamports at all: it is the payer, while the group and
    // asset authority is a separate funded key.
    let broke = put(&f, payer_account(0));
    let group = put(&f, GroupSpec::new(authority).build());
    let asset = put(&f, AssetSpec::new(authority).build());

    let result = f.run(&ix::add_assets_to_group_v1(
        group,
        broke,
        Some(authority),
        &[writable(asset)],
    ));

    // The failure comes from the system program's transfer, surfaced as
    // `Custom(SystemError::ResultWithNegativeLamports)`, not from mpl-core.
    assert_program_err(
        &result,
        ProgramError::Custom(SystemError::ResultWithNegativeLamports as u32),
    );
    assert_eq!(
        f.lamports(&broke),
        0,
        "the failed instruction must not have moved lamports"
    );
    assert!(
        read_plugin(&f.account(&asset), PluginType::Groups).is_none(),
        "the asset must not have gained a Groups plugin"
    );
}
