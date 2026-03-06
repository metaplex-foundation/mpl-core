// Account Ownership Tests for mpl-core
//
// These tests use Mollusk to verify that mpl-core instructions properly reject
// accounts owned by other programs, even when those accounts contain valid
// mpl-core discriminator bytes and correctly serialized data.
//
// The tests prove that the existing defense layers (discriminator checks,
// authority validation, and Solana's runtime write protection) already prevent
// exploitation of fake accounts. The explicit owner check in SolanaAccount::load
// is a defense-in-depth measure that provides clearer error messages and
// earlier rejection.

#[allow(deprecated)]
use {
    borsh::BorshSerialize,
    mollusk_svm::Mollusk,
    mpl_core_program::{
        plugins::{
            FreezeDelegate, PermanentBurnDelegate, PermanentFreezeDelegate,
            PermanentTransferDelegate, Plugin, PluginHeaderV1, PluginRegistryV1, PluginType,
            RegistryRecord,
        },
        state::{AssetV1, Authority, CollectionV1, DataBlob, Key, UpdateAuthority},
        ID as MPL_CORE_ID,
    },
    solana_sdk::{
        account::Account,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        system_program,
    },
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// A fake program ID representing an attacker's program.
const FAKE_PROGRAM_ID: Pubkey = Pubkey::new_from_array([0xAA; 32]);

/// Minimum lamports for accounts to be rent-exempt-ish in tests.
const ACCOUNT_LAMPORTS: u64 = 1_000_000_000;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Creates a Mollusk instance for the mpl-core program.
fn core_mollusk() -> Mollusk {
    Mollusk::new(&MPL_CORE_ID, "mpl_core_program")
}

/// Serializes an AssetV1 into an Account owned by the given program.
fn fake_asset_account(owner_pubkey: &Pubkey, program_owner: &Pubkey) -> Account {
    let asset = AssetV1::new(
        *owner_pubkey,
        UpdateAuthority::Address(*owner_pubkey),
        "Fake Asset".to_string(),
        "https://example.com/fake".to_string(),
    );
    let data = asset.try_to_vec().unwrap();
    Account {
        lamports: ACCOUNT_LAMPORTS,
        data,
        owner: *program_owner,
        executable: false,
        rent_epoch: 0,
    }
}

/// Serializes a CollectionV1 into an Account owned by the given program.
fn fake_collection_account(update_authority: &Pubkey, program_owner: &Pubkey) -> Account {
    let collection = CollectionV1::new(
        *update_authority,
        "Fake".to_string(),
        "https://example.com".to_string(),
        0,
        0,
    );
    let data = collection.try_to_vec().unwrap();
    Account {
        lamports: ACCOUNT_LAMPORTS,
        data,
        owner: *program_owner,
        executable: false,
        rent_epoch: 0,
    }
}

/// Creates a valid asset account owned by the mpl-core program.
fn valid_asset_account(owner_pubkey: &Pubkey) -> Account {
    fake_asset_account(owner_pubkey, &MPL_CORE_ID)
}

/// Creates a payer account with lamports.
fn payer_account() -> Account {
    Account {
        lamports: ACCOUNT_LAMPORTS,
        data: vec![],
        owner: system_program::ID,
        executable: false,
        rent_epoch: 0,
    }
}

/// Builds a TransferV1 instruction with the collection account set to a real pubkey.
fn transfer_v1_with_collection_instruction(
    asset: Pubkey,
    collection: Pubkey,
    payer: Pubkey,
    new_owner: Pubkey,
) -> Instruction {
    let data = vec![14u8, 0u8]; // TransferV1 discriminator + None compression_proof
    Instruction::new_with_bytes(
        MPL_CORE_ID,
        &data,
        vec![
            AccountMeta::new(asset, false),               // 0: asset (writable)
            AccountMeta::new_readonly(collection, false), // 1: collection
            AccountMeta::new(payer, true),                // 2: payer (writable, signer)
            AccountMeta::new_readonly(payer, true),       // 3: authority (signer)
            AccountMeta::new_readonly(new_owner, false),  // 4: new_owner
            AccountMeta::new_readonly(system_program::ID, false), // 5: system_program
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 6: log_wrapper (optional)
        ],
    )
}

/// Constructs an Account with valid AssetV1 data + serialized plugins, owned by `program_owner`.
///
/// The account data layout is:
///   [AssetV1 data][PluginHeaderV1][Plugin data...][PluginRegistryV1]
fn build_asset_with_plugins(
    owner: &Pubkey,
    program_owner: &Pubkey,
    plugins: &[(Plugin, Authority)],
) -> Account {
    let asset = AssetV1::new(
        *owner,
        UpdateAuthority::Address(*owner),
        "Fake Asset".to_string(),
        "https://example.com/fake".to_string(),
    );
    let asset_data = asset.try_to_vec().unwrap();
    let asset_len = asset.len();

    // Build plugin data and registry records.
    let header_offset = asset_len;
    let plugins_start = header_offset + 9; // PluginHeaderV1 is 9 bytes (Key + usize)

    let mut plugin_data = Vec::new();
    let mut registry_records = Vec::new();

    for (plugin, authority) in plugins {
        let offset = plugins_start + plugin_data.len();
        let plugin_bytes = plugin.try_to_vec().unwrap();
        plugin_data.extend_from_slice(&plugin_bytes);

        let plugin_type = match plugin {
            Plugin::FreezeDelegate(_) => PluginType::FreezeDelegate,
            Plugin::PermanentFreezeDelegate(_) => PluginType::PermanentFreezeDelegate,
            Plugin::PermanentTransferDelegate(_) => PluginType::PermanentTransferDelegate,
            Plugin::PermanentBurnDelegate(_) => PluginType::PermanentBurnDelegate,
            _ => panic!("Unsupported plugin type in test helper"),
        };

        registry_records.push(RegistryRecord {
            plugin_type,
            authority: *authority,
            offset,
        });
    }

    let registry_offset = plugins_start + plugin_data.len();

    let header = PluginHeaderV1 {
        key: Key::PluginHeaderV1,
        plugin_registry_offset: registry_offset,
    };

    let registry = PluginRegistryV1 {
        key: Key::PluginRegistryV1,
        registry: registry_records,
        external_registry: vec![],
    };

    let header_bytes = header.try_to_vec().unwrap();
    let registry_bytes = registry.try_to_vec().unwrap();

    let mut data = Vec::new();
    data.extend_from_slice(&asset_data);
    data.extend_from_slice(&header_bytes);
    data.extend_from_slice(&plugin_data);
    data.extend_from_slice(&registry_bytes);

    Account {
        lamports: ACCOUNT_LAMPORTS,
        data,
        owner: *program_owner,
        executable: false,
        rent_epoch: 0,
    }
}

/// Constructs an Account with valid CollectionV1 data + serialized plugins, owned by `program_owner`.
fn build_collection_with_plugins(
    update_authority: &Pubkey,
    program_owner: &Pubkey,
    plugins: &[(Plugin, Authority)],
) -> Account {
    let collection = CollectionV1::new(
        *update_authority,
        "Fake Collection".to_string(),
        "https://example.com".to_string(),
        0,
        0,
    );
    let collection_data = collection.try_to_vec().unwrap();
    let collection_len = collection.len();

    let header_offset = collection_len;
    let plugins_start = header_offset + 9;

    let mut plugin_data = Vec::new();
    let mut registry_records = Vec::new();

    for (plugin, authority) in plugins {
        let offset = plugins_start + plugin_data.len();
        let plugin_bytes = plugin.try_to_vec().unwrap();
        plugin_data.extend_from_slice(&plugin_bytes);

        let plugin_type = match plugin {
            Plugin::FreezeDelegate(_) => PluginType::FreezeDelegate,
            Plugin::PermanentFreezeDelegate(_) => PluginType::PermanentFreezeDelegate,
            Plugin::PermanentTransferDelegate(_) => PluginType::PermanentTransferDelegate,
            Plugin::PermanentBurnDelegate(_) => PluginType::PermanentBurnDelegate,
            _ => panic!("Unsupported plugin type in test helper"),
        };

        registry_records.push(RegistryRecord {
            plugin_type,
            authority: *authority,
            offset,
        });
    }

    let registry_offset = plugins_start + plugin_data.len();

    let header = PluginHeaderV1 {
        key: Key::PluginHeaderV1,
        plugin_registry_offset: registry_offset,
    };

    let registry = PluginRegistryV1 {
        key: Key::PluginRegistryV1,
        registry: registry_records,
        external_registry: vec![],
    };

    let header_bytes = header.try_to_vec().unwrap();
    let registry_bytes = registry.try_to_vec().unwrap();

    let mut data = Vec::new();
    data.extend_from_slice(&collection_data);
    data.extend_from_slice(&header_bytes);
    data.extend_from_slice(&plugin_data);
    data.extend_from_slice(&registry_bytes);

    Account {
        lamports: ACCOUNT_LAMPORTS,
        data,
        owner: *program_owner,
        executable: false,
        rent_epoch: 0,
    }
}

/// Builds a TransferV1 instruction.
/// Discriminator 14, with Option::None compression_proof.
fn transfer_v1_instruction(asset: Pubkey, payer: Pubkey, new_owner: Pubkey) -> Instruction {
    let data = vec![14u8, 0u8]; // TransferV1 discriminator + None compression_proof
    Instruction::new_with_bytes(
        MPL_CORE_ID,
        &data,
        vec![
            AccountMeta::new(asset, false),                // 0: asset (writable)
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 1: collection (optional, use program ID)
            AccountMeta::new(payer, true),                 // 2: payer (writable, signer)
            AccountMeta::new_readonly(payer, true),        // 3: authority (signer)
            AccountMeta::new_readonly(new_owner, false),   // 4: new_owner
            AccountMeta::new_readonly(system_program::ID, false), // 5: system_program
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 6: log_wrapper (optional)
        ],
    )
}

/// Builds a BurnV1 instruction.
/// Discriminator 12, with Option::None compression_proof.
fn burn_v1_instruction(asset: Pubkey, payer: Pubkey) -> Instruction {
    let data = vec![12u8, 0u8]; // BurnV1 discriminator + None compression_proof
    Instruction::new_with_bytes(
        MPL_CORE_ID,
        &data,
        vec![
            AccountMeta::new(asset, false),                // 0: asset (writable)
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 1: collection (optional)
            AccountMeta::new(payer, true),                 // 2: payer (writable, signer)
            AccountMeta::new_readonly(payer, true),        // 3: authority (signer)
            AccountMeta::new_readonly(system_program::ID, false), // 4: system_program (optional)
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 5: log_wrapper (optional)
        ],
    )
}

/// Builds an UpdateV1 instruction that changes the name to "X".
/// Discriminator 15, with Some(name) and None for the rest.
fn update_v1_instruction(asset: Pubkey, payer: Pubkey) -> Instruction {
    // UpdateV1Args { new_name: Some("X"), new_uri: None, new_update_authority: None }
    let mut data = vec![15u8]; // discriminator
    data.push(1); // Option::Some for new_name
    data.extend_from_slice(&1u32.to_le_bytes()); // string length = 1
    data.push(b'X'); // "X"
    data.push(0); // Option::None for new_uri
    data.push(0); // Option::None for new_update_authority
    Instruction::new_with_bytes(
        MPL_CORE_ID,
        &data,
        vec![
            AccountMeta::new(asset, false),                // 0: asset (writable)
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 1: collection (optional)
            AccountMeta::new(payer, true),                 // 2: payer (writable, signer)
            AccountMeta::new_readonly(payer, true),        // 3: authority (signer)
            AccountMeta::new_readonly(system_program::ID, false), // 4: system_program
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 5: log_wrapper (optional)
        ],
    )
}

/// Builds an UpdateCollectionV1 instruction that changes the name to "X".
/// Discriminator 16, with Some(name) and None uri.
fn update_collection_v1_instruction(collection: Pubkey, payer: Pubkey) -> Instruction {
    // UpdateCollectionV1Args { new_name: Some("X"), new_uri: None }
    let mut data = vec![16u8]; // discriminator
    data.push(1); // Option::Some for new_name
    data.extend_from_slice(&1u32.to_le_bytes()); // string length = 1
    data.push(b'X'); // "X"
    data.push(0); // Option::None for new_uri
    Instruction::new_with_bytes(
        MPL_CORE_ID,
        &data,
        vec![
            AccountMeta::new(collection, false), // 0: collection (writable)
            AccountMeta::new(payer, true),       // 1: payer (writable, signer)
            AccountMeta::new_readonly(payer, true), // 2: authority (signer)
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 3: new_update_authority (optional)
            AccountMeta::new_readonly(system_program::ID, false), // 4: system_program
            AccountMeta::new_readonly(MPL_CORE_ID, false), // 5: log_wrapper (optional)
        ],
    )
}

/// Converts accounts Vec into the shared data format Mollusk expects.
fn to_mollusk_accounts(accounts: Vec<(Pubkey, Account)>) -> Vec<(Pubkey, Account)> {
    let mut result = accounts;

    // System program account must be present and executable.
    if let Some(pos) = result.iter().position(|(k, _)| *k == system_program::ID) {
        result.remove(pos);
    }
    let (sys_key, sys_account) = mollusk_svm::program::keyed_account_for_system_program();
    result.push((sys_key, sys_account));

    result
}

/// Asserts the instruction failed (any error).
fn assert_failure(result: &mollusk_svm::result::InstructionResult) {
    assert!(
        !matches!(
            result.program_result,
            mollusk_svm::result::ProgramResult::Success
        ),
        "Expected instruction to fail, but it succeeded"
    );
}

// ===========================================================================
// Section 1: Fake assets owned by a different program
//
// These tests create accounts with valid mpl-core AssetV1 data (correct
// discriminator byte, valid Borsh-serialized fields) but owned by a
// different program. They verify that mpl-core instructions reject these
// fake accounts.
// ===========================================================================

#[test]
fn transfer_rejects_fake_asset_owned_by_different_program() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    // Create a fake asset owned by FAKE_PROGRAM_ID with payer as the asset owner.
    let fake_asset = fake_asset_account(&payer, &FAKE_PROGRAM_ID);

    let instruction = transfer_v1_instruction(asset_key, payer, new_owner);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, fake_asset),
        (MPL_CORE_ID, Account::default()),
        (payer, payer_account()),
        (new_owner, Account::default()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

#[test]
fn burn_rejects_fake_asset_owned_by_different_program() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();

    let fake_asset = fake_asset_account(&payer, &FAKE_PROGRAM_ID);

    let instruction = burn_v1_instruction(asset_key, payer);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, fake_asset),
        (MPL_CORE_ID, Account::default()),
        (payer, payer_account()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

#[test]
fn update_rejects_fake_asset_owned_by_different_program() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();

    let fake_asset = fake_asset_account(&payer, &FAKE_PROGRAM_ID);

    let instruction = update_v1_instruction(asset_key, payer);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, fake_asset),
        (MPL_CORE_ID, Account::default()),
        (payer, payer_account()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

// ===========================================================================
// Section 2: Fake collections owned by a different program
// ===========================================================================

#[test]
fn update_collection_rejects_fake_collection_owned_by_different_program() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let collection_key = Pubkey::new_unique();

    let fake_collection = fake_collection_account(&payer, &FAKE_PROGRAM_ID);

    let instruction = update_collection_v1_instruction(collection_key, payer);

    let accounts = to_mollusk_accounts(vec![
        (collection_key, fake_collection),
        (payer, payer_account()),
        (MPL_CORE_ID, Account::default()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

// ===========================================================================
// Section 3: Accounts with wrong discriminator
//
// Even with valid Borsh data, an account with the wrong discriminator byte
// should be rejected immediately.
// ===========================================================================

#[test]
fn transfer_rejects_account_with_wrong_discriminator() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    // Create account data with CollectionV1 discriminator (5) instead of AssetV1 (1).
    let collection = CollectionV1::new(
        payer,
        "Wrong".to_string(),
        "https://wrong.com".to_string(),
        0,
        0,
    );
    let data = collection.try_to_vec().unwrap();
    let wrong_disc_account = Account {
        lamports: ACCOUNT_LAMPORTS,
        data,
        owner: MPL_CORE_ID, // Even with correct owner!
        executable: false,
        rent_epoch: 0,
    };

    let instruction = transfer_v1_instruction(asset_key, payer, new_owner);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, wrong_disc_account),
        (MPL_CORE_ID, Account::default()),
        (payer, payer_account()),
        (new_owner, Account::default()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

#[test]
fn burn_rejects_account_with_wrong_discriminator() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();

    // Use Uninitialized key (0) - completely wrong discriminator.
    let mut data = vec![0u8; 100]; // Key::Uninitialized = 0
    data[0] = Key::Uninitialized as u8;
    let wrong_disc_account = Account {
        lamports: ACCOUNT_LAMPORTS,
        data,
        owner: MPL_CORE_ID,
        executable: false,
        rent_epoch: 0,
    };

    let instruction = burn_v1_instruction(asset_key, payer);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, wrong_disc_account),
        (MPL_CORE_ID, Account::default()),
        (payer, payer_account()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

// ===========================================================================
// Section 4: Random / garbage data accounts
//
// Accounts with completely random data should fail at deserialization.
// ===========================================================================

#[test]
fn transfer_rejects_random_data_account() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    let garbage_account = Account {
        lamports: ACCOUNT_LAMPORTS,
        data: vec![0xFF; 200],
        owner: MPL_CORE_ID,
        executable: false,
        rent_epoch: 0,
    };

    let instruction = transfer_v1_instruction(asset_key, payer, new_owner);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, garbage_account),
        (MPL_CORE_ID, Account::default()),
        (payer, payer_account()),
        (new_owner, Account::default()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

// ===========================================================================
// Section 5: Empty accounts
//
// Completely empty accounts should fail immediately.
// ===========================================================================

#[test]
fn transfer_rejects_empty_account() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    let empty_account = Account {
        lamports: ACCOUNT_LAMPORTS,
        data: vec![],
        owner: MPL_CORE_ID,
        executable: false,
        rent_epoch: 0,
    };

    let instruction = transfer_v1_instruction(asset_key, payer, new_owner);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, empty_account),
        (MPL_CORE_ID, Account::default()),
        (payer, payer_account()),
        (new_owner, Account::default()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

// ===========================================================================
// Section 6: Valid accounts work correctly (sanity checks)
//
// Ensure that properly owned mpl-core accounts ARE accepted so we know
// our test harness works correctly.
// ===========================================================================

#[test]
fn transfer_succeeds_with_valid_asset() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    let asset = valid_asset_account(&payer);

    let instruction = transfer_v1_instruction(asset_key, payer, new_owner);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, asset),
        (MPL_CORE_ID, Account::default()),
        (payer, payer_account()),
        (new_owner, Account::default()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    // This should succeed because the asset is owned by mpl-core, the
    // discriminator matches, and the payer is the asset owner.
    assert!(
        matches!(
            result.program_result,
            mollusk_svm::result::ProgramResult::Success
        ),
        "Expected transfer to succeed with valid asset, got: {:?}",
        result.program_result
    );
}

// ===========================================================================
// Section 7: System-program-owned accounts (uninitialized)
//
// A system-program-owned account with an AssetV1 discriminator should fail.
// This simulates someone trying to use an unrelated system account.
// ===========================================================================

#[test]
fn transfer_rejects_system_owned_account_with_asset_discriminator() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    // Create account with valid AssetV1 data but owned by system program.
    let fake_asset = fake_asset_account(&payer, &system_program::ID);

    let instruction = transfer_v1_instruction(asset_key, payer, new_owner);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, fake_asset),
        (MPL_CORE_ID, Account::default()),
        (payer, payer_account()),
        (new_owner, Account::default()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

#[test]
fn update_rejects_system_owned_account_with_asset_discriminator() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();

    let fake_asset = fake_asset_account(&payer, &system_program::ID);

    let instruction = update_v1_instruction(asset_key, payer);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, fake_asset),
        (MPL_CORE_ID, Account::default()),
        (payer, payer_account()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

// ===========================================================================
// Section 8: Fake asset with mismatched authority
//
// Even if an account has valid AssetV1 data and is owned by mpl-core,
// the authority check should reject unauthorized callers.
// ===========================================================================

#[test]
fn transfer_rejects_unauthorized_caller_on_valid_asset() {
    let mollusk = core_mollusk();
    let actual_owner = Pubkey::new_unique();
    let attacker = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    // Asset is owned by actual_owner, but attacker is trying to transfer.
    let asset = valid_asset_account(&actual_owner);

    let instruction = transfer_v1_instruction(asset_key, attacker, new_owner);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, asset),
        (MPL_CORE_ID, Account::default()),
        (attacker, payer_account()),
        (new_owner, Account::default()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

#[test]
fn burn_rejects_unauthorized_caller_on_valid_asset() {
    let mollusk = core_mollusk();
    let actual_owner = Pubkey::new_unique();
    let attacker = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();

    let asset = valid_asset_account(&actual_owner);

    let instruction = burn_v1_instruction(asset_key, attacker);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, asset),
        (MPL_CORE_ID, Account::default()),
        (attacker, payer_account()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

// ===========================================================================
// Section 9: Fake assets with freeze plugins (owned by different program)
//
// These tests create accounts with valid AssetV1 data AND embedded
// FreezeDelegate plugin data, but owned by FAKE_PROGRAM_ID. They verify
// that mpl-core rejects these fake accounts regardless of freeze state.
// ===========================================================================

#[test]
fn transfer_rejects_fake_frozen_asset_owned_by_different_program() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    // Fake asset with FreezeDelegate{frozen: true}, owned by attacker's program.
    let fake_asset = build_asset_with_plugins(
        &payer,
        &FAKE_PROGRAM_ID,
        &[(
            Plugin::FreezeDelegate(FreezeDelegate { frozen: true }),
            Authority::Owner,
        )],
    );

    let instruction = transfer_v1_instruction(asset_key, payer, new_owner);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, fake_asset),
        (MPL_CORE_ID, Account::default()),
        (payer, payer_account()),
        (new_owner, Account::default()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

#[test]
fn transfer_rejects_fake_unfrozen_asset_owned_by_different_program() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    // Fake asset with FreezeDelegate{frozen: false} -- even an "unfrozen" fake must fail.
    let fake_asset = build_asset_with_plugins(
        &payer,
        &FAKE_PROGRAM_ID,
        &[(
            Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
            Authority::Owner,
        )],
    );

    let instruction = transfer_v1_instruction(asset_key, payer, new_owner);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, fake_asset),
        (MPL_CORE_ID, Account::default()),
        (payer, payer_account()),
        (new_owner, Account::default()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

#[test]
fn burn_rejects_fake_frozen_asset_owned_by_different_program() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();

    let fake_asset = build_asset_with_plugins(
        &payer,
        &FAKE_PROGRAM_ID,
        &[(
            Plugin::FreezeDelegate(FreezeDelegate { frozen: true }),
            Authority::Owner,
        )],
    );

    let instruction = burn_v1_instruction(asset_key, payer);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, fake_asset),
        (MPL_CORE_ID, Account::default()),
        (payer, payer_account()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

// ===========================================================================
// Section 10: Fake assets with permanent delegate plugins
//
// Permanent delegates can force-approve transfers/burns. These tests verify
// that even with PermanentTransferDelegate or PermanentBurnDelegate embedded
// in a fake account, the instruction is still rejected because the account
// is not owned by mpl-core.
// ===========================================================================

#[test]
fn transfer_rejects_fake_asset_with_permanent_transfer_delegate() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    // PermanentTransferDelegate allows anyone to transfer -- but not on a fake account.
    let fake_asset = build_asset_with_plugins(
        &payer,
        &FAKE_PROGRAM_ID,
        &[(
            Plugin::PermanentTransferDelegate(PermanentTransferDelegate {}),
            Authority::Owner,
        )],
    );

    let instruction = transfer_v1_instruction(asset_key, payer, new_owner);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, fake_asset),
        (MPL_CORE_ID, Account::default()),
        (payer, payer_account()),
        (new_owner, Account::default()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

#[test]
fn burn_rejects_fake_asset_with_permanent_burn_delegate() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();

    // PermanentBurnDelegate allows anyone to burn -- but not on a fake account.
    let fake_asset = build_asset_with_plugins(
        &payer,
        &FAKE_PROGRAM_ID,
        &[(
            Plugin::PermanentBurnDelegate(PermanentBurnDelegate {}),
            Authority::Owner,
        )],
    );

    let instruction = burn_v1_instruction(asset_key, payer);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, fake_asset),
        (MPL_CORE_ID, Account::default()),
        (payer, payer_account()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

#[test]
fn transfer_rejects_fake_asset_with_permanent_freeze_frozen() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    // PermanentFreezeDelegate{frozen: true} on a fake account.
    let fake_asset = build_asset_with_plugins(
        &payer,
        &FAKE_PROGRAM_ID,
        &[(
            Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen: true }),
            Authority::Owner,
        )],
    );

    let instruction = transfer_v1_instruction(asset_key, payer, new_owner);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, fake_asset),
        (MPL_CORE_ID, Account::default()),
        (payer, payer_account()),
        (new_owner, Account::default()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

// ===========================================================================
// Section 11: Fake assets with multiple conflicting plugins
//
// Tests that even with "favorable" plugin combinations (e.g., frozen but
// with PermanentTransferDelegate), fake accounts are still rejected.
// ===========================================================================

#[test]
fn transfer_rejects_fake_asset_frozen_but_with_permanent_transfer() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    // FreezeDelegate{frozen:true} + PermanentTransferDelegate -- the permanent
    // delegate would normally force-approve, but the fake account fails first.
    let fake_asset = build_asset_with_plugins(
        &payer,
        &FAKE_PROGRAM_ID,
        &[
            (
                Plugin::FreezeDelegate(FreezeDelegate { frozen: true }),
                Authority::Owner,
            ),
            (
                Plugin::PermanentTransferDelegate(PermanentTransferDelegate {}),
                Authority::Owner,
            ),
        ],
    );

    let instruction = transfer_v1_instruction(asset_key, payer, new_owner);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, fake_asset),
        (MPL_CORE_ID, Account::default()),
        (payer, payer_account()),
        (new_owner, Account::default()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

#[test]
fn burn_rejects_fake_asset_unfrozen_with_permanent_burn() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();

    // PermanentBurnDelegate + FreezeDelegate{frozen:false} -- all favorable
    // plugin state, but fake account ownership means rejection.
    let fake_asset = build_asset_with_plugins(
        &payer,
        &FAKE_PROGRAM_ID,
        &[
            (
                Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
                Authority::Owner,
            ),
            (
                Plugin::PermanentBurnDelegate(PermanentBurnDelegate {}),
                Authority::Owner,
            ),
        ],
    );

    let instruction = burn_v1_instruction(asset_key, payer);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, fake_asset),
        (MPL_CORE_ID, Account::default()),
        (payer, payer_account()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

// ===========================================================================
// Section 12: Fake collections with plugins
//
// Valid assets that reference a fake collection (owned by FAKE_PROGRAM_ID)
// with favorable plugin data. The owner check in SolanaAccount::load
// rejects the fake collection with InvalidAccountOwner before any plugin
// data is even evaluated.
// ===========================================================================

#[test]
fn transfer_rejects_when_fake_collection_has_permanent_freeze() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let collection_key = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    // Valid asset that belongs to a collection.
    let asset = AssetV1::new(
        payer,
        UpdateAuthority::Collection(collection_key),
        "Real Asset".to_string(),
        "https://example.com/real".to_string(),
    );
    let asset_account = Account {
        lamports: ACCOUNT_LAMPORTS,
        data: asset.try_to_vec().unwrap(),
        owner: MPL_CORE_ID,
        executable: false,
        rent_epoch: 0,
    };

    // Fake collection with PermanentFreezeDelegate{frozen: false}, owned by
    // FAKE_PROGRAM_ID. The owner check rejects this before plugins are evaluated.
    let fake_collection = build_collection_with_plugins(
        &payer,
        &FAKE_PROGRAM_ID,
        &[(
            Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen: false }),
            Authority::UpdateAuthority,
        )],
    );

    let instruction =
        transfer_v1_with_collection_instruction(asset_key, collection_key, payer, new_owner);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, asset_account),
        (collection_key, fake_collection),
        (payer, payer_account()),
        (new_owner, Account::default()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

#[test]
fn transfer_rejects_when_fake_collection_has_permanent_transfer_delegate() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let collection_key = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    let asset = AssetV1::new(
        payer,
        UpdateAuthority::Collection(collection_key),
        "Real Asset".to_string(),
        "https://example.com/real".to_string(),
    );
    let asset_account = Account {
        lamports: ACCOUNT_LAMPORTS,
        data: asset.try_to_vec().unwrap(),
        owner: MPL_CORE_ID,
        executable: false,
        rent_epoch: 0,
    };

    // Fake collection with PermanentTransferDelegate -- owned by FAKE_PROGRAM_ID.
    // The owner check rejects this before plugins are evaluated.
    let fake_collection = build_collection_with_plugins(
        &payer,
        &FAKE_PROGRAM_ID,
        &[(
            Plugin::PermanentTransferDelegate(PermanentTransferDelegate {}),
            Authority::UpdateAuthority,
        )],
    );

    let instruction =
        transfer_v1_with_collection_instruction(asset_key, collection_key, payer, new_owner);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, asset_account),
        (collection_key, fake_collection),
        (payer, payer_account()),
        (new_owner, Account::default()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_failure(&result);
}

// ===========================================================================
// Section 13: Valid frozen asset sanity checks
//
// Verify that plugin enforcement actually works on valid mpl-core-owned
// accounts: frozen assets should reject transfer, unfrozen assets with
// freeze plugin should allow transfer.
// ===========================================================================

#[test]
fn transfer_rejects_valid_frozen_asset() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    // Valid mpl-core-owned asset with FreezeDelegate{frozen: true}.
    let frozen_asset = build_asset_with_plugins(
        &payer,
        &MPL_CORE_ID,
        &[(
            Plugin::FreezeDelegate(FreezeDelegate { frozen: true }),
            Authority::Owner,
        )],
    );

    let instruction = transfer_v1_instruction(asset_key, payer, new_owner);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, frozen_asset),
        (MPL_CORE_ID, Account::default()),
        (payer, payer_account()),
        (new_owner, Account::default()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    // A valid frozen asset should reject transfer due to freeze plugin.
    assert_failure(&result);
}

#[test]
fn transfer_succeeds_valid_unfrozen_asset_with_freeze_plugin() {
    let mollusk = core_mollusk();
    let payer = Pubkey::new_unique();
    let asset_key = Pubkey::new_unique();
    let new_owner = Pubkey::new_unique();

    // Valid mpl-core-owned asset with FreezeDelegate{frozen: false}.
    let unfrozen_asset = build_asset_with_plugins(
        &payer,
        &MPL_CORE_ID,
        &[(
            Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
            Authority::Owner,
        )],
    );

    let instruction = transfer_v1_instruction(asset_key, payer, new_owner);

    let accounts = to_mollusk_accounts(vec![
        (asset_key, unfrozen_asset),
        (MPL_CORE_ID, Account::default()),
        (payer, payer_account()),
        (new_owner, Account::default()),
        (system_program::ID, Account::default()),
    ]);

    let result = mollusk.process_instruction(&instruction, &accounts);

    // An unfrozen valid asset should allow transfer by the owner.
    assert!(
        matches!(
            result.program_result,
            mollusk_svm::result::ProgramResult::Success
        ),
        "Expected transfer to succeed with valid unfrozen asset, got: {:?}",
        result.program_result
    );
}
