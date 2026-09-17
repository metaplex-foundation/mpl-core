//! Group instructions: `CreateGroupV1`, `CloseGroupV1`, `UpdateGroupV1`, the
//! four add/remove membership instructions, and the `Groups` plugin the
//! members carry.
//!
//! Mirrors `clients/js/test/{createGroup,closeGroup,group,groupComplexRelations,
//! groupsPluginBlocking,updateGroupAuthority}.test.ts`, plus the branches no JS
//! test can reach: 256-entry vectors, eight parent groups, a `CreateGroupV1`
//! whose relationship list exceeds the on-chain transaction size, the
//! account-count and key-mismatch guards, and the inconsistent bidirectional
//! state that no writer of the program can produce (roadmap section 10, note 3).
//!
//! A `GroupV1` account is flat Borsh with no plugin metadata, so every fixture
//! here is one [`GroupSpec`] literal; the members (assets and collections) are
//! the usual [`AssetSpec`] / [`CollectionSpec`] accounts.

use {
    crate::common::*,
    mpl_core::types::{RelationshipEntry, RelationshipKind},
    mpl_core_program::{
        error::MplCoreError,
        plugins::{FreezeDelegate, Groups, Plugin, PluginType, Royalties, RuleSet, UpdateDelegate},
        state::{
            Authority, GroupV1, Key, UpdateAuthority, MAX_GROUP_NESTING_DEPTH,
            MAX_GROUP_VECTOR_SIZE,
        },
        ID as MPL_CORE_ID,
    },
    solana_account::Account,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

/// A fake program ID standing in for an attacker's program.
const FAKE_PROGRAM_ID: Pubkey = Pubkey::new_from_array([0xAA; 32]);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A fixture with one funded payer, which is also the default authority.
fn world() -> (Fixture, Pubkey) {
    let fixture = Fixture::new();
    let payer = fixture.fund(ACCOUNT_LAMPORTS);
    (fixture, payer)
}

/// Stores a fabricated group account and returns its key.
fn put_group(fixture: &Fixture, spec: GroupSpec) -> Pubkey {
    let key = Pubkey::new_unique();
    fixture.store(key, spec.build());
    key
}

/// Stores a fabricated account and returns its key.
fn put(fixture: &Fixture, account: Account) -> Pubkey {
    let key = Pubkey::new_unique();
    fixture.store(key, account);
    key
}

/// A key with a zero-lamport system account, ready for `CreateGroupV1`.
fn new_group_key(fixture: &Fixture) -> Pubkey {
    put(fixture, empty_account())
}

fn writable(key: Pubkey) -> AccountMeta {
    AccountMeta::new(key, false)
}

fn readonly(key: Pubkey) -> AccountMeta {
    AccountMeta::new_readonly(key, false)
}

fn rel(kind: RelationshipKind, key: Pubkey) -> RelationshipEntry {
    RelationshipEntry { kind, key }
}

/// Replaces the system program meta with a key that is not the system program.
fn break_system_program(fixture: &Fixture, ix: &mut Instruction, index: usize) {
    let impostor = put(fixture, payer_account(ACCOUNT_LAMPORTS));
    ix::set_account(ix, index, readonly(impostor));
}

/// Clears `is_writable` on the meta at `index`, keeping its signer flag.
fn make_readonly(ix: &mut Instruction, index: usize) {
    let meta = ix.accounts[index].clone();
    ix::set_account(
        ix,
        index,
        AccountMeta {
            pubkey: meta.pubkey,
            is_signer: meta.is_signer,
            is_writable: false,
        },
    );
}

/// The `Groups` plugin of a member account, with the authority it was
/// registered under.
fn groups_plugin(account: &Account) -> Option<(Authority, Vec<Pubkey>)> {
    match read_plugin(account, PluginType::Groups) {
        Some((authority, Plugin::Groups(groups))) => Some((authority, groups.groups)),
        Some((_, other)) => panic!("the Groups registry record points at {other:?}"),
        None => None,
    }
}

/// Asserts a member carries exactly `expected` in an authority-managed
/// `Groups` plugin, and that its plugin layout is still intact.
fn assert_member_groups(account: &Account, expected: &[Pubkey]) {
    let (authority, groups) =
        groups_plugin(account).unwrap_or_else(|| panic!("account has no Groups plugin"));
    assert_eq!(
        authority,
        Authority::UpdateAuthority,
        "the Groups plugin is authority-managed"
    );
    assert_eq!(groups, expected, "unexpected Groups plugin contents");
    assert_registry_consistent(account);
}

/// An `UpdateDelegate` plugin naming `delegate` as an additional delegate.
fn update_delegate_for(delegate: Pubkey) -> Plugin {
    Plugin::UpdateDelegate(UpdateDelegate {
        additional_delegates: vec![delegate],
    })
}

/// A collection-managed asset: `UpdateAuthority::Collection(collection)`.
fn collection_managed_asset(owner: Pubkey, collection: Pubkey) -> Account {
    AssetSpec::new(owner)
        .update_authority(UpdateAuthority::Collection(collection))
        .build()
}

/// Runs `CreateGroupV1` with no relationships against a fresh group key.
fn create_empty_group(fixture: &Fixture, payer: Pubkey) -> Pubkey {
    let group = new_group_key(fixture);
    fixture.run_ok(&ix::create_group_v1(
        group,
        None,
        payer,
        "Test Group",
        "https://example.com/group",
        vec![],
        &[],
    ));
    group
}

/// The group account as the program left it.
fn group_of(fixture: &Fixture, key: &Pubkey) -> GroupV1 {
    read_group(&fixture.account(key))
}

/// Asserts the stored group account is exactly `GroupV1::len()` bytes and
/// rent exempt for that size, which every `save_flat_group` must maintain.
fn assert_group_sized(fixture: &Fixture, key: &Pubkey) {
    let account = fixture.account(key);
    let group = read_group(&account);
    let len = borsh::to_vec(&group).expect("group serializes").len();
    assert_eq!(
        account.data.len(),
        len,
        "group account is {} bytes but its contents serialize to {len}",
        account.data.len()
    );
    assert!(
        account.lamports >= rent_exempt_balance(len),
        "group account of {len} bytes holds {} lamports, below the rent minimum {}",
        account.lamports,
        rent_exempt_balance(len)
    );
}

// ===========================================================================
// CreateGroupV1: happy paths
// ===========================================================================

/// JS: createGroup.test.ts :: it can create a new group
#[test]
fn create_group_minimal() {
    let (f, payer) = world();
    let group = new_group_key(&f);

    let result = f.run_ok(&ix::create_group_v1(
        group,
        None,
        payer,
        "My Group",
        "https://example.com/g.json",
        vec![],
        &[],
    ));

    let stored = group_of(&f, &group);
    assert_eq!(stored.key, Key::GroupV1);
    assert_eq!(stored.update_authority, payer, "the payer is the authority");
    assert_eq!(stored.name, "My Group");
    assert_eq!(stored.uri, "https://example.com/g.json");
    assert!(stored.collections.is_empty());
    assert!(stored.groups.is_empty());
    assert!(stored.parent_groups.is_empty());
    assert!(stored.assets.is_empty());
    assert_group_sized(&f, &group);
    assert_eq!(
        account_of(&result, &group).owner,
        MPL_CORE_ID,
        "the created group is owned by mpl-core"
    );
}

#[test]
fn create_group_with_update_authority_signer() {
    let (f, payer) = world();
    let authority = f.fund(ACCOUNT_LAMPORTS);
    let group = new_group_key(&f);

    f.run_ok(&ix::create_group_v1(
        group,
        Some(authority),
        payer,
        "Shared",
        "uri",
        vec![],
        &[],
    ));

    assert_eq!(
        group_of(&f, &group).update_authority,
        authority,
        "account 1 becomes the group update authority"
    );
}

/// JS: createGroup.test.ts :: it can createGroupV1 with all four relationship kinds in one call
#[test]
fn create_group_all_relationship_kinds() {
    let (f, payer) = world();
    let collection = put(&f, CollectionSpec::new(payer).build());
    let child = put_group(&f, GroupSpec::new(payer));
    let parent = put_group(&f, GroupSpec::new(payer));
    let asset = put(&f, AssetSpec::new(payer).build());
    let group = new_group_key(&f);

    f.run_ok(&ix::create_group_v1(
        group,
        None,
        payer,
        "All Kinds",
        "uri",
        vec![
            rel(RelationshipKind::Collection, collection),
            rel(RelationshipKind::ChildGroup, child),
            rel(RelationshipKind::ParentGroup, parent),
            rel(RelationshipKind::Asset, asset),
        ],
        &[
            writable(collection),
            writable(child),
            writable(parent),
            writable(asset),
        ],
    ));

    let stored = group_of(&f, &group);
    assert_eq!(stored.collections, vec![collection]);
    assert_eq!(stored.groups, vec![child]);
    assert_eq!(stored.parent_groups, vec![parent]);
    assert_eq!(stored.assets, vec![asset]);
    assert_group_sized(&f, &group);

    assert_eq!(
        group_of(&f, &child).parent_groups,
        vec![group],
        "the child group gained the new group as a parent"
    );
    assert_eq!(
        group_of(&f, &parent).groups,
        vec![group],
        "the parent group gained the new group as a child"
    );
    assert_group_sized(&f, &child);
    assert_group_sized(&f, &parent);

    assert_member_groups(&f.account(&collection), &[group]);
    assert_member_groups(&f.account(&asset), &[group]);
}

/// JS: createGroup.test.ts :: it allows collection authority to link collection-managed assets in createGroupV1
#[test]
fn create_group_links_collection_managed_asset_with_supplemental_account() {
    let (f, payer) = world();
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let collection = put(&f, CollectionSpec::new(payer).build());
    let asset = put(&f, collection_managed_asset(owner, collection));
    let group = new_group_key(&f);

    f.run_ok(&ix::create_group_v1(
        group,
        None,
        payer,
        "G",
        "uri",
        vec![rel(RelationshipKind::Asset, asset)],
        // The asset itself, then the collection as a read-only supplemental
        // account so `is_valid_asset_authority` can find it.
        &[writable(asset), readonly(collection)],
    ));

    assert_eq!(group_of(&f, &group).assets, vec![asset]);
    assert_member_groups(&f.account(&asset), &[group]);
}

// ===========================================================================
// CreateGroupV1: guards and argument validation
// ===========================================================================

#[test]
fn create_group_rejects_bad_system_program() {
    let (f, payer) = world();
    let group = new_group_key(&f);
    let mut ix = ix::create_group_v1(group, None, payer, "G", "uri", vec![], &[]);
    break_system_program(&f, &mut ix, 3);

    assert_core_err(&f.run(&ix), MplCoreError::InvalidSystemProgram);
}

#[test]
fn create_group_rejects_non_writable_group() {
    let (f, payer) = world();
    let group = new_group_key(&f);
    let mut ix = ix::create_group_v1(group, None, payer, "G", "uri", vec![], &[]);
    make_readonly(&mut ix, 0);

    assert_program_err(&f.run(&ix), ProgramError::InvalidAccountData);
}

#[test]
fn create_group_rejects_missing_signers() {
    let (f, payer) = world();

    let group = new_group_key(&f);
    let mut without_group_signer = ix::create_group_v1(group, None, payer, "G", "uri", vec![], &[]);
    ix::unsign(&mut without_group_signer, &group);
    assert_program_err(
        &f.run(&without_group_signer),
        ProgramError::MissingRequiredSignature,
    );

    let group = new_group_key(&f);
    let mut without_payer_signer = ix::create_group_v1(group, None, payer, "G", "uri", vec![], &[]);
    ix::unsign(&mut without_payer_signer, &payer);
    assert_program_err(
        &f.run(&without_payer_signer),
        ProgramError::MissingRequiredSignature,
    );

    let group = new_group_key(&f);
    let authority = f.fund(ACCOUNT_LAMPORTS);
    let mut without_authority_signer =
        ix::create_group_v1(group, Some(authority), payer, "G", "uri", vec![], &[]);
    ix::unsign(&mut without_authority_signer, &authority);
    assert_program_err(
        &f.run(&without_authority_signer),
        ProgramError::MissingRequiredSignature,
    );
}

#[test]
fn create_group_rejects_duplicate_relationship_entries() {
    let (f, payer) = world();
    let group = new_group_key(&f);
    let collection = put(&f, CollectionSpec::new(payer).build());

    // The same key twice, once in each of two categories.
    let result = f.run(&ix::create_group_v1(
        group,
        None,
        payer,
        "G",
        "uri",
        vec![
            rel(RelationshipKind::Collection, collection),
            rel(RelationshipKind::Asset, collection),
        ],
        &[writable(collection), writable(collection)],
    ));

    assert_core_err(&result, MplCoreError::DuplicateEntry);
}

/// JS: createGroup.test.ts :: it rejects creating a group with itself as a child relationship
#[test]
fn create_group_rejects_self_as_child() {
    let (f, payer) = world();
    let group = new_group_key(&f);

    let result = f.run(&ix::create_group_v1(
        group,
        None,
        payer,
        "G",
        "uri",
        vec![rel(RelationshipKind::ChildGroup, group)],
        &[writable(group)],
    ));

    assert_core_err(&result, MplCoreError::IncorrectAccount);
}

/// JS: createGroup.test.ts :: it rejects creating a group with itself as a parent relationship
#[test]
fn create_group_rejects_self_as_parent() {
    let (f, payer) = world();
    let group = new_group_key(&f);

    let result = f.run(&ix::create_group_v1(
        group,
        None,
        payer,
        "G",
        "uri",
        vec![rel(RelationshipKind::ParentGroup, group)],
        &[writable(group)],
    ));

    assert_core_err(&result, MplCoreError::IncorrectAccount);
}

/// The vector-full check runs before the account is created, so no remaining
/// accounts are needed and the keys can be random. 257 relationship entries
/// are 8+ KB of instruction data: this case cannot be sent on-chain, which is
/// why the JS suite has no equivalent (roadmap section 10).
#[test]
fn create_group_rejects_vector_full_at_creation() {
    let (f, payer) = world();
    let overfull = MAX_GROUP_VECTOR_SIZE + 1;

    for kind in [
        RelationshipKind::Collection,
        RelationshipKind::ChildGroup,
        RelationshipKind::Asset,
    ] {
        let group = new_group_key(&f);
        let relationships = random_keys(overfull)
            .into_iter()
            .map(|key| rel(kind.clone(), key))
            .collect();

        let result = f.run(&ix::create_group_v1(
            group,
            None,
            payer,
            "G",
            "uri",
            relationships,
            &[],
        ));

        // The two execution modes genuinely differ here, so assert what each
        // one really does rather than the one that is convenient.
        //
        // Native execution has no heap limit and reaches the check. Under SBF
        // the program aborts first: the runtime's bump allocator never
        // reclaims, so growing `seen` and the per-kind vector through their
        // doubling steps exhausts the 32 KB heap before the length check runs
        // (`create_group.rs:80-111`).
        //
        // Neither mode contradicts the other about on-chain behaviour, because
        // this branch cannot be reached on chain at all: 257 entries is about
        // 8.5 KB of instruction data against a 1232-byte transaction limit.
        // The branch is covered in the report because coverage is measured
        // natively; CI's SBF run pins the abort.
        if native_program_enabled() {
            assert_core_err(&result, MplCoreError::GroupVectorFull);
        } else {
            assert_instruction_err(
                &result,
                solana_program::instruction::InstructionError::ProgramFailedToComplete,
            );
        }
    }
}

/// JS: createGroup.test.ts :: it rejects createGroupV1 when parent relationships exceed nesting depth
#[test]
fn create_group_rejects_nesting_depth_at_creation() {
    let (f, payer) = world();
    let group = new_group_key(&f);
    let relationships = random_keys(MAX_GROUP_NESTING_DEPTH + 1)
        .into_iter()
        .map(|key| rel(RelationshipKind::ParentGroup, key))
        .collect();

    let result = f.run(&ix::create_group_v1(
        group,
        None,
        payer,
        "G",
        "uri",
        relationships,
        &[],
    ));

    assert_core_err(&result, MplCoreError::GroupNestingDepthExceeded);
}

#[test]
fn create_group_rejects_not_enough_remaining_accounts() {
    let (f, payer) = world();
    let group = new_group_key(&f);
    let collection = put(&f, CollectionSpec::new(payer).build());

    let result = f.run(&ix::create_group_v1(
        group,
        None,
        payer,
        "G",
        "uri",
        vec![rel(RelationshipKind::Collection, collection)],
        &[],
    ));

    // The system CPI has already created the account at this point; the
    // transaction reverts, so nothing is persisted.
    assert_program_err(&result, ProgramError::NotEnoughAccountKeys);
    assert!(
        f.account(&group).data.is_empty(),
        "the group account must not survive a failed CreateGroupV1"
    );
}

#[test]
fn create_group_rejects_non_collection_supplemental_account() {
    let (f, payer) = world();
    let group = new_group_key(&f);
    let asset = put(&f, AssetSpec::new(payer).build());
    let extra = put(&f, AssetSpec::new(payer).build());

    let result = f.run(&ix::create_group_v1(
        group,
        None,
        payer,
        "G",
        "uri",
        vec![rel(RelationshipKind::Asset, asset)],
        &[writable(asset), readonly(extra)],
    ));

    assert_core_err(&result, MplCoreError::IncorrectAccount);
}

// ===========================================================================
// CreateGroupV1: per-target link failures
// ===========================================================================

#[test]
fn create_group_collection_link_failures() {
    let (f, payer) = world();
    let stranger = f.fund(ACCOUNT_LAMPORTS);

    // Key mismatch: the remaining account is not the one named in the args.
    let collection = put(&f, CollectionSpec::new(payer).build());
    let other = put(&f, CollectionSpec::new(payer).build());
    let group = new_group_key(&f);
    assert_core_err(
        &f.run(&ix::create_group_v1(
            group,
            None,
            payer,
            "G",
            "uri",
            vec![rel(RelationshipKind::Collection, collection)],
            &[writable(other)],
        )),
        MplCoreError::IncorrectAccount,
    );

    // The collection account is not writable.
    let group = new_group_key(&f);
    assert_program_err(
        &f.run(&ix::create_group_v1(
            group,
            None,
            payer,
            "G",
            "uri",
            vec![rel(RelationshipKind::Collection, collection)],
            &[readonly(collection)],
        )),
        ProgramError::InvalidAccountData,
    );

    // The signer is neither the collection update authority nor a delegate.
    // The collection carries an `UpdateDelegate` naming someone else, so the
    // delegate lookup runs and returns false. A collection with no plugin
    // metadata at all panics instead; see
    // `link_target_without_plugin_metadata_panics_instead_of_rejecting`.
    let foreign = put(
        &f,
        CollectionSpec::new(stranger)
            .plugin(update_delegate_for(stranger), Authority::UpdateAuthority)
            .build(),
    );
    let group = new_group_key(&f);
    assert_core_err(
        &f.run(&ix::create_group_v1(
            group,
            None,
            payer,
            "G",
            "uri",
            vec![rel(RelationshipKind::Collection, foreign)],
            &[writable(foreign)],
        )),
        MplCoreError::InvalidAuthority,
    );

    // ... but an `UpdateDelegate.additional_delegates` entry is accepted.
    let delegated = put(
        &f,
        CollectionSpec::new(stranger)
            .plugin(update_delegate_for(payer), Authority::UpdateAuthority)
            .build(),
    );
    let group = new_group_key(&f);
    f.run_ok(&ix::create_group_v1(
        group,
        None,
        payer,
        "G",
        "uri",
        vec![rel(RelationshipKind::Collection, delegated)],
        &[writable(delegated)],
    ));
    assert_eq!(group_of(&f, &group).collections, vec![delegated]);
    assert_member_groups(&f.account(&delegated), &[group]);
}

#[test]
fn create_group_child_link_failures() {
    let (f, payer) = world();
    let stranger = f.fund(ACCOUNT_LAMPORTS);

    let child = put_group(&f, GroupSpec::new(payer));
    let other = put_group(&f, GroupSpec::new(payer));

    // Key mismatch.
    let group = new_group_key(&f);
    assert_core_err(
        &f.run(&ix::create_group_v1(
            group,
            None,
            payer,
            "G",
            "uri",
            vec![rel(RelationshipKind::ChildGroup, child)],
            &[writable(other)],
        )),
        MplCoreError::IncorrectAccount,
    );

    // Not writable.
    let group = new_group_key(&f);
    assert_program_err(
        &f.run(&ix::create_group_v1(
            group,
            None,
            payer,
            "G",
            "uri",
            vec![rel(RelationshipKind::ChildGroup, child)],
            &[readonly(child)],
        )),
        ProgramError::InvalidAccountData,
    );

    // An asset passed where a group is expected: the discriminator differs.
    let asset = put(&f, AssetSpec::new(payer).build());
    let group = new_group_key(&f);
    assert_core_err(
        &f.run(&ix::create_group_v1(
            group,
            None,
            payer,
            "G",
            "uri",
            vec![rel(RelationshipKind::ChildGroup, asset)],
            &[writable(asset)],
        )),
        MplCoreError::DeserializationError,
    );

    // A group account owned by another program.
    let foreign_owned = put_group(&f, GroupSpec::new(payer).program_owner(FAKE_PROGRAM_ID));
    let group = new_group_key(&f);
    assert_program_err(
        &f.run(&ix::create_group_v1(
            group,
            None,
            payer,
            "G",
            "uri",
            vec![rel(RelationshipKind::ChildGroup, foreign_owned)],
            &[writable(foreign_owned)],
        )),
        ProgramError::InvalidAccountOwner,
    );

    // A group with a different update authority.
    let foreign = put_group(&f, GroupSpec::new(stranger));
    let group = new_group_key(&f);
    assert_core_err(
        &f.run(&ix::create_group_v1(
            group,
            None,
            payer,
            "G",
            "uri",
            vec![rel(RelationshipKind::ChildGroup, foreign)],
            &[writable(foreign)],
        )),
        MplCoreError::InvalidAuthority,
    );

    // A child already at the maximum nesting depth. On-chain this takes eight
    // prior transactions; here it is one fabricated account.
    let saturated = put_group(
        &f,
        GroupSpec::new(payer).parent_groups(random_keys(MAX_GROUP_NESTING_DEPTH)),
    );
    let group = new_group_key(&f);
    assert_core_err(
        &f.run(&ix::create_group_v1(
            group,
            None,
            payer,
            "G",
            "uri",
            vec![rel(RelationshipKind::ChildGroup, saturated)],
            &[writable(saturated)],
        )),
        MplCoreError::GroupNestingDepthExceeded,
    );
}

#[test]
fn create_group_parent_link_failures() {
    let (f, payer) = world();
    let stranger = f.fund(ACCOUNT_LAMPORTS);

    let parent = put_group(&f, GroupSpec::new(payer));
    let other = put_group(&f, GroupSpec::new(payer));

    let group = new_group_key(&f);
    assert_core_err(
        &f.run(&ix::create_group_v1(
            group,
            None,
            payer,
            "G",
            "uri",
            vec![rel(RelationshipKind::ParentGroup, parent)],
            &[writable(other)],
        )),
        MplCoreError::IncorrectAccount,
    );

    let group = new_group_key(&f);
    assert_program_err(
        &f.run(&ix::create_group_v1(
            group,
            None,
            payer,
            "G",
            "uri",
            vec![rel(RelationshipKind::ParentGroup, parent)],
            &[readonly(parent)],
        )),
        ProgramError::InvalidAccountData,
    );

    let foreign = put_group(&f, GroupSpec::new(stranger));
    let group = new_group_key(&f);
    assert_core_err(
        &f.run(&ix::create_group_v1(
            group,
            None,
            payer,
            "G",
            "uri",
            vec![rel(RelationshipKind::ParentGroup, foreign)],
            &[writable(foreign)],
        )),
        MplCoreError::InvalidAuthority,
    );

    // A parent whose child-group vector is already full.
    let saturated = put_group(
        &f,
        GroupSpec::new(payer).groups(random_keys(MAX_GROUP_VECTOR_SIZE)),
    );
    let group = new_group_key(&f);
    assert_core_err(
        &f.run(&ix::create_group_v1(
            group,
            None,
            payer,
            "G",
            "uri",
            vec![rel(RelationshipKind::ParentGroup, saturated)],
            &[writable(saturated)],
        )),
        MplCoreError::GroupVectorFull,
    );
}

#[test]
fn create_group_asset_link_failures() {
    let (f, payer) = world();
    let stranger = f.fund(ACCOUNT_LAMPORTS);

    let asset = put(&f, AssetSpec::new(payer).build());
    let other = put(&f, AssetSpec::new(payer).build());

    let group = new_group_key(&f);
    assert_core_err(
        &f.run(&ix::create_group_v1(
            group,
            None,
            payer,
            "G",
            "uri",
            vec![rel(RelationshipKind::Asset, asset)],
            &[writable(other)],
        )),
        MplCoreError::IncorrectAccount,
    );

    let group = new_group_key(&f);
    assert_program_err(
        &f.run(&ix::create_group_v1(
            group,
            None,
            payer,
            "G",
            "uri",
            vec![rel(RelationshipKind::Asset, asset)],
            &[readonly(asset)],
        )),
        ProgramError::InvalidAccountData,
    );

    // Foreign update authority. As above, the asset carries an
    // `UpdateDelegate` listing someone else so the delegate lookup can run.
    let foreign = put(
        &f,
        AssetSpec::new(stranger)
            .plugin(update_delegate_for(stranger), Authority::UpdateAuthority)
            .build(),
    );
    let group = new_group_key(&f);
    assert_core_err(
        &f.run(&ix::create_group_v1(
            group,
            None,
            payer,
            "G",
            "uri",
            vec![rel(RelationshipKind::Asset, foreign)],
            &[writable(foreign)],
        )),
        MplCoreError::InvalidAuthority,
    );

    // Collection-managed, but the collection account is not in the
    // transaction: `is_valid_asset_authority` logs and falls through.
    let collection = put(&f, CollectionSpec::new(payer).build());
    let managed = put(
        &f,
        AssetSpec::new(stranger)
            .update_authority(UpdateAuthority::Collection(collection))
            .plugin(update_delegate_for(stranger), Authority::UpdateAuthority)
            .build(),
    );
    let group = new_group_key(&f);
    assert_core_err(
        &f.run(&ix::create_group_v1(
            group,
            None,
            payer,
            "G",
            "uri",
            vec![rel(RelationshipKind::Asset, managed)],
            &[writable(managed)],
        )),
        MplCoreError::InvalidAuthority,
    );

    // An `UpdateDelegate` on the asset itself accepts the signer.
    let delegated = put(
        &f,
        AssetSpec::new(stranger)
            .plugin(update_delegate_for(payer), Authority::UpdateAuthority)
            .build(),
    );
    let group = new_group_key(&f);
    f.run_ok(&ix::create_group_v1(
        group,
        None,
        payer,
        "G",
        "uri",
        vec![rel(RelationshipKind::Asset, delegated)],
        &[writable(delegated)],
    ));
    assert_eq!(group_of(&f, &group).assets, vec![delegated]);
    assert_member_groups(&f.account(&delegated), &[group]);
}

/// The delegate lookup in `is_valid_asset_authority` /
/// `is_valid_collection_authority` (`utils/mod.rs:586,627`) calls
/// `fetch_wrapped_plugin` with `core: Some(..)`, which skips the
/// `len() == data_len()` guard at `plugins/utils.rs:166` and reads the plugin
/// header at `data[data.len()]`. On a member with **no plugin metadata at all**
/// — the normal state of a freshly created asset or collection — that index is
/// out of bounds and the program aborts with `ProgramFailedToComplete` instead
/// of returning `InvalidAuthority`.
///
/// This is the same class of finding as the `load_key` panics in roadmap
/// section 6 ("panics on caller-supplied input instead of clean errors"), but
/// it is reached with an entirely well-formed account, so it fires for any
/// caller who is simply not the update authority. The tests pin the real
/// behaviour; only the assertions change if the guard is added.
#[test]
fn link_target_without_plugin_metadata_panics_instead_of_rejecting() {
    let (f, payer) = world();
    let stranger = f.fund(ACCOUNT_LAMPORTS);

    // A bare collection (no header, no registry) whose update authority is
    // not the signer.
    let collection = put(&f, CollectionSpec::new(stranger).build());
    let group = new_group_key(&f);
    assert_instruction_err(
        &f.run(&ix::create_group_v1(
            group,
            None,
            payer,
            "G",
            "uri",
            vec![rel(RelationshipKind::Collection, collection)],
            &[writable(collection)],
        )),
        solana_program::instruction::InstructionError::ProgramFailedToComplete,
    );

    // The same for a bare asset.
    let asset = put(&f, AssetSpec::new(stranger).build());
    let group = new_group_key(&f);
    assert_instruction_err(
        &f.run(&ix::create_group_v1(
            group,
            None,
            payer,
            "G",
            "uri",
            vec![rel(RelationshipKind::Asset, asset)],
            &[writable(asset)],
        )),
        solana_program::instruction::InstructionError::ProgramFailedToComplete,
    );

    // An empty plugin header and registry (the state after every plugin was
    // removed) is enough to get the clean rejection instead.
    let with_meta = put(&f, CollectionSpec::new(stranger).with_empty_meta().build());
    let group = new_group_key(&f);
    assert_core_err(
        &f.run(&ix::create_group_v1(
            group,
            None,
            payer,
            "G",
            "uri",
            vec![rel(RelationshipKind::Collection, with_meta)],
            &[writable(with_meta)],
        )),
        MplCoreError::InvalidAuthority,
    );
}

// ===========================================================================
// CloseGroupV1
// ===========================================================================

/// JS: closeGroup.test.ts :: it can close a group
#[test]
fn close_group_success() {
    let (f, payer) = world();
    let group = create_empty_group(&f, payer);

    let data_len = f.account(&group).data.len();
    let group_lamports = f.lamports(&group);
    let payer_lamports = f.lamports(&payer);
    let refund = rent_exempt_balance(data_len) - rent_exempt_balance(1);

    f.run_ok(&ix::close_group_v1(group, payer, None));

    let closed = f.account(&group);
    assert_eq!(
        closed.data,
        vec![Key::Uninitialized as u8],
        "a closed group keeps one Uninitialized byte"
    );
    assert_eq!(
        closed.owner, MPL_CORE_ID,
        "a closed group is still owned by mpl-core"
    );
    assert_eq!(
        closed.lamports,
        group_lamports - refund,
        "the group keeps rent for one byte"
    );
    assert_eq!(
        f.lamports(&payer),
        payer_lamports + refund,
        "the payer receives rent(len) - rent(1)"
    );
}

#[test]
fn close_group_rejects_non_empty_vectors() {
    let (f, payer) = world();
    let member = Pubkey::new_unique();

    // One case per vector: the `&&` chain short-circuits, so each needs its
    // own fixture.
    let specs = [
        GroupSpec::new(payer).collections(vec![member]),
        GroupSpec::new(payer).groups(vec![member]),
        GroupSpec::new(payer).parent_groups(vec![member]),
        GroupSpec::new(payer).assets(vec![member]),
    ];

    for spec in specs {
        let group = put_group(&f, spec);
        assert_core_err(
            &f.run(&ix::close_group_v1(group, payer, None)),
            MplCoreError::GroupMustBeEmpty,
        );
        assert_eq!(
            read_group(&f.account(&group)).key,
            Key::GroupV1,
            "a rejected close must leave the group intact"
        );
    }
}

#[test]
fn close_group_rejects_auth_and_account_errors() {
    let (f, payer) = world();
    let stranger = f.fund(ACCOUNT_LAMPORTS);

    // Wrong authority: groups have no delegate mechanism, the check is an
    // exact match against `update_authority`.
    let foreign = put_group(&f, GroupSpec::new(stranger));
    assert_core_err(
        &f.run(&ix::close_group_v1(foreign, payer, None)),
        MplCoreError::InvalidAuthority,
    );

    // Non-writable group.
    let group = put_group(&f, GroupSpec::new(payer));
    let mut ix = ix::close_group_v1(group, payer, None);
    make_readonly(&mut ix, 0);
    assert_program_err(&f.run(&ix), ProgramError::InvalidAccountData);

    // An asset account in the group slot.
    let asset = put(&f, AssetSpec::new(payer).build());
    assert_core_err(
        &f.run(&ix::close_group_v1(asset, payer, None)),
        MplCoreError::DeserializationError,
    );

    // A group account owned by another program.
    let foreign_owned = put_group(&f, GroupSpec::new(payer).program_owner(FAKE_PROGRAM_ID));
    assert_program_err(
        &f.run(&ix::close_group_v1(foreign_owned, payer, None)),
        ProgramError::InvalidAccountOwner,
    );

    // The payer must sign.
    let group = put_group(&f, GroupSpec::new(payer));
    let mut ix = ix::close_group_v1(group, payer, None);
    ix::unsign(&mut ix, &payer);
    assert_program_err(&f.run(&ix), ProgramError::MissingRequiredSignature);
}

// ===========================================================================
// UpdateGroupV1
// ===========================================================================

#[test]
fn update_group_transfer_authority_then_reject_old_authority() {
    let (f, payer) = world();
    let new_authority = f.fund(ACCOUNT_LAMPORTS);
    let group = create_empty_group(&f, payer);

    // 1. Hand the group to a new update authority (which does not sign).
    f.run_ok(&ix::update_group_v1(
        group,
        payer,
        None,
        Some(new_authority),
        None,
        None,
    ));
    assert_eq!(group_of(&f, &group).update_authority, new_authority);

    // 2. The new authority can rename it.
    f.run_ok(&ix::update_group_v1(
        group,
        payer,
        Some(new_authority),
        None,
        Some("Renamed".to_string()),
        None,
    ));
    assert_eq!(group_of(&f, &group).name, "Renamed");

    // 3. The old authority can no longer act.
    assert_core_err(
        &f.run(&ix::update_group_v1(
            group,
            payer,
            None,
            None,
            Some("Nope".to_string()),
            None,
        )),
        MplCoreError::InvalidAuthority,
    );
    assert_eq!(group_of(&f, &group).name, "Renamed");
}

/// JS: updateGroupAuthority.test.ts :: it can updateGroup with both name and URI simultaneously
#[test]
fn update_group_name_uri_grow_shrink_and_noop() {
    let (f, payer) = world();
    let group = new_group_key(&f);
    f.run_ok(&ix::create_group_v1(
        group,
        None,
        payer,
        "abc",
        "u",
        vec![],
        &[],
    ));
    let original_len = f.account(&group).data.len();

    // Grow: both fields get longer, so `save_flat_group` reallocates upward
    // and the payer tops up the rent.
    f.run_ok(&ix::update_group_v1(
        group,
        payer,
        None,
        None,
        Some("a much longer group name".to_string()),
        Some("https://example.com/a/much/longer/uri.json".to_string()),
    ));
    let grown = f.account(&group);
    assert_eq!(read_group(&grown).name, "a much longer group name");
    assert_eq!(
        read_group(&grown).uri,
        "https://example.com/a/much/longer/uri.json"
    );
    assert!(
        grown.data.len() > original_len,
        "the account should have grown"
    );
    assert_group_sized(&f, &group);

    // Shrink: this is the cheapest way to reach the lamport-return branch of
    // `resize_or_reallocate_account`.
    let payer_before = f.lamports(&payer);
    let group_before = f.lamports(&group);
    f.run_ok(&ix::update_group_v1(
        group,
        payer,
        None,
        None,
        Some("x".to_string()),
        Some("y".to_string()),
    ));
    let shrunk = f.account(&group);
    assert_eq!(read_group(&shrunk).name, "x");
    assert_eq!(read_group(&shrunk).uri, "y");
    assert!(shrunk.data.len() < grown.data.len());
    assert_group_sized(&f, &group);
    let returned = group_before - f.lamports(&group);
    assert!(returned > 0, "shrinking must return rent to the payer");
    assert_eq!(
        f.lamports(&payer),
        payer_before + returned,
        "the payer receives exactly what the group gave up"
    );

    // No-op: no account 3, no args, so `dirty` stays false and nothing is
    // written.
    let before = f.account(&group);
    f.run_ok(&ix::update_group_v1(group, payer, None, None, None, None));
    let after = f.account(&group);
    assert_eq!(after.data, before.data, "a no-op update must not rewrite");
    assert_eq!(after.lamports, before.lamports);
}

#[test]
fn update_group_rejects_bad_system_program_and_non_writable() {
    let (f, payer) = world();
    let group = put_group(&f, GroupSpec::new(payer));

    let mut ix = ix::update_group_v1(group, payer, None, None, Some("X".to_string()), None);
    break_system_program(&f, &mut ix, 4);
    assert_core_err(&f.run(&ix), MplCoreError::InvalidSystemProgram);

    let mut ix = ix::update_group_v1(group, payer, None, None, Some("X".to_string()), None);
    make_readonly(&mut ix, 0);
    assert_program_err(&f.run(&ix), ProgramError::InvalidAccountData);
}

// ===========================================================================
// AddAssetsToGroupV1
// ===========================================================================

/// A `Groups` plugin holding `groups`.
fn groups_plugin_with(groups: Vec<Pubkey>) -> Plugin {
    Plugin::Groups(Groups { groups })
}

/// An unfrozen `FreezeDelegate`, used as a trailing plugin whose bytes must
/// survive the `Groups` grow and shrink memmoves.
fn freeze_delegate() -> Plugin {
    Plugin::FreezeDelegate(FreezeDelegate { frozen: false })
}

/// JS: groupsPluginBlocking.test.ts :: it blocks generic asset plugin operations for Groups
#[test]
fn add_assets_success_fresh_and_with_existing_plugins() {
    let (f, payer) = world();
    let group = put_group(&f, GroupSpec::new(payer));
    // A bare asset: `create_meta_idempotent` has to create the header and
    // registry before the plugin can be initialized.
    let fresh = put(&f, AssetSpec::new(payer).build());
    // An asset that already carries a plugin: the load branch instead.
    let with_plugin = put(
        &f,
        AssetSpec::new(payer)
            .plugin(freeze_delegate(), Authority::Owner)
            .build(),
    );

    f.run_ok(&ix::add_assets_to_group_v1(
        group,
        payer,
        None,
        &[writable(fresh), writable(with_plugin)],
    ));

    assert_eq!(group_of(&f, &group).assets, vec![fresh, with_plugin]);
    assert_group_sized(&f, &group);
    assert_member_groups(&f.account(&fresh), &[group]);
    assert_member_groups(&f.account(&with_plugin), &[group]);
    assert_eq!(
        read_plugin(&f.account(&with_plugin), PluginType::FreezeDelegate),
        Some((Authority::Owner, freeze_delegate())),
        "the pre-existing plugin must survive untouched"
    );
}

/// JS: group.test.ts :: it allows collection update authority to add collection-managed assets to a group
#[test]
fn add_assets_collection_managed_with_separate_authority() {
    let (f, payer) = world();
    let shared = f.fund(ACCOUNT_LAMPORTS);
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let collection = put(&f, CollectionSpec::new(shared).build());
    let asset = put(&f, collection_managed_asset(owner, collection));
    let group = put_group(&f, GroupSpec::new(shared));

    // The authority differs from the payer, so the processor additionally
    // asserts that it signed.
    f.run_ok(&ix::add_assets_to_group_v1(
        group,
        payer,
        Some(shared),
        &[writable(asset), readonly(collection)],
    ));

    assert_eq!(group_of(&f, &group).assets, vec![asset]);
    assert_member_groups(&f.account(&asset), &[group]);
}

#[test]
fn add_assets_zero_remaining_accounts_is_a_silent_no_op() {
    let (f, payer) = world();
    let group = put_group(&f, GroupSpec::new(payer));
    let before = f.account(&group);

    // Documents roadmap section 10, note 5: with no remaining accounts the
    // instruction succeeds and simply rewrites the group unchanged.
    f.run_ok(&ix::add_assets_to_group_v1(group, payer, None, &[]));

    assert_eq!(f.account(&group).data, before.data);
}

/// JS: group.test.ts :: it rejects adding an already-member asset (duplicate entry)
#[test]
fn add_assets_rejects_duplicates_and_full_vector() {
    let (f, payer) = world();
    let asset = put(&f, AssetSpec::new(payer).build());

    // Already a member of the group.
    let group = put_group(&f, GroupSpec::new(payer).assets(vec![asset]));
    assert_core_err(
        &f.run(&ix::add_assets_to_group_v1(
            group,
            payer,
            None,
            &[writable(asset)],
        )),
        MplCoreError::DuplicateEntry,
    );

    // JS: group.test.ts :: it rejects duplicate asset in remaining accounts for addAssetsToGroup
    let group = put_group(&f, GroupSpec::new(payer));
    assert_core_err(
        &f.run(&ix::add_assets_to_group_v1(
            group,
            payer,
            None,
            &[writable(asset), writable(asset)],
        )),
        MplCoreError::DuplicateEntry,
    );

    // A full asset vector. On-chain this needs 256 transactions; here it is
    // one fabricated account.
    let group = put_group(
        &f,
        GroupSpec::new(payer).assets(random_keys(MAX_GROUP_VECTOR_SIZE)),
    );
    assert_core_err(
        &f.run(&ix::add_assets_to_group_v1(
            group,
            payer,
            None,
            &[writable(asset)],
        )),
        MplCoreError::GroupVectorFull,
    );
}

/// JS: group.test.ts :: it rejects addAssetsToGroup when signer is not group authority
#[test]
fn add_assets_rejections() {
    let (f, payer) = world();
    let stranger = f.fund(ACCOUNT_LAMPORTS);
    let asset = put(&f, AssetSpec::new(payer).build());
    let group = put_group(&f, GroupSpec::new(payer));

    // Wrong system program.
    let mut ix = ix::add_assets_to_group_v1(group, payer, None, &[writable(asset)]);
    break_system_program(&f, &mut ix, 3);
    assert_core_err(&f.run(&ix), MplCoreError::InvalidSystemProgram);

    // Non-writable group.
    let mut ix = ix::add_assets_to_group_v1(group, payer, None, &[writable(asset)]);
    make_readonly(&mut ix, 0);
    assert_program_err(&f.run(&ix), ProgramError::InvalidAccountData);

    // A signer who is not the group update authority.
    let foreign_group = put_group(&f, GroupSpec::new(stranger));
    assert_core_err(
        &f.run(&ix::add_assets_to_group_v1(
            foreign_group,
            payer,
            None,
            &[writable(asset)],
        )),
        MplCoreError::InvalidAuthority,
    );

    // A remaining account that is neither an asset nor a collection.
    let other_group = put_group(&f, GroupSpec::new(payer));
    assert_core_err(
        &f.run(&ix::add_assets_to_group_v1(
            group,
            payer,
            None,
            &[writable(other_group)],
        )),
        MplCoreError::IncorrectAccount,
    );

    // Only supplemental collections and no asset at all.
    let collection = put(&f, CollectionSpec::new(payer).build());
    assert_core_err(
        &f.run(&ix::add_assets_to_group_v1(
            group,
            payer,
            None,
            &[readonly(collection)],
        )),
        MplCoreError::IncorrectAccount,
    );

    // A non-writable asset.
    assert_program_err(
        &f.run(&ix::add_assets_to_group_v1(
            group,
            payer,
            None,
            &[readonly(asset)],
        )),
        ProgramError::InvalidAccountData,
    );

    // An asset whose update authority is somebody else.
    let foreign_asset = put(
        &f,
        AssetSpec::new(stranger)
            .plugin(update_delegate_for(stranger), Authority::UpdateAuthority)
            .build(),
    );
    assert_core_err(
        &f.run(&ix::add_assets_to_group_v1(
            group,
            payer,
            None,
            &[writable(foreign_asset)],
        )),
        MplCoreError::InvalidAuthority,
    );

    // A collection-managed asset whose collection is not in the transaction.
    let foreign_collection = put(&f, CollectionSpec::new(stranger).build());
    let managed = put(
        &f,
        AssetSpec::new(stranger)
            .update_authority(UpdateAuthority::Collection(foreign_collection))
            .plugin(update_delegate_for(stranger), Authority::UpdateAuthority)
            .build(),
    );
    assert_core_err(
        &f.run(&ix::add_assets_to_group_v1(
            group,
            payer,
            None,
            &[writable(managed)],
        )),
        MplCoreError::InvalidAuthority,
    );

    // An `UpdateDelegate` additional delegate on the asset is accepted.
    let delegated = put(
        &f,
        AssetSpec::new(stranger)
            .plugin(update_delegate_for(payer), Authority::UpdateAuthority)
            .build(),
    );
    f.run_ok(&ix::add_assets_to_group_v1(
        group,
        payer,
        None,
        &[writable(delegated)],
    ));
    assert_eq!(group_of(&f, &group).assets, vec![delegated]);
}

// ===========================================================================
// RemoveAssetsFromGroupV1
// ===========================================================================

/// JS: burn.test.ts :: it allows burning an asset after removing it from all groups
#[test]
fn remove_assets_success() {
    let (f, payer) = world();
    let asset_key = Pubkey::new_unique();
    let group = put_group(&f, GroupSpec::new(payer).assets(vec![asset_key]));
    f.store(
        asset_key,
        AssetSpec::new(payer)
            .plugin(groups_plugin_with(vec![group]), Authority::UpdateAuthority)
            .build(),
    );
    let group_len_before = f.account(&group).data.len();
    let asset_len_before = f.account(&asset_key).data.len();

    f.run_ok(&ix::remove_assets_from_group_v1(
        group,
        payer,
        None,
        vec![asset_key],
        &[writable(asset_key)],
    ));

    assert!(group_of(&f, &group).assets.is_empty());
    assert_eq!(
        f.account(&group).data.len(),
        group_len_before - 32,
        "the group shrinks by one pubkey"
    );
    assert_group_sized(&f, &group);

    // The plugin is emptied, not deleted.
    assert_member_groups(&f.account(&asset_key), &[]);
    assert_eq!(
        f.account(&asset_key).data.len(),
        asset_len_before - 32,
        "the asset shrinks by one pubkey"
    );
}

/// JS: group.test.ts :: it allows collection update authority to remove collection-managed assets from a group
#[test]
fn remove_assets_collection_managed_with_supplemental_account() {
    let (f, payer) = world();
    let shared = f.fund(ACCOUNT_LAMPORTS);
    let owner = f.fund(ACCOUNT_LAMPORTS);
    let collection = put(&f, CollectionSpec::new(shared).build());
    let asset = Pubkey::new_unique();
    let group = put_group(&f, GroupSpec::new(shared).assets(vec![asset]));
    f.store(
        asset,
        AssetSpec::new(owner)
            .update_authority(UpdateAuthority::Collection(collection))
            .plugin(groups_plugin_with(vec![group]), Authority::UpdateAuthority)
            .build(),
    );

    f.run_ok(&ix::remove_assets_from_group_v1(
        group,
        payer,
        Some(shared),
        vec![asset],
        &[writable(asset), readonly(collection)],
    ));

    assert!(group_of(&f, &group).assets.is_empty());
    assert_member_groups(&f.account(&asset), &[]);
}

/// JS: group.test.ts :: it rejects removeAssetsFromGroup when signer is not group authority
#[test]
fn remove_assets_rejections() {
    let (f, payer) = world();
    let stranger = f.fund(ACCOUNT_LAMPORTS);
    let asset = Pubkey::new_unique();
    let member_asset = || {
        AssetSpec::new(payer)
            .plugin(groups_plugin_with(vec![]), Authority::UpdateAuthority)
            .build()
    };
    f.store(asset, member_asset());
    let group = put_group(&f, GroupSpec::new(payer).assets(vec![asset]));

    let base = |remaining: &[AccountMeta]| {
        ix::remove_assets_from_group_v1(group, payer, None, vec![asset], remaining)
    };

    // Wrong system program.
    let mut ix = base(&[writable(asset)]);
    break_system_program(&f, &mut ix, 3);
    assert_core_err(&f.run(&ix), MplCoreError::InvalidSystemProgram);

    // Non-writable group.
    let mut ix = base(&[writable(asset)]);
    make_readonly(&mut ix, 0);
    assert_program_err(&f.run(&ix), ProgramError::InvalidAccountData);

    // Fewer remaining accounts than pubkeys in the args.
    assert_program_err(&f.run(&base(&[])), ProgramError::NotEnoughAccountKeys);

    // A supplemental account that is not a collection.
    let extra = put(&f, AssetSpec::new(payer).build());
    assert_core_err(
        &f.run(&base(&[writable(asset), readonly(extra)])),
        MplCoreError::IncorrectAccount,
    );

    // A signer who is not the group update authority.
    let foreign_group = put_group(&f, GroupSpec::new(stranger).assets(vec![asset]));
    assert_core_err(
        &f.run(&ix::remove_assets_from_group_v1(
            foreign_group,
            payer,
            None,
            vec![asset],
            &[writable(asset)],
        )),
        MplCoreError::InvalidAuthority,
    );

    // The remaining account does not match the pubkey at the same index.
    let other = put(&f, AssetSpec::new(payer).build());
    assert_core_err(
        &f.run(&base(&[writable(other)])),
        MplCoreError::IncorrectAccount,
    );

    // A non-writable asset.
    assert_program_err(
        &f.run(&base(&[readonly(asset)])),
        ProgramError::InvalidAccountData,
    );

    // An asset the signer has no authority over.
    let foreign_asset = Pubkey::new_unique();
    f.store(
        foreign_asset,
        AssetSpec::new(stranger)
            .plugin(update_delegate_for(stranger), Authority::UpdateAuthority)
            .build(),
    );
    let group_with_foreign = put_group(&f, GroupSpec::new(payer).assets(vec![foreign_asset]));
    assert_core_err(
        &f.run(&ix::remove_assets_from_group_v1(
            group_with_foreign,
            payer,
            None,
            vec![foreign_asset],
            &[writable(foreign_asset)],
        )),
        MplCoreError::InvalidAuthority,
    );

    // An asset that is not a member of the group.
    let empty_group = put_group(&f, GroupSpec::new(payer));
    assert_core_err(
        &f.run(&ix::remove_assets_from_group_v1(
            empty_group,
            payer,
            None,
            vec![asset],
            &[writable(asset)],
        )),
        MplCoreError::IncorrectAccount,
    );
}

// ===========================================================================
// AddCollectionsToGroupV1 / RemoveCollectionsFromGroupV1
// ===========================================================================

/// A `Royalties` plugin, used as a pre-existing plugin on a collection.
fn royalties() -> Plugin {
    Plugin::Royalties(Royalties {
        basis_points: 500,
        creators: vec![],
        rule_set: RuleSet::None,
    })
}

/// JS: groupsPluginBlocking.test.ts :: it blocks generic collection plugin operations for Groups
#[test]
fn add_collections_success_fresh_and_with_existing_plugins() {
    let (f, payer) = world();
    let group = put_group(&f, GroupSpec::new(payer));
    let fresh = put(&f, CollectionSpec::new(payer).build());
    let with_plugin = put(
        &f,
        CollectionSpec::new(payer)
            .plugin(royalties(), Authority::UpdateAuthority)
            .build(),
    );

    f.run_ok(&ix::add_collections_to_group_v1(
        group,
        payer,
        None,
        &[writable(fresh), writable(with_plugin)],
    ));

    assert_eq!(group_of(&f, &group).collections, vec![fresh, with_plugin]);
    assert_group_sized(&f, &group);
    assert_member_groups(&f.account(&fresh), &[group]);
    assert_member_groups(&f.account(&with_plugin), &[group]);
    assert_eq!(
        read_plugin(&f.account(&with_plugin), PluginType::Royalties),
        Some((Authority::UpdateAuthority, royalties())),
        "the pre-existing plugin must survive untouched"
    );
}

/// JS: group.test.ts :: it rejects adding an already-member collection (duplicate entry)
#[test]
fn add_collections_rejects_duplicates_and_full_vector() {
    let (f, payer) = world();
    let collection = put(&f, CollectionSpec::new(payer).build());

    let group = put_group(&f, GroupSpec::new(payer).collections(vec![collection]));
    assert_core_err(
        &f.run(&ix::add_collections_to_group_v1(
            group,
            payer,
            None,
            &[writable(collection)],
        )),
        MplCoreError::DuplicateEntry,
    );

    // JS: group.test.ts :: it rejects duplicate collection in remaining accounts for addCollectionsToGroup
    let group = put_group(&f, GroupSpec::new(payer));
    assert_core_err(
        &f.run(&ix::add_collections_to_group_v1(
            group,
            payer,
            None,
            &[writable(collection), writable(collection)],
        )),
        MplCoreError::DuplicateEntry,
    );

    let group = put_group(
        &f,
        GroupSpec::new(payer).collections(random_keys(MAX_GROUP_VECTOR_SIZE)),
    );
    assert_core_err(
        &f.run(&ix::add_collections_to_group_v1(
            group,
            payer,
            None,
            &[writable(collection)],
        )),
        MplCoreError::GroupVectorFull,
    );
}

/// JS: group.test.ts :: it rejects addCollectionsToGroup when signer is not group authority
#[test]
fn add_collections_rejections() {
    let (f, payer) = world();
    let stranger = f.fund(ACCOUNT_LAMPORTS);
    let collection = put(&f, CollectionSpec::new(payer).build());
    let group = put_group(&f, GroupSpec::new(payer));

    let mut ix = ix::add_collections_to_group_v1(group, payer, None, &[writable(collection)]);
    break_system_program(&f, &mut ix, 3);
    assert_core_err(&f.run(&ix), MplCoreError::InvalidSystemProgram);

    let mut ix = ix::add_collections_to_group_v1(group, payer, None, &[writable(collection)]);
    make_readonly(&mut ix, 0);
    assert_program_err(&f.run(&ix), ProgramError::InvalidAccountData);

    let foreign_group = put_group(&f, GroupSpec::new(stranger));
    assert_core_err(
        &f.run(&ix::add_collections_to_group_v1(
            foreign_group,
            payer,
            None,
            &[writable(collection)],
        )),
        MplCoreError::InvalidAuthority,
    );

    // A non-writable collection.
    assert_program_err(
        &f.run(&ix::add_collections_to_group_v1(
            group,
            payer,
            None,
            &[readonly(collection)],
        )),
        ProgramError::InvalidAccountData,
    );

    // An asset passed where a collection is expected.
    let asset = put(&f, AssetSpec::new(payer).build());
    assert_core_err(
        &f.run(&ix::add_collections_to_group_v1(
            group,
            payer,
            None,
            &[writable(asset)],
        )),
        MplCoreError::DeserializationError,
    );

    // A collection owned by another program.
    let foreign_owned = put(
        &f,
        CollectionSpec::new(payer)
            .program_owner(FAKE_PROGRAM_ID)
            .build(),
    );
    assert_program_err(
        &f.run(&ix::add_collections_to_group_v1(
            group,
            payer,
            None,
            &[writable(foreign_owned)],
        )),
        ProgramError::InvalidAccountOwner,
    );

    // A collection the signer has no authority over.
    let foreign = put(
        &f,
        CollectionSpec::new(stranger)
            .plugin(update_delegate_for(stranger), Authority::UpdateAuthority)
            .build(),
    );
    assert_core_err(
        &f.run(&ix::add_collections_to_group_v1(
            group,
            payer,
            None,
            &[writable(foreign)],
        )),
        MplCoreError::InvalidAuthority,
    );

    // ... unless it lists the signer as an additional update delegate.
    let delegated = put(
        &f,
        CollectionSpec::new(stranger)
            .plugin(update_delegate_for(payer), Authority::UpdateAuthority)
            .build(),
    );
    f.run_ok(&ix::add_collections_to_group_v1(
        group,
        payer,
        None,
        &[writable(delegated)],
    ));
    assert_eq!(group_of(&f, &group).collections, vec![delegated]);
}

/// JS: burnCollection.test.ts :: it allows burning a collection after removing it from all groups
#[test]
fn remove_collections_success() {
    let (f, payer) = world();
    let collection = Pubkey::new_unique();
    let group = put_group(&f, GroupSpec::new(payer).collections(vec![collection]));
    f.store(
        collection,
        CollectionSpec::new(payer)
            .plugin(groups_plugin_with(vec![group]), Authority::UpdateAuthority)
            .build(),
    );
    let collection_len_before = f.account(&collection).data.len();

    f.run_ok(&ix::remove_collections_from_group_v1(
        group,
        payer,
        None,
        vec![collection],
        &[writable(collection)],
    ));

    assert!(group_of(&f, &group).collections.is_empty());
    assert_group_sized(&f, &group);
    assert_member_groups(&f.account(&collection), &[]);
    assert_eq!(
        f.account(&collection).data.len(),
        collection_len_before - 32
    );
}

/// JS: group.test.ts :: it rejects removeCollectionsFromGroup when signer is not group authority
#[test]
fn remove_collections_rejections() {
    let (f, payer) = world();
    let stranger = f.fund(ACCOUNT_LAMPORTS);
    let collection = Pubkey::new_unique();
    f.store(
        collection,
        CollectionSpec::new(payer)
            .plugin(groups_plugin_with(vec![]), Authority::UpdateAuthority)
            .build(),
    );
    let group = put_group(&f, GroupSpec::new(payer).collections(vec![collection]));

    let base = |remaining: &[AccountMeta]| {
        ix::remove_collections_from_group_v1(group, payer, None, vec![collection], remaining)
    };

    let mut ix = base(&[writable(collection)]);
    break_system_program(&f, &mut ix, 3);
    assert_core_err(&f.run(&ix), MplCoreError::InvalidSystemProgram);

    let mut ix = base(&[writable(collection)]);
    make_readonly(&mut ix, 0);
    assert_program_err(&f.run(&ix), ProgramError::InvalidAccountData);

    // Unlike the asset variant, the count must match exactly in both
    // directions: no supplemental accounts are allowed.
    assert_program_err(&f.run(&base(&[])), ProgramError::NotEnoughAccountKeys);
    let extra = put(&f, CollectionSpec::new(payer).build());
    assert_program_err(
        &f.run(&base(&[writable(collection), writable(extra)])),
        ProgramError::NotEnoughAccountKeys,
    );

    let foreign_group = put_group(&f, GroupSpec::new(stranger).collections(vec![collection]));
    assert_core_err(
        &f.run(&ix::remove_collections_from_group_v1(
            foreign_group,
            payer,
            None,
            vec![collection],
            &[writable(collection)],
        )),
        MplCoreError::InvalidAuthority,
    );

    // Key mismatch between the args and the remaining accounts.
    assert_core_err(
        &f.run(&base(&[writable(extra)])),
        MplCoreError::IncorrectAccount,
    );

    // Non-writable collection.
    assert_program_err(
        &f.run(&base(&[readonly(collection)])),
        ProgramError::InvalidAccountData,
    );

    // An asset where a collection is expected.
    let asset = put(&f, AssetSpec::new(payer).build());
    assert_core_err(
        &f.run(&ix::remove_collections_from_group_v1(
            group,
            payer,
            None,
            vec![asset],
            &[writable(asset)],
        )),
        MplCoreError::DeserializationError,
    );

    // A collection the signer has no authority over.
    let foreign_collection = Pubkey::new_unique();
    f.store(
        foreign_collection,
        CollectionSpec::new(stranger)
            .plugin(update_delegate_for(stranger), Authority::UpdateAuthority)
            .build(),
    );
    let group_with_foreign = put_group(
        &f,
        GroupSpec::new(payer).collections(vec![foreign_collection]),
    );
    assert_core_err(
        &f.run(&ix::remove_collections_from_group_v1(
            group_with_foreign,
            payer,
            None,
            vec![foreign_collection],
            &[writable(foreign_collection)],
        )),
        MplCoreError::InvalidAuthority,
    );

    // A collection that is not a member.
    let empty_group = put_group(&f, GroupSpec::new(payer));
    assert_core_err(
        &f.run(&ix::remove_collections_from_group_v1(
            empty_group,
            payer,
            None,
            vec![collection],
            &[writable(collection)],
        )),
        MplCoreError::IncorrectAccount,
    );
}

// ===========================================================================
// AddGroupsToGroupV1 / RemoveGroupsFromGroupV1
// ===========================================================================

/// JS: groupComplexRelations.test.ts :: it keeps parentGroups in sync with groups when adding and removing child groups
#[test]
fn add_and_remove_groups_keep_both_sides_in_sync() {
    let (f, payer) = world();
    let parent = put_group(&f, GroupSpec::new(payer));
    let child1 = put_group(&f, GroupSpec::new(payer));
    // A child that is already at seven of the eight allowed parents.
    let existing_parents = random_keys(MAX_GROUP_NESTING_DEPTH - 1);
    let child2 = put_group(
        &f,
        GroupSpec::new(payer).parent_groups(existing_parents.clone()),
    );

    f.run_ok(&ix::add_groups_to_group_v1(
        parent,
        payer,
        None,
        vec![child1, child2],
        &[writable(child1), writable(child2)],
    ));

    assert_eq!(group_of(&f, &parent).groups, vec![child1, child2]);
    assert_eq!(group_of(&f, &child1).parent_groups, vec![parent]);
    let mut expected = existing_parents.clone();
    expected.push(parent);
    assert_eq!(group_of(&f, &child2).parent_groups, expected);
    assert_group_sized(&f, &parent);
    assert_group_sized(&f, &child1);
    assert_group_sized(&f, &child2);

    // Removing keeps both sides in sync too, and both accounts shrink.
    let parent_len = f.account(&parent).data.len();
    let child_len = f.account(&child1).data.len();
    f.run_ok(&ix::remove_groups_from_group_v1(
        parent,
        payer,
        None,
        vec![child1],
        &[writable(child1)],
    ));

    assert_eq!(group_of(&f, &parent).groups, vec![child2]);
    assert!(group_of(&f, &child1).parent_groups.is_empty());
    assert_eq!(f.account(&parent).data.len(), parent_len - 32);
    assert_eq!(f.account(&child1).data.len(), child_len - 32);
    assert_group_sized(&f, &parent);
    assert_group_sized(&f, &child1);
}

/// JS: groupComplexRelations.test.ts :: it rejects addGroupsToGroup when signer is not parent group authority
#[test]
fn add_groups_rejections() {
    let (f, payer) = world();
    let stranger = f.fund(ACCOUNT_LAMPORTS);
    let parent = put_group(&f, GroupSpec::new(payer));
    let child = put_group(&f, GroupSpec::new(payer));

    let base = |remaining: &[AccountMeta]| {
        ix::add_groups_to_group_v1(parent, payer, None, vec![child], remaining)
    };

    let mut ix = base(&[writable(child)]);
    break_system_program(&f, &mut ix, 3);
    assert_core_err(&f.run(&ix), MplCoreError::InvalidSystemProgram);

    let mut ix = base(&[writable(child)]);
    make_readonly(&mut ix, 0);
    assert_program_err(&f.run(&ix), ProgramError::InvalidAccountData);

    // The account count must match the argument count exactly.
    assert_program_err(&f.run(&base(&[])), ProgramError::NotEnoughAccountKeys);

    let foreign_parent = put_group(&f, GroupSpec::new(stranger));
    assert_core_err(
        &f.run(&ix::add_groups_to_group_v1(
            foreign_parent,
            payer,
            None,
            vec![child],
            &[writable(child)],
        )),
        MplCoreError::InvalidAuthority,
    );

    // Key mismatch.
    let other = put_group(&f, GroupSpec::new(payer));
    assert_core_err(
        &f.run(&base(&[writable(other)])),
        MplCoreError::IncorrectAccount,
    );

    // Non-writable child.
    assert_program_err(
        &f.run(&base(&[readonly(child)])),
        ProgramError::InvalidAccountData,
    );

    // JS: groupComplexRelations.test.ts :: it rejects adding a parent group as its own child group
    assert_core_err(
        &f.run(&ix::add_groups_to_group_v1(
            parent,
            payer,
            None,
            vec![parent],
            &[writable(parent)],
        )),
        MplCoreError::IncorrectAccount,
    );

    // An asset where a group is expected.
    let asset = put(&f, AssetSpec::new(payer).build());
    assert_core_err(
        &f.run(&ix::add_groups_to_group_v1(
            parent,
            payer,
            None,
            vec![asset],
            &[writable(asset)],
        )),
        MplCoreError::DeserializationError,
    );

    // A child with a different update authority: groups have no delegate
    // mechanism, so this can never be worked around.
    let foreign_child = put_group(&f, GroupSpec::new(stranger));
    assert_core_err(
        &f.run(&ix::add_groups_to_group_v1(
            parent,
            payer,
            None,
            vec![foreign_child],
            &[writable(foreign_child)],
        )),
        MplCoreError::InvalidAuthority,
    );

    // JS: groupComplexRelations.test.ts :: it rejects duplicate child group in addGroupsToGroup
    assert_core_err(
        &f.run(&ix::add_groups_to_group_v1(
            parent,
            payer,
            None,
            vec![child, child],
            &[writable(child), writable(child)],
        )),
        MplCoreError::DuplicateEntry,
    );

    // A parent whose child vector is already full.
    let saturated_parent = put_group(
        &f,
        GroupSpec::new(payer).groups(random_keys(MAX_GROUP_VECTOR_SIZE)),
    );
    assert_core_err(
        &f.run(&ix::add_groups_to_group_v1(
            saturated_parent,
            payer,
            None,
            vec![child],
            &[writable(child)],
        )),
        MplCoreError::GroupVectorFull,
    );

    // A child already at the maximum number of parents. The JS suite only
    // covers the `CreateGroupV1` variant of this check.
    let saturated_child = put_group(
        &f,
        GroupSpec::new(payer).parent_groups(random_keys(MAX_GROUP_NESTING_DEPTH)),
    );
    assert_core_err(
        &f.run(&ix::add_groups_to_group_v1(
            parent,
            payer,
            None,
            vec![saturated_child],
            &[writable(saturated_child)],
        )),
        MplCoreError::GroupNestingDepthExceeded,
    );
}

/// The child's `parent_groups` already naming the parent is a state the
/// program cannot produce (every writer updates both sides), so the skip at
/// `add_groups_to_group.rs:122` needs a fabricated account. The child is left
/// untouched while the parent still gains the link.
#[test]
fn add_groups_skips_saving_a_child_that_already_lists_the_parent() {
    let (f, payer) = world();
    let parent = Pubkey::new_unique();
    let child = put_group(&f, GroupSpec::new(payer).parent_groups(vec![parent]));
    f.store(parent, GroupSpec::new(payer).build());
    let child_before = f.account(&child);

    f.run_ok(&ix::add_groups_to_group_v1(
        parent,
        payer,
        None,
        vec![child],
        &[writable(child)],
    ));

    assert_eq!(group_of(&f, &parent).groups, vec![child]);
    assert_eq!(
        f.account(&child).data,
        child_before.data,
        "the child was already linked, so it must not be rewritten"
    );
}

/// JS: groupComplexRelations.test.ts :: it rejects removeGroupsFromGroup when signer is not parent group authority
#[test]
fn remove_groups_rejections() {
    let (f, payer) = world();
    let stranger = f.fund(ACCOUNT_LAMPORTS);
    let parent = Pubkey::new_unique();
    let child = put_group(&f, GroupSpec::new(payer).parent_groups(vec![parent]));
    f.store(parent, GroupSpec::new(payer).groups(vec![child]).build());

    let base = |remaining: &[AccountMeta]| {
        ix::remove_groups_from_group_v1(parent, payer, None, vec![child], remaining)
    };

    let mut ix = base(&[writable(child)]);
    break_system_program(&f, &mut ix, 3);
    assert_core_err(&f.run(&ix), MplCoreError::InvalidSystemProgram);

    let mut ix = base(&[writable(child)]);
    make_readonly(&mut ix, 0);
    assert_program_err(&f.run(&ix), ProgramError::InvalidAccountData);

    assert_program_err(&f.run(&base(&[])), ProgramError::NotEnoughAccountKeys);

    let foreign_parent = put_group(&f, GroupSpec::new(stranger).groups(vec![child]));
    assert_core_err(
        &f.run(&ix::remove_groups_from_group_v1(
            foreign_parent,
            payer,
            None,
            vec![child],
            &[writable(child)],
        )),
        MplCoreError::InvalidAuthority,
    );

    let other = put_group(&f, GroupSpec::new(payer));
    assert_core_err(
        &f.run(&base(&[writable(other)])),
        MplCoreError::IncorrectAccount,
    );

    assert_program_err(
        &f.run(&base(&[readonly(child)])),
        ProgramError::InvalidAccountData,
    );

    let asset = put(&f, AssetSpec::new(payer).build());
    assert_core_err(
        &f.run(&ix::remove_groups_from_group_v1(
            parent,
            payer,
            None,
            vec![asset],
            &[writable(asset)],
        )),
        MplCoreError::DeserializationError,
    );

    let foreign_child = put_group(&f, GroupSpec::new(stranger));
    let parent_of_foreign = put_group(&f, GroupSpec::new(payer).groups(vec![foreign_child]));
    assert_core_err(
        &f.run(&ix::remove_groups_from_group_v1(
            parent_of_foreign,
            payer,
            None,
            vec![foreign_child],
            &[writable(foreign_child)],
        )),
        MplCoreError::InvalidAuthority,
    );

    // JS: groupComplexRelations.test.ts :: it rejects removing a child group that is not linked
    let unrelated_parent = put_group(&f, GroupSpec::new(payer));
    assert_core_err(
        &f.run(&ix::remove_groups_from_group_v1(
            unrelated_parent,
            payer,
            None,
            vec![child],
            &[writable(child)],
        )),
        MplCoreError::IncorrectAccount,
    );
}

/// `InconsistentGroupRelationship` is unreachable through the program's own
/// writers (roadmap section 10, note 3): it needs a parent that lists the
/// child while the child does not list the parent.
#[test]
fn remove_groups_rejects_inconsistent_bidirectional_state() {
    let (f, payer) = world();
    let child = put_group(&f, GroupSpec::new(payer));
    let parent = put_group(&f, GroupSpec::new(payer).groups(vec![child]));

    assert_core_err(
        &f.run(&ix::remove_groups_from_group_v1(
            parent,
            payer,
            None,
            vec![child],
            &[writable(child)],
        )),
        MplCoreError::InconsistentGroupRelationship,
    );
}

/// `AddGroupsToGroupV1` only rejects self-links, so a two-group cycle is
/// accepted and both groups end up as each other's parent and child
/// (roadmap section 10, note 2). Pinned so a future cycle check is a
/// deliberate change.
#[test]
fn add_groups_accepts_a_two_group_cycle() {
    let (f, payer) = world();
    let a = put_group(&f, GroupSpec::new(payer));
    let b = put_group(&f, GroupSpec::new(payer));

    f.run_ok(&ix::add_groups_to_group_v1(
        a,
        payer,
        None,
        vec![b],
        &[writable(b)],
    ));
    f.run_ok(&ix::add_groups_to_group_v1(
        b,
        payer,
        None,
        vec![a],
        &[writable(a)],
    ));

    assert_eq!(group_of(&f, &a).groups, vec![b]);
    assert_eq!(group_of(&f, &a).parent_groups, vec![b]);
    assert_eq!(group_of(&f, &b).groups, vec![a]);
    assert_eq!(group_of(&f, &b).parent_groups, vec![a]);
}

// ===========================================================================
// The Groups plugin on the member: layout, and the states only fabrication
// can produce
// ===========================================================================

/// The `Groups` plugin grows and shrinks by exactly one pubkey, and when it is
/// not the last plugin in the account the trailing plugin has to be moved and
/// its registry offset bumped (`save_updated_groups_plugin`'s two `sol_memmove`
/// branches plus `PluginRegistryV1::bump_offsets`).
#[test]
fn member_in_two_groups_grows_and_shrinks_around_a_trailing_plugin() {
    let (f, payer) = world();
    let asset = Pubkey::new_unique();
    let g1 = put_group(&f, GroupSpec::new(payer).assets(vec![asset]));
    let g2 = put_group(&f, GroupSpec::new(payer));
    f.store(
        asset,
        AssetSpec::new(payer)
            // `Groups` first, so the FreezeDelegate bytes after it have to
            // move both ways.
            .plugin(groups_plugin_with(vec![g1]), Authority::UpdateAuthority)
            .plugin(freeze_delegate(), Authority::Owner)
            .build(),
    );
    let freeze_offset = |account: &Account| {
        parse_asset(&account.data)
            .1
            .plugins
            .iter()
            .find(|(record, _)| record.plugin_type == PluginType::FreezeDelegate)
            .expect("the asset carries a FreezeDelegate")
            .0
            .offset
    };
    let offset_before = freeze_offset(&f.account(&asset));
    let len_before = f.account(&asset).data.len();

    // Grow: realloc first, then memmove the trailing plugin up by 32 bytes.
    f.run_ok(&ix::add_assets_to_group_v1(
        g2,
        payer,
        None,
        &[writable(asset)],
    ));

    let grown = f.account(&asset);
    assert_member_groups(&grown, &[g1, g2]);
    assert_eq!(grown.data.len(), len_before + 32);
    assert_eq!(
        freeze_offset(&grown),
        offset_before + 32,
        "the trailing plugin's registry offset must be bumped by the growth"
    );
    assert_eq!(
        read_plugin(&grown, PluginType::FreezeDelegate),
        Some((Authority::Owner, freeze_delegate())),
        "the trailing plugin bytes must survive the memmove"
    );

    // Shrink: memmove down first, then realloc.
    f.run_ok(&ix::remove_assets_from_group_v1(
        g1,
        payer,
        None,
        vec![asset],
        &[writable(asset)],
    ));

    let shrunk = f.account(&asset);
    assert_member_groups(&shrunk, &[g2]);
    assert_eq!(shrunk.data.len(), len_before);
    assert_eq!(freeze_offset(&shrunk), offset_before);
    assert_eq!(
        read_plugin(&shrunk, PluginType::FreezeDelegate),
        Some((Authority::Owner, freeze_delegate())),
    );
}

/// The same for a collection, whose `Groups` plugin is manipulated through the
/// `CollectionV1` monomorphization of the helpers.
#[test]
fn collection_in_two_groups_grows_and_shrinks_around_a_trailing_plugin() {
    let (f, payer) = world();
    let collection = Pubkey::new_unique();
    let g1 = put_group(&f, GroupSpec::new(payer).collections(vec![collection]));
    let g2 = put_group(&f, GroupSpec::new(payer));
    f.store(
        collection,
        CollectionSpec::new(payer)
            .plugin(groups_plugin_with(vec![g1]), Authority::UpdateAuthority)
            .plugin(royalties(), Authority::UpdateAuthority)
            .build(),
    );
    let len_before = f.account(&collection).data.len();

    f.run_ok(&ix::add_collections_to_group_v1(
        g2,
        payer,
        None,
        &[writable(collection)],
    ));
    let grown = f.account(&collection);
    assert_member_groups(&grown, &[g1, g2]);
    assert_eq!(grown.data.len(), len_before + 32);
    assert_eq!(
        read_plugin(&grown, PluginType::Royalties),
        Some((Authority::UpdateAuthority, royalties()))
    );

    f.run_ok(&ix::remove_collections_from_group_v1(
        g1,
        payer,
        None,
        vec![collection],
        &[writable(collection)],
    ));
    let shrunk = f.account(&collection);
    assert_member_groups(&shrunk, &[g2]);
    assert_eq!(shrunk.data.len(), len_before);
    assert_eq!(
        read_plugin(&shrunk, PluginType::Royalties),
        Some((Authority::UpdateAuthority, royalties()))
    );
}

/// The member's plugin already naming the group is a state the program cannot
/// produce; `process_*_groups_plugin_add` returns early and leaves the plugin
/// untouched while the group still records the membership.
#[test]
fn groups_plugin_add_skips_a_member_that_already_lists_the_group() {
    let (f, payer) = world();
    let group = put_group(&f, GroupSpec::new(payer));
    let asset = put(
        &f,
        AssetSpec::new(payer)
            .plugin(groups_plugin_with(vec![group]), Authority::UpdateAuthority)
            .build(),
    );
    let before = f.account(&asset);

    f.run_ok(&ix::add_assets_to_group_v1(
        group,
        payer,
        None,
        &[writable(asset)],
    ));

    assert_eq!(group_of(&f, &group).assets, vec![asset]);
    assert_eq!(
        f.account(&asset).data,
        before.data,
        "the member's plugin was already correct and must not be rewritten"
    );
}

/// A registry record that claims to be `Groups` while the bytes it points at
/// are a different plugin. No instruction can write this, so it is built by
/// retyping the record of a well-formed account.
#[test]
fn groups_plugin_rejects_a_registry_record_of_the_wrong_type() {
    let (f, payer) = world();
    let group = put_group(&f, GroupSpec::new(payer));

    let asset = put(
        &f,
        retype_registry_record(
            &AssetSpec::new(payer)
                .plugin(freeze_delegate(), Authority::Owner)
                .build(),
            PluginType::FreezeDelegate,
            PluginType::Groups,
        ),
    );
    assert_core_err(
        &f.run(&ix::add_assets_to_group_v1(
            group,
            payer,
            None,
            &[writable(asset)],
        )),
        MplCoreError::InvalidPlugin,
    );

    // The remove path has the same guard.
    let member = Pubkey::new_unique();
    let group_with_member = put_group(&f, GroupSpec::new(payer).assets(vec![member]));
    f.store(
        member,
        retype_registry_record(
            &AssetSpec::new(payer)
                .plugin(freeze_delegate(), Authority::Owner)
                .build(),
            PluginType::FreezeDelegate,
            PluginType::Groups,
        ),
    );
    assert_core_err(
        &f.run(&ix::remove_assets_from_group_v1(
            group_with_member,
            payer,
            None,
            vec![member],
            &[writable(member)],
        )),
        MplCoreError::InvalidPlugin,
    );

    // ... and so does the collection variant.
    let collection = Pubkey::new_unique();
    let group_with_collection = put_group(&f, GroupSpec::new(payer).collections(vec![collection]));
    f.store(
        collection,
        retype_registry_record(
            &CollectionSpec::new(payer)
                .plugin(royalties(), Authority::UpdateAuthority)
                .build(),
            PluginType::Royalties,
            PluginType::Groups,
        ),
    );
    assert_core_err(
        &f.run(&ix::remove_collections_from_group_v1(
            group_with_collection,
            payer,
            None,
            vec![collection],
            &[writable(collection)],
        )),
        MplCoreError::InvalidPlugin,
    );
}

/// Two more inconsistent states on the remove path: the member's plugin lists
/// a different group, and the member has no plugin metadata at all. Both
/// succeed, and the second one has `create_meta_idempotent` allocate an empty
/// header and registry on the member before discovering there is nothing to
/// remove (roadmap section 10, note 4).
#[test]
fn remove_assets_tolerates_inconsistent_member_plugin_states() {
    let (f, payer) = world();
    let other_group = Pubkey::new_unique();

    // (a) The plugin lists a different group.
    let asset = Pubkey::new_unique();
    let group = put_group(&f, GroupSpec::new(payer).assets(vec![asset]));
    f.store(
        asset,
        AssetSpec::new(payer)
            .plugin(
                groups_plugin_with(vec![other_group]),
                Authority::UpdateAuthority,
            )
            .build(),
    );
    let plugin_before = f.account(&asset).data.clone();

    f.run_ok(&ix::remove_assets_from_group_v1(
        group,
        payer,
        None,
        vec![asset],
        &[writable(asset)],
    ));
    assert!(group_of(&f, &group).assets.is_empty());
    assert_eq!(
        f.account(&asset).data,
        plugin_before,
        "the member's plugin does not list this group, so it stays as it was"
    );

    // (b) The member has no plugins at all.
    let bare = Pubkey::new_unique();
    let group = put_group(&f, GroupSpec::new(payer).assets(vec![bare]));
    f.store(bare, AssetSpec::new(payer).build());
    let len_before = f.account(&bare).data.len();

    f.run_ok(&ix::remove_assets_from_group_v1(
        group,
        payer,
        None,
        vec![bare],
        &[writable(bare)],
    ));
    assert!(group_of(&f, &group).assets.is_empty());
    let after = f.account(&bare);
    assert!(
        after.data.len() > len_before,
        "an empty plugin header and registry are created even though there was nothing to remove"
    );
    assert!(
        read_plugin(&after, PluginType::Groups).is_none(),
        "no Groups plugin should have been created"
    );
    assert_registry_consistent(&after);
}

// ===========================================================================
// The Groups plugin's burn validation
//
// `Groups::validate_burn` is only reachable through `BurnV1` /
// `BurnCollectionV1`; these are the only tests that cover
// `plugins/internal/authority_managed/groups.rs`.
// ===========================================================================

/// JS: burn.test.ts :: it rejects burning an asset that belongs to a group
#[test]
fn groups_plugin_rejects_burning_a_member_asset() {
    let (f, payer) = world();
    let group = put_group(&f, GroupSpec::new(payer));
    let asset = put(
        &f,
        AssetSpec::new(payer)
            .plugin(groups_plugin_with(vec![group]), Authority::UpdateAuthority)
            .build(),
    );

    // `reject!()` surfaces as `InvalidAuthority`, not a group-specific error.
    assert_core_err(
        &f.run(&ix::burn_v1(asset, None, payer, None, None, None)),
        MplCoreError::InvalidAuthority,
    );
    assert_eq!(read_asset(&f.account(&asset)).owner, payer);
}

/// JS: burn.test.ts :: it allows burning an asset after removing it from all groups
#[test]
fn groups_plugin_allows_burning_an_asset_with_an_empty_group_list() {
    let (f, payer) = world();
    let asset = put(
        &f,
        AssetSpec::new(payer)
            .plugin(groups_plugin_with(vec![]), Authority::UpdateAuthority)
            .build(),
    );

    f.run_ok(&ix::burn_v1(asset, None, payer, None, None, None));
    f.assert_burned(&asset);
}

/// JS: burnCollection.test.ts :: it rejects burning a collection that belongs to a group
#[test]
fn groups_plugin_rejects_burning_a_member_collection() {
    let (f, payer) = world();
    let group = put_group(&f, GroupSpec::new(payer));
    let collection = put(
        &f,
        CollectionSpec::new(payer)
            .plugin(groups_plugin_with(vec![group]), Authority::UpdateAuthority)
            .build(),
    );

    assert_core_err(
        &f.run(&ix::burn_collection_v1(collection, payer, None, None, None)),
        MplCoreError::InvalidAuthority,
    );
}

/// JS: burnCollection.test.ts :: it allows burning a collection after removing it from all groups
#[test]
fn groups_plugin_allows_burning_a_collection_with_an_empty_group_list() {
    let (f, payer) = world();
    let collection = put(
        &f,
        CollectionSpec::new(payer)
            .plugin(groups_plugin_with(vec![]), Authority::UpdateAuthority)
            .build(),
    );

    f.run_ok(&ix::burn_collection_v1(collection, payer, None, None, None));
    f.assert_burned(&collection);
}

/// JS: burn.test.ts :: it allows burning an asset in a collection that belongs to a group
#[test]
fn groups_plugin_allows_burning_an_asset_inside_a_member_collection() {
    let (f, payer) = world();
    let group = put_group(&f, GroupSpec::new(payer));
    let collection = Pubkey::new_unique();
    let asset = Pubkey::new_unique();
    f.store(
        collection,
        CollectionSpec::new(payer)
            .sizes(1, 1)
            .plugin(groups_plugin_with(vec![group]), Authority::UpdateAuthority)
            .build(),
    );
    f.store(
        asset,
        AssetSpec::new(payer)
            .update_authority(UpdateAuthority::Collection(collection))
            .build(),
    );

    // The plugin is inherited from the collection but the burn target is the
    // asset, so `Groups::validate_burn` abstains.
    f.run_ok(&ix::burn_v1(
        asset,
        Some(collection),
        payer,
        None,
        None,
        None,
    ));
    f.assert_burned(&asset);
    assert_eq!(
        read_collection(&f.account(&collection)).current_size,
        0,
        "burning a member asset decrements the collection"
    );
}

/// The collection mirror of the inconsistent-member cases: the
/// `CollectionV1` monomorphizations of `process_collection_groups_plugin_add`'s
/// already-contains early return and of the remove path's "plugin lacks this
/// group" and "no plugin at all" branches.
#[test]
fn collection_membership_tolerates_inconsistent_member_plugin_states() {
    let (f, payer) = world();
    let other_group = Pubkey::new_unique();

    // Add: the collection's plugin already lists the group, so it is left
    // exactly as it was while the group records the membership.
    let group = put_group(&f, GroupSpec::new(payer));
    let collection = put(
        &f,
        CollectionSpec::new(payer)
            .plugin(groups_plugin_with(vec![group]), Authority::UpdateAuthority)
            .build(),
    );
    let before = f.account(&collection);
    f.run_ok(&ix::add_collections_to_group_v1(
        group,
        payer,
        None,
        &[writable(collection)],
    ));
    assert_eq!(group_of(&f, &group).collections, vec![collection]);
    assert_eq!(f.account(&collection).data, before.data);

    // Remove: the plugin lists a different group.
    let other = Pubkey::new_unique();
    let group = put_group(&f, GroupSpec::new(payer).collections(vec![other]));
    f.store(
        other,
        CollectionSpec::new(payer)
            .plugin(
                groups_plugin_with(vec![other_group]),
                Authority::UpdateAuthority,
            )
            .build(),
    );
    let before = f.account(&other);
    f.run_ok(&ix::remove_collections_from_group_v1(
        group,
        payer,
        None,
        vec![other],
        &[writable(other)],
    ));
    assert!(group_of(&f, &group).collections.is_empty());
    assert_eq!(f.account(&other).data, before.data);

    // Remove: the collection has no plugin metadata at all, which
    // `create_meta_idempotent` creates on the way to finding nothing.
    let bare = Pubkey::new_unique();
    let group = put_group(&f, GroupSpec::new(payer).collections(vec![bare]));
    f.store(bare, CollectionSpec::new(payer).build());
    let len_before = f.account(&bare).data.len();
    f.run_ok(&ix::remove_collections_from_group_v1(
        group,
        payer,
        None,
        vec![bare],
        &[writable(bare)],
    ));
    assert!(group_of(&f, &group).collections.is_empty());
    let after = f.account(&bare);
    assert!(after.data.len() > len_before);
    assert!(read_plugin(&after, PluginType::Groups).is_none());
    assert_registry_consistent(&after);
}

/// `AddCollectionsToGroupV1` with no remaining accounts is the same silent
/// no-op as the asset variant.
#[test]
fn add_collections_zero_remaining_accounts_is_a_silent_no_op() {
    let (f, payer) = world();
    let group = put_group(&f, GroupSpec::new(payer));
    let before = f.account(&group);

    f.run_ok(&ix::add_collections_to_group_v1(group, payer, None, &[]));

    assert_eq!(f.account(&group).data, before.data);
}

/// `CloseGroupV1` credits the *payer*, not the authority, even when they
/// differ (`close_group.rs:57`). Both have to sign, so this is by design, but
/// it is the only lamport destination the instruction has.
#[test]
fn close_group_refunds_the_payer_not_the_authority() {
    let (f, payer) = world();
    let authority = f.fund(ACCOUNT_LAMPORTS);
    let group = put_group(&f, GroupSpec::new(authority));

    let data_len = f.account(&group).data.len();
    let refund = rent_exempt_balance(data_len) - rent_exempt_balance(1);
    let payer_before = f.lamports(&payer);
    let authority_before = f.lamports(&authority);

    f.run_ok(&ix::close_group_v1(group, payer, Some(authority)));

    assert_eq!(f.lamports(&payer), payer_before + refund);
    assert_eq!(
        f.lamports(&authority),
        authority_before,
        "the authority receives nothing"
    );
}
